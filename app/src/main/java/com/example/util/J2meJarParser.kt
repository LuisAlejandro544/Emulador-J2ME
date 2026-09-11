package com.example.util

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import com.example.data.J2meGame
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.FileOutputStream
import java.io.InputStream
import java.util.jar.Manifest
import java.util.zip.ZipEntry
import java.util.zip.ZipInputStream

/**
 * Resultado estructurado de la lectura y extracción de metadatos de un archivo .jar J2ME.
 */
data class ParsedJarInfo(
    val title: String,
    val vendor: String,
    val version: String,
    val mainClass: String,
    val fileName: String,
    val fileSizeBytes: Long,
    val iconPath: String?
)

/**
 * Analizador y extractor para paquetes J2ME (.jar).
 *
 * Lee el archivo de manifiesto (META-INF/MANIFEST.MF) según la especificación MIDP
 * para obtener el nombre del juego, autor, versión y punto de entrada (MIDlet-1).
 * También extrae y almacena localmente el icono del juego si está presente en el archivo .jar.
 */
object J2meJarParser {

    /**
     * Procesa un archivo .jar desde su URI de almacenamiento y retorna un objeto [J2meGame]
     * listo para ser guardado en la base de datos local.
     */
    suspend fun parseJarFromUri(context: Context, uri: Uri): J2meGame = withContext(Dispatchers.IO) {
        val contentResolver = context.contentResolver

        // 1. Obtener nombre del archivo y tamaño original desde el ContentResolver
        var fileName = "juego.jar"
        var fileSize = 0L

        contentResolver.query(uri, null, null, null, null)?.use { cursor ->
            if (cursor.moveToFirst()) {
                val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                val sizeIndex = cursor.getColumnIndex(OpenableColumns.SIZE)
                if (nameIndex != -1) {
                    val name = cursor.getString(nameIndex)
                    if (!name.isNullOrBlank()) {
                        fileName = name
                    }
                }
                if (sizeIndex != -1) {
                    fileSize = cursor.getLong(sizeIndex)
                }
            }
        }

        // 2. Extraer metadatos y posible icono desde el contenido ZIP del .jar
        val parsedInfo = parseZipStream(
            context = context,
            inputStreamProvider = { contentResolver.openInputStream(uri) },
            defaultFileName = fileName,
            defaultFileSize = fileSize
        )

        return@withContext J2meGame(
            title = parsedInfo.title,
            vendor = parsedInfo.vendor,
            version = parsedInfo.version,
            mainClass = parsedInfo.mainClass,
            fileUri = uri.toString(),
            fileName = parsedInfo.fileName,
            fileSizeBytes = parsedInfo.fileSizeBytes,
            iconPath = parsedInfo.iconPath,
            addedTimestamp = System.currentTimeMillis()
        )
    }

