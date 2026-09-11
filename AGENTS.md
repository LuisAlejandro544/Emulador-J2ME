# Guía de Comportamiento para Agentes (AGENTS.md)

Este documento define las reglas operativas y de desarrollo que los agentes de IA deben seguir estrictamente en este proyecto.

---

## 🛑 Reglas Absolutas e Invariantes

1. **Razonar Profundamente antes de Actuar**:
   - Analizar el impacto de cada cambio antes de tocar el código.
   - Respetar siempre el trío técnico: **Java** para el estándar/APIs de los juegos J2ME, **Rust** para el núcleo JVM/CLDC y memoria segura, y **C++** para el puente JNI, gráficos nativos y audio.

2. **Entorno del Usuario**:
   - El usuario programa y utiliza este proyecto directamente desde un teléfono móvil, no desde una PC de escritorio. Las respuestas deben ser directas, claras, bien estructuradas y fáciles de leer en pantalla vertical.

3. **Inclusión Obligatoria en el Build System**:
   - Cuando se utilice C++, Rust u otros componentes nativos, **deben estar 100% integrados en el pipeline de Gradle y CMake**.
   - No se deben dejar pasos manuales de compilación fuera del ciclo de Gradle.
   - No reemplazar código o frameworks solicitados con funciones sustitutas (*fallbacks*) a menos que el usuario lo solicite expresamente.

4. **Soporte de Arquitecturas**:
   - Asegurarse de que el código nativo contemple arquitecturas de **32 bits** (`armeabi-v7a`) y **64 bits** (`arm64-v8a`, `x86_64`).

5. **Limpieza Continua de Archivos Basura**:
   - Mantener las carpetas `target/` de Rust, `.cxx/` de CMake y archivos intermedios de compilación fuera del repositorio y del control de versiones.
   - Utilizar y mantener actualizados los scripts `clean_native_artifacts.sh` y `clean_native_artifacts.py`.

6. **Información de Commits**:
   - Si existe un archivo `commit_message.txt`, su contenido debe redactarse siempre en **español** y no debe modificarse a menos que el usuario lo pida.

7. **Prohibición de Propiedades Restringidas**:
   - En caso de optimizaciones o boosters de rendimiento, **nunca** utilizar propiedades del sistema con prefijo `persist.sys.*`.

8. **Dependencias Funcionales**:
   - Priorizar dependencias y librerías que sean 100% funcionales y probadas sobre implementaciones artesanales propensas a fallos.
   - No usar dependencias con licencias que obliguen al proyecto a ser de código abierto o que exijan atribuciones legales forzadas.
