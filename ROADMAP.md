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

### ✅ Fase 3: Intérprete de Bytecode y Máquina Virtual CLDC (Completada)
- [x] Estructuras fundamentales del Runtime de la JVM (`vm/types.rs`, `vm/stack.rs`):
  - `Value`: Representación tipada de valores enteros, flotantes, long, double, referencias (`ObjectRef`) y `Null`.
  - `OperandStack`: Pila de operandos con protección ante `StackOverflow` y `StackUnderflow`, incluyendo `dup`, `dup_x1`, `dup2` y `swap`.
  - `LocalVariables`: Tabla de variables locales indexadas y comprobación de límites.
- [x] Gestor de memoria dinámica (`Heap` en `vm/heap.rs`):
  - Asignación segura de objetos (`ObjectInstance`) y almacenamiento asociativo de campos.
  - Asignación y manipulación de arrays primitivos y de objetos (`ArrayInstance`) con control estricto de límites (`ArrayIndexOutOfBounds`).
  - Estadísticas diagnósticas de objetos y arrays vivos en memoria.
- [x] Intérprete de opcodes estándar de JVM (CLDC en `vm/frame.rs`):
  - Constantes y carga: `nop`, `aconst_null`, `iconst_m1..5`, `bipush`, `sipush`, `ldc`, `ldc_w`.
  - Carga y almacenamiento local: `iload`, `aload`, `iload_0..3`, `aload_0..3`, `istore`, `astore`, `istore_0..3`, `astore_0..3`.
  - Operaciones de pila: `pop`, `pop2`, `dup`, `dup_x1`, `dup2`, `swap`.
  - Aritmética entera y lógica: `iadd`, `isub`, `imul`, `idiv`, `irem`, `ineg`, `iand`, `ior`, `ixor`, `iinc`.
  - Saltos condicionales y branching: `ifeq`, `ifne`, `iflt`, `ifge`, `ifgt`, `ifle`, `if_icmpeq..le`, `goto`, `ifnull`, `ifnonnull`.
  - Control de retorno: `ireturn`, `areturn`, `return`.
  - Objetos y campos: `new`, `getfield`, `putfield`, `getstatic`, `putstatic`, `checkcast`, `instanceof`.
  - Arrays: `newarray`, `anewarray`, `arraylength`, `iaload`, `baload`, `caload`, `saload`, `aaload`, `iastore`, `bastore`, `castore`, `sastore`, `aastore`.
  - Sincronización: `monitorenter`, `monitorexit`.
  - Invocación de métodos: `invokevirtual`, `invokespecial`, `invokestatic` con resolución de firmas y descriptores.
- [x] Pila de llamadas y Runtime global (`vm/runtime.rs`):
  - `VirtualMachine`: Call Stack con profundidad acotada (`max_call_depth = 512`) y paso de argumentos.
  - Registro de clases cargadas y ejecución de métodos por nombre y descriptor.
  - Built-ins nativos interceptados: `Object.<init>`, `MIDlet.<init>`, `System.currentTimeMillis`, `System.gc`, `Math.abs/max/min`, `Thread.sleep`.
- [x] Integración FFI, C++ y JNI (`lib.rs`, `native-lib.cpp`, `J2meNativeBridge.kt`):
  - Métodos expuestos: `vmReset()`, `vmLoadClass()`, `vmExecuteMethod()`, `vmGetStats()`.
  - Suite de tests unitarios de integración en Rust (7/7 pruebas superadas exitosamente).

---

### ✅ Fase 4: Subsistema Gráfico y LCDUI (Completada)
- [x] Implementación estándar de las APIs de UI de J2ME en Java:
  - `javax.microedition.midlet.MIDlet`, `MIDletStateChangeException`
  - `javax.microedition.lcdui.Display`, `Displayable`
  - `javax.microedition.lcdui.Canvas`, `Command`, `CommandListener`
  - `javax.microedition.lcdui.Graphics`, `Font`, `Image`
- [x] Rasterizador 2D acelerado en C++ (`framebuffer.h`, `framebuffer.cpp`):
  - Búfer de píxeles ARGB8888 con sincronización mutex thread-safe.
  - Primitivas gráficas: `drawLine` (algoritmo de Bresenham), `drawRect`, `fillRect`, `drawArc`, `fillArc`, `drawString` (fuente bitmap 8x8 integrada), `drawImage` (blitting ARGB con canal alfa y clipping rectangular).
  - Volcado de alta velocidad a `android.graphics.Bitmap` utilizando `jnigraphics` (`AndroidBitmap_lockPixels`).
- [x] Pantalla de Emulación y Controles Táctiles Retro (`J2meEmulatorScreen.kt`):
  - Pantalla LCD virtual a 60 FPS estables con escalado pixel-art (`FilterQuality.None`).
  - Teclado clásico completo: SoftKey 1 y 2, tecla central FIRE, cruceta direccional D-Pad y teclado alfanumérico (1-9, *, 0, #).
  - Soporte de gestos táctiles directos sobre la pantalla (`pointerPressed`, `pointerDragged`).
  - Canvas interactivo reactivo y puente de despacho de eventos bidireccional hacia el MIDlet.
- [x] Integración en pipeline NDK/CMake para 32 bits (`armeabi-v7a`) y 64 bits (`arm64-v8a`, `x86_64`).

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

---

### 🚀 Fase 8: Automatización CI/CD y Compilación Automatizada de APK Debug
- [x] Flujo de GitHub Actions (`.github/workflows/build-debug.yml`) para compilar APK Debug.
- [x] Script `generate_debug_keystore.sh` para forzar la creación de firmas debug desde cero sin esperar archivos o secretos externos.
- [x] Descarga e instalación automatizada de dependencias de C++ (NDK 27.2.12479018, CMake 3.22.1) y Rust (toolchain estable, targets móviles `aarch64`, `armv7`, `x86_64`, `i686` y `cargo fetch`).
- [x] Compilación obligatoria sin caché (`--no-build-cache --no-configuration-cache`) con publicación del artefacto `app-debug.apk`.

