//! ============================================================================
//! PILA DE OPERANDOS Y VARIABLES LOCALES DE LA JVM (vm::stack)
//! ============================================================================
//!
//! Este módulo implementa:
//! 1. `OperandStack`: Pila acotada que previene desbordamientos de búfer en memoria.
//! 2. `LocalVariables`: Ranuras de variables locales para paso de argumentos y estado.

use crate::vm::types::{Value, VmError};

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

    pub fn dup_x1(&mut self) -> Result<(), VmError> {
        if self.values.len() < 2 {
            return Err(VmError::StackUnderflow);
        }
        let top = self.pop()?;
        let second = self.pop()?;
        self.push(top.clone())?;
        self.push(second)?;
        self.push(top)?;
        Ok(())
    }

    pub fn dup2(&mut self) -> Result<(), VmError> {
        if self.values.len() < 2 {
            return Err(VmError::StackUnderflow);
        }
        let top = self.pop()?;
        let second = self.pop()?;
        self.push(second.clone())?;
        self.push(top.clone())?;
        self.push(second)?;
        self.push(top)?;
        Ok(())
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

    pub fn max_locals(&self) -> usize {
        self.max_locals
    }
}
