#!/usr/bin/env python3
"""
==============================================================================
SCRIPT DE LIMPIEZA DE ARTEFACTOS BASURA DE C++ Y RUST (clean_native_artifacts.py)
==============================================================================

Elimina de forma segura y automatizada los archivos temporales y residuos
de compilación generados por:
1. Rust (carpeta target/ con los objetos compilados de aarch64, armv7, etc.)
2. C++ / CMake / Android NDK (carpetas .cxx/ y caché de Ninja/CMake)
3. Temporales de cargo y compilación en el sistema

Uso:
    python3 clean_native_artifacts.py
"""

import os
import shutil
import sys
from pathlib import Path


def get_size(path: Path) -> int:
    """Calcula el tamaño total en bytes de un directorio."""
    total = 0
    if path.is_file():
        return path.stat().st_size
    for root, _, files in os.walk(path):
        for f in files:
            fp = os.path.join(root, f)
            try:
                total += os.path.getsize(fp)
            except OSError:
                pass
    return total


def main():
    print("=" * 60)
    print("🧹 Iniciando limpieza de archivos basura de C++ y Rust")
    print("=" * 60)

    # Raíz del proyecto
    project_root = Path(__file__).resolve().parent

    # Lista de carpetas temporales y basura a remover
    targets_to_clean = [
        project_root / "app" / "src" / "main" / "rust" / "j2me_core" / "target",
        project_root / "app" / ".cxx",
        project_root / "app" / "build" / "intermediates" / "cxx",
        project_root / "app" / "build" / "intermediates" / "cmake",
        project_root / "app" / "build" / "intermediates" / "merged_native_libs",
        Path.home() / ".cargo" / "registry" / "cache",
    ]

    total_bytes_freed = 0
    cleaned_count = 0

    for target in targets_to_clean:
        if target.exists():
            size = get_size(target)
            total_bytes_freed += size
            size_mb = size / (1024 * 1024)
            print(f"🗑️  Eliminando: {target} ({size_mb:.2f} MB)")
            try:
                if target.is_dir():
                    shutil.rmtree(target)
                else:
                    target.unlink()
                cleaned_count += 1
            except Exception as e:
                print(f"⚠️  Error al eliminar {target}: {e}")
        else:
            print(f"✨ Ya limpio: {target}")

    # Limpiar temporales de cargo en /tmp
    tmp_path = Path("/tmp")
    if tmp_path.exists():
        for item in tmp_path.glob("cargo*"):
            try:
                if item.is_dir():
                    shutil.rmtree(item)
                else:
                    item.unlink()
            except Exception:
                pass

    freed_mb = total_bytes_freed / (1024 * 1024)
    print("=" * 60)
    print("✅ Limpieza completada con éxito.")
    print(f"📦 Total de ubicaciones limpiadas: {cleaned_count}")
    print(f"💾 Espacio liberado estimado: ~{freed_mb:.2f} MB")
    print("=" * 60)


if __name__ == "__main__":
    main()
