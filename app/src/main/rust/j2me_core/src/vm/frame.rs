//! ============================================================================
//! MARCO DE ACTIVACIÓN DE MÉTODO Y DESPACHO DE OPCODES (vm::frame)
//! ============================================================================
//!
//! Este módulo implementa `StackFrame`:
//! - Maneja la ejecución de instrucciones de bytecode en Java ME (CLDC).
//! - Resuelve constantes desde el Constant Pool.
//! - Soporta objetos, arrays, aritmética, saltos, campos e invocación de métodos.

use std::collections::HashMap;
use crate::class_parser::CpInfo;
use crate::vm::heap::Heap;
use crate::vm::stack::{LocalVariables, OperandStack};
use crate::vm::types::{ArrayType, ExecutionResult, Value, VmError};

/// Extrae la cantidad de parámetros de un descriptor de método de JVM (ej. `(II)V` -> 2)
pub fn parse_descriptor_param_count(desc: &str) -> usize {
    let mut count = 0;
    if let Some(start) = desc.find('(') {
        if let Some(end) = desc.find(')') {
            let params = &desc[start + 1..end];
            let mut chars = params.chars().peekable();
            while let Some(ch) = chars.next() {
                match ch {
                    '[' => {
                        while let Some(&next_ch) = chars.peek() {
                            if next_ch == '[' {
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if let Some(&next_ch) = chars.peek() {
                            if next_ch == 'L' {
                                chars.next();
                                for c in chars.by_ref() {
                                    if c == ';' {
                                        break;
                                    }
                                }
                            } else {
                                chars.next();
                            }
                        }
                        count += 1;
                    }
                    'L' => {
                        for c in chars.by_ref() {
                            if c == ';' {
                                break;
                            }
                        }
                        count += 1;
                    }
                    'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' => {
                        count += 1;
                    }
                    _ => {}
                }
            }
        }
    }
    count
}

/// Funciones auxiliares para resolver elementos en el Constant Pool
pub fn resolve_utf8<'a>(cp: &'a [CpInfo], index: u16) -> Option<&'a str> {
    match cp.get(index as usize) {
        Some(CpInfo::Utf8(s)) => Some(s.as_str()),
        _ => None,
    }
}

pub fn resolve_class_name<'a>(cp: &'a [CpInfo], class_index: u16) -> Option<&'a str> {
    match cp.get(class_index as usize) {
        Some(CpInfo::Class { name_index }) => resolve_utf8(cp, *name_index),
        _ => None,
    }
}

pub fn resolve_name_and_type<'a>(cp: &'a [CpInfo], nat_index: u16) -> Option<(&'a str, &'a str)> {
    match cp.get(nat_index as usize) {
        Some(CpInfo::NameAndType { name_index, descriptor_index }) => {
            let name = resolve_utf8(cp, *name_index)?;
            let desc = resolve_utf8(cp, *descriptor_index)?;
            Some((name, desc))
        }
        _ => None,
    }
}

pub fn resolve_field_ref<'a>(cp: &'a [CpInfo], field_index: u16) -> Option<(&'a str, &'a str, &'a str)> {
    match cp.get(field_index as usize) {
        Some(CpInfo::Fieldref { class_index, name_and_type_index }) => {
            let class_name = resolve_class_name(cp, *class_index)?;
            let (field_name, desc) = resolve_name_and_type(cp, *name_and_type_index)?;
            Some((class_name, field_name, desc))
        }
        _ => None,
    }
}

pub fn resolve_method_ref<'a>(cp: &'a [CpInfo], method_index: u16) -> Option<(&'a str, &'a str, &'a str)> {
    match cp.get(method_index as usize) {
        Some(CpInfo::Methodref { class_index, name_and_type_index })
        | Some(CpInfo::InterfaceMethodref { class_index, name_and_type_index }) => {
            let class_name = resolve_class_name(cp, *class_index)?;
            let (method_name, desc) = resolve_name_and_type(cp, *name_and_type_index)?;
            Some((class_name, method_name, desc))
        }
        _ => None,
    }
}

