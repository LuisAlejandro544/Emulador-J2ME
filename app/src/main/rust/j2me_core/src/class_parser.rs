//! ============================================================================
//! PARSER BINARIO DE ARCHIVOS .CLASS DE JAVA (JVM / CLDC)
//! ============================================================================
//!
//! Este módulo implementa la lectura y validación estricta del formato binario
//! de archivos compilados de Java (`.class`) según las especificaciones de
//! Java Virtual Machine (JVM Specification) y Java ME CLDC 1.0 / 1.1.
//!
//! Estructura de un archivo ClassFile:
//! 1. Magic Number: `0xCAFEBABE` (4 bytes, obligatorio).
//! 2. Versión: `minor_version` (2 bytes) y `major_version` (2 bytes).
//! 3. Constant Pool: Tabla de constantes tipadas (UTF8, enteros, clases, métodos).
//! 4. Banderas de acceso: `public`, `final`, `super`, `abstract`, etc.
//! 5. Índices de Clase: `this_class` y `super_class` apuntando al Constant Pool.
//! 6. Interfaces implementadas.
//! 7. Campos de la clase (Fields).
//! 8. Métodos de la clase (Methods) con sus atributos de Bytecode (`Code`).
//! 9. Atributos generales de la clase (SourceFile, etc.).
//!
//! Diseñado para máxima seguridad de memoria en Rust, evitando desbordamientos
//! de búfer y manejando datos corruptos o incompletos con errores descriptivos.

/// Número mágico identificador de todo archivo .class de Java
pub const JAVA_CLASS_MAGIC: u32 = 0xCAFEBABE;

/// Tags identificadores para los elementos del Constant Pool
pub const CONSTANT_UTF8: u8 = 1;
pub const CONSTANT_INTEGER: u8 = 3;
pub const CONSTANT_FLOAT: u8 = 4;
pub const CONSTANT_LONG: u8 = 5;
pub const CONSTANT_DOUBLE: u8 = 6;
pub const CONSTANT_CLASS: u8 = 7;
pub const CONSTANT_STRING: u8 = 8;
pub const CONSTANT_FIELDREF: u8 = 9;
pub const CONSTANT_METHODREF: u8 = 10;
pub const CONSTANT_INTERFACE_METHODREF: u8 = 11;
pub const CONSTANT_NAME_AND_TYPE: u8 = 12;

/// Entrada individual dentro del Constant Pool de la clase.
/// El Constant Pool almacena literales, referencias a métodos, campos y nombres.
#[derive(Debug, Clone, PartialEq)]
pub enum CpInfo {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class { name_index: u16 },
    StringRef { string_index: u16 },
    Fieldref { class_index: u16, name_and_type_index: u16 },
    Methodref { class_index: u16, name_and_type_index: u16 },
    InterfaceMethodref { class_index: u16, name_and_type_index: u16 },
    NameAndType { name_index: u16, descriptor_index: u16 },
    /// Ranura vacía para compensar el doble slot que ocupan Long y Double en la JVM
    Unusable,
}

/// Atributo binario genérico dentro de una clase, campo o método
#[derive(Debug, Clone, PartialEq)]
pub struct RawAttribute {
    pub name_index: u16,
    pub data: Vec<u8>,
}

/// Entrada de la tabla de excepciones de un método
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

/// Atributo 'Code' decodificado: contiene el bytecode ejecutable del método
#[derive(Debug, Clone, PartialEq)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub bytecode: Vec<u8>,
    pub exception_table: Vec<ExceptionTableEntry>,
}

/// Información de un campo (Field) de la clase Java
#[derive(Debug, Clone, PartialEq)]
pub struct FieldInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<RawAttribute>,
}

/// Información de un método (Method) de la clase Java
#[derive(Debug, Clone, PartialEq)]
pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<RawAttribute>,
}

