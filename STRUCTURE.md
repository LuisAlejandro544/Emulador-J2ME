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
├── clean_native_artifacts.sh      # Script Bash para purgar artefactos de C++ y Rust
├── clean_native_artifacts.py      # Script Python equivalente para purga
├── .gitignore                     # Filtros de exclusión de Git (blindado contra artefactos)
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
            │   └── com/example/
            │       ├── MainActivity.kt        # Actividad principal contenedora
            │       ├── ui/
            │       │   ├── J2meMenuScreen.kt  # Interfaz del emulador (pantalla retro y controles)
            │       │   ├── J2meNativeBridge.kt# Puente JNI que enlaza con C++ y Rust
            │       │   └── theme/             # Sistema de diseño y temas visuales
            │       └── javax/                 # (Próximo) Implementación de APIs J2ME en Java
            │           └── microedition/
            │               ├── midlet/        # Ciclo de vida MIDlet
            │               ├── lcdui/         # Gráficos y pantalla (Canvas, Graphics)
            │               ├── media/         # Audio y reproducción
            │               └── rms/           # Persistencia RecordStore
            │
            ├── cpp/               # Capa C++ (Abstracción de Hardware & JNI)
            │   ├── CMakeLists.txt # Script de compilación de CMake, invoca a Cargo y compila C++
            │   ├── native-lib.cpp # Implementación de métodos nativos JNI y llamadas FFI
            │   └── graphics/      # (Próximo) Renderizador nativo OpenGL ES
            │
            ├── rust/              # Capa Rust (Núcleo de la Máquina Virtual)
            │   └── j2me_core/
            │       ├── Cargo.toml # Definición del paquete Rust, cdylib y staticlib
            │       ├── .cargo/
            │       │   └── config.toml # Linkers de NDK para ARM32, ARM64, x86, x86_64
            │       └── src/
            │           ├── lib.rs # Punto de entrada FFI con C (`extern "C"`)
            │           ├── class/ # (Próximo) Parser de archivos .class y Constant Pool
            │           ├── vm/    # (Próximo) Intérprete de bytecode, pila y registros
            │           └── jar/   # (Próximo) Lector y descompresor de paquetes JAR
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
