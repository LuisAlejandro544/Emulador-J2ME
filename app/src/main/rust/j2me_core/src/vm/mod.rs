//! ============================================================================
//! ESTRUCTURAS DE EJECUCIÓN DE LA MÁQUINA VIRTUAL CLDC / JVM EN RUST (vm)
//! ============================================================================
//!
//! Este módulo define las estructuras de tiempo de ejecución (Runtime) de la JVM:
//! 1. `types`: `Value`, `VmError`, `ExecutionResult`, `ArrayType`.
//! 2. `stack`: `OperandStack` y `LocalVariables`.
//! 3. `heap`: `Heap`, `ObjectInstance`, `ArrayInstance`.
//! 4. `frame`: `StackFrame` y despacho de opcodes.
//! 5. `runtime`: `VirtualMachine` y gestión del Call Stack entre clases Java.

pub mod types;
pub use types::{ArrayType, ExecutionResult, Value, VmError};

pub mod stack;
pub use stack::{LocalVariables, OperandStack};

pub mod heap;
pub use heap::{ArrayInstance, Heap, ObjectInstance};

pub mod frame;
pub use frame::{parse_descriptor_param_count, StackFrame};

pub mod runtime;
pub use runtime::VirtualMachine;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::class_parser::CpInfo;

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
        let code = vec![
            0x03,             // 0: iconst_0
            0x3B,             // 1: istore_0
            0x1A,             // 2: iload_0
            0x06,             // 3: iconst_3
            0xA2, 0x00, 0x09, // 4: if_icmpge a pc=13 (offset = 9 desde opcode pos 4)
            0x84, 0x00, 0x01, // 7: iinc 0, 1
            0xA7, 0xFF, 0xF8, // 10: goto pc=2 (offset = -8 desde pos 10)
            0x1A,             // 13: iload_0
            0xAC,             // 14: ireturn
        ];
        let mut frame = StackFrame::new(4, 2, code, "testLoop");
        let result = frame.run_to_completion(200).expect("Bucle ejecutado");
        assert_eq!(result, ExecutionResult::ReturnValue(Value::Int(3)));
    }

    #[test]
    fn test_heap_and_array_execution() {
        // bipush 5 (0x10, 0x05), newarray int (0xBC, 10), astore_0 (0x4B),
        // aload_0 (0x2A), iconst_2 (0x05), bipush 42 (0x10, 0x2A), iastore (0x4F),
        // aload_0 (0x2A), iconst_2 (0x05), iaload (0x2E), ireturn (0xAC)
        let code = vec![
            0x10, 0x05,       // bipush 5 (longitud del array)
            0xBC, 10,         // newarray T_INT (10) -> guarda ref en stack
            0x4B,             // astore_0 (guarda en local 0)
            0x2A,             // aload_0
            0x05,             // iconst_2 (índice 2)
            0x10, 0x2A,       // bipush 42 (valor)
            0x4F,             // iastore (arr[2] = 42)
            0x2A,             // aload_0
            0x05,             // iconst_2
            0x2E,             // iaload (lee arr[2])
            0xAC,             // ireturn
        ];
        let mut frame = StackFrame::new(6, 2, code, "testArray");
        let result = frame.run_to_completion(100).expect("Array manipulado correctamente");
        assert_eq!(result, ExecutionResult::ReturnValue(Value::Int(42)));
    }

    #[test]
    fn test_object_allocation_and_fields() {
        // Preparar constant pool simulado con una clase y un campo
        // CP[1] = Class(name_index: 2)
        // CP[2] = Utf8("com/example/Hero")
        // CP[3] = Fieldref(class_index: 1, name_and_type_index: 4)
        // CP[4] = NameAndType(name_index: 5, descriptor_index: 6)
        // CP[5] = Utf8("health")
        // CP[6] = Utf8("I")
        let cp = vec![
            CpInfo::Unusable,
            CpInfo::Class { name_index: 2 },
            CpInfo::Utf8("com/example/Hero".to_string()),
            CpInfo::Fieldref { class_index: 1, name_and_type_index: 4 },
            CpInfo::NameAndType { name_index: 5, descriptor_index: 6 },
            CpInfo::Utf8("health".to_string()),
            CpInfo::Utf8("I".to_string()),
        ];

        // new CP[1] (0xBB, 0x00, 0x01), astore_0 (0x4B),
        // aload_0 (0x2A), bipush 100 (0x10, 0x64), putfield CP[3] (0xB5, 0x00, 0x03),
        // aload_0 (0x2A), getfield CP[3] (0xB4, 0x00, 0x03), ireturn (0xAC)
        let code = vec![
            0xBB, 0x00, 0x01, // new Hero
            0x4B,             // astore_0
            0x2A,             // aload_0
            0x10, 0x64,       // bipush 100
            0xB5, 0x00, 0x03, // putfield health = 100
            0x2A,             // aload_0
            0xB4, 0x00, 0x03, // getfield health
            0xAC,             // ireturn
        ];

        let mut heap = Heap::new();
        let mut static_fields = std::collections::HashMap::new();
        let mut frame = StackFrame::new_with_cp(4, 2, code, cp, "com/example/Hero", "testObject");

        let mut res = ExecutionResult::Continue;
        while res == ExecutionResult::Continue {
            res = frame.step(&mut heap, &mut static_fields).expect("Instrucción válida");
        }

        assert_eq!(res, ExecutionResult::ReturnValue(Value::Int(100)));
        assert_eq!(heap.object_count(), 1);
    }
}
