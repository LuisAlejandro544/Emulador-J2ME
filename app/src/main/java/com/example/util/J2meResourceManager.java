package com.example.util;

import java.io.ByteArrayInputStream;
import java.io.InputStream;
import com.example.ui.J2meNativeBridge;

/**
 * ============================================================================
 * GESTOR DE RECURSOS INTERNOS DE J2ME (J2meResourceManager)
 * ============================================================================
 *
 * Provee acceso directo y transparente a los archivos internos del juego (JAR),
 * tales como texturas, mapas (.dat, .bin), sonidos (.mid, .wav) y datos de niveles.
 *
 * Emula fielmente el comportamiento de `java.lang.Class.getResourceAsStream`:
 * - Admite rutas absolutas con barra inicial ("/maps/level1.dat").
 * - Admite rutas relativas ("data/sprites.png").
 * - Normaliza automáticamente las rutas y consulta el contenedor JAR nativo en Rust.
 * - Retorna un flujo `InputStream` binario reutilizable o `null` si no existe.
 */
public final class J2meResourceManager {

    private J2meResourceManager() {}

    /**
     * Carga un recurso interno del JAR como un InputStream.
     *
     * @param resourcePath Ruta del recurso interno (ej: "/levels/level1.bin" o "icon.png").
     * @return InputStream con el contenido del archivo o null si no fue encontrado.
     */
    public static InputStream getResourceAsStream(String resourcePath) {
        if (resourcePath == null || resourcePath.trim().isEmpty()) {
            return null;
        }

        byte[] data = getResourceAsBytes(resourcePath);
        if (data != null && data.length > 0) {
            return new ByteArrayInputStream(data);
        }

        // Intento de fallback mediante ClassLoader de Java estándar
        return J2meResourceManager.class.getResourceAsStream(resourcePath);
    }

    /**
     * Extrae los bytes directos de un recurso interno del JAR a través del puente nativo Rust.
     */
    public static byte[] getResourceAsBytes(String resourcePath) {
        if (resourcePath == null) return null;

        String cleanPath = resourcePath.trim();
        // 1. Consulta directa con la ruta provista
        byte[] data = J2meNativeBridge.INSTANCE.extractJarResource(cleanPath);

        // 2. Si tenía barra inicial y falló, probar sin barra
        if (data == null && cleanPath.startsWith("/")) {
            data = J2meNativeBridge.INSTANCE.extractJarResource(cleanPath.substring(1));
        }

        // 3. Si no tenía barra inicial y falló, probar con barra
        if (data == null && !cleanPath.startsWith("/")) {
            data = J2meNativeBridge.INSTANCE.extractJarResource("/" + cleanPath);
        }

        return data;
    }
}
