//! ============================================================================
//! MÁQUINA VIRTUAL CLDC Y GESTOR DE PILA DE LLAMADAS (vm::runtime)
//! ============================================================================
//!
//! Este módulo implementa la estructura `VirtualMachine`:
//! - Mantiene el estado global del runtime: Heap, variables estáticas y clases cargadas.
//! - Gestiona la pila de llamadas (Call Stack) entre métodos de clases `.class`.
//! - Resuelve e intercepta métodos nativos y APIs fundamentales de Java ME.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::class_parser::JavaClassFile;
use crate::vm::frame::StackFrame;
use crate::vm::heap::Heap;
use crate::vm::types::{ExecutionResult, Value, VmError};

/// Máquina virtual Java ME CLDC completa
#[derive(Debug, Clone)]
pub struct VirtualMachine {
    pub heap: Heap,
    pub static_fields: HashMap<String, Value>,
    pub loaded_classes: HashMap<String, JavaClassFile>,
    pub call_stack: Vec<StackFrame>,
    pub max_call_depth: usize,
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualMachine {
    /// Inicializa una nueva instancia de la máquina virtual CLDC
    pub fn new() -> Self {
        Self {
            heap: Heap::new(),
            static_fields: HashMap::new(),
            loaded_classes: HashMap::new(),
            call_stack: Vec::with_capacity(64),
            max_call_depth: 512,
        }
    }

    /// Carga una clase parseada en el registro de clases de la VM
    pub fn load_class(&mut self, class_file: JavaClassFile) {
        if let Some(name) = class_file.get_class_name() {
            self.loaded_classes.insert(name, class_file);
        }
    }

    /// Obtiene una clase cargada por su nombre
    pub fn get_class(&self, name: &str) -> Option<&JavaClassFile> {
        self.loaded_classes.get(name)
    }

    /// Ejecuta un método específico de una clase dada pasando los argumentos requeridos
    pub fn execute_method(
        &mut self,
        class_name: &str,
        method_name: &str,
        descriptor: &str,
        args: Vec<Value>,
        max_instructions: usize,
    ) -> Result<ExecutionResult, VmError> {
        // 1. Verificar si es una llamada a un método builtin del sistema
        if let Some(res) = self.handle_builtin(class_name, method_name, descriptor, &args)? {
            return Ok(res);
        }

        // 2. Buscar la clase y el método
        let class_file = self
            .loaded_classes
            .get(class_name)
            .ok_or_else(|| VmError::ClassNotFound(class_name.to_string()))?;

        let method = class_file
            .find_method(method_name, descriptor)
            .ok_or_else(|| VmError::MethodNotFound {
                class: class_name.to_string(),
                method: method_name.to_string(),
                descriptor: descriptor.to_string(),
            })?;

        let code_attr = method
            .parse_code_attribute(&class_file.constant_pool)
            .ok_or_else(|| VmError::MethodNotFound {
                class: class_name.to_string(),
                method: method_name.to_string(),
                descriptor: descriptor.to_string(),
            })?;

        // 3. Crear el marco de activación inicial
        let mut frame = StackFrame::new_with_cp(
            code_attr.max_stack as usize,
            code_attr.max_locals as usize,
            code_attr.bytecode,
            class_file.constant_pool.clone(),
            class_name,
            method_name,
        );

        // 4. Copiar argumentos en las variables locales del marco
        for (i, arg) in args.into_iter().enumerate() {
            if i < frame.locals.max_locals() {
                frame.locals.set(i, arg)?;
            }
        }

        self.call_stack.push(frame);

        // 5. Ciclo de ejecución del Call Stack
        let mut instructions_executed = 0;
        while !self.call_stack.is_empty() {
            if instructions_executed >= max_instructions {
                return Ok(ExecutionResult::Continue);
            }

            let step_res = {
                let current_frame = self.call_stack.last_mut().unwrap();
                current_frame.step(&mut self.heap, &mut self.static_fields)?
            };
            instructions_executed += 1;

            match step_res {
                ExecutionResult::Continue => {}
                ExecutionResult::InvokeMethod {
                    class_name,
                    method_name,
                    descriptor,
                    is_static: _,
                    args,
                } => {
                    // Interceptar built-in primero
                    if let Some(res) = self.handle_builtin(&class_name, &method_name, &descriptor, &args)? {
                        let caller = self.call_stack.last_mut().unwrap();
                        match res {
                            ExecutionResult::ReturnValue(val) => {
                                caller.stack.push(val)?;
                            }
                            ExecutionResult::ReturnVoid => {}
                            _ => {}
                        }
                    } else {
                        // Invocar método en clase de usuario
                        if self.call_stack.len() >= self.max_call_depth {
                            return Err(VmError::MaxCallStackDepthExceeded(self.max_call_depth));
                        }

                        let target_class = self
                            .loaded_classes
                            .get(&class_name)
                            .ok_or_else(|| VmError::ClassNotFound(class_name.clone()))?;

                        let target_method = target_class
                            .find_method(&method_name, &descriptor)
                            .ok_or_else(|| VmError::MethodNotFound {
                                class: class_name.clone(),
                                method: method_name.clone(),
                                descriptor: descriptor.clone(),
                            })?;

                        let target_code = target_method
                            .parse_code_attribute(&target_class.constant_pool)
                            .ok_or_else(|| VmError::MethodNotFound {
                                class: class_name.clone(),
                                method: method_name.clone(),
                                descriptor: descriptor.clone(),
                            })?;

                        let mut new_frame = StackFrame::new_with_cp(
                            target_code.max_stack as usize,
                            target_code.max_locals as usize,
                            target_code.bytecode,
                            target_class.constant_pool.clone(),
                            &class_name,
                            &method_name,
                        );

                        for (i, arg) in args.into_iter().enumerate() {
                            if i < new_frame.locals.max_locals() {
                                new_frame.locals.set(i, arg)?;
                            }
                        }

                        self.call_stack.push(new_frame);
                    }
                }
                ExecutionResult::ReturnValue(val) => {
                    self.call_stack.pop();
                    if self.call_stack.is_empty() {
                        return Ok(ExecutionResult::ReturnValue(val));
                    } else {
                        let caller = self.call_stack.last_mut().unwrap();
                        caller.stack.push(val)?;
                    }
                }
                ExecutionResult::ReturnVoid => {
                    self.call_stack.pop();
                    if self.call_stack.is_empty() {
                        return Ok(ExecutionResult::ReturnVoid);
                    }
                }
            }
        }

        Ok(ExecutionResult::ReturnVoid)
    }

    /// Maneja métodos estándar de la biblioteca base de Java ME (built-ins)
    fn handle_builtin(
        &mut self,
        class_name: &str,
        method_name: &str,
        descriptor: &str,
        args: &[Value],
    ) -> Result<Option<ExecutionResult>, VmError> {
        match (class_name, method_name, descriptor) {
            // java/lang/Object.<init>()V
            ("java/lang/Object", "<init>", "()V") => Ok(Some(ExecutionResult::ReturnVoid)),

            // javax/microedition/midlet/MIDlet.<init>()V
            ("javax/microedition/midlet/MIDlet", "<init>", "()V") => Ok(Some(ExecutionResult::ReturnVoid)),

            // java/lang/System.currentTimeMillis()J
            ("java/lang/System", "currentTimeMillis", "()J") => {
                let millis = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);
                Ok(Some(ExecutionResult::ReturnValue(Value::Long(millis))))
            }

            // java/lang/System.gc()V
            ("java/lang/System", "gc", "()V") => Ok(Some(ExecutionResult::ReturnVoid)),

            // java/lang/Math.abs(I)I
            ("java/lang/Math", "abs", "(I)I") => {
                let val = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                Ok(Some(ExecutionResult::ReturnValue(Value::Int(val.abs()))))
            }

            // java/lang/Math.max(II)I
            ("java/lang/Math", "max", "(II)I") => {
                let a = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                let b = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                Ok(Some(ExecutionResult::ReturnValue(Value::Int(a.max(b)))))
            }

            // java/lang/Math.min(II)I
            ("java/lang/Math", "min", "(II)I") => {
                let a = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                let b = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                Ok(Some(ExecutionResult::ReturnValue(Value::Int(a.min(b)))))
            }

            // java/lang/Thread.sleep(J)V
            ("java/lang/Thread", "sleep", "(J)V") => Ok(Some(ExecutionResult::ReturnVoid)),

            _ => Ok(None),
        }
    }

    /// Reinicia completamente el estado de la VM (Heap, variables estáticas y pila de llamadas)
    pub fn reset(&mut self) {
        self.heap.clear();
        self.static_fields.clear();
        self.call_stack.clear();
    }
}
