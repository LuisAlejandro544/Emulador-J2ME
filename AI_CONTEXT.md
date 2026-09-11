# AI Context: Emulador J2ME Retro

Este documento contiene el contexto de dominio, restricciones arquitectónicas e invariantes que cualquier modelo de inteligencia artificial o asistente debe respetar al interactuar con este repositorio.

---

## 📌 Contexto del Dominio J2ME (Java Micro Edition)

1. **Java es el Entorno de Ejecución Objetivo**:
   - En la época original de los teléfonos móviles (era de Nokia, Sony Ericsson, Motorola con Symbian, Series 40, etc.), los juegos se escribieron estrictamente en **Java** bajo las especificaciones **CLDC 1.0/1.1** y **MIDP 1.0/2.0**.
   - No se debe asumir que el código de emulación del juego deba reescribirse o reemplazarse por Kotlin. La lógica de clases del juego debe interpretarse o emularse fiel al estándar de Java de esa época.

2. **Rol Estricto de los Tres Lenguajes**:
   - **Java**: Se usa para definir las APIs originales de J2ME que los juegos esperan (`javax.microedition.*`) y el contenedor de aplicaciones.
   - **Rust**: Se encarga del parser seguro de contenedores JAR/ZIP (`jar_parser.rs`), extracción de `MANIFEST.MF`, intérprete de la máquina virtual (JVM/CLDC), procesamiento de bytecode, parser de clases binarias y control seguro de memoria.
   - **C++**: Se encarga del renderizado gráfico nativo de alta velocidad (OpenGL ES), subsistema de audio nativo (AAudio/Oboe) y la interfaz JNI con Android.

3. **Arquitecturas y Compatibilidad**:
   - Se debe soportar tanto **32 bits** (`armeabi-v7a`, `i686`) como **64 bits** (`arm64-v8a`, `x86_64`).
   - Todo código nativo en C++ y Rust debe compilarse limpiamente para ambas clases de procesadores.

4. **Regla de Cero Archivos Basura**:
   - La carpeta `target/` de Rust y las carpetas `.cxx/` o `build/` de C++ intermedias generan cientos de megabytes en binarios temporales.
   - No deben incluirse en el control de versiones (`.gitignore` debe mantenerlas bloqueadas).
   - Siempre que se añadan nuevas herramientas o compilaciones, debe verificarse que los artefactos no queden expuestos.

5. **Licencias y Privacidad**:
   - No utilizar librerías de terceros que obliguen a usar licencias virales restrictivas como GPLv3 que fuercen la apertura del código si el usuario no lo desea, ni librerías que requieran atribuciones forzadas.
   - No incluir nombres comerciales registrados o marcas protegidas por derechos de autor que puedan representar un riesgo legal.

6. **Estado del Motor y Máquina Virtual**:
   - El núcleo en Rust tiene completado el parser de JAR/ZIP (`jar_parser`), el parser de ClassFile (`class_parser`), la memoria dinámica Heap (`vm/heap`), el despacho de opcodes e invocación de métodos (`vm/frame`), y la Máquina Virtual con Call Stack (`vm/runtime`).
   - Dispone de **ClassLoader automático en Rust**: la VM carga y parsea archivos `.class` bajo demanda desde los bytes del JAR en memoria ante llamadas a métodos o instanciaciones no resueltas previamente.
   - Cuenta con **Soporte Multihilo y Bucle de Juego (Thread / Runnable)**: intercepción de `java/lang/Thread`, asociación de `Runnable`, despacho de `start()` hacia `run()` y `Thread.sleep()` con tope de 20ms para no bloquear el hilo de interfaz.
   - Todo cambio o adición posterior debe respetar la modularidad de `vm/` y mantener las llamadas FFI/JNI sincronizadas con `native-lib.cpp` y `J2meNativeBridge.kt`.

7. **Subsistema Gráfico, Cargador Real y Teclado a Pantalla Completa**:
   - La capa Java proporciona la API estándar de J2ME (`javax.microedition.lcdui.*` y `midlet.*`), implementando `Canvas`, `Graphics`, `Display`, `Image`, `Font` y `Command`.
   - Las operaciones de rasterizado pesado (Bresenham para líneas, rectángulos, arcos, clipping y texto con fuente bitmap 8x8) se ejecutan en C++ con un `Framebuffer` nativo ARGB8888 sincronizado con mutex.
   - La pantalla Compose `J2meEmulatorScreen` implementa el **Cargador Real**: lee el archivo `.jar` desde SAF/URI, lo envía a la VM en Rust y ejecuta el ciclo de vida del `MIDlet` (`<init>` y `startApp`).
   - El layout maximiza el área vertical del teléfono móvil: el Framebuffer LCDUI 240x320 se renderiza casi a pantalla completa, acompañado de un teclado virtual táctil grande con rotulación de letras alfanuméricas reales (1..9 con ABC, DEF, etc.), D-Pad y SoftKeys ergonómicas.

8. **Pipeline CI/CD y Generación Automática de Firmas Debug**:
   - `.github/workflows/build-debug.yml` implementa el flujo oficial para compilar el APK Debug completo en GitHub Actions sin requerir caché (`--no-build-cache --no-configuration-cache`).
   - El script `generate_debug_keystore.sh` genera un `debug.keystore` autofirmado desde cero sin requerir archivos externos ni contraseñas interactivas, garantizando compilaciones desatendidas y reproducibles tanto en CI como en entornos locales.
   - `CMakeLists.txt` resuelve dinámicamente las rutas de `$HOME`, `$CARGO_HOME` y `$RUSTUP_HOME` según el usuario en ejecución, evitando fallos de permisos (`Permission denied /root/.rustup`) en runners no-root como GitHub Actions (`runner`).

9. **API de Juegos MIDP 2.0 y Carga de Recursos Internos**:
   - Soporte completo del paquete `javax.microedition.lcdui.game.*`: `Layer` (base), `Sprite` (animación, 8 transformaciones y colisiones AABB/píxel-perfect), `TiledLayer` (escenarios en cuadrícula, tiles estáticos/animados), `LayerManager` (cámara de visualización y Z-order) y `GameCanvas` (doble búfer off-screen y sondeo instantáneo `getKeyStates`).
   - Carga transparente de recursos del juego empaquetados en el JAR (`getResourceAsStream`), soportada en `Class`, `MIDlet` y `J2meResourceManager`, con contraparte en Rust mediante streams seguros (`ByteArrayInputStream`).
   - `Graphics.drawRegion`: recorte y renderizado con rotaciones ortogonales y espejos.

