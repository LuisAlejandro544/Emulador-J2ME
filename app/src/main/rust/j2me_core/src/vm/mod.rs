//! ============================================================================
//! ESTRUCTURAS DE EJECUCIÓN DE LA MÁQUINA VIRTUAL CLDC / JVM EN RUST (vm)
//! ============================================================================
//!
//! Este módulo define las estructuras de tiempo de ejecución (Runtime) de la JVM:
//! 1. `Value`: Representación tipada de valores (enteros, flotantes, referencias a objetos, nulo).
//! 2. `OperandStack`: Pila de operandos de tamaño acotado con validación de límites.
//! 3. `LocalVariables`: Tabla de variables locales indexadas para argumentos y variables.
//! 4. `StackFrame`: Marco de activación de un método (`pc`, pila, variables locales y bytecode).
//! 5. `Interpreter`: Bucle de ejecución para las instrucciones estándar de JVM (Java ME CLDC).
//!
//! Garantías de diseño:
//! - Cero desbordamientos de búfer (control estricto de límites).
//! - Detección controlada de división por cero y NullPointerExceptions.
//! - Arquitectura agnóstica compatible con ARM32, ARM64 y x86/x86_64.

use std::fmt;

/// Errores durante la ejecución de una instrucción de la máquina virtual
#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    StackOverflow,
    StackUnderflow,
    LocalVariableOutOfBounds { index: usize, max: usize },
    InvalidOpcode(u8),
    DivisionByZero,
    NullPointer,
    PcOutOfBounds { pc: usize, code_len: usize },
    TypeMismatch,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::StackOverflow => write!(f, "Desbordamiento de pila de operandos (StackOverflow)"),
            VmError::StackUnderflow => write!(f, "Subdesbordamiento de pila de operandos (StackUnderflow)"),
            VmError::LocalVariableOutOfBounds { index, max } => {
                write!(f, "Índice de variable local fuera de rango: {} (máx: {})", index, max)
            }
            VmError::InvalidOpcode(op) => write!(f, "Opcode no reconocido o no soportado: 0x{:02X}", op),
            VmError::DivisionByZero => write!(f, "Intento de división o módulo por cero"),
            VmError::NullPointer => write!(f, "Acceso a referencia nula (NullPointerException)"),
            VmError::PcOutOfBounds { pc, code_len } => {
                write!(f, "Contador de programa fuera de límites: {} (longitud: {})", pc, code_len)
            }
            VmError::TypeMismatch => write!(f, "Incompatibilidad de tipos en la pila de operandos"),
        }
    }
}

/// Representación de un valor de 32/64 bits en la máquina virtual CLDC
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    ObjectRef(u32),
    Null,
}

impl Value {
    pub fn as_int(&self) -> Result<i32, VmError> {
        match self {
            Value::Int(v) => Ok(*v),
            _ => Err(VmError::TypeMismatch),
        }
    }

    pub fn as_ref(&self) -> Result<Option<u32>, VmError> {
        match self {
            Value::Null => Ok(None),
            Value::ObjectRef(id) => Ok(Some(*id)),
            _ => Err(VmError::TypeMismatch),
        }
    }
}

/// Pila de operandos de la JVM para un marco de ejecución (StackFrame)
#[derive(Debug, Clone)]
pub struct OperandStack {
    values: Vec<Value>,
    max_stack: usize,
}

impl OperandStack {
    pub fn new(max_stack: usize) -> Self {
        Self {
            values: Vec::with_capacity(max_stack),
            max_stack,
        }
    }