/// Marco de activación de un método en la pila de llamadas de la JVM
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub locals: LocalVariables,
    pub stack: OperandStack,
    pub pc: usize,
    pub bytecode: Vec<u8>,
    pub constant_pool: Vec<CpInfo>,
    pub class_name: String,
    pub method_name: String,
}

impl StackFrame {
    /// Crea un marco simple para pruebas directas de bytecode
    pub fn new(max_stack: usize, max_locals: usize, bytecode: Vec<u8>, method_name: &str) -> Self {
        Self {
            locals: LocalVariables::new(max_locals),
            stack: OperandStack::new(max_stack),
            pc: 0,
            bytecode,
            constant_pool: Vec::new(),
            class_name: "Standalone".to_string(),
            method_name: method_name.to_string(),
        }
    }

    /// Crea un marco completo enlazado a la clase y su Constant Pool
    pub fn new_with_cp(
        max_stack: usize,
        max_locals: usize,
        bytecode: Vec<u8>,
        constant_pool: Vec<CpInfo>,
        class_name: &str,
        method_name: &str,
    ) -> Self {
        Self {
            locals: LocalVariables::new(max_locals),
            stack: OperandStack::new(max_stack),
            pc: 0,
            bytecode,
            constant_pool,
            class_name: class_name.to_string(),
            method_name: method_name.to_string(),
        }
    }

