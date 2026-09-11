package com.example.ui

import org.json.JSONObject

/**
 * Representa los metadatos parseados desde META-INF/MANIFEST.MF de un paquete JAR de J2ME.
 */
data class J2meManifestInfo(
    val midletName: String,
    val vendor: String,
    val version: String,
    val profile: String,
    val configuration: String,
    val mainClass: String,
    val iconPath: String,
    val filesCount: Int
) {
    companion object {
        fun fromJson(jsonStr: String): J2meManifestInfo {
            val json = JSONObject(jsonStr)
            return J2meManifestInfo(
                midletName = json.optString("midletName", "Juego J2ME"),
                vendor = json.optString("vendor", "Desconocido"),
                version = json.optString("version", "1.0"),
                profile = json.optString("profile", "MIDP-2.0"),
                configuration = json.optString("configuration", "CLDC-1.1"),
                mainClass = json.optString("mainClass", ""),
                iconPath = json.optString("iconPath", ""),
                filesCount = json.optInt("filesCount", 0)
            )
        }
    }
}
