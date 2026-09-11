//! ============================================================================
//! NÚCLEO DEL EMULADOR J2ME EN RUST (j2me_core)
//! ============================================================================
//! 
//! Este módulo constituye el núcleo de ejecución de la máquina virtual (JVM/CLDC).
//! 
//! Responsabilidades clave de este módulo en Rust:
//! 1. Intérprete de Bytecode y Pila de Ejecución: Despacho seguro de opcodes de Java ME.
//! 2. Carga y Verificación de Clases (.class): Parseo del Constant Pool, firmas de métodos y campos.
//! 3. Seguridad de Memoria: Control estricto de accesos sin peligro de desbordamiento de búfer
//!    ni NullPointerExceptions a nivel nativo.
//! 4. Soporte Multi-Arquitectura: Compilado tanto para 32 bits (ARMv7, x86) como para
//!    64 bits (ARM64-v8a, x86_64).
//! 5. Enlace FFI con C++: Exporta funciones mediante `extern "C"` con convención C estándar
//!    para ser consumidas directamente por la capa C++ y el puente JNI de Android.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

pub mod jar_parser;
use jar_parser::JarArchive;

pub mod class_parser;
use class_parser::JavaClassFile;

pub mod vm;
use vm::{ExecutionResult, StackFrame, Value, VirtualMachine};

/// Estado global del motor de emulación en Rust
static CORE_INITIALIZED: AtomicBool = AtomicBool::new(false);
static INSTRUCTION_CYCLE_COUNT: AtomicU64 = AtomicU64::new(0);

/// Búfer en memoria del último JAR cargado
static CURRENT_JAR_DATA: Mutex<Option<Vec<u8>>> = Mutex::new(None);

/// Instancia global de la Máquina Virtual CLDC en Rust
static CURRENT_VM: Mutex<Option<VirtualMachine>> = Mutex::new(None);

/// Cadena constante con la versión del núcleo Rust
const CORE_VERSION: &str = "0.1.0-alpha (Rust JVM/CLDC Core)";

/// Inicializa el núcleo del emulador J2ME en Rust.
/// 
/// Retorna:
/// - 0 si se inicializó correctamente o ya estaba activo.
/// - Menor a 0 si ocurrió algún fallo crítico de memoria.
#[no_mangle]
pub extern "C" fn j2me_core_init() -> i32 {
    CORE_INITIALIZED.store(true, Ordering::SeqCst);
    INSTRUCTION_CYCLE_COUNT.store(0, Ordering::SeqCst);
    0
}

/// Retorna la cadena de versión del motor Rust en formato C-string (puntero a char terminado en nulo).
#[no_mangle]
pub extern "C" fn j2me_core_get_version() -> *const c_char {
    static VERSION_CSTRING: std::sync::OnceLock<CString> = std::sync::OnceLock::new();
    VERSION_CSTRING
        .get_or_init(|| CString::new(CORE_VERSION).unwrap_or_default())
        .as_ptr()
}

/// Retorna la arquitectura de CPU detectada para la cual fue compilado este binario Rust.
/// Soporta arquitecturas de 32 bits y 64 bits.
#[no_mangle]
pub extern "C" fn j2me_core_get_architecture() -> *const c_char {
    let arch_name = if cfg!(target_arch = "aarch64") {
        "ARM64-v8a (64-bit)"
    } else if cfg!(target_arch = "arm") {
        "ARMv7-A (32-bit)"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64 (64-bit)"
    } else if cfg!(target_arch = "x86") {
        "x86 (32-bit)"
    } else {
        "Desconocida"
    };

    static ARCH_CSTRING: std::sync::OnceLock<CString> = std::sync::OnceLock::new();
    ARCH_CSTRING
        .get_or_init(|| CString::new(arch_name).unwrap_or_default())
        .as_ptr()
}

