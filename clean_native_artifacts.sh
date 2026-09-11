#!/usr/bin/env bash
# ==============================================================================
# SCRIPT DE LIMPIEZA DE ARCHIVOS BASURA DE C++ Y RUST
# ==============================================================================
# Este script elimina de forma segura los archivos temporales y artefactos
# de compilación generados por:
# 1. Rust Cargo: carpeta 'target/' (aarch64, armv7, x86_64, cache de objetos).
# 2. CMake y C++ NDK: carpetas '.cxx/', 'build/intermediates/cxx/'.
# 3. Archivos temporales de instalación y caché de paquetes.
#
# Uso:
#   bash clean_native_artifacts.sh
#   o
#   chmod +x clean_native_artifacts.sh && ./clean_native_artifacts.sh
# ==============================================================================

set -euo pipefail

echo "======================================================"
echo "🧹 Iniciando limpieza de artefactos basura de C++ y Rust"
echo "======================================================"

# Directorio raíz del proyecto
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${PROJECT_ROOT}"

# Lista de directorios de compilación intermedia y basura a eliminar
TARGET_DIRS=(
    "${PROJECT_ROOT}/app/src/main/rust/j2me_core/target"
    "${PROJECT_ROOT}/app/.cxx"
    "${PROJECT_ROOT}/app/build/intermediates/cxx"
    "${PROJECT_ROOT}/app/build/intermediates/cmake"
    "${PROJECT_ROOT}/app/build/intermediates/merged_native_libs"
    "${HOME}/.cargo/registry/cache"
)

TOTAL_FREED=0

for dir in "${TARGET_DIRS[@]}"; do
    if [ -d "${dir}" ]; then
        SIZE_KB=$(du -sk "${dir}" 2>/dev/null | cut -f1 || echo "0")
        TOTAL_FREED=$((TOTAL_FREED + SIZE_KB))
        echo "🗑️  Eliminando: ${dir} (~${SIZE_KB} KB)"
        rm -rf "${dir}"
    else
        echo "✨ Ya limpio: ${dir}"
    fi
done

# Eliminar archivos temporales de compiladores en /tmp si existen
echo "🧹 Limpiando temporales en /tmp..."
find /tmp -maxdepth 1 -name "cargo*" -exec rm -rf {} + 2>/dev/null || true
find /tmp -maxdepth 1 -name "rust*" -exec rm -rf {} + 2>/dev/null || true

FREED_MB=$(awk "BEGIN {printf \"%.2f\", ${TOTAL_FREED}/1024}")

echo "======================================================"
echo "✅ Limpieza completada con éxito."
echo "📦 Espacio liberado estimado: ~${FREED_MB} MB"
echo "======================================================"
