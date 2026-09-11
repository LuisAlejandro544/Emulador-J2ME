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

### ⏳ Fase 2: Parser de Archivos JAR/JAD y Formato .class (En Progreso)
- [ ] Lector de archivos ZIP/JAR en Rust para descomprimir y listar entradas.
- [ ] Parser del manifiesto `MANIFEST.MF` (detección de `MIDlet-1`, versión CLDC/MIDP y tamaño).
- [ ] Parser binario de archivos `.class` de Java:
  - Constant Pool (Utf8, Integer, Float, Methodref, Fieldref, Class).
  - Tabla de métodos, firmas y atributos `Code`.
  - Pila de operandos y variables locales de la JVM.

---

### 🔄 Fase 3: Intérprete de Bytecode y Máquina Virtual CLDC (Rust)
- [ ] Bucle principal de ejecución de opcodes (instrucciones estándar de JVM de 1 byte).
- [ ] Soporte de instrucciones aritméticas, de salto condicional (`ifeq`, `if_icmpne`, etc.) y de pila (`iload`, `istore`, `dup`, `swap`).
- [ ] Manejo de llamadas a métodos (`invokevirtual`, `invokestatic`, `invokespecial`).
- [ ] Asignador de memoria para objetos e instancias de clases de forma segura.

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