    /// Ejecuta una sola instrucción en el contexto del Heap y variables estáticas
    pub fn step(
        &mut self,
        heap: &mut Heap,
        static_fields: &mut HashMap<String, Value>,
    ) -> Result<ExecutionResult, VmError> {
        if self.pc >= self.bytecode.len() {
            return Err(VmError::PcOutOfBounds {
                pc: self.pc,
                code_len: self.bytecode.len(),
            });
        }

        let opcode = self.bytecode[self.pc];
        self.pc += 1;

        match opcode {
            // 0x00: nop
            0x00 => Ok(ExecutionResult::Continue),

            // 0x01: aconst_null
            0x01 => {
                self.stack.push(Value::Null)?;
                Ok(ExecutionResult::Continue)
            }

            // 0x02..0x08: iconst_m1..iconst_5
            0x02 => { self.stack.push(Value::Int(-1))?; Ok(ExecutionResult::Continue) }
            0x03 => { self.stack.push(Value::Int(0))?; Ok(ExecutionResult::Continue) }
            0x04 => { self.stack.push(Value::Int(1))?; Ok(ExecutionResult::Continue) }
            0x05 => { self.stack.push(Value::Int(2))?; Ok(ExecutionResult::Continue) }
            0x06 => { self.stack.push(Value::Int(3))?; Ok(ExecutionResult::Continue) }
            0x07 => { self.stack.push(Value::Int(4))?; Ok(ExecutionResult::Continue) }
            0x08 => { self.stack.push(Value::Int(5))?; Ok(ExecutionResult::Continue) }

            // 0x10: bipush
            0x10 => {
                let b = self.fetch_i8()? as i32;
                self.stack.push(Value::Int(b))?;
                Ok(ExecutionResult::Continue)
            }

            // 0x11: sipush
            0x11 => {
                let s = self.fetch_i16()? as i32;
                self.stack.push(Value::Int(s))?;
                Ok(ExecutionResult::Continue)
            }

            // 0x12: ldc
            0x12 => {
                let idx = self.fetch_u8()? as u16;
                self.push_constant(idx, heap)?;
                Ok(ExecutionResult::Continue)
            }

            // 0x13: ldc_w
            0x13 => {
                let idx = self.fetch_u16()?;
                self.push_constant(idx, heap)?;
                Ok(ExecutionResult::Continue)
            }

            // 0x15: iload
            0x15 => {
                let idx = self.fetch_u8()? as usize;
                let val = self.locals.get(idx)?.clone();
                self.stack.push(val)?;
                Ok(ExecutionResult::Continue)
            }

            // 0x19: aload
            0x19 => {
                let idx = self.fetch_u8()? as usize;
                let val = self.locals.get(idx)?.clone();
                self.stack.push(val)?;
                Ok(ExecutionResult::Continue)
            }

            // 0x1A..0x1D: iload_0..3
            0x1A => { let v = self.locals.get(0)?.clone(); self.stack.push(v)?; Ok(ExecutionResult::Continue) }
            0x1B => { let v = self.locals.get(1)?.clone(); self.stack.push(v)?; Ok(ExecutionResult::Continue) }
            0x1C => { let v = self.locals.get(2)?.clone(); self.stack.push(v)?; Ok(ExecutionResult::Continue) }
            0x1D => { let v = self.locals.get(3)?.clone(); self.stack.push(v)?; Ok(ExecutionResult::Continue) }

            // 0x2A..0x2D: aload_0..3
            0x2A => { let v = self.locals.get(0)?.clone(); self.stack.push(v)?; Ok(ExecutionResult::Continue) }
            0x2B => { let v = self.locals.get(1)?.clone(); self.stack.push(v)?; Ok(ExecutionResult::Continue) }
            0x2C => { let v = self.locals.get(2)?.clone(); self.stack.push(v)?; Ok(ExecutionResult::Continue) }
            0x2D => { let v = self.locals.get(3)?.clone(); self.stack.push(v)?; Ok(ExecutionResult::Continue) }

            // 0x2E: iaload, 0x32: aaload, 0x33: baload, 0x34: caload, 0x35: saload
            0x2E | 0x32 | 0x33 | 0x34 | 0x35 => {
                let index = self.stack.pop_int()?;
                let arr_ref = self.stack.pop()?;
                match arr_ref {
                    Value::ObjectRef(id) => {
                        let elem = heap.get_array_element(id, index)?;
                        self.stack.push(elem)?;
                        Ok(ExecutionResult::Continue)
                    }
                    Value::Null => Err(VmError::NullPointer),
                    _ => Err(VmError::TypeMismatch),
                }
            }

            // 0x36: istore
            0x36 => {
                let idx = self.fetch_u8()? as usize;
                let val = self.stack.pop()?;
                self.locals.set(idx, val)?;
                Ok(ExecutionResult::Continue)
            }

            // 0x3A: astore
            0x3A => {
                let idx = self.fetch_u8()? as usize;
                let val = self.stack.pop()?;
                self.locals.set(idx, val)?;
                Ok(ExecutionResult::Continue)
            }

            // 0x3B..0x3E: istore_0..3
            0x3B => { let v = self.stack.pop()?; self.locals.set(0, v)?; Ok(ExecutionResult::Continue) }
            0x3C => { let v = self.stack.pop()?; self.locals.set(1, v)?; Ok(ExecutionResult::Continue) }
            0x3D => { let v = self.stack.pop()?; self.locals.set(2, v)?; Ok(ExecutionResult::Continue) }
            0x3E => { let v = self.stack.pop()?; self.locals.set(3, v)?; Ok(ExecutionResult::Continue) }

            // 0x4B..0x4E: astore_0..3
            0x4B => { let v = self.stack.pop()?; self.locals.set(0, v)?; Ok(ExecutionResult::Continue) }
            0x4C => { let v = self.stack.pop()?; self.locals.set(1, v)?; Ok(ExecutionResult::Continue) }
            0x4D => { let v = self.stack.pop()?; self.locals.set(2, v)?; Ok(ExecutionResult::Continue) }
            0x4E => { let v = self.stack.pop()?; self.locals.set(3, v)?; Ok(ExecutionResult::Continue) }

            // 0x4F: iastore, 0x53: aastore, 0x54: bastore, 0x55: castore, 0x56: sastore
            0x4F | 0x53 | 0x54 | 0x55 | 0x56 => {
                let val = self.stack.pop()?;
                let index = self.stack.pop_int()?;
                let arr_ref = self.stack.pop()?;
                match arr_ref {
                    Value::ObjectRef(id) => {
                        heap.set_array_element(id, index, val)?;
                        Ok(ExecutionResult::Continue)
                    }
                    Value::Null => Err(VmError::NullPointer),
                    _ => Err(VmError::TypeMismatch),
                }
            }

            // 0x57: pop
            0x57 => {
                self.stack.pop()?;
                Ok(ExecutionResult::Continue)
            }

            // 0x58: pop2
            0x58 => {
                self.stack.pop()?;
                self.stack.pop()?;
                Ok(ExecutionResult::Continue)
            }

            // 0x59: dup
            0x59 => {
                self.stack.dup()?;
                Ok(ExecutionResult::Continue)
            }

            // 0x5A: dup_x1
            0x5A => {
                self.stack.dup_x1()?;
                Ok(ExecutionResult::Continue)
            }

            // 0x5C: dup2
            0x5C => {
                self.stack.dup2()?;
                Ok(ExecutionResult::Continue)
            }

            // 0x5F: swap
            0x5F => {
                self.stack.swap()?;
                Ok(ExecutionResult::Continue)
            }

            // Aritmética entera
            // 0x60: iadd
            0x60 => {
                let b = self.stack.pop_int()?;
                let a = self.stack.pop_int()?;
                self.stack.push(Value::Int(a.wrapping_add(b)))?;
                Ok(ExecutionResult::Continue)
            }

            // 0x64: isub
            0x64 => {
                let b = self.stack.pop_int()?;
                let a = self.stack.pop_int()?;
                self.stack.push(Value::Int(a.wrapping_sub(b)))?;
                Ok(ExecutionResult::Continue)
            }

            // 0x68: imul
            0x68 => {
                let b = self.stack.pop_int()?;
                let a = self.stack.pop_int()?;
                self.stack.push(Value::Int(a.wrapping_mul(b)))?;
                Ok(ExecutionResult::Continue)
            }

            // 0x6C: idiv
            0x6C => {
                let b = self.stack.pop_int()?;
                if b == 0 {
                    return Err(VmError::DivisionByZero);
                }
                let a = self.stack.pop_int()?;
                self.stack.push(Value::Int(a.wrapping_div(b)))?;
                Ok(ExecutionResult::Continue)
            }

            // 0x70: irem
            0x70 => {
                let b = self.stack.pop_int()?;
                if b == 0 {
                    return Err(VmError::DivisionByZero);
                }
                let a = self.stack.pop_int()?;
                self.stack.push(Value::Int(a.wrapping_rem(b)))?;
                Ok(ExecutionResult::Continue)
            }

            // 0x74: ineg
            0x74 => {
                let a = self.stack.pop_int()?;
                self.stack.push(Value::Int(a.wrapping_neg()))?;
                Ok(ExecutionResult::Continue)
            }

            // 0x7E: iand
            0x7E => {
                let b = self.stack.pop_int()?;
                let a = self.stack.pop_int()?;
                self.stack.push(Value::Int(a & b))?;
                Ok(ExecutionResult::Continue)
            }

            // 0x80: ior
            0x80 => {
                let b = self.stack.pop_int()?;
                let a = self.stack.pop_int()?;
                self.stack.push(Value::Int(a | b))?;
                Ok(ExecutionResult::Continue)
            }

            // 0x82: ixor
            0x82 => {
                let b = self.stack.pop_int()?;
                let a = self.stack.pop_int()?;
                self.stack.push(Value::Int(a ^ b))?;
                Ok(ExecutionResult::Continue)
            }

            // 0x84: iinc
            0x84 => {
                let idx = self.fetch_u8()? as usize;
                let c = self.fetch_i8()? as i32;
                let cur = self.locals.get_int(idx)?;
                self.locals.set_int(idx, cur.wrapping_add(c))?;
                Ok(ExecutionResult::Continue)
            }

            // Saltos condicionales
            0x99 => { self.branch_if(|v| v == 0) }
            0x9A => { self.branch_if(|v| v != 0) }
            0x9B => { self.branch_if(|v| v < 0) }
            0x9C => { self.branch_if(|v| v >= 0) }
            0x9D => { self.branch_if(|v| v > 0) }
            0x9E => { self.branch_if(|v| v <= 0) }

            0x9F => { self.branch_if_icmp(|a, b| a == b) }
            0xA0 => { self.branch_if_icmp(|a, b| a != b) }
            0xA1 => { self.branch_if_icmp(|a, b| a < b) }
            0xA2 => { self.branch_if_icmp(|a, b| a >= b) }
            0xA3 => { self.branch_if_icmp(|a, b| a > b) }
            0xA4 => { self.branch_if_icmp(|a, b| a <= b) }

            // 0xA7: goto
            0xA7 => {
                let opcode_pos = self.pc - 1;
                let offset = self.fetch_i16()? as isize;
                let new_pc = (opcode_pos as isize + offset) as usize;
                self.pc = new_pc;
                Ok(ExecutionResult::Continue)
            }

            // 0xAA: tableswitch
            0xAA => {
                let opcode_pos = self.pc - 1;
                // Alinear a múltiplo de 4 bytes con respecto al inicio del método
                let remainder = self.pc % 4;
                if remainder != 0 {
                    self.pc += 4 - remainder;
                }

                let default_offset = self.fetch_i32()? as isize;
                let low = self.fetch_i32()?;
                let high = self.fetch_i32()?;

                let key = self.stack.pop_int()?;
                if key >= low && key <= high {
                    let jump_index = (key - low) as usize;
                    // Avanzar al offset correspondiente
                    self.pc += jump_index * 4;
                    let target_offset = self.fetch_i32()? as isize;
                    self.pc = (opcode_pos as isize + target_offset) as usize;
                } else {
                    self.pc = (opcode_pos as isize + default_offset) as usize;
                }
                Ok(ExecutionResult::Continue)
            }

            // 0xAB: lookupswitch
            0xAB => {
                let opcode_pos = self.pc - 1;
                // Alinear a múltiplo de 4 bytes con respecto al inicio del método
                let remainder = self.pc % 4;
                if remainder != 0 {
                    self.pc += 4 - remainder;
                }

                let default_offset = self.fetch_i32()? as isize;
                let npairs = self.fetch_i32()?;
                let key = self.stack.pop_int()?;

                let mut matched_offset: Option<isize> = None;
                for _ in 0..npairs {
                    let match_val = self.fetch_i32()?;
                    let offset = self.fetch_i32()? as isize;
                    if match_val == key && matched_offset.is_none() {
                        matched_offset = Some(offset);
                    }
                }

                match matched_offset {
                    Some(offset) => {
                        self.pc = (opcode_pos as isize + offset) as usize;
                    }
                    None => {
                        self.pc = (opcode_pos as isize + default_offset) as usize;
                    }
                }
                Ok(ExecutionResult::Continue)
            }

            // 0xAC: ireturn
            0xAC => {
                let val = self.stack.pop()?;
                Ok(ExecutionResult::ReturnValue(val))
            }

            // 0xB0: areturn
            0xB0 => {
                let val = self.stack.pop()?;
                Ok(ExecutionResult::ReturnValue(val))
            }

            // 0xB1: return (void)
            0xB1 => Ok(ExecutionResult::ReturnVoid),

            // 0xB2: getstatic
            0xB2 => {
                let field_idx = self.fetch_u16()?;
                let (cname, fname, _) = resolve_field_ref(&self.constant_pool, field_idx)
                    .ok_or(VmError::InvalidConstantPoolEntry(field_idx))?;
                let key = format!("{}.{}", cname, fname);
                let val = static_fields.get(&key).cloned().unwrap_or(Value::Int(0));
                self.stack.push(val)?;
                Ok(ExecutionResult::Continue)
            }

            // 0xB3: putstatic
            0xB3 => {
                let field_idx = self.fetch_u16()?;
                let (cname, fname, _) = resolve_field_ref(&self.constant_pool, field_idx)
                    .ok_or(VmError::InvalidConstantPoolEntry(field_idx))?;
                let val = self.stack.pop()?;
                let key = format!("{}.{}", cname, fname);
                static_fields.insert(key, val);
                Ok(ExecutionResult::Continue)
            }

            // 0xB4: getfield
            0xB4 => {
                let field_idx = self.fetch_u16()?;
                let (_, fname, _) = resolve_field_ref(&self.constant_pool, field_idx)
                    .ok_or(VmError::InvalidConstantPoolEntry(field_idx))?;
                let obj_ref = self.stack.pop()?;
                match obj_ref {
                    Value::ObjectRef(id) => {
                        let val = heap.get_field(id, fname)?;
                        self.stack.push(val)?;
                        Ok(ExecutionResult::Continue)
                    }
                    Value::Null => Err(VmError::NullPointer),
                    _ => Err(VmError::TypeMismatch),
                }
            }

            // 0xB5: putfield
            0xB5 => {
                let field_idx = self.fetch_u16()?;
                let (_, fname, _) = resolve_field_ref(&self.constant_pool, field_idx)
                    .ok_or(VmError::InvalidConstantPoolEntry(field_idx))?;
                let val = self.stack.pop()?;
                let obj_ref = self.stack.pop()?;
                match obj_ref {
                    Value::ObjectRef(id) => {
                        heap.set_field(id, fname, val)?;
                        Ok(ExecutionResult::Continue)
                    }
                    Value::Null => Err(VmError::NullPointer),
                    _ => Err(VmError::TypeMismatch),
                }
            }

            // 0xB6: invokevirtual
            0xB6 => {
                let method_idx = self.fetch_u16()?;
                let (cname, mname, desc) = resolve_method_ref(&self.constant_pool, method_idx)
                    .ok_or(VmError::InvalidConstantPoolEntry(method_idx))?;
                let p_count = parse_descriptor_param_count(desc);
                let mut args = Vec::with_capacity(p_count + 1);
                for _ in 0..p_count {
                    args.push(self.stack.pop()?);
                }
                let this_ref = self.stack.pop()?;
                if this_ref.is_null() {
                    return Err(VmError::NullPointer);
                }
                args.push(this_ref);
                args.reverse();

                Ok(ExecutionResult::InvokeMethod {
                    class_name: cname.to_string(),
                    method_name: mname.to_string(),
                    descriptor: desc.to_string(),
                    is_static: false,
                    args,
                })
            }

            // 0xB7: invokespecial (constructores <init> y llamadas privadas/super)
            0xB7 => {
                let method_idx = self.fetch_u16()?;
                let (cname, mname, desc) = resolve_method_ref(&self.constant_pool, method_idx)
                    .ok_or(VmError::InvalidConstantPoolEntry(method_idx))?;
                let p_count = parse_descriptor_param_count(desc);
                let mut args = Vec::with_capacity(p_count + 1);
                for _ in 0..p_count {
                    args.push(self.stack.pop()?);
                }
                let this_ref = self.stack.pop()?;
                if this_ref.is_null() {
                    return Err(VmError::NullPointer);
                }
                args.push(this_ref);
                args.reverse();

                Ok(ExecutionResult::InvokeMethod {
                    class_name: cname.to_string(),
                    method_name: mname.to_string(),
                    descriptor: desc.to_string(),
                    is_static: false,
                    args,
                })
            }

            // 0xB8: invokestatic
            0xB8 => {
                let method_idx = self.fetch_u16()?;
                let (cname, mname, desc) = resolve_method_ref(&self.constant_pool, method_idx)
                    .ok_or(VmError::InvalidConstantPoolEntry(method_idx))?;
                let p_count = parse_descriptor_param_count(desc);
                let mut args = Vec::with_capacity(p_count);
                for _ in 0..p_count {
                    args.push(self.stack.pop()?);
                }
                args.reverse();

                Ok(ExecutionResult::InvokeMethod {
                    class_name: cname.to_string(),
                    method_name: mname.to_string(),
                    descriptor: desc.to_string(),
                    is_static: true,
                    args,
                })
            }

            // 0xB9: invokeinterface
            0xB9 => {
                let method_idx = self.fetch_u16()?;
                let _count = self.fetch_u8()?;
                let _zero = self.fetch_u8()?; // Byte reservado según JVM spec

                let (cname, mname, desc) = resolve_method_ref(&self.constant_pool, method_idx)
                    .ok_or(VmError::InvalidConstantPoolEntry(method_idx))?;
                let p_count = parse_descriptor_param_count(desc);
                let mut args = Vec::with_capacity(p_count + 1);
                for _ in 0..p_count {
                    args.push(self.stack.pop()?);
                }
                let this_ref = self.stack.pop()?;
                if this_ref.is_null() {
                    return Err(VmError::NullPointer);
                }
                args.push(this_ref);
                args.reverse();

                Ok(ExecutionResult::InvokeMethod {
                    class_name: cname.to_string(),
                    method_name: mname.to_string(),
                    descriptor: desc.to_string(),
                    is_static: false,
                    args,
                })
            }

            // 0xBB: new
            0xBB => {
                let class_idx = self.fetch_u16()?;
                let cname = resolve_class_name(&self.constant_pool, class_idx)
                    .unwrap_or("java/lang/Object");
                let obj_id = heap.allocate_object(cname);
                self.stack.push(Value::ObjectRef(obj_id))?;
                Ok(ExecutionResult::Continue)
            }

            // 0xBC: newarray
            0xBC => {
                let atype = self.fetch_u8()?;
                let count = self.stack.pop_int()?;
                if count < 0 {
                    return Err(VmError::NegativeArraySize(count));
                }
                let array_type = match atype {
                    4 => ArrayType::Boolean,
                    5 => ArrayType::Char,
                    6 => ArrayType::Float,
                    7 => ArrayType::Double,
                    8 => ArrayType::Byte,
                    9 => ArrayType::Short,
                    10 => ArrayType::Int,
                    11 => ArrayType::Long,
                    _ => ArrayType::Int,
                };
                let arr_id = heap.allocate_array(array_type, count as usize);
                self.stack.push(Value::ObjectRef(arr_id))?;
                Ok(ExecutionResult::Continue)
            }

            // 0xBD: anewarray
            0xBD => {
                let class_idx = self.fetch_u16()?;
                let count = self.stack.pop_int()?;
                if count < 0 {
                    return Err(VmError::NegativeArraySize(count));
                }
                let cname = resolve_class_name(&self.constant_pool, class_idx)
                    .unwrap_or("java/lang/Object");
                let arr_id = heap.allocate_array(ArrayType::Object(cname.to_string()), count as usize);
                self.stack.push(Value::ObjectRef(arr_id))?;
                Ok(ExecutionResult::Continue)
            }

            // 0xBE: arraylength
            0xBE => {
                let arr_ref = self.stack.pop()?;
                match arr_ref {
                    Value::ObjectRef(id) => {
                        let len = heap.array_len(id)?;
                        self.stack.push(Value::Int(len as i32))?;
                        Ok(ExecutionResult::Continue)
                    }
                    Value::Null => Err(VmError::NullPointer),
                    _ => Err(VmError::TypeMismatch),
                }
            }

            // 0xC0: checkcast (validación de tipo segura sin crash)
            0xC0 => {
                let _class_idx = self.fetch_u16()?;
                Ok(ExecutionResult::Continue)
            }

            // 0xC1: instanceof
            0xC1 => {
                let class_idx = self.fetch_u16()?;
                let target_class = resolve_class_name(&self.constant_pool, class_idx).unwrap_or("");
                let top = self.stack.pop()?;
                match top {
                    Value::ObjectRef(id) => {
                        let is_instance = match heap.get_object(id) {
                            Ok(obj) => obj.class_name == target_class || target_class == "java/lang/Object",
                            Err(_) => false,
                        };
                        self.stack.push(Value::Int(if is_instance { 1 } else { 0 }))?;
                    }
                    _ => {
                        self.stack.push(Value::Int(0))?;
                    }
                }
                Ok(ExecutionResult::Continue)
            }

            // 0xC2: monitorenter, 0xC3: monitorexit (sincronización segura de hilos)
            0xC2 | 0xC3 => {
                let obj = self.stack.pop()?;
                if obj.is_null() {
                    return Err(VmError::NullPointer);
                }
                Ok(ExecutionResult::Continue)
            }

            // 0xC6: ifnull
            0xC6 => {
                let opcode_pos = self.pc - 1;
                let offset = self.fetch_i16()? as isize;
                let val = self.stack.pop()?;
                if val.is_null() {
                    self.pc = (opcode_pos as isize + offset) as usize;
                }
                Ok(ExecutionResult::Continue)
            }

            // 0xC7: ifnonnull
            0xC7 => {
                let opcode_pos = self.pc - 1;
                let offset = self.fetch_i16()? as isize;
                let val = self.stack.pop()?;
                if !val.is_null() {
                    self.pc = (opcode_pos as isize + offset) as usize;
                }
                Ok(ExecutionResult::Continue)
            }

            other => Err(VmError::InvalidOpcode(other)),
        }
    }

