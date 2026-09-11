//! ============================================================================
//! GESTOR DE MEMORIA Y ASIGNADOR DE OBJETOS / ARRAYS (vm::heap)
//! ============================================================================
//!
//! Este módulo implementa la memoria dinámica (Heap) de la JVM CLDC:
//! 1. `ObjectInstance`: Instancia de una clase Java con su tabla de campos.
//! 2. `ArrayInstance`: Array tipado en memoria con comprobación de límites.
//! 3. `Heap`: Asignador seguro de referencias con protección contra fugas de memoria.

use std::collections::HashMap;
use crate::vm::types::{ArrayType, Value, VmError};

/// Instancia de un objeto Java asignado en el Heap
#[derive(Debug, Clone)]
pub struct ObjectInstance {
    pub id: u32,
    pub class_name: String,
    pub fields: HashMap<String, Value>,
}

impl ObjectInstance {
    pub fn new(id: u32, class_name: &str) -> Self {
        Self {
            id,
            class_name: class_name.to_string(),
            fields: HashMap::new(),
        }
    }

    pub fn get_field(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }

    pub fn set_field(&mut self, name: &str, val: Value) {
        self.fields.insert(name.to_string(), val);
    }
}

/// Instancia de un array Java asignado en el Heap
#[derive(Debug, Clone)]
pub struct ArrayInstance {
    pub id: u32,
    pub array_type: ArrayType,
    pub elements: Vec<Value>,
}

impl ArrayInstance {
    pub fn new(id: u32, array_type: ArrayType, length: usize) -> Self {
        let default_val = match array_type {
            ArrayType::Object(_) => Value::Null,
            ArrayType::Float => Value::Float(0.0),
            ArrayType::Double => Value::Double(0.0),
            ArrayType::Long => Value::Long(0),
            _ => Value::Int(0),
        };
        Self {
            id,
            array_type,
            elements: vec![default_val; length],
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

/// Gestor de memoria del Heap para la máquina virtual CLDC
#[derive(Debug, Clone)]
pub struct Heap {
    objects: HashMap<u32, ObjectInstance>,
    arrays: HashMap<u32, ArrayInstance>,
    next_id: u32,
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}

impl Heap {
    /// Crea un nuevo Heap. El id 0 se reserva para referencias nulas (`null`).
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            arrays: HashMap::new(),
            next_id: 1,
        }
    }

    /// Asigna una nueva instancia de objeto y retorna su identificador único
    pub fn allocate_object(&mut self, class_name: &str) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.objects.insert(id, ObjectInstance::new(id, class_name));
        id
    }

    /// Asigna un nuevo array del tipo y tamaño especificados
    pub fn allocate_array(&mut self, array_type: ArrayType, length: usize) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.arrays.insert(id, ArrayInstance::new(id, array_type, length));
        id
    }

    /// Obtiene una referencia inmutable a un objeto por su ID
    pub fn get_object(&self, id: u32) -> Result<&ObjectInstance, VmError> {
        if id == 0 {
            return Err(VmError::NullPointer);
        }
        self.objects.get(&id).ok_or(VmError::NullPointer)
    }

    /// Obtiene una referencia mutable a un objeto por su ID
    pub fn get_object_mut(&mut self, id: u32) -> Result<&mut ObjectInstance, VmError> {
        if id == 0 {
            return Err(VmError::NullPointer);
        }
        self.objects.get_mut(&id).ok_or(VmError::NullPointer)
    }

    /// Obtiene una referencia inmutable a un array por su ID
    pub fn get_array(&self, id: u32) -> Result<&ArrayInstance, VmError> {
        if id == 0 {
            return Err(VmError::NullPointer);
        }
        self.arrays.get(&id).ok_or(VmError::NullPointer)
    }

    /// Obtiene una referencia mutable a un array por su ID
    pub fn get_array_mut(&mut self, id: u32) -> Result<&mut ArrayInstance, VmError> {
        if id == 0 {
            return Err(VmError::NullPointer);
        }
        self.arrays.get_mut(&id).ok_or(VmError::NullPointer)
    }

    /// Lee el valor de un campo de una instancia de objeto
    pub fn get_field(&self, obj_id: u32, field_name: &str) -> Result<Value, VmError> {
        let obj = self.get_object(obj_id)?;
        Ok(obj.get_field(field_name).cloned().unwrap_or(Value::Int(0)))
    }

    /// Escribe el valor de un campo en una instancia de objeto
    pub fn set_field(&mut self, obj_id: u32, field_name: &str, value: Value) -> Result<(), VmError> {
        let obj = self.get_object_mut(obj_id)?;
        obj.set_field(field_name, value);
        Ok(())
    }

    /// Obtiene la longitud de un array en el Heap
    pub fn array_len(&self, array_id: u32) -> Result<usize, VmError> {
        let arr = self.get_array(array_id)?;
        Ok(arr.len())
    }

    /// Lee un elemento de un array con comprobación estricta de límites
    pub fn get_array_element(&self, array_id: u32, index: i32) -> Result<Value, VmError> {
        let arr = self.get_array(array_id)?;
        if index < 0 || (index as usize) >= arr.len() {
            return Err(VmError::ArrayIndexOutOfBounds {
                index,
                length: arr.len(),
            });
        }
        Ok(arr.elements[index as usize].clone())
    }

    /// Escribe un elemento en un array con comprobación estricta de límites
    pub fn set_array_element(&mut self, array_id: u32, index: i32, value: Value) -> Result<(), VmError> {
        let arr = self.get_array_mut(array_id)?;
        if index < 0 || (index as usize) >= arr.len() {
            return Err(VmError::ArrayIndexOutOfBounds {
                index,
                length: arr.len(),
            });
        }
        arr.elements[index as usize] = value;
        Ok(())
    }

    /// Retorna la cantidad de objetos vivos en el Heap
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// Retorna la cantidad de arrays vivos en el Heap
    pub fn array_count(&self) -> usize {
        self.arrays.len()
    }

    /// Limpia completamente el Heap
    pub fn clear(&mut self) {
        self.objects.clear();
        self.arrays.clear();
        self.next_id = 1;
    }
}
