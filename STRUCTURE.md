# Estructura del Proyecto (Architecture & Directory Tree)

Este documento detalla la estructura física y lógica de directorios del proyecto, explicando el rol de cada componente dentro del trío de lenguajes (**Java**, **Rust**, **C++**).

---

## 🌳 Árbol de Directorios Principal

```text
.
├── README.md                      # Documentación general y guía rápida
├── ROADMAP.md                     # Fases de desarrollo planificadas
├── STRUCTURE.md                   # Este documento (mapa del proyecto)
├── AI_CONTEXT.md                  # Contexto y directrices de IA
├── AGENTS.md                      # Reglas de comportamiento para agentes
├── generate_debug_keystore.sh     # Script que genera debug.keystore desde cero para CI y local
├── clean_native_artifacts.sh      # Script Bash para purgar artefactos de C++ y Rust
├── clean_native_artifacts.py      # Script Python equivalente para purga
├── .gitignore                     # Filtros de exclusión de Git (blindado contra artefactos)
├── .github/
│   └── workflows/
│       └── build-debug.yml        # Pipeline CI/CD GitHub Actions para compilar APK Debug sin caché
├── build.gradle.kts               # Configuración Gradle raíz
├── settings.gradle.kts            # Módulos del proyecto
│
└── app/
    ├── build.gradle.kts           # Configuración de compilación de Android, NDK y CMake
    └── src/
        └── main/
            ├── AndroidManifest.xml # Manifiesto de permisos y actividades
            │
            ├── java/              # Capa Java / Android
            │   ├── com/example/
            │   │   ├── MainActivity.kt        # Actividad principal contenedora
            │   │   ├── ui/
            │   │   │   ├── J2meMenuScreen.kt      # Biblioteca y explorador de juegos .jar
            │   │   │   ├── J2meEmulatorScreen.kt  # Pantalla de emulación LCDUI a 60 FPS y teclado retro
            │   │   │   ├── J2meMenuViewModel.kt   # ViewModel de gestión de biblioteca y estado activo
            │   │   │   ├── J2meNativeBridge.kt    # Declaraciones JNI que enlazan con C++ y Rust
            │   │   │   ├── J2meManifestInfo.kt    # Modelo de datos para metadatos MIDlet parseados
            │   │   │   └── theme/                 # Sistema de diseño y temas visuales
            │   │   ├── data/                      # Persistencia Room (J2meGame, Dao, Database, Repo)
            │   │   └── util/                      # Parser y utilidades SAF
            │   └── javax/                         # Implementación de APIs estándar J2ME en Java
            │       └── microedition/
            │           ├── midlet/                # MIDlet y MIDletStateChangeException
            │           ├── lcdui/                 # Canvas, Graphics, Display, Image, Font, Command
            │           ├── media/                 # (Próximo) Audio y reproducción
            │           └── rms/                   # (Próximo) Persistencia RecordStore
            │
            ├── cpp/               # Capa C++ (Abstracción de Hardware, JNI & Rasterizador)
            │   ├── CMakeLists.txt # Script de compilación CMake, integra Cargo, C++ y jnigraphics
            │   ├── native-lib.cpp # Implementación de métodos nativos JNI, FFI y puente gráfico
            │   ├── framebuffer.h  # Definición del Framebuffer nativo ARGB8888 y primitivas 2D
            │   └── framebuffer.cpp# Implementación del rasterizador 2D, Bresenham y volcado Bitmap
            │
            ├── rust/              # Capa Rust (Núcleo de la Máquina Virtual)
            │   └── j2me_core/
            │       ├── Cargo.toml # Definición del paquete Rust con miniz_oxide
            │       ├── .cargo/
            │       │   └── config.toml # Linkers de NDK para ARM32, ARM64, x86, x86_64
            │       └── src/
            │           ├── lib.rs          # Punto de entrada FFI con C (`extern "C"`) y runtime global
            │           ├── jar_parser.rs   # Parser seguro de archivos JAR, ZIP y MANIFEST.MF
            │           ├── class_parser.rs # Parser binario de archivos .class, Constant Pool y Code
            │           └── vm/             # Arquitectura modular de la Máquina Virtual CLDC
            │               ├── mod.rs      # Re-exportaciones y tests de integración
            │               ├── types.rs    # Definiciones de Value, VmError, ExecutionResult y ArrayType
            │               ├── stack.rs    # Pila de operandos (OperandStack) y variables locales (LocalVariables)
            │               ├── heap.rs     # Gestor de memoria dinámica (Heap), objetos y arrays tipados
            │               ├── frame.rs    # Marco de activación (StackFrame), Constant Pool y opcodes
            │               └── runtime.rs  # Máquina Virtual (VirtualMachine), Call Stack y built-ins nativos
            │
            └── res/               # Recursos de interfaz (iconos, temas, layouts)
```

---

## 🔄 Flujo de Interacción entre Componentes

```text
┌─────────────────────────────────────────────────────────────┐
│                    Entorno Android (UI)                     │
│  - Captura toques en teclado virtual (D-Pad, 0-9, SoftKeys) │
│  - Presenta la pantalla de visualización                    │
└──────────────────────────────┬──────────────────────────────┘
                               │ JNI (Java Native Interface)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                       Capa C++ (NDK)                        │
│  - Recibe llamadas JNI y eventos de teclado                 │
│  - Gestiona el búfer de píxeles y aceleración gráfica       │
│  - Controla el audio de baja latencia                       │
└──────────────────────────────┬──────────────────────────────┘
                               │ FFI (Foreign Function Interface)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                   Núcleo Rust (j2me_core)                   │
│  - Parsea el bytecode de los juegos J2ME (.class)           │
│  - Ejecuta el ciclo de instrucciones de la máquina virtual  │
│  - Administra la memoria de forma segura y sin riesgos      │
└─────────────────────────────────────────────────────────────┘
```