    /**
     * Lee las entradas del archivo ZIP (.jar) para encontrar el manifiesto y los iconos del juego.
     */
    private fun parseZipStream(
        context: Context,
        inputStreamProvider: () -> InputStream?,
        defaultFileName: String,
        defaultFileSize: Long
    ): ParsedJarInfo {
        var midletName: String? = null
        var midletVendor: String? = null
        var midletVersion: String? = null
        var midlet1Raw: String? = null
        var midletIconPath: String? = null

        var iconBytes: ByteArray? = null
        var targetIconName: String? = null

        // Primera pasada: extraer META-INF/MANIFEST.MF
        val inputStream1 = inputStreamProvider()
        if (inputStream1 != null) {
            ZipInputStream(inputStream1).use { zis ->
                var entry: ZipEntry? = zis.nextEntry
                while (entry != null) {
                    val entryName = entry.name.replace('\\', '/')
                    if (entryName.equals("META-INF/MANIFEST.MF", ignoreCase = true)) {
                        val manifestBytes = readBytesFromEntry(zis)
                        try {
                            val manifest = Manifest(manifestBytes.inputStream())
                            val attributes = manifest.mainAttributes

                            midletName = attributes.getValue("MIDlet-Name")
                            midletVendor = attributes.getValue("MIDlet-Vendor")
                            midletVersion = attributes.getValue("MIDlet-Version")
                            midlet1Raw = attributes.getValue("MIDlet-1")
                            midletIconPath = attributes.getValue("MIDlet-Icon")
                        } catch (e: Exception) {
                            // En caso de error en formato manifest estricto, parseo alternativo por líneas
                            val manifestText = String(manifestBytes, Charsets.UTF_8)
                            for (line in manifestText.lines()) {
                                val separatorIndex = line.indexOf(':')
                                if (separatorIndex > 0) {
                                    val key = line.substring(0, separatorIndex).trim()
                                    val value = line.substring(separatorIndex + 1).trim()
                                    when {
                                        key.equals("MIDlet-Name", true) -> midletName = value
                                        key.equals("MIDlet-Vendor", true) -> midletVendor = value
                                        key.equals("MIDlet-Version", true) -> midletVersion = value
                                        key.equals("MIDlet-1", true) -> midlet1Raw = value
                                        key.equals("MIDlet-Icon", true) -> midletIconPath = value
                                    }
                                }
                            }
                        }
                        break
                    }
                    entry = zis.nextEntry
                }
            }
        }

        // Determinar qué ruta de icono buscar en el JAR
        var candidateIcon = midletIconPath
        var extractedMainClass = ""

        if (!midlet1Raw.isNullOrBlank()) {
            val parts = midlet1Raw.split(",").map { it.trim() }
            if (parts.size >= 2 && candidateIcon.isNullOrBlank() && parts[1].isNotBlank()) {
                candidateIcon = parts[1]
            }
            if (parts.size >= 3) {
                extractedMainClass = parts[2]
            }
        }

        if (!candidateIcon.isNullOrBlank()) {
            targetIconName = candidateIcon.trim().removePrefix("/").replace('\\', '/')
        }

        // Segunda pasada: extraer los bytes del icono si se especificó
        val inputStream2 = inputStreamProvider()
        if (inputStream2 != null) {
            ZipInputStream(inputStream2).use { zis ->
                var entry: ZipEntry? = zis.nextEntry
                while (entry != null) {
                    val entryName = entry.name.replace('\\', '/').removePrefix("/")

                    val isTargetIcon = targetIconName != null && entryName.equals(targetIconName, ignoreCase = true)
                    val isFallbackIcon = targetIconName == null && (
                        entryName.endsWith("icon.png", ignoreCase = true) ||
                        entryName.endsWith(".png", ignoreCase = true)
                    ) && !entry.isDirectory

                    if (isTargetIcon || (iconBytes == null && isFallbackIcon)) {
                        val bytes = readBytesFromEntry(zis)
                        if (bytes.isNotEmpty() && bytes.size < 500_000) { // Límite seguro de 500KB para iconos
                            iconBytes = bytes
                            if (isTargetIcon) break
                        }
                    }
                    entry = zis.nextEntry
                }
            }
        }

        // Guardar el icono extraído en el almacenamiento de la app
        var savedIconFilePath: String? = null
        if (iconBytes != null && iconBytes.isNotEmpty()) {
            try {
                val iconDir = File(context.filesDir, "j2me_icons")
                if (!iconDir.exists()) {
                    iconDir.mkdirs()
                }
                val iconFile = File(iconDir, "icon_${System.currentTimeMillis()}_${(1000..9999).random()}.png")
                FileOutputStream(iconFile).use { fos ->
                    fos.write(iconBytes)
                }
                savedIconFilePath = iconFile.absolutePath
            } catch (e: Exception) {
                savedIconFilePath = null
            }
        }

        // Fallback de título si el manifiesto no lo contiene
        val fallbackTitle = defaultFileName.substringBeforeLast(".jar")
            .replace('_', ' ')
            .replace('-', ' ')
            .trim()
            .ifBlank { "Juego J2ME" }

        val finalTitle = midletName?.trim()?.takeIf { it.isNotBlank() } ?: fallbackTitle

        return ParsedJarInfo(
            title = finalTitle,
            vendor = midletVendor?.trim()?.takeIf { it.isNotBlank() } ?: "Desconocido",
            version = midletVersion?.trim()?.takeIf { it.isNotBlank() } ?: "1.0",
            mainClass = extractedMainClass,
            fileName = defaultFileName,
            fileSizeBytes = defaultFileSize,
            iconPath = savedIconFilePath
        )
    }

    /**
     * Lee de forma segura los bytes de una entrada del archivo zip.
     */
    private fun readBytesFromEntry(zis: ZipInputStream): ByteArray {
        val buffer = ByteArray(4096)
        val baos = ByteArrayOutputStream()
        var len: Int
        while (zis.read(buffer).also { len = it } != -1) {
            baos.write(buffer, 0, len)
        }
        return baos.toByteArray()
    }
}
