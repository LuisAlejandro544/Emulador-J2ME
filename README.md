# Emulador J2ME Retro (Java + Rust + C++)

Emulador de juegos y aplicaciones Java ME (J2ME CLDC/MIDP) para Android, diseñado con una arquitectura híbrida de alto rendimiento y bajo consumo que combina **Java**, **Rust** y **C++**.

---

## 🎯 Visión General

En la era dorada de los teléfonos móviles con teclado numérico, los juegos móviles fueron desarrollados en **Java** bajo los perfiles CLDC (*Connected Limited Device Configuration*) y MIDP (*Mobile Information Device Profile*). 

Este proyecto recrea ese ecosistema en dispositivos Android modernos aprovechando lo mejor de tres tecnologías complementarias:

| Componente | Rol Técnico | Responsabilidad |
| :--- | :--- | :--- |
| **Java** | Entorno J2ME | Clases de la API J2ME (`javax.microedition.*`, `MIDlet`, `Canvas`, `RecordStore`) |
| **Rust** | Núcleo de Emulación | Intérprete seguro de bytecode JVM, despacho de opcodes y gestión de memoria estricta |
| **C++** | Abstracción de Hardware | Puente JNI, renderizado acelerado por hardware (OpenGL ES) y audio de baja latencia |

---

## 📱 Características Principales

- **Arquitectura Nativa Híbrida**: Máximo rendimiento sin ralentizaciones en arquitecturas de 32 bits (`armeabi-v7a`) y 64 bits (`arm64-v8a`, `x86_64`).
- **Control Táctil Fiel**: Interfaz con teclado alfanumérico retro (0-9, *, #), D-Pad direccional, tecla de acción central y SoftKeys de menú.
- **Gestión de Memoria Segura**: El núcleo en Rust previene fugas de memoria y fallos de segmentación al parsear archivos JAR/JAD corruptos.
- **Compatibilidad Extensible**: Diseñado para soportar juegos clásicos de 128x128, 176x208, 240x320 y pantallas táctiles de 360x640.

---

## 🚀 Requisitos y Configuración

- **Android SDK**: API 24 (Android 7.0) o superior.
- **Android NDK**: Versión 27.2.12479018 (o compatible vía CMake 3.22+).
- **Rust Toolchain**: `rustc` y `cargo` con soporte para los targets:
  - `aarch64-linux-android`
  - `armv7-linux-androideabi`
  - `x86_64-linux-android`
  - `i686-linux-android`

---

## 🛠️ Compilación y Construcción

El sistema de compilación está automatizado mediante **Gradle + CMake + Cargo**:

```bash
# Compilar el proyecto completo y generar el APK
./gradlew assembleDebug

# O mediante la herramienta gradle en el entorno:
gradle :app:assembleDebug
```

CMake detectará automáticamente la arquitectura (`ANDROID_ABI`) objetivo y compilará la librería estática de Rust antes de vincularla a la librería compartida nativa `libj2me_native.so`.

---

## 🧹 Limpieza de Artefactos de Compilación

Para evitar subir archivos residuales pesados (`target/`, `.cxx/`, caches de objetos):

```bash
# Con script Bash:
./clean_native_artifacts.sh

# O con script Python:
python3 clean_native_artifacts.py
```

---

## 📄 Licencia

Este proyecto está bajo una licencia de uso libre sin restricciones comerciales que obliguen a dar atribuciones no deseadas.