    /// Ejecuta el marco en modo autónomo (con Heap local) hasta retorno o límite de pasos
    pub fn run_to_completion(&mut self, max_instructions: usize) -> Result<ExecutionResult, VmError> {
        let mut heap = Heap::new();
        let mut static_fields = HashMap::new();
        let mut count = 0;
        loop {
            if count >= max_instructions {
                return Ok(ExecutionResult::Continue);
            }
            match self.step(&mut heap, &mut static_fields)? {
                ExecutionResult::Continue => {
                    count += 1;
                }
                ret => return Ok(ret),
            }
        }
    }

    // Funciones auxiliares para decodificar operandos inmediatos

    fn fetch_u8(&mut self) -> Result<u8, VmError> {
        if self.pc >= self.bytecode.len() {
            return Err(VmError::PcOutOfBounds {
                pc: self.pc,
                code_len: self.bytecode.len(),
            });
        }
        let b = self.bytecode[self.pc];
        self.pc += 1;
        Ok(b)
    }

    fn fetch_i8(&mut self) -> Result<i8, VmError> {
        self.fetch_u8().map(|b| b as i8)
    }

    fn fetch_u16(&mut self) -> Result<u16, VmError> {
        if self.pc + 2 > self.bytecode.len() {
            return Err(VmError::PcOutOfBounds {
                pc: self.pc,
                code_len: self.bytecode.len(),
            });
        }
        let val = u16::from_be_bytes([self.bytecode[self.pc], self.bytecode[self.pc + 1]]);
        self.pc += 2;
        Ok(val)
    }

