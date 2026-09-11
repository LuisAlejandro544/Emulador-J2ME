//! ============================================================================
//! PARSER DE ARCHIVOS JAR Y MANIFIESTO J2ME (jar_parser.rs)
//! ============================================================================
//!
//! En J2ME, un paquete .jar es una estructura ZIP estándar que contiene:
//! 1. "META-INF/MANIFEST.MF": Metadatos del MIDlet (Nombre, Versión, Clase principal, etc.)
//! 2. Clases Java compiladas (".class")
//! 3. Recursos multimedia (imágenes PNG, sonidos MIDI/WAV)
//!
//! Este módulo parsea el contenedor ZIP directamente en memoria de forma segura,
//! extrayendo el manifiesto y listando los recursos sin peligro de desbordamiento.

use miniz_oxide::inflate::decompress_to_vec;
use std::collections::HashMap;

/// Metadatos clave extraídos de META-INF/MANIFEST.MF de un juego J2ME
#[derive(Debug, Clone, Default)]
pub struct MidletManifest {
    pub midlet_name: String,
    pub midlet_vendor: String,
    pub midlet_version: String,
    pub microedition_profile: String,
    pub microedition_configuration: String,
    pub main_class: String,
    pub icon_path: String,
    pub raw_attributes: HashMap<String, String>,
}

/// Entrada de archivo contenida dentro del JAR
#[derive(Debug, Clone)]
pub struct JarEntry {
    pub name: String,
    pub compressed_size: usize,
    pub uncompressed_size: usize,
    pub compression_method: u16,
    pub data_offset: usize,
}

/// Estructura del paquete JAR cargado en memoria
pub struct JarArchive<'a> {
    data: &'a [u8],
    entries: HashMap<String, JarEntry>,
}

impl<'a> JarArchive<'a> {
    /// Carga y parsea el índice ZIP de un paquete JAR desde un búfer de bytes.
    pub fn parse(data: &'a [u8]) -> Result<Self, String> {
        if data.len() < 22 {
            return Err("El archivo es demasiado pequeño para ser un archivo ZIP/JAR válido".to_string());
        }

        // Buscar el End of Central Directory Record (EOCD), con firma 0x06054b50
        let eocd_offset = Self::find_eocd(data)
            .ok_or_else(|| "No se encontró el registro EOCD de ZIP en el archivo JAR".to_string())?;

        if eocd_offset + 22 > data.len() {
            return Err("Estructura de EOCD corrupta".to_string());
        }

        let total_entries = u16::from_le_bytes([data[eocd_offset + 10], data[eocd_offset + 11]]) as usize;
        let cd_size = u32::from_le_bytes([
            data[eocd_offset + 12],
            data[eocd_offset + 13],
            data[eocd_offset + 14],
            data[eocd_offset + 15],
        ]) as usize;
        let cd_offset = u32::from_le_bytes([
            data[eocd_offset + 16],
            data[eocd_offset + 17],
            data[eocd_offset + 18],
            data[eocd_offset + 19],
        ]) as usize;

        if cd_offset + cd_size > data.len() {
            return Err("El directorio central excede el tamaño del archivo JAR".to_string());
        }

        let mut entries = HashMap::new();
        let mut cursor = cd_offset;

        // Parsear cada entrada en el Directorio Central (firma 0x02014b50)
        for _ in 0..total_entries {
            if cursor + 46 > data.len() {
                break;
            }

            let sig = u32::from_le_bytes([data[cursor], data[cursor + 1], data[cursor + 2], data[cursor + 3]]);
            if sig != 0x02014b50 {
                break;
            }

            let comp_method = u16::from_le_bytes([data[cursor + 10], data[cursor + 11]]);
            let comp_size = u32::from_le_bytes([
                data[cursor + 20],
                data[cursor + 21],
                data[cursor + 22],
                data[cursor + 23],
            ]) as usize;
            let uncomp_size = u32::from_le_bytes([
                data[cursor + 24],
                data[cursor + 25],
                data[cursor + 26],
                data[cursor + 27],
            ]) as usize;
            let name_len = u16::from_le_bytes([data[cursor + 28], data[cursor + 29]]) as usize;
            let extra_len = u16::from_le_bytes([data[cursor + 30], data[cursor + 31]]) as usize;
            let comment_len = u16::from_le_bytes([data[cursor + 32], data[cursor + 33]]) as usize;
            let local_header_offset = u32::from_le_bytes([
                data[cursor + 42],
                data[cursor + 43],
                data[cursor + 44],
                data[cursor + 45],
            ]) as usize;

            cursor += 46;
            if cursor + name_len > data.len() {
                break;
            }

            let name_bytes = &data[cursor..cursor + name_len];
            let name = String::from_utf8_lossy(name_bytes).to_string();

            cursor += name_len + extra_len + comment_len;

            // Calcular el offset de datos reales a partir del encabezado local de archivo (firma 0x04034b50)
            if local_header_offset + 30 <= data.len() {
                let local_sig = u32::from_le_bytes([
                    data[local_header_offset],
                    data[local_header_offset + 1],
                    data[local_header_offset + 2],
                    data[local_header_offset + 3],
                ]);

                if local_sig == 0x04034b50 {
                    let loc_name_len = u16::from_le_bytes([
                        data[local_header_offset + 26],
                        data[local_header_offset + 27],
                    ]) as usize;
                    let loc_extra_len = u16::from_le_bytes([
                        data[local_header_offset + 28],
                        data[local_header_offset + 29],
                    ]) as usize;
                    let data_offset = local_header_offset + 30 + loc_name_len + loc_extra_len;

                    entries.insert(
                        name.clone(),
                        JarEntry {
                            name,
                            compressed_size: comp_size,
                            uncompressed_size: uncomp_size,
                            compression_method: comp_method,
                            data_offset,
                        },
                    );
                }
            }
        }

        Ok(JarArchive { data, entries })
    }