impl MethodInfo {
    /// Extrae y decodifica el atributo 'Code' que contiene los opcodes del método
    pub fn parse_code_attribute(&self, cp: &[CpInfo]) -> Option<CodeAttribute> {
        for attr in &self.attributes {
            if let Some(CpInfo::Utf8(name)) = cp.get(attr.name_index as usize) {
                if name == "Code" && attr.data.len() >= 8 {
                    let mut cursor = 0;
                    let max_stack = u16::from_be_bytes([attr.data[cursor], attr.data[cursor + 1]]);
                    cursor += 2;
                    let max_locals = u16::from_be_bytes([attr.data[cursor], attr.data[cursor + 1]]);
                    cursor += 2;
                    let code_length = u32::from_be_bytes([
                        attr.data[cursor],
                        attr.data[cursor + 1],
                        attr.data[cursor + 2],
                        attr.data[cursor + 3],
                    ]) as usize;
                    cursor += 4;

                    if cursor + code_length > attr.data.len() {
                        return None;
                    }
                    let bytecode = attr.data[cursor..cursor + code_length].to_vec();
                    cursor += code_length;

                    if cursor + 2 > attr.data.len() {
                        return None;
                    }
                    let exception_count = u16::from_be_bytes([attr.data[cursor], attr.data[cursor + 1]]) as usize;
                    cursor += 2;

                    let mut exception_table = Vec::with_capacity(exception_count);
                    for _ in 0..exception_count {
                        if cursor + 8 > attr.data.len() {
                            return None;
                        }
                        let start_pc = u16::from_be_bytes([attr.data[cursor], attr.data[cursor + 1]]);
                        let end_pc = u16::from_be_bytes([attr.data[cursor + 2], attr.data[cursor + 3]]);
                        let handler_pc = u16::from_be_bytes([attr.data[cursor + 4], attr.data[cursor + 5]]);
                        let catch_type = u16::from_be_bytes([attr.data[cursor + 6], attr.data[cursor + 7]]);
                        cursor += 8;
                        exception_table.push(ExceptionTableEntry {
                            start_pc,
                            end_pc,
                            handler_pc,
                            catch_type,
                        });
                    }

                    return Some(CodeAttribute {
                        max_stack,
                        max_locals,
                        bytecode,
                        exception_table,
                    });
                }
            }
        }
        None
    }
}

/// Representación completa de un archivo de clase Java (`.class`)
#[derive(Debug, Clone)]
pub struct JavaClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    /// El Constant Pool es 1-indexed. La posición 0 contiene `CpInfo::Unusable`.
    pub constant_pool: Vec<CpInfo>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<RawAttribute>,
}

/// Errores posibles durante el parseo de un archivo .class
#[derive(Debug, Clone, PartialEq)]
pub enum ClassParseError {
    UnexpectedEndOfData,
    InvalidMagicNumber(u32),
    UnsupportedVersion { major: u16, minor: u16 },
    InvalidConstantTag(u8),
    InvalidUtf8Encoding,
    CorruptedAttributeData,
}