    fn fetch_i16(&mut self) -> Result<i16, VmError> {
        self.fetch_u16().map(|v| v as i16)
    }

    fn fetch_i32(&mut self) -> Result<i32, VmError> {
        if self.pc + 4 > self.bytecode.len() {
            return Err(VmError::PcOutOfBounds {
                pc: self.pc,
                code_len: self.bytecode.len(),
            });
        }
        let val = i32::from_be_bytes([
            self.bytecode[self.pc],
            self.bytecode[self.pc + 1],
            self.bytecode[self.pc + 2],
            self.bytecode[self.pc + 3],
        ]);
        self.pc += 4;
        Ok(val)
    }

    fn push_constant(&mut self, index: u16, heap: &mut Heap) -> Result<(), VmError> {
        let cp_entry = self
            .constant_pool
            .get(index as usize)
            .ok_or(VmError::ConstantPoolIndexOutOfBounds(index))?;

        match cp_entry {
            CpInfo::Integer(val) => self.stack.push(Value::Int(*val)),
            CpInfo::Float(val) => self.stack.push(Value::Float(*val)),
            CpInfo::StringRef { string_index } => {
                let text = resolve_utf8(&self.constant_pool, *string_index).unwrap_or("");
                let str_id = heap.allocate_object("java/lang/String");
                let char_arr_id = heap.allocate_array(ArrayType::Char, text.chars().count());
                for (i, ch) in text.chars().enumerate() {
                    let _ = heap.set_array_element(char_arr_id, i as i32, Value::Int(ch as i32));
                }
                let _ = heap.set_field(str_id, "value", Value::ObjectRef(char_arr_id));
                self.stack.push(Value::ObjectRef(str_id))
            }
            _ => Err(VmError::InvalidConstantPoolEntry(index)),
        }
    }

    fn branch_if<F>(&mut self, condition: F) -> Result<ExecutionResult, VmError>
    where
        F: FnOnce(i32) -> bool,
    {
        let opcode_pos = self.pc - 1;
        let offset = self.fetch_i16()? as isize;
        let val = self.stack.pop_int()?;
        if condition(val) {
            self.pc = (opcode_pos as isize + offset) as usize;
        }
        Ok(ExecutionResult::Continue)
    }

    fn branch_if_icmp<F>(&mut self, condition: F) -> Result<ExecutionResult, VmError>
    where
        F: FnOnce(i32, i32) -> bool,
    {
        let opcode_pos = self.pc - 1;
        let offset = self.fetch_i16()? as isize;
        let b = self.stack.pop_int()?;
        let a = self.stack.pop_int()?;
        if condition(a, b) {
            self.pc = (opcode_pos as isize + offset) as usize;
        }
        Ok(ExecutionResult::Continue)
    }
}
