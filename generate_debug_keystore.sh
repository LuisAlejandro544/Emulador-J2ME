#!/usr/bin/env bash
# ==============================================================================
# SCRIPT: GENERAR KEYSTORE DEBUG DESDE CERO
# ==============================================================================
# Este script obliga a generar un archivo debug.keystore nuevo y autofirmado
# en la raíz del proyecto, sin esperar firmas externas ni pedir contraseñas.
#
# Es invocado directamente por el flujo de CI/CD (GitHub Actions) o localmente.
# ==============================================================================

set -euo pipefail

# Obtener directorio raíz del proyecto
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KEYSTORE_FILE="${PROJECT_ROOT}/debug.keystore"

echo "=========================================================="
echo "🔐 [Keystore Debug] Iniciando generación de firma desde cero"
echo "=========================================================="

# Si existe un debug.keystore previo, eliminarlo para forzar regeneración limpia
if [ -f "${KEYSTORE_FILE}" ]; then
    echo "⚠️  Keystore existente detectado. Eliminando para regenerar desde cero..."
    rm -f "${KEYSTORE_FILE}"
fi

# Generar keystore debug estándar con keytool
# Parámetros estándar de Android debug keystore:
# - keystore: debug.keystore
# - storepass: android
# - keypass: android
# - alias: androiddebugkey
# - dname: CN=Android Debug,O=Android,C=US
# - keyalg: RSA, keysize: 2048, validity: 10000 días
echo "⚙️  Generando nuevo debug.keystore con algoritmo RSA 2048 bits..."

keytool -genkeypair \
    -v \
    -keystore "${KEYSTORE_FILE}" \
    -alias androiddebugkey \
    -keypass android \
    -storepass android \
    -keyalg RSA \
    -keysize 2048 \
    -validity 10000 \
    -dname "CN=Android Debug,O=Android,C=US"

if [ -f "${KEYSTORE_FILE}" ]; then
    FILE_SIZE=$(wc -c < "${KEYSTORE_FILE}")
    echo "✅ Keystore debug generado exitosamente:"
    echo "   📍 Ruta: ${KEYSTORE_FILE}"
    echo "   📦 Tamaño: ${FILE_SIZE} bytes"
    echo "   🔑 Alias: androiddebugkey"
    echo "   🔒 Pass: android"
else
    echo "❌ Error crítico: No se pudo generar ${KEYSTORE_FILE}"
    exit 1
fi

echo "=========================================================="
echo "🚀 Keystore listo. La compilación de APK Debug puede proceder."
echo "=========================================================="
