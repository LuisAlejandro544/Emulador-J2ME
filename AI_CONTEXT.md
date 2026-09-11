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
