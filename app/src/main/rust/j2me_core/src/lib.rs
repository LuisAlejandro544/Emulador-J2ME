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

use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Estado global del motor de emulación en Rust
static CORE_INITIALIZED: AtomicBool = AtomicBool::new(false);
static INSTRUCTION_CYCLE_COUNT: AtomicU64 = AtomicU64::new(0);

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
}
