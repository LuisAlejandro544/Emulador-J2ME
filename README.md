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
- **Parser de Paquetes JAR en Rust**: Descompresión en memoria segura (Deflate RFC 1951 con `miniz_oxide`) e inspección instantánea de `META-INF/MANIFEST.MF` (nombre, versión, autor, icono y clase de inicio `MIDlet-1`).
- **Parser Binario de Clases Java (.class)**: Decodificación estricta y segura del formato binario ClassFile (`0xCAFEBABE`), Constant Pool tipado (strings, enteros, floats, clases, métodos), tabla de métodos y extracción de secuencias de bytecode (`Code`).
- **Núcleo de Ejecución JVM CLDC (Rust)**: Máquina virtual completa y modular en Rust:
  - Pila de operandos (`OperandStack`) con protección estricta contra desbordamientos.
  - Tabla de variables locales (`LocalVariables`) con indexación segura.
  - Gestor de memoria dinámica (`Heap`): asignación de objetos (`ObjectInstance`), campos dinámicos y arrays tipados (`ArrayInstance`) con control estricto de límites (`ArrayIndexOutOfBounds`).
  - Marco de activación (`StackFrame`): resolución del Constant Pool tipado, despacho de más de 45 opcodes (aritmética, saltos, manipulación de pila, objetos, arrays y campos estáticos/de instancia).
  - Pila de llamadas (`VirtualMachine`): resolución y ejecución de llamadas anidadas a métodos (`invokevirtual`, `invokestatic`, `invokespecial`), soporte para métodos built-in (`Object.<init>`, `MIDlet.<init>`, `System.currentTimeMillis`, `Math`) y estadísticas en tiempo real en formato JSON.
- **Control Táctil Fiel y Pantalla Retro**: Interfaz con pantalla LCDUI a 60 FPS estables, teclado alfanumérico retro (0-9, *, #), D-Pad direccional, tecla de acción central (FIRE) y SoftKeys de menú, con soporte táctil directo (`pointerPressed` / `pointerDragged`).
- **Cargador Real de Juegos y Ejecución en VM**: Integración en Compose (`J2meEmulatorScreen.kt`) del cargador real que lee los bytes del archivo `.jar` seleccionado, inicializa el motor nativo, transfiere los datos a la VM en Rust y ejecuta el ciclo de vida del `MIDlet` (`<init>` y `startApp`).
- **Carga Automática de Clases en la VM (ClassLoader en Rust)**: La máquina virtual resuelve y carga clases `.class` bajo demanda directamente desde el contenedor JAR en memoria cuando son requeridas por `invokevirtual`, `invokestatic` o `new`, eliminando la necesidad de precarga manual de clases.
- **Soporte Multihilo y Bucle de Juego (Thread / Runnable)**: Soporte completo en la VM de Rust para instanciar `java/lang/Thread` con objetos `Runnable`, despachar `Thread.start()` ejecutando el método `run()` y gestionar retardos no bloqueantes con `Thread.sleep()`.
- **Interfaz a Pantalla Completa y Teclado Alfanumérico Retro**: Pantalla LCDUI rediseñada para ocupar el máximo espacio vertical en smartphones, con teclado virtual de teclas grandes que incorpora las letras de cada tecla (2: ABC, 3: DEF, etc.), D-Pad y SoftKeys ergonómicas.
- **Subsistema Gráfico LCDUI y Rasterizador C++**:
  - Implementación completa de la especificación `javax.microedition.lcdui.*` (`Display`, `Canvas`, `Graphics`, `Image`, `Font`) y `javax.microedition.midlet.*` (`MIDlet`).
  - Motor rasterizador 2D en C++ con Framebuffer ARGB8888, sincronización multihilo con mutex y primitivas aceleradas (líneas con algoritmo Bresenham, rectángulos, arcos, clipping y fuente bitmap 8x8 integrada).
  - Volcado directo de alto rendimiento a `android.graphics.Bitmap` mediante `jnigraphics` (`AndroidBitmap_lockPixels`).
  - Método `Graphics.drawRegion` con soporte para rotaciones (90°, 180°, 270°) y espejos horizontales/verticales.
- **API Completa de Juegos MIDP 2.0 (`javax.microedition.lcdui.game.*`)**:
  - `GameCanvas`: Lienzo de juego de alto rendimiento con doble búfer off-screen (`flushGraphics`) y lectura instantánea de estado de teclado (`getKeyStates`).
  - `Sprite`: Sprites animados con secuencias dinámicas de cuadros, rotaciones ortogonales, espejos y detección precisa de colisiones (caja envolvente AABB y colisión píxel-perfect por canal alfa).
  - `TiledLayer`: Motor de escenarios y mapas en cuadrícula con soporte para tiles estáticos y animados, optimizado para desplazamiento y renderizado por frustum clipping.
  - `LayerManager`: Gestor de profundidad Z-order para composición de capas y cámara de visualización (`setViewWindow`).
- **Carga de Recursos Internos de Juegos (`getResourceAsStream`)**:
  - Carga directa y transparente de activos internos empaquetados en el JAR (texturas de sprites `.png`, mapas binarios `.dat`/`.bin`, tablas de puntuación y música).
  - Soporte universal mediante `J2meResourceManager`, `MIDlet.getResourceAsStream()` y `Class.getResourceAsStream()`.
  - Integración en la VM de Rust con asignación en Heap de `ByteArrayInputStream` y manejo de `InputStream.read`, `available`, `skip` y `close`.
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

## 🤖 Integración Continua (CI/CD) y GitHub Actions

El proyecto incluye un flujo automatizado de compilación en `.github/workflows/build-debug.yml`:

- **Descarga completa de código y dependencias**: Instala JDK 17, Android NDK `27.2.12479018`, CMake `3.22.1`, el toolchain de Rust con targets Android (`aarch64`, `armv7`, `x86_64`, `i686`) y ejecuta `cargo fetch`.
- **Firma Debug generada desde cero**: Ejecuta el script `./generate_debug_keystore.sh`, creando un keystore debug autofirmado al vuelo sin requerir firmas almacenadas ni secretos.
- **Compilación sin caché**: Ejecuta Gradle con las banderas `--no-build-cache` y `--no-configuration-cache` para garantizar una compilación 100% limpia y reproducible.
- **Artefacto generado**: Sube el archivo `app-debug.apk` directamente a los artefactos de la ejecución en GitHub Actions.

Para generar la firma debug de forma local o en cualquier entorno:

```bash
chmod +x generate_debug_keystore.sh
./generate_debug_keystore.sh
```

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
