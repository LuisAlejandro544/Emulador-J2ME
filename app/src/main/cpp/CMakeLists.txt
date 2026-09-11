cmake_minimum_required(VERSION 3.22.1)
project("j2me_native")

# ==============================================================================
# CONFIGURACIÓN DEL MOTOR NATIVO J2ME (C++ y RUST)
# ==============================================================================
# Este archivo CMake configura:
# 1. Compilación del código nativo en C++ (interfaz JNI, gráficos y audio nativo).
# 2. Compilación y vinculación con la librería estática generada por Rust (j2me_core).
# 3. Soporte para arquitecturas de 32 bits (armeabi-v7a) y 64 bits (arm64-v8a, x86_64).
# ==============================================================================

# Determinación del target de Rust según la arquitectura Android (ABI)
if(ANDROID_ABI STREQUAL "arm64-v8a")
    set(RUST_TARGET "aarch64-linux-android")
    set(RUST_LINKER "${ANDROID_NDK}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang")
    set(RUST_LINKER_ENV "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=${RUST_LINKER}")
elseif(ANDROID_ABI STREQUAL "armeabi-v7a")
    set(RUST_TARGET "armv7-linux-androideabi")
    set(RUST_LINKER "${ANDROID_NDK}/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi24-clang")
    set(RUST_LINKER_ENV "CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER=${RUST_LINKER}")
elseif(ANDROID_ABI STREQUAL "x86_64")
    set(RUST_TARGET "x86_64-linux-android")
    set(RUST_LINKER "${ANDROID_NDK}/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android24-clang")
    set(RUST_LINKER_ENV "CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER=${RUST_LINKER}")
elseif(ANDROID_ABI STREQUAL "x86")
    set(RUST_TARGET "i686-linux-android")
    set(RUST_LINKER "${ANDROID_NDK}/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android24-clang")
    set(RUST_LINKER_ENV "CARGO_TARGET_I686_LINUX_ANDROID_LINKER=${RUST_LINKER}")
else()
    message(FATAL_ERROR "Arquitectura no soportada: ${ANDROID_ABI}")
endif()

set(RUST_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../rust/j2me_core")
set(RUST_LIB_PATH "${RUST_DIR}/target/${RUST_TARGET}/release/libj2me_core.a")

# Configuración dinámica del entorno de Rust (evita hardcodear /root que provoca error de permisos en CI y multiusuario)
if(DEFINED ENV{HOME} AND NOT "$ENV{HOME}" STREQUAL "")
    set(EFFECTIVE_HOME "$ENV{HOME}")
else()
    set(EFFECTIVE_HOME "/root")
endif()

if(DEFINED ENV{CARGO_HOME} AND NOT "$ENV{CARGO_HOME}" STREQUAL "")
    set(EFFECTIVE_CARGO_HOME "$ENV{CARGO_HOME}")
else()
    set(EFFECTIVE_CARGO_HOME "${EFFECTIVE_HOME}/.cargo")
endif()

if(DEFINED ENV{RUSTUP_HOME} AND NOT "$ENV{RUSTUP_HOME}" STREQUAL "")
    set(EFFECTIVE_RUSTUP_HOME "$ENV{RUSTUP_HOME}")
else()
    set(EFFECTIVE_RUSTUP_HOME "${EFFECTIVE_HOME}/.rustup")
endif()

# Detección de fuentes en Rust para recompilación automática cuando cambien
file(GLOB_RECURSE RUST_SOURCES "${RUST_DIR}/src/*.rs" "${RUST_DIR}/Cargo.toml")

# Comando para invocar Cargo y compilar el núcleo en Rust de manera automática
add_custom_command(
    OUTPUT "${RUST_LIB_PATH}"
    COMMAND ${CMAKE_COMMAND} -E env "HOME=${EFFECTIVE_HOME}" "RUSTUP_HOME=${EFFECTIVE_RUSTUP_HOME}" "CARGO_HOME=${EFFECTIVE_CARGO_HOME}" "PATH=${EFFECTIVE_CARGO_HOME}/bin:/root/.cargo/bin:/usr/local/bin:$ENV{PATH}" "${RUST_LINKER_ENV}" cargo build --manifest-path "${RUST_DIR}/Cargo.toml" --target "${RUST_TARGET}" --release
    WORKING_DIRECTORY "${RUST_DIR}"
    DEPENDS ${RUST_SOURCES}
    COMMENT "Compilando núcleo J2ME en Rust para arquitectura ${ANDROID_ABI} (${RUST_TARGET})..."
    VERBATIM
)

add_custom_target(
    build_rust_core
    DEPENDS "${RUST_LIB_PATH}"
)

# Declaración de la librería estática de Rust para ser enlazada en C++
add_library(j2me_rust_core STATIC IMPORTED GLOBAL)
set_target_properties(j2me_rust_core PROPERTIES IMPORTED_LOCATION "${RUST_LIB_PATH}")

# Librería nativa compartida (C++) que contiene el puente JNI y el rasterizador para Android
add_library(
    j2me_native
    SHARED
    native-lib.cpp
    framebuffer.cpp
)

add_dependencies(j2me_native build_rust_core)

# Configuración del estándar C++ moderno
set_target_properties(j2me_native PROPERTIES
    CXX_STANDARD 17
    CXX_STANDARD_REQUIRED ON
)

target_include_directories(
    j2me_native
    PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# Localización de las librerías nativas del sistema Android
find_library(log-lib log)
find_library(android-lib android)
find_library(jnigraphics-lib jnigraphics)

# Enlace final: C++ + Rust + Librerías del sistema Android
target_link_libraries(
    j2me_native
    j2me_rust_core
    ${log-lib}
    ${android-lib}
    ${jnigraphics-lib}
)
