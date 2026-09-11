//! ============================================================================
//! TIPOS DE DATOS Y ERRORES DEL RUNTIME JVM / CLDC EN RUST
//! ============================================================================
//!
//! Este archivo define los tipos de valores de tiempo de ejecución y los errores
//! controlados de la máquina virtual para garantizar seguridad estricta de memoria.

use std::fmt;

/// Errores durante la ejecución de una instrucción o método de la máquina virtual
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
    ConstantPoolIndexOutOfBounds(u16),
    InvalidConstantPoolEntry(u16),
    ClassNotFound(String),
    MethodNotFound { class: String, method: String, descriptor: String },
    FieldNotFound { class: String, field: String },
    ArrayIndexOutOfBounds { index: i32, length: usize },
    NegativeArraySize(i32),
    MaxCallStackDepthExceeded(usize),
    ClassCastException,
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
            VmError::ConstantPoolIndexOutOfBounds(idx) => {
                write!(f, "Índice del Constant Pool fuera de límites: {}", idx)
            }
            VmError::InvalidConstantPoolEntry(idx) => {
                write!(f, "Entrada del Constant Pool inválida en el índice: {}", idx)
            }
            VmError::ClassNotFound(name) => write!(f, "Clase Java no encontrada en el runtime: {}", name),
            VmError::MethodNotFound { class, method, descriptor } => {
                write!(f, "Método no encontrado: {}.{}:{}", class, method, descriptor)
            }
            VmError::FieldNotFound { class, field } => {
                write!(f, "Campo no encontrado: {}.{}", class, field)
            }
            VmError::ArrayIndexOutOfBounds { index, length } => {
                write!(f, "Índice de array fuera de rango: {} (longitud: {})", index, length)
            }
            VmError::NegativeArraySize(size) => {
                write!(f, "Tamaño negativo de array: {}", size)
            }
            VmError::MaxCallStackDepthExceeded(max) => {
                write!(f, "Profundidad máxima de pila de llamadas superada ({})", max)
            }
            VmError::ClassCastException => write!(f, "Error de conversión de tipo de clase (ClassCastException)"),
        }
    }
}

/// Tipos de elementos admitidos en arrays del Heap
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayType {
    Boolean,
    Char,
    Float,
    Double,
    Byte,
    Short,
    Int,
    Long,
    Object(String),
}

/// Representación de un valor tipado de 32 o 64 bits en la máquina virtual CLDC
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

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

/// Resultado de la ejecución de una instrucción o ciclo
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionResult {
    Continue,
    ReturnVoid,
    ReturnValue(Value),
    InvokeMethod {
        class_name: String,
        method_name: String,
        descriptor: String,
        is_static: bool,
        args: Vec<Value>,
    },
}