impl JavaClassFile {
    /// Parsea un archivo `.class` a partir de un slice binario en memoria.
    pub fn parse(bytes: &[u8]) -> Result<Self, ClassParseError> {
        let mut cursor = 0;

        // 1. Verificación del Magic Number (0xCAFEBABE)
        if bytes.len() < 4 {
            return Err(ClassParseError::UnexpectedEndOfData);
        }
        let magic = read_u32(bytes, &mut cursor)?;
        if magic != JAVA_CLASS_MAGIC {
            return Err(ClassParseError::InvalidMagicNumber(magic));
        }

        // 2. Versión de bytecode (menor y mayor)
        let minor_version = read_u16(bytes, &mut cursor)?;
        let major_version = read_u16(bytes, &mut cursor)?;

        // 3. Constant Pool
        let cp_count = read_u16(bytes, &mut cursor)? as usize;
        let mut constant_pool = Vec::with_capacity(cp_count);
        // El índice 0 no se utiliza en la especificación de JVM
        constant_pool.push(CpInfo::Unusable);

        let mut i = 1;
        while i < cp_count {
            let tag = read_u8(bytes, &mut cursor)?;
            match tag {
                CONSTANT_UTF8 => {
                    let len = read_u16(bytes, &mut cursor)? as usize;
                    if cursor + len > bytes.len() {
                        return Err(ClassParseError::UnexpectedEndOfData);
                    }
                    let utf8_slice = &bytes[cursor..cursor + len];
                    cursor += len;
                    let text = String::from_utf8_lossy(utf8_slice).into_owned();
                    constant_pool.push(CpInfo::Utf8(text));
                    i += 1;
                }
                CONSTANT_INTEGER => {
                    let val = read_u32(bytes, &mut cursor)? as i32;
                    constant_pool.push(CpInfo::Integer(val));
                    i += 1;
                }
                CONSTANT_FLOAT => {
                    let bits = read_u32(bytes, &mut cursor)?;
                    constant_pool.push(CpInfo::Float(f32::from_bits(bits)));
                    i += 1;
                }
                CONSTANT_LONG => {
                    let high = read_u32(bytes, &mut cursor)? as u64;
                    let low = read_u32(bytes, &mut cursor)? as u64;
                    let val = ((high << 32) | low) as i64;
                    constant_pool.push(CpInfo::Long(val));
                    // Ocupa dos entradas en el Constant Pool
                    constant_pool.push(CpInfo::Unusable);
                    i += 2;
                }
                CONSTANT_DOUBLE => {
                    let high = read_u32(bytes, &mut cursor)? as u64;
                    let low = read_u32(bytes, &mut cursor)? as u64;
                    let bits = (high << 32) | low;
                    constant_pool.push(CpInfo::Double(f64::from_bits(bits)));
                    // Ocupa dos entradas en el Constant Pool
                    constant_pool.push(CpInfo::Unusable);
                    i += 2;
                }
                CONSTANT_CLASS => {
                    let name_index = read_u16(bytes, &mut cursor)?;
                    constant_pool.push(CpInfo::Class { name_index });
                    i += 1;
                }
                CONSTANT_STRING => {
                    let string_index = read_u16(bytes, &mut cursor)?;
                    constant_pool.push(CpInfo::StringRef { string_index });
                    i += 1;
                }
                CONSTANT_FIELDREF => {
                    let class_index = read_u16(bytes, &mut cursor)?;
                    let name_and_type_index = read_u16(bytes, &mut cursor)?;
                    constant_pool.push(CpInfo::Fieldref {
                        class_index,
                        name_and_type_index,
                    });
                    i += 1;
                }
                CONSTANT_METHODREF => {
                    let class_index = read_u16(bytes, &mut cursor)?;
                    let name_and_type_index = read_u16(bytes, &mut cursor)?;
                    constant_pool.push(CpInfo::Methodref {
                        class_index,
                        name_and_type_index,
                    });
                    i += 1;
                }
                CONSTANT_INTERFACE_METHODREF => {
                    let class_index = read_u16(bytes, &mut cursor)?;
                    let name_and_type_index = read_u16(bytes, &mut cursor)?;
                    constant_pool.push(CpInfo::InterfaceMethodref {
                        class_index,
                        name_and_type_index,
                    });
                    i += 1;
                }
                CONSTANT_NAME_AND_TYPE => {
                    let name_index = read_u16(bytes, &mut cursor)?;
                    let descriptor_index = read_u16(bytes, &mut cursor)?;
                    constant_pool.push(CpInfo::NameAndType {
                        name_index,
                        descriptor_index,
                    });
                    i += 1;
                }
                other => return Err(ClassParseError::InvalidConstantTag(other)),
            }
        }

        // 4. Banderas de acceso e índices de clase
        let access_flags = read_u16(bytes, &mut cursor)?;
        let this_class = read_u16(bytes, &mut cursor)?;
        let super_class = read_u16(bytes, &mut cursor)?;

        // 5. Interfaces
        let interfaces_count = read_u16(bytes, &mut cursor)? as usize;
        let mut interfaces = Vec::with_capacity(interfaces_count);
        for _ in 0..interfaces_count {
            interfaces.push(read_u16(bytes, &mut cursor)?);
        }

        // 6. Campos
        let fields_count = read_u16(bytes, &mut cursor)? as usize;
        let mut fields = Vec::with_capacity(fields_count);
        for _ in 0..fields_count {
            let field_flags = read_u16(bytes, &mut cursor)?;
            let name_index = read_u16(bytes, &mut cursor)?;
            let desc_index = read_u16(bytes, &mut cursor)?;
            let attr_count = read_u16(bytes, &mut cursor)? as usize;
            let mut attrs = Vec::with_capacity(attr_count);
            for _ in 0..attr_count {
                attrs.push(read_attribute(bytes, &mut cursor)?);
            }
            fields.push(FieldInfo {
                access_flags: field_flags,
                name_index,
                descriptor_index: desc_index,
                attributes: attrs,
            });
        }

        // 7. Métodos
        let methods_count = read_u16(bytes, &mut cursor)? as usize;
        let mut methods = Vec::with_capacity(methods_count);
        for _ in 0..methods_count {
            let method_flags = read_u16(bytes, &mut cursor)?;
            let name_index = read_u16(bytes, &mut cursor)?;
            let desc_index = read_u16(bytes, &mut cursor)?;
            let attr_count = read_u16(bytes, &mut cursor)? as usize;
            let mut attrs = Vec::with_capacity(attr_count);
            for _ in 0..attr_count {
                attrs.push(read_attribute(bytes, &mut cursor)?);
            }
            methods.push(MethodInfo {
                access_flags: method_flags,
                name_index,
                descriptor_index: desc_index,
                attributes: attrs,
            });
        }

        // 8. Atributos de clase
        let attributes_count = read_u16(bytes, &mut cursor)? as usize;
        let mut attributes = Vec::with_capacity(attributes_count);
        for _ in 0..attributes_count {
            attributes.push(read_attribute(bytes, &mut cursor)?);
        }

        Ok(JavaClassFile {
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }

    /// Obtiene una cadena UTF8 del Constant Pool por su índice.
    pub fn get_utf8(&self, index: u16) -> Option<&str> {
        match self.constant_pool.get(index as usize) {
            Some(CpInfo::Utf8(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Resuelve el nombre calificado de la clase actual (ej. "com/game/MainMidlet").
    pub fn get_class_name(&self) -> Option<String> {
        match self.constant_pool.get(self.this_class as usize) {
            Some(CpInfo::Class { name_index }) => self.get_utf8(*name_index).map(|s| s.to_string()),
            _ => None,
        }
    }

    /// Resuelve el nombre de la superclase (ej. "javax/microedition/midlet/MIDlet").
    pub fn get_super_class_name(&self) -> Option<String> {
        if self.super_class == 0 {
            return None; // java/lang/Object no tiene superclase
        }
        match self.constant_pool.get(self.super_class as usize) {
            Some(CpInfo::Class { name_index }) => self.get_utf8(*name_index).map(|s| s.to_string()),
            _ => None,
        }
    }

    /// Busca un método específico por su nombre y su descriptor de parámetros/retorno.
    pub fn find_method(&self, name: &str, descriptor: &str) -> Option<&MethodInfo> {
        self.methods.iter().find(|m| {
            let m_name = self.get_utf8(m.name_index);
            let m_desc = self.get_utf8(m.descriptor_index);
            m_name == Some(name) && m_desc == Some(descriptor)
        })
    }

    /// Genera un resumen serializable en JSON con información de diagnóstico de la clase.
    pub fn to_summary_json(&self) -> String {
        let class_name = self.get_class_name().unwrap_or_else(|| "Desconocida".to_string());
        let super_name = self.get_super_class_name().unwrap_or_else(|| "Ninguna".to_string());

        let mut method_entries = Vec::new();
        for m in &self.methods {
            let m_name = self.get_utf8(m.name_index).unwrap_or("?");
            let m_desc = self.get_utf8(m.descriptor_index).unwrap_or("?");
            let code = m.parse_code_attribute(&self.constant_pool);
            let (max_stack, max_locals, code_size) = match code {
                Some(ref c) => (c.max_stack, c.max_locals, c.bytecode.len()),
                None => (0, 0, 0),
            };
            method_entries.push(format!(
                "{{\"name\":\"{}\",\"descriptor\":\"{}\",\"maxStack\":{},\"maxLocals\":{},\"codeBytes\":{}}}",
                escape_json(m_name),
                escape_json(m_desc),
                max_stack,
                max_locals,
                code_size
            ));
        }

        format!(
            "{{\"className\":\"{}\",\"superClass\":\"{}\",\"majorVersion\":{},\"minorVersion\":{},\"fieldsCount\":{},\"methodsCount\":{},\"methods\":[{}]}}",
            escape_json(&class_name),
            escape_json(&super_name),
            self.major_version,
            self.minor_version,
            self.fields.len(),
            self.methods.len(),
            method_entries.join(",")
        )
    }
}

// ============================================================================
// FUNCIONES AUXILIARES DE LECTURA BINARIA CON PROTECCIÓN CONTRA OVERFLOW
// ============================================================================

fn read_u8(bytes: &[u8], cursor: &mut usize) -> Result<u8, ClassParseError> {
    if *cursor >= bytes.len() {
        return Err(ClassParseError::UnexpectedEndOfData);
    }
    let val = bytes[*cursor];
    *cursor += 1;
    Ok(val)
}

fn read_u16(bytes: &[u8], cursor: &mut usize) -> Result<u16, ClassParseError> {
    if *cursor + 2 > bytes.len() {
        return Err(ClassParseError::UnexpectedEndOfData);
    }
    let val = u16::from_be_bytes([bytes[*cursor], bytes[*cursor + 1]]);
    *cursor += 2;
    Ok(val)
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, ClassParseError> {
    if *cursor + 4 > bytes.len() {
        return Err(ClassParseError::UnexpectedEndOfData);
    }
    let val = u32::from_be_bytes([
        bytes[*cursor],
        bytes[*cursor + 1],
        bytes[*cursor + 2],
        bytes[*cursor + 3],
    ]);
    *cursor += 4;
    Ok(val)
}

fn read_attribute(bytes: &[u8], cursor: &mut usize) -> Result<RawAttribute, ClassParseError> {
    let name_index = read_u16(bytes, cursor)?;
    let length = read_u32(bytes, cursor)? as usize;
    if *cursor + length > bytes.len() {
        return Err(ClassParseError::UnexpectedEndOfData);
    }
    let data = bytes[*cursor..*cursor + length].to_vec();
    *cursor += length;
    Ok(RawAttribute { name_index, data })
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_magic_number() {
        let fake_bytes = [0x00, 0x01, 0x02, 0x03];
        let result = JavaClassFile::parse(&fake_bytes);
        assert!(matches!(result, Err(ClassParseError::InvalidMagicNumber(0x00010203))));
    }

    #[test]
    fn test_truncated_header() {
        let fake_bytes = [0xCA, 0xFE];
        let result = JavaClassFile::parse(&fake_bytes);
        assert!(matches!(result, Err(ClassParseError::UnexpectedEndOfData)));
    }
}