    pub fn push(&mut self, value: Value) -> Result<(), VmError> {
        if self.values.len() >= self.max_stack {
            return Err(VmError::StackOverflow);
        }
        self.values.push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Value, VmError> {
        self.values.pop().ok_or(VmError::StackUnderflow)
    }

    pub fn pop_int(&mut self) -> Result<i32, VmError> {
        let val = self.pop()?;
        val.as_int()
    }

    pub fn peek(&self) -> Result<&Value, VmError> {
        self.values.last().ok_or(VmError::StackUnderflow)
    }

    pub fn dup(&mut self) -> Result<(), VmError> {
        let top = self.peek()?.clone();
        self.push(top)
    }

    pub fn swap(&mut self) -> Result<(), VmError> {
        if self.values.len() < 2 {
            return Err(VmError::StackUnderflow);
        }
        let len = self.values.len();
        self.values.swap(len - 1, len - 2);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }
}

/// Tabla de variables locales para parámetros y variables de método
#[derive(Debug, Clone)]
pub struct LocalVariables {
    slots: Vec<Value>,
    max_locals: usize,
}

impl LocalVariables {
    pub fn new(max_locals: usize) -> Self {
        let mut slots = Vec::with_capacity(max_locals);
        for _ in 0..max_locals {
            slots.push(Value::Null);
        }
        Self { slots, max_locals }
    }

    pub fn get(&self, index: usize) -> Result<&Value, VmError> {
        if index >= self.max_locals {
            return Err(VmError::LocalVariableOutOfBounds {
                index,
                max: self.max_locals,
            });
        }
        Ok(&self.slots[index])
    }

    pub fn get_int(&self, index: usize) -> Result<i32, VmError> {
        self.get(index)?.as_int()
    }

    pub fn set(&mut self, index: usize, value: Value) -> Result<(), VmError> {
        if index >= self.max_locals {
            return Err(VmError::LocalVariableOutOfBounds {
                index,
                max: self.max_locals,
            });
        }
        self.slots[index] = value;
        Ok(())
    }

    pub fn set_int(&mut self, index: usize, value: i32) -> Result<(), VmError> {
        self.set(index, Value::Int(value))
    }
}

/// Estado de finalización del paso o método
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionResult {
    Continue,
    ReturnVoid,
    ReturnValue(Value),
}

/// Marco de activación de un método en la pila de llamadas de la JVM
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub locals: LocalVariables,
    pub stack: OperandStack,
    pub pc: usize,
    pub bytecode: Vec<u8>,
    pub method_name: String,
}

impl StackFrame {
    pub fn new(max_stack: usize, max_locals: usize, bytecode: Vec<u8>, method_name: &str) -> Self {
        Self {
            locals: LocalVariables::new(max_locals),
            stack: OperandStack::new(max_stack),
            pc: 0,
            bytecode,
            method_name: method_name.to_string(),
        }
    }