    /// Busca la firma de End of Central Directory (0x06054b50) desde el final del archivo.
    fn find_eocd(data: &[u8]) -> Option<usize> {
        let min_offset = if data.len() > 65557 {
            data.len() - 65557
        } else {
            0
        };

        for i in (min_offset..=data.len() - 22).rev() {
            if data[i] == 0x50 && data[i + 1] == 0x4b && data[i + 2] == 0x05 && data[i + 3] == 0x06 {
                return Some(i);
            }
        }
        None
    }

    /// Retorna una lista con los nombres de todos los archivos y recursos dentro del JAR.
    pub fn list_files(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    /// Extrae y descomprime el contenido de un archivo dentro del JAR por su nombre.
    pub fn read_file(&self, filename: &str) -> Result<Vec<u8>, String> {
        // Permitir búsqueda sin distinguir mayúsculas/minúsculas o variaciones de barra
        let entry = self
            .entries
            .get(filename)
            .or_else(|| {
                self.entries
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(filename))
                    .map(|(_, v)| v)
            })
            .ok_or_else(|| format!("Archivo '{}' no encontrado en el JAR", filename))?;

        if entry.data_offset + entry.compressed_size > self.data.len() {
            return Err("El segmento comprimido excede los límites del archivo".to_string());
        }

        let raw_slice = &self.data[entry.data_offset..entry.data_offset + entry.compressed_size];

        match entry.compression_method {
            0 => {
                // Almacenado sin compresión (STORED)
                Ok(raw_slice.to_vec())
            }
            8 => {
                // Compresión DEFLATE estándar (RFC 1951)
                decompress_to_vec(raw_slice)
                    .map_err(|e| format!("Fallo al descomprimir '{}': {:?}", filename, e))
            }
            other => Err(format!(
                "Método de compresión {} no soportado en J2ME JAR",
                other
            )),
        }
    }

    /// Parsea los atributos del manifiesto 'META-INF/MANIFEST.MF'.
    pub fn parse_manifest(&self) -> Result<MidletManifest, String> {
        let manifest_bytes = self.read_file("META-INF/MANIFEST.MF")?;
        let manifest_text = String::from_utf8_lossy(&manifest_bytes);

        let mut manifest = MidletManifest::default();
        let mut current_key = String::new();
        let mut current_val = String::new();

        for line in manifest_text.lines() {
            if line.starts_with(' ') || line.starts_with('\t') {
                // Continuación de línea larga según estándar JAR
                current_val.push_str(line.trim_start());
            } else {
                if !current_key.is_empty() {
                    manifest.raw_attributes.insert(current_key.clone(), current_val.trim().to_string());
                }

                if let Some((k, v)) = line.split_once(':') {
                    current_key = k.trim().to_string();
                    current_val = v.trim().to_string();
                } else {
                    current_key.clear();
                    current_val.clear();
                }
            }
        }

        if !current_key.is_empty() {
            manifest.raw_attributes.insert(current_key, current_val.trim().to_string());
        }

        // Mapear atributos estándar de J2ME
        if let Some(name) = manifest.raw_attributes.get("MIDlet-Name") {
            manifest.midlet_name = name.clone();
        }
        if let Some(vendor) = manifest.raw_attributes.get("MIDlet-Vendor") {
            manifest.midlet_vendor = vendor.clone();
        }
        if let Some(version) = manifest.raw_attributes.get("MIDlet-Version") {
            manifest.midlet_version = version.clone();
        }
        if let Some(prof) = manifest.raw_attributes.get("MicroEdition-Profile") {
            manifest.microedition_profile = prof.clone();
        }
        if let Some(conf) = manifest.raw_attributes.get("MicroEdition-Configuration") {
            manifest.microedition_configuration = conf.clone();
        }

        // Extraer la clase principal a partir de MIDlet-1 (formato: "Nombre, /icon.png, com.package.MainMIDlet")
        if let Some(midlet_1) = manifest.raw_attributes.get("MIDlet-1") {
            let parts: Vec<&str> = midlet_1.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 3 {
                if manifest.midlet_name.is_empty() {
                    manifest.midlet_name = parts[0].to_string();
                }
                manifest.icon_path = parts[1].to_string();
                manifest.main_class = parts[2].to_string();
            } else if parts.len() == 1 {
                manifest.main_class = parts[0].to_string();
            }
        }

        Ok(manifest)
    }
}
