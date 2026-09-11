# Roadmap de Desarrollo: Emulador J2ME

Este documento define la ruta de evolución técnica y las fases estratégicas para la construcción completa del emulador.

---

## 🗺️ Fases del Proyecto

### ✅ Fase 1: Cimientos e Infraestructura Nativa (Completada)
- [x] Configuración del compilador cruzado NDK para C++ (CMake 3.22+).
- [x] Instalación y configuración de Rust y sus toolchains Android (`aarch64`, `armv7`, `x86_64`, `i686`).
- [x] Creación del puente JNI C++ (`native-lib.cpp`) y enlace estático con la librería Rust (`libj2me_core.a`).
- [x] Verificación de exportación de funciones FFI y soporte para arquitecturas de 32 y 64 bits.
- [x] Creación de scripts de limpieza (`clean_native_artifacts.sh` y `.py`) y blindaje de `.gitignore`.

---

### ✅ Fase 2: Parser de Archivos JAR/JAD y Formato .class (Completada)
- [x] Lector de archivos ZIP/JAR en Rust para descomprimir y listar entradas (`jar_parser.rs`).
- [x] Parser del manifiesto `MANIFEST.MF` (detección de `MIDlet-1`, versión CLDC/MIDP, nombre, autor e icono).
- [x] Extracción en memoria de recursos binarios e imágenes del JAR vía FFI / JNI.
- [x] Parser binario de archivos `.class` de Java (`class_parser.rs`):
  - Constant Pool tipado (Utf8, Integer, Float, Long, Double, Methodref, Fieldref, Class, NameAndType).
  - Tabla de métodos, firmas, campos y modificadores de acceso.
  - Decodificación del atributo `Code`: secuencia de bytecode, tabla de excepciones, `max_stack` y `max_locals`.
  - Inspección directa de clases de un JAR en memoria y serialización diagnóstica en JSON vía FFI/JNI.

---

### 🔄 Fase 3: Intérprete de Bytecode y Máquina Virtual CLDC (Rust - En Progreso)
- [x] Estructuras fundamentales del Runtime de la JVM (`vm/mod.rs`):
  - `Value`: Representación tipada de valores enteros, flotantes, long, double, referencias y null.
  - `OperandStack`: Pila de operandos con protección ante StackOverflow y StackUnderflow.
  - `LocalVariables`: Tabla de variables locales indexadas y comprobación de límites.
  - `StackFrame`: Marco de pila con contador de programa (`pc`), pila, variables locales y bytecode.
- [x] Intérprete de opcodes estándar de JVM (CLDC):
  - Constantes y carga: `nop`, `aconst_null`, `iconst_m1..5`, `bipush`, `sipush`.
  - Carga y almacenamiento local: `iload`, `aload`, `iload_0..3`, `aload_0..3`, `istore`, `astore`, `istore_0..3`, `astore_0..3`.
  - Operaciones de pila: `pop`, `dup`, `swap`.
  - Aritmética entera y lógica: `iadd`, `isub`, `imul`, `idiv`, `irem`, `ineg`, `iand`, `ior`, `ixor`, `iinc`.
  - Saltos condicionales y branching: `ifeq`, `ifne`, `iflt`, `ifge`, `ifgt`, `ifle`, `if_icmpeq..le`, `goto`.
  - Control de retorno: `ireturn`, `areturn`, `return`.
- [x] Exposición en FFI y JNI (`executeBytecode`) para ejecución y pruebas directas desde Kotlin/C++.
- [ ] Manejo de llamadas a métodos (`invokevirtual`, `invokestatic`, `invokespecial`).
- [ ] Asignador de memoria para objetos e instancias de clases de forma segura (Heap).

---

### 🎨 Fase 4: Subsistema Gráfico y LCDUI (Java & C++)
- [ ] Implementación de las clases de UI de J2ME:
  - `javax.microedition.lcdui.Display`
  - `javax.microedition.lcdui.Canvas`
  - `javax.microedition.lcdui.Graphics`
  - `javax.microedition.lcdui.Image`
- [ ] Primitivas de dibujo aceleradas en C++ (líneas, rectángulos, texto con fuente bitmap, rotaciones).
- [ ] Renderizado en pantalla mediante `ANativeWindow` y OpenGL ES a 60 FPS estables.

---

### 🔊 Fase 5: Subsistema de Sonido y Multimedia (C++)
- [ ] Implementación de `javax.microedition.media.Manager` y `Player`.
- [ ] Motor de síntesis MIDI ligero y decodificación de audio PCM/WAV/AMR nativo con AAudio/Oboe.
- [ ] Manejo de hilos independientes de baja latencia para efectos de sonido sin retardos.

---

### 💾 Fase 6: Persistencia RMS (Record Management System)
- [ ] Implementación de `javax.microedition.rms.RecordStore`.
- [ ] Almacenamiento seguro de partidas y configuraciones en archivos locales por juego.
- [ ] Exportación e importación de partidas guardadas.

---

### 🎮 Fase 7: Controles Táctiles, Feedback Háptico y Perfiles
- [ ] Personalización del layout de controles (opacidad, posición y tamaño).
- [ ] Soporte para gamepads físicos bluetooth y USB.
- [ ] Selección de perfiles de pantalla y paletas de color retro (retroiluminación verde/azul estilo teléfonos de época).