    /// Ejecuta una sola instrucción de bytecode (un paso).
    pub fn step(&mut self) -> Result<ExecutionResult, VmError> {
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

            // 0x10: bipush (byte inmediato con signo)
            0x10 => {
                let b = self.fetch_i8()? as i32;
                self.stack.push(Value::Int(b))?;
                Ok(ExecutionResult::Continue)
            }

            // 0x11: sipush (short inmediato con signo)
            0x11 => {
                let s = self.fetch_i16()? as i32;
                self.stack.push(Value::Int(s))?;
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

            // 0x57: pop
            0x57 => {
                self.stack.pop()?;
                Ok(ExecutionResult::Continue)
            }

            // 0x59: dup
            0x59 => {
                self.stack.dup()?;
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

            // 0x84: iinc (incremento en variable local)
            0x84 => {
                let idx = self.fetch_u8()? as usize;
                let c = self.fetch_i8()? as i32;
                let cur = self.locals.get_int(idx)?;
                self.locals.set_int(idx, cur.wrapping_add(c))?;
                Ok(ExecutionResult::Continue)
            }

            // Saltos condicionales y branching
            // 0x99..0x9E: ifeq, ifne, iflt, ifge, ifgt, ifle
            0x99 => { self.branch_if(|v| v == 0) }
            0x9A => { self.branch_if(|v| v != 0) }
            0x9B => { self.branch_if(|v| v < 0) }
            0x9C => { self.branch_if(|v| v >= 0) }
            0x9D => { self.branch_if(|v| v > 0) }
            0x9E => { self.branch_if(|v| v <= 0) }

            // 0x9F..0xA4: if_icmpeq, if_icmpne, if_icmplt, if_icmpge, if_icmpgt, if_icmple
            0x9F => { self.branch_if_icmp(|a, b| a == b) }
            0xA0 => { self.branch_if_icmp(|a, b| a != b) }
            0xA1 => { self.branch_if_icmp(|a, b| a < b) }
            0xA2 => { self.branch_if_icmp(|a, b| a >= b) }
            0xA3 => { self.branch_if_icmp(|a, b| a > b) }
            0xA4 => { self.branch_if_icmp(|a, b| a <= b) }

            // 0xA7: goto (salto incondicional de 16 bits relativo al opcode)
            0xA7 => {
                let opcode_pos = self.pc - 1;
                let offset = self.fetch_i16()? as isize;
                let new_pc = (opcode_pos as isize + offset) as usize;
                self.pc = new_pc;
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

            other => Err(VmError::InvalidOpcode(other)),
        }
    }

    /// Ejecuta el marco de forma continua hasta un retorno o error
    pub fn run_to_completion(&mut self, max_instructions: usize) -> Result<ExecutionResult, VmError> {
        let mut count = 0;
        loop {
            if count >= max_instructions {
                return Ok(ExecutionResult::Continue);
            }
            match self.step()? {
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

    fn fetch_i16(&mut self) -> Result<i16, VmError> {
        if self.pc + 2 > self.bytecode.len() {
            return Err(VmError::PcOutOfBounds {
                pc: self.pc,
                code_len: self.bytecode.len(),
            });
        }
        let val = i16::from_be_bytes([self.bytecode[self.pc], self.bytecode[self.pc + 1]]);
        self.pc += 2;
        Ok(val)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_push_pop_overflow() {
        let mut stack = OperandStack::new(2);
        assert!(stack.push(Value::Int(10)).is_ok());
        assert!(stack.push(Value::Int(20)).is_ok());
        assert_eq!(stack.push(Value::Int(30)), Err(VmError::StackOverflow));
        assert_eq!(stack.pop_int(), Ok(20));
        assert_eq!(stack.pop_int(), Ok(10));
        assert_eq!(stack.pop(), Err(VmError::StackUnderflow));
    }

    #[test]
    fn test_iadd_execution() {
        // iconst_5 (0x08), bipush 10 (0x10, 0x0A), iadd (0x60), ireturn (0xAC)
        let code = vec![0x08, 0x10, 0x0A, 0x60, 0xAC];
        let mut frame = StackFrame::new(4, 2, code, "testMethod");
        let result = frame.run_to_completion(100).expect("Ejecución correcta");
        assert_eq!(result, ExecutionResult::ReturnValue(Value::Int(15)));
    }

    #[test]
    fn test_loop_with_goto_and_branch() {
        // int x = 0; while (x < 3) { x++; } return x;
        // 0: iconst_0 (0x03)
        // 1: istore_0 (0x3B)
        // 2: iload_0  (0x1A)
        // 3: iconst_3 (0x06)
        // 4: if_icmpge +8 -> offset = 8 -> destino: pc = 4 + 8 = 12
        //    (0xA2, 0x00, 0x08)
        // 7: iinc 0, 1 (0x84, 0x00, 0x01)
        // 10: goto -8 -> offset = -8 -> destino: pc = 10 - 8 = 2
        //    (0xA7, 0xFF, 0xF8)
        // 13: iload_0 (0x1A)
        // 14: ireturn (0xAC)
        let code = vec![
            0x03,       // 0: iconst_0
            0x3B,       // 1: istore_0
            0x1A,       // 2: iload_0
            0x06,       // 3: iconst_3
            0xA2, 0x00, 0x09, // 4: if_icmpge a pc=13 (offset = 9 desde opcode pos 4)
            0x84, 0x00, 0x01, // 7: iinc 0, 1
            0xA7, 0xFF, 0xF8, // 10: goto pc=2 (offset = -8 desde pos 10)
            0x1A,       // 13: iload_0
            0xAC,       // 14: ireturn
        ];
        let mut frame = StackFrame::new(4, 2, code, "testLoop");
        let result = frame.run_to_completion(200).expect("Bucle ejecutado");
        assert_eq!(result, ExecutionResult::ReturnValue(Value::Int(3)));
    }
}
