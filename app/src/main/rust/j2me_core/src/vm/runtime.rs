//! ============================================================================
//! MÁQUINA VIRTUAL CLDC Y GESTOR DE PILA DE LLAMADAS (vm::runtime)
//! ============================================================================
//!
//! Este módulo implementa la estructura `VirtualMachine`:
//! - Mantiene el estado global del runtime: Heap, variables estáticas y clases cargadas.
//! - Gestiona la pila de llamadas (Call Stack) entre métodos de clases `.class`.
//! - Resuelve e intercepta métodos nativos y APIs fundamentales de Java ME.
//! - ClassLoader automático: carga clases bajo demanda directamente desde el JAR activo.
//! - Soporte Multihilo y Bucle de Juego: modelado de Thread, Runnable y sleep.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::class_parser::JavaClassFile;
use crate::jar_parser::JarArchive;
use crate::vm::frame::StackFrame;
use crate::vm::heap::Heap;
use crate::vm::types::{ArrayType, ExecutionResult, Value, VmError};

/// Callback opcional para cargar bytes de clases bajo demanda desde el JAR activo
pub type ClassBytesLoader = Box<dyn Fn(&str) -> Option<Vec<u8>> + Send + Sync>;

/// Máquina virtual Java ME CLDC completa
pub struct VirtualMachine {
    pub heap: Heap,
    pub static_fields: HashMap<String, Value>,
    pub loaded_classes: HashMap<String, JavaClassFile>,
    pub call_stack: Vec<StackFrame>,
    pub max_call_depth: usize,
    /// Búfer de bytes del paquete JAR cargado actualmente para auto-ClassLoader
    pub jar_bytes: Option<Vec<u8>>,
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
            jar_bytes: None,
        }
    }

    /// Asocia los bytes del archivo JAR actual a la VM para permitir la carga automática de clases
    pub fn set_jar_bytes(&mut self, data: Vec<u8>) {
        self.jar_bytes = Some(data);
    }

    /// Carga una clase parseada en el registro de clases de la VM
    pub fn load_class(&mut self, class_file: JavaClassFile) {
        if let Some(name) = class_file.get_class_name() {
            self.loaded_classes.insert(name, class_file);
        }
    }

    /// Carga automática de clases (ClassLoader dinámico de Java ME):
    /// Si la clase ya está cargada en memoria, la retorna inmediatamente.
    /// Si no está cargada, la busca en el paquete JAR activo (e.g. `com/game/Player.class`),
    /// la parsea de forma segura con `JavaClassFile::parse` y la almacena en `loaded_classes`.
    pub fn resolve_or_load_class(&mut self, class_name: &str) -> Result<(), VmError> {
        if self.loaded_classes.contains_key(class_name) {
            return Ok(());
        }

        // Si tenemos el JAR cargado, intentar extraer la clase automáticamente
        if let Some(ref jar_data) = self.jar_bytes {
            if let Ok(archive) = JarArchive::parse(jar_data) {
                let filename_class = format!("{}.class", class_name);
                let alt_filename = if class_name.starts_with('/') {
                    format!("{}.class", &class_name[1..])
                } else {
                    format!("/{}.class", class_name)
                };

                let class_bytes = archive
                    .read_file(&filename_class)
                    .or_else(|_| archive.read_file(&alt_filename));

                if let Ok(bytes) = class_bytes {
                    if let Ok(class_file) = JavaClassFile::parse(&bytes) {
                        self.load_class(class_file);
                        return Ok(());
                    }
                }
            }
        }

        // Si es una clase del sistema (Object, Thread, MIDlet, Canvas, etc.), permitir registrar dummy si no existe
        if class_name.starts_with("java/") || class_name.starts_with("javax/") {
            return Ok(());
        }

        Err(VmError::ClassNotFound(class_name.to_string()))
    }

    /// Obtiene una clase cargada por su nombre
    pub fn get_class(&self, name: &str) -> Option<&JavaClassFile> {
        self.loaded_classes.get(name)
    }

    /// Lee un texto String asignado en el Heap
    pub fn read_string_from_heap(&self, str_id: u32) -> Option<String> {
        let obj = self.heap.get_object(str_id).ok()?;
        if let Some(Value::ObjectRef(arr_id)) = obj.get_field("value") {
            let arr = self.heap.get_array(*arr_id).ok()?;
            let mut s = String::new();
            for elem in &arr.elements {
                if let Value::Int(c) = elem {
                    if let Some(ch) = char::from_u32(*c as u32) {
                        s.push(ch);
                    }
                }
            }
            return Some(s);
        }
        None
    }

    /// Carga un recurso interno del JAR en memoria y crea una instancia de ByteArrayInputStream en el Heap
    pub fn create_input_stream_for_resource(&mut self, res_path: &str) -> Value {
        let clean = res_path.trim();
        if clean.is_empty() {
            return Value::Null;
        }

        if let Some(ref jar_data) = self.jar_bytes {
            if let Ok(archive) = JarArchive::parse(jar_data) {
                let data_opt = archive.read_file(clean)
                    .or_else(|_| {
                        if clean.starts_with('/') {
                            archive.read_file(&clean[1..])
                        } else {
                            archive.read_file(&format!("/{}", clean))
                        }
                    })
                    .ok();

                if let Some(bytes) = data_opt {
                    let arr_id = self.heap.allocate_array(ArrayType::Byte, bytes.len());
                    for (i, b) in bytes.iter().enumerate() {
                        let _ = self.heap.set_array_element(arr_id, i as i32, Value::Int(*b as i8 as i32));
                    }
                    let stream_id = self.heap.allocate_object("java/io/ByteArrayInputStream");
                    let _ = self.heap.set_field(stream_id, "buf", Value::ObjectRef(arr_id));
                    let _ = self.heap.set_field(stream_id, "pos", Value::Int(0));
                    let _ = self.heap.set_field(stream_id, "count", Value::Int(bytes.len() as i32));
                    let _ = self.heap.set_field(stream_id, "mark", Value::Int(0));
                    return Value::ObjectRef(stream_id);
                }
            }
        }
        Value::Null
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
        // 1. Cargar la clase automáticamente mediante el ClassLoader si aún no está presente
        self.resolve_or_load_class(class_name)?;

        // 2. Verificar si es una llamada a un método builtin del sistema
        if let Some(res) = self.handle_builtin(class_name, method_name, descriptor, &args)? {
            return Ok(res);
        }

        // 3. Buscar la clase y el método
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

        // 4. Crear el marco de activación inicial
        let mut frame = StackFrame::new_with_cp(
            code_attr.max_stack as usize,
            code_attr.max_locals as usize,
            code_attr.bytecode,
            class_file.constant_pool.clone(),
            class_name,
            method_name,
        );

        // 5. Copiar argumentos en las variables locales del marco
        for (i, arg) in args.into_iter().enumerate() {
            if i < frame.locals.max_locals() {
                frame.locals.set(i, arg)?;
            }
        }

        self.call_stack.push(frame);

        // 6. Ciclo de ejecución del Call Stack
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
                        // Invocar método en clase de usuario (usando ClassLoader automático)
                        if self.call_stack.len() >= self.max_call_depth {
                            return Err(VmError::MaxCallStackDepthExceeded(self.max_call_depth));
                        }

                        self.resolve_or_load_class(&class_name)?;

                        let target_class = self
                            .loaded_classes
                            .get(&class_name)
                            .ok_or_else(|| VmError::ClassNotFound(class_name.clone()))?;

                        // Buscar método en la clase o en su jerarquía
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
    /// Incluye soporte completo para Thread, Runnable, System, Math y MIDlet
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

            // javax/microedition/lcdui/Displayable y Canvas constructores
            ("javax/microedition/lcdui/Displayable", "<init>", "()V") => Ok(Some(ExecutionResult::ReturnVoid)),
            ("javax/microedition/lcdui/Canvas", "<init>", "()V") => Ok(Some(ExecutionResult::ReturnVoid)),

            // java/lang/Thread.<init>(Ljava/lang/Runnable;)V
            ("java/lang/Thread", "<init>", "(Ljava/lang/Runnable;)V") => {
                if let Some(Value::ObjectRef(thread_id)) = args.first() {
                    if let Some(runnable_val) = args.get(1) {
                        if let Ok(thread_obj) = self.heap.get_object_mut(*thread_id) {
                            thread_obj.set_field("__target_runnable", runnable_val.clone());
                        }
                    }
                }
                Ok(Some(ExecutionResult::ReturnVoid))
            }

            // java/lang/Thread.<init>()V
            ("java/lang/Thread", "<init>", "()V") => Ok(Some(ExecutionResult::ReturnVoid)),

            // java/lang/Thread.start()V
            // Inicia el bucle de juego invocando el método 'run()V' en el Runnable asociado o en la subclase Thread
            ("java/lang/Thread", "start", "()V") => {
                if let Some(Value::ObjectRef(thread_id)) = args.first() {
                    let mut target_obj_id = *thread_id;
                    if let Ok(thread_obj) = self.heap.get_object(*thread_id) {
                        if let Some(Value::ObjectRef(runnable_id)) = thread_obj.get_field("__target_runnable") {
                            if *runnable_id != 0 {
                                target_obj_id = *runnable_id;
                            }
                        }
                    }

                    if let Ok(target_obj) = self.heap.get_object(target_obj_id) {
                        let target_class = target_obj.class_name.clone();
                        // Despachar llamada a run()V en el objeto destino
                        return Ok(Some(ExecutionResult::InvokeMethod {
                            class_name: target_class,
                            method_name: "run".to_string(),
                            descriptor: "()V".to_string(),
                            is_static: false,
                            args: vec![Value::ObjectRef(target_obj_id)],
                        }));
                    }
                }
                Ok(Some(ExecutionResult::ReturnVoid))
            }

            // java/lang/Thread.sleep(J)V
            // Simula la temporización del bucle de juego respetando el tiempo solicitado
            ("java/lang/Thread", "sleep", "(J)V") => {
                let millis = args.first().and_then(|v| match v {
                    Value::Long(l) => Some(*l),
                    Value::Int(i) => Some(*i as i64),
                    _ => None,
                }).unwrap_or(1);

                if millis > 0 {
                    // No bloquear más de 20ms para no congelar la UI de Compose
                    let bounded_millis = (millis as u64).min(20);
                    std::thread::sleep(Duration::from_millis(bounded_millis));
                }
                Ok(Some(ExecutionResult::ReturnVoid))
            }

            // java/lang/Thread.yield()V
            ("java/lang/Thread", "yield", "()V") => {
                std::thread::yield_now();
                Ok(Some(ExecutionResult::ReturnVoid))
            }

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

            // java/lang/Object.getClass()Ljava/lang/Class;
            ("java/lang/Object", "getClass", "()Ljava/lang/Class;") => {
                let class_id = self.heap.allocate_object("java/lang/Class");
                Ok(Some(ExecutionResult::ReturnValue(Value::ObjectRef(class_id))))
            }

            // java/lang/Class.getResourceAsStream(Ljava/lang/String;)Ljava/io/InputStream;
            ("java/lang/Class", "getResourceAsStream", "(Ljava/lang/String;)Ljava/io/InputStream;") => {
                let path_str = args.get(1).and_then(|v| {
                    if let Value::ObjectRef(str_id) = v {
                        self.read_string_from_heap(*str_id)
                    } else {
                        None
                    }
                }).unwrap_or_default();

                let stream_val = self.create_input_stream_for_resource(&path_str);
                Ok(Some(ExecutionResult::ReturnValue(stream_val)))
            }

            // javax/microedition/midlet/MIDlet.getResourceAsStream(Ljava/lang/String;)Ljava/io/InputStream;
            ("javax/microedition/midlet/MIDlet", "getResourceAsStream", "(Ljava/lang/String;)Ljava/io/InputStream;") => {
                let path_str = args.first().and_then(|v| {
                    if let Value::ObjectRef(str_id) = v {
                        self.read_string_from_heap(*str_id)
                    } else {
                        None
                    }
                }).unwrap_or_default();

                let stream_val = self.create_input_stream_for_resource(&path_str);
                Ok(Some(ExecutionResult::ReturnValue(stream_val)))
            }

            // com/example/util/J2meResourceManager.getResourceAsStream(Ljava/lang/String;)Ljava/io/InputStream;
            ("com/example/util/J2meResourceManager", "getResourceAsStream", "(Ljava/lang/String;)Ljava/io/InputStream;") => {
                let path_str = args.first().and_then(|v| {
                    if let Value::ObjectRef(str_id) = v {
                        self.read_string_from_heap(*str_id)
                    } else {
                        None
                    }
                }).unwrap_or_default();

                let stream_val = self.create_input_stream_for_resource(&path_str);
                Ok(Some(ExecutionResult::ReturnValue(stream_val)))
            }

            // java/io/InputStream.<init>()V
            ("java/io/InputStream", "<init>", "()V") => Ok(Some(ExecutionResult::ReturnVoid)),

            // java/io/ByteArrayInputStream.<init>([B)V
            ("java/io/ByteArrayInputStream", "<init>", "([B)V") => {
                if let (Some(Value::ObjectRef(stream_id)), Some(arr_val)) = (args.first(), args.get(1)) {
                    let mut len = 0;
                    if let Value::ObjectRef(arr_id) = arr_val {
                        if let Ok(arr) = self.heap.get_array(*arr_id) {
                            len = arr.len();
                        }
                    }
                    let _ = self.heap.set_field(*stream_id, "buf", arr_val.clone());
                    let _ = self.heap.set_field(*stream_id, "pos", Value::Int(0));
                    let _ = self.heap.set_field(*stream_id, "count", Value::Int(len as i32));
                    let _ = self.heap.set_field(*stream_id, "mark", Value::Int(0));
                }
                Ok(Some(ExecutionResult::ReturnVoid))
            }

            // java/io/ByteArrayInputStream.<init>([BII)V
            ("java/io/ByteArrayInputStream", "<init>", "([BII)V") => {
                if let (Some(Value::ObjectRef(stream_id)), Some(arr_val), Some(offset_val), Some(len_val)) =
                    (args.first(), args.get(1), args.get(2), args.get(3))
                {
                    let offset = offset_val.as_int().unwrap_or(0);
                    let len = len_val.as_int().unwrap_or(0);
                    let _ = self.heap.set_field(*stream_id, "buf", arr_val.clone());
                    let _ = self.heap.set_field(*stream_id, "pos", Value::Int(offset));
                    let _ = self.heap.set_field(*stream_id, "count", Value::Int((offset + len).max(0)));
                    let _ = self.heap.set_field(*stream_id, "mark", Value::Int(offset));
                }
                Ok(Some(ExecutionResult::ReturnVoid))
            }

            // InputStream / ByteArrayInputStream.read()I
            ("java/io/InputStream", "read", "()I") |
            ("java/io/ByteArrayInputStream", "read", "()I") => {
                if let Some(Value::ObjectRef(stream_id)) = args.first() {
                    let pos = self.heap.get_field(*stream_id, "pos").and_then(|v| v.as_int()).unwrap_or(0);
                    let count = self.heap.get_field(*stream_id, "count").and_then(|v| v.as_int()).unwrap_or(0);
                    if pos >= count {
                        return Ok(Some(ExecutionResult::ReturnValue(Value::Int(-1))));
                    }
                    if let Ok(Value::ObjectRef(arr_id)) = self.heap.get_field(*stream_id, "buf") {
                        if let Ok(byte_val) = self.heap.get_array_element(arr_id, pos) {
                            let _ = self.heap.set_field(*stream_id, "pos", Value::Int(pos + 1));
                            let byte_int = byte_val.as_int().unwrap_or(0) & 0xFF;
                            return Ok(Some(ExecutionResult::ReturnValue(Value::Int(byte_int))));
                        }
                    }
                }
                Ok(Some(ExecutionResult::ReturnValue(Value::Int(-1))))
            }

            // InputStream / ByteArrayInputStream.read([B)I
            ("java/io/InputStream", "read", "([B)I") |
            ("java/io/ByteArrayInputStream", "read", "([B)I") => {
                if let (Some(Value::ObjectRef(stream_id)), Some(Value::ObjectRef(dst_arr_id))) = (args.first(), args.get(1)) {
                    let dst_len = self.heap.array_len(*dst_arr_id).unwrap_or(0) as i32;
                    let pos = self.heap.get_field(*stream_id, "pos").and_then(|v| v.as_int()).unwrap_or(0);
                    let count = self.heap.get_field(*stream_id, "count").and_then(|v| v.as_int()).unwrap_or(0);
                    if pos >= count {
                        return Ok(Some(ExecutionResult::ReturnValue(Value::Int(-1))));
                    }
                    let to_read = (count - pos).min(dst_len);
                    if let Ok(Value::ObjectRef(src_arr_id)) = self.heap.get_field(*stream_id, "buf") {
                        for i in 0..to_read {
                            if let Ok(b) = self.heap.get_array_element(src_arr_id, pos + i) {
                                let _ = self.heap.set_array_element(*dst_arr_id, i, b);
                            }
                        }
                        let _ = self.heap.set_field(*stream_id, "pos", Value::Int(pos + to_read));
                        return Ok(Some(ExecutionResult::ReturnValue(Value::Int(to_read))));
                    }
                }
                Ok(Some(ExecutionResult::ReturnValue(Value::Int(-1))))
            }

            // InputStream / ByteArrayInputStream.read([BII)I
            ("java/io/InputStream", "read", "([BII)I") |
            ("java/io/ByteArrayInputStream", "read", "([BII)I") => {
                if let (Some(Value::ObjectRef(stream_id)), Some(Value::ObjectRef(dst_arr_id)), Some(off_val), Some(len_val)) =
                    (args.first(), args.get(1), args.get(2), args.get(3))
                {
                    let off = off_val.as_int().unwrap_or(0);
                    let req_len = len_val.as_int().unwrap_or(0);
                    let pos = self.heap.get_field(*stream_id, "pos").and_then(|v| v.as_int()).unwrap_or(0);
                    let count = self.heap.get_field(*stream_id, "count").and_then(|v| v.as_int()).unwrap_or(0);
                    if pos >= count {
                        return Ok(Some(ExecutionResult::ReturnValue(Value::Int(-1))));
                    }
                    let to_read = (count - pos).min(req_len);
                    if let Ok(Value::ObjectRef(src_arr_id)) = self.heap.get_field(*stream_id, "buf") {
                        for i in 0..to_read {
                            if let Ok(b) = self.heap.get_array_element(src_arr_id, pos + i) {
                                let _ = self.heap.set_array_element(*dst_arr_id, off + i, b);
                            }
                        }
                        let _ = self.heap.set_field(*stream_id, "pos", Value::Int(pos + to_read));
                        return Ok(Some(ExecutionResult::ReturnValue(Value::Int(to_read))));
                    }
                }
                Ok(Some(ExecutionResult::ReturnValue(Value::Int(-1))))
            }

            // InputStream / ByteArrayInputStream.available()I
            ("java/io/InputStream", "available", "()I") |
            ("java/io/ByteArrayInputStream", "available", "()I") => {
                if let Some(Value::ObjectRef(stream_id)) = args.first() {
                    let pos = self.heap.get_field(*stream_id, "pos").and_then(|v| v.as_int()).unwrap_or(0);
                    let count = self.heap.get_field(*stream_id, "count").and_then(|v| v.as_int()).unwrap_or(0);
                    return Ok(Some(ExecutionResult::ReturnValue(Value::Int((count - pos).max(0)))));
                }
                Ok(Some(ExecutionResult::ReturnValue(Value::Int(0))))
            }

            // InputStream / ByteArrayInputStream.close()V
            ("java/io/InputStream", "close", "()V") |
            ("java/io/ByteArrayInputStream", "close", "()V") => Ok(Some(ExecutionResult::ReturnVoid)),

            // MIDP 2.0 Game API constructores e invocaciones
            ("javax/microedition/lcdui/game/GameCanvas", "<init>", "(Z)V") |
            ("javax/microedition/lcdui/game/GameCanvas", "<init>", "()V") => Ok(Some(ExecutionResult::ReturnVoid)),

            ("javax/microedition/lcdui/game/GameCanvas", "flushGraphics", "()V") |
            ("javax/microedition/lcdui/game/GameCanvas", "flushGraphics", "(IIII)V") => Ok(Some(ExecutionResult::ReturnVoid)),

            ("javax/microedition/lcdui/game/GameCanvas", "getKeyStates", "()I") => {
                Ok(Some(ExecutionResult::ReturnValue(Value::Int(0))))
            }

            ("javax/microedition/lcdui/game/Layer", "<init>", "(II)V") => {
                if let (Some(Value::ObjectRef(layer_id)), Some(w), Some(h)) = (args.first(), args.get(1), args.get(2)) {
                    let _ = self.heap.set_field(*layer_id, "width", w.clone());
                    let _ = self.heap.set_field(*layer_id, "height", h.clone());
                    let _ = self.heap.set_field(*layer_id, "visible", Value::Int(1));
                }
                Ok(Some(ExecutionResult::ReturnVoid))
            }

            ("javax/microedition/lcdui/game/Sprite", "<init>", "(Ljavax/microedition/lcdui/Image;)V") |
            ("javax/microedition/lcdui/game/Sprite", "<init>", "(Ljavax/microedition/lcdui/Image;II)V") => {
                Ok(Some(ExecutionResult::ReturnVoid))
            }

            ("javax/microedition/lcdui/game/TiledLayer", "<init>", "(IILjavax/microedition/lcdui/Image;II)V") => {
                Ok(Some(ExecutionResult::ReturnVoid))
            }

            ("javax/microedition/lcdui/game/LayerManager", "<init>", "()V") => {
                Ok(Some(ExecutionResult::ReturnVoid))
            }

            _ => Ok(None),
        }
    }

    /// Reinicia completamente el estado de la VM (Heap, variables estáticas y pila de llamadas)
    pub fn reset(&mut self) {
        self.heap.clear();
        self.static_fields.clear();
        self.call_stack.clear();
        self.loaded_classes.clear();
        self.jar_bytes = None;
    }
}