/// Ejecuta un ciclo de procesamiento de instrucciones en la máquina virtual.
/// Retorna el total de ciclos ejecutados hasta el momento.
#[no_mangle]
pub extern "C" fn j2me_core_execute_cycle() -> u64 {
    if !CORE_INITIALIZED.load(Ordering::SeqCst) {
        return 0;
    }
    INSTRUCTION_CYCLE_COUNT.fetch_add(1, Ordering::SeqCst) + 1
}

/// Libera los recursos del motor Rust al pausar o destruir la actividad.
#[no_mangle]
pub extern "C" fn j2me_core_cleanup() {
    CORE_INITIALIZED.store(false, Ordering::SeqCst);
    INSTRUCTION_CYCLE_COUNT.store(0, Ordering::SeqCst);
    if let Ok(mut lock) = CURRENT_JAR_DATA.lock() {
        *lock = None;
    }
    if let Ok(mut vm_lock) = CURRENT_VM.lock() {
        *vm_lock = None;
    }
}

/// Carga un paquete JAR desde un búfer en memoria proporcionado por C++/JNI.
/// Retorna:
/// - 0 si fue parseado correctamente.
/// - Menor a 0 si hubo un error de parseo o archivo inválido.
#[no_mangle]
pub unsafe extern "C" fn j2me_core_load_jar(bytes: *const u8, length: usize) -> i32 {
    if bytes.is_null() || length == 0 {
        return -1;
    }

    let slice = std::slice::from_raw_parts(bytes, length);
    match JarArchive::parse(slice) {
        Ok(_) => {
            if let Ok(mut lock) = CURRENT_JAR_DATA.lock() {
                *lock = Some(slice.to_vec());
            }
            0
        }
        Err(_) => -2,
    }
}

/// Obtiene los metadatos del manifiesto J2ME en formato JSON C-String.
/// El puntero retornado debe ser liberado llamando a `j2me_core_free_string`.
#[no_mangle]
pub extern "C" fn j2me_core_get_manifest_json() -> *mut c_char {
    let lock = match CURRENT_JAR_DATA.lock() {
        Ok(l) => l,
        Err(_) => return std::ptr::null_mut(),
    };

    let data = match lock.as_ref() {
        Some(d) => d,
        None => return std::ptr::null_mut(),
    };

    let archive = match JarArchive::parse(data) {
        Ok(a) => a,
        Err(_) => return std::ptr::null_mut(),
    };

    let manifest = match archive.parse_manifest() {
        Ok(m) => m,
        Err(_) => return std::ptr::null_mut(),
    };

    // Construcción de JSON seguro y ligero sin dependencias externas
    let escape = |s: &str| -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    };

    let json = format!(
        "{{\"midletName\":\"{}\",\"vendor\":\"{}\",\"version\":\"{}\",\"profile\":\"{}\",\"configuration\":\"{}\",\"mainClass\":\"{}\",\"iconPath\":\"{}\",\"filesCount\":{}}}",
        escape(&manifest.midlet_name),
        escape(&manifest.midlet_vendor),
        escape(&manifest.midlet_version),
        escape(&manifest.microedition_profile),
        escape(&manifest.microedition_configuration),
        escape(&manifest.main_class),
        escape(&manifest.icon_path),
        archive.list_files().len()
    );

    match CString::new(json) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Extrae un recurso o clase del JAR por su nombre.
/// Escribe el puntero al búfer y su longitud en `out_data` y `out_len`.
/// Retorna 0 en éxito, menor a 0 en fallo.
#[no_mangle]
pub unsafe extern "C" fn j2me_core_read_jar_resource(
    filename: *const c_char,
    out_data: *mut *mut u8,
    out_len: *mut usize,
) -> i32 {
    if filename.is_null() || out_data.is_null() || out_len.is_null() {
        return -1;
    }

    let c_str = CStr::from_ptr(filename);
    let name = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let lock = match CURRENT_JAR_DATA.lock() {
        Ok(l) => l,
        Err(_) => return -3,
    };

    let data = match lock.as_ref() {
        Some(d) => d,
        None => return -4,
    };

    let archive = match JarArchive::parse(data) {
        Ok(a) => a,
        Err(_) => return -5,
    };

    match archive.read_file(name) {
        Ok(content) => {
            let mut boxed_slice = content.into_boxed_slice();
            *out_len = boxed_slice.len();
            *out_data = boxed_slice.as_mut_ptr();
            std::mem::forget(boxed_slice); // Transferir posesión a la capa receptora
            0
        }
        Err(_) => -6,
    }
}

/// Libera la memoria de una cadena C asignada por Rust.
#[no_mangle]
pub unsafe extern "C" fn j2me_core_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Libera la memoria de un búfer de bytes de recurso asignado por Rust.
#[no_mangle]
pub unsafe extern "C" fn j2me_core_free_resource_bytes(ptr: *mut u8, length: usize) {
    if !ptr.is_null() && length > 0 {
        let slice = std::slice::from_raw_parts_mut(ptr, length);
        drop(Box::from_raw(slice.as_mut_ptr()));
    }
}

/// Parsea un archivo `.class` a partir de un búfer binario en memoria.
/// Retorna un resumen en formato JSON con la clase, superclase, métodos y bytecodes.
/// La cadena retornada debe liberarse llamando a `j2me_core_free_string`.
#[no_mangle]
pub unsafe extern "C" fn j2me_core_parse_class_bytes(
    bytes: *const u8,
    length: usize,
) -> *mut c_char {
    if bytes.is_null() || length == 0 {
        return std::ptr::null_mut();
    }

    let slice = std::slice::from_raw_parts(bytes, length);
    match JavaClassFile::parse(slice) {
        Ok(class_file) => {
            let json = class_file.to_summary_json();
            match CString::new(json) {
                Ok(c_str) => c_str.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Busca una clase en el archivo JAR cargado actualmente, la parsea y retorna su resumen JSON.
/// La cadena retornada debe liberarse llamando a `j2me_core_free_string`.
#[no_mangle]
pub unsafe extern "C" fn j2me_core_inspect_jar_class(class_name: *const c_char) -> *mut c_char {
    if class_name.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = CStr::from_ptr(class_name);
    let name = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    // Formatear la ruta esperada dentro del archivo JAR (ej. "com/game/Main.class")
    let normalized_path = if name.ends_with(".class") {
        name.to_string()
    } else {
        format!("{}.class", name.replace('.', "/"))
    };

    let lock = match CURRENT_JAR_DATA.lock() {
        Ok(l) => l,
        Err(_) => return std::ptr::null_mut(),
    };

    let data = match lock.as_ref() {
        Some(d) => d,
        None => return std::ptr::null_mut(),
    };

    let archive = match JarArchive::parse(data) {
        Ok(a) => a,
        Err(_) => return std::ptr::null_mut(),
    };

    match archive.read_file(&normalized_path) {
        Ok(class_bytes) => match JavaClassFile::parse(&class_bytes) {
            Ok(class_file) => {
                let json = class_file.to_summary_json();
                match CString::new(json) {
                    Ok(c_str) => c_str.into_raw(),
                    Err(_) => std::ptr::null_mut(),
                }
            }
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Ejecuta un bloque de bytecode de JVM con los límites de pila y variables dados.
/// Retorna 0 en caso de éxito y almacena el resultado en `out_result`.
/// Retorna menor a 0 si ocurrió una excepción de VM o error de límites.
#[no_mangle]
pub unsafe extern "C" fn j2me_core_execute_bytecode(
    bytecode: *const u8,
    bytecode_len: usize,
    max_stack: u16,
    max_locals: u16,
    out_result: *mut i32,
) -> i32 {
    if bytecode.is_null() || bytecode_len == 0 {
        return -1;
    }

    let slice = std::slice::from_raw_parts(bytecode, bytecode_len).to_vec();
    let mut frame = StackFrame::new(
        max_stack.max(1) as usize,
        max_locals.max(1) as usize,
        slice,
        "nativeInvokedMethod",
    );

    match frame.run_to_completion(100_000) {
        Ok(ExecutionResult::ReturnValue(Value::Int(v))) => {
            if !out_result.is_null() {
                *out_result = v;
            }
            0
        }
        Ok(ExecutionResult::ReturnVoid) => {
            if !out_result.is_null() {
                *out_result = 0;
            }
            0
        }
        Ok(ExecutionResult::Continue) => {
            // Se agotaron los ciclos máximos (posible bucle infinito)
            -2
        }
        Ok(_) => 0,
        Err(_) => -3,
    }
}

/// Reinicia o inicializa la Máquina Virtual CLDC global en Rust.
#[no_mangle]
pub extern "C" fn j2me_core_vm_reset() -> i32 {
    let mut vm_lock = match CURRENT_VM.lock() {
        Ok(l) => l,
        Err(_) => return -1,
    };
    *vm_lock = Some(VirtualMachine::new());
    0
}

/// Carga una clase en la Máquina Virtual global a partir de sus bytes binarios.
#[no_mangle]
pub unsafe extern "C" fn j2me_core_vm_load_class(bytes: *const u8, length: usize) -> i32 {
    if bytes.is_null() || length == 0 {
        return -1;
    }
    let slice = std::slice::from_raw_parts(bytes, length);
    let class_file = match JavaClassFile::parse(slice) {
        Ok(cf) => cf,
        Err(_) => return -2,
    };

    let mut vm_lock = match CURRENT_VM.lock() {
        Ok(l) => l,
        Err(_) => return -3,
    };

    if vm_lock.is_none() {
        *vm_lock = Some(VirtualMachine::new());
    }

    if let Some(ref mut vm) = *vm_lock {
        vm.load_class(class_file);
        0
    } else {
        -4
    }
}

/// Ejecuta un método de una clase en la VM global (resolviendo llamadas anidadas en el Heap).
#[no_mangle]
pub unsafe extern "C" fn j2me_core_vm_execute_method(
    class_name: *const c_char,
    method_name: *const c_char,
    descriptor: *const c_char,
    out_result: *mut i32,
) -> i32 {
    if class_name.is_null() || method_name.is_null() || descriptor.is_null() {
        return -1;
    }

    let c_cname = match CStr::from_ptr(class_name).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    let c_mname = match CStr::from_ptr(method_name).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    let c_desc = match CStr::from_ptr(descriptor).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let mut vm_lock = match CURRENT_VM.lock() {
        Ok(l) => l,
        Err(_) => return -3,
    };

    if vm_lock.is_none() {
        *vm_lock = Some(VirtualMachine::new());
    }

    let vm = vm_lock.as_mut().unwrap();
    match vm.execute_method(c_cname, c_mname, c_desc, Vec::new(), 200_000) {
        Ok(ExecutionResult::ReturnValue(Value::Int(v))) => {
            if !out_result.is_null() {
                *out_result = v;
            }
            0
        }
        Ok(ExecutionResult::ReturnVoid) => {
            if !out_result.is_null() {
                *out_result = 0;
            }
            0
        }
        Ok(ExecutionResult::Continue) => -4,
        Ok(_) => 0,
        Err(_) => -5,
    }
}

/// Retorna estadísticas diagnósticas de la VM (clases cargadas, objetos y arrays en el Heap) en JSON.
#[no_mangle]
pub extern "C" fn j2me_core_vm_get_stats() -> *mut c_char {
    let vm_lock = match CURRENT_VM.lock() {
        Ok(l) => l,
        Err(_) => return std::ptr::null_mut(),
    };

    let (classes_count, objects_count, arrays_count) = match vm_lock.as_ref() {
        Some(vm) => (
            vm.loaded_classes.len(),
            vm.heap.object_count(),
            vm.heap.array_count(),
        ),
        None => (0, 0, 0),
    };

    let json = format!(
        "{{\"loadedClasses\":{},\"heapObjects\":{},\"heapArrays\":{},\"vmStatus\":\"Active\"}}",
        classes_count, objects_count, arrays_count
    );

    match CString::new(json) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}


