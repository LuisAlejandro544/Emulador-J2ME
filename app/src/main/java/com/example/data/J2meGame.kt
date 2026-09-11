package com.example.data

import androidx.room.Entity
import androidx.room.PrimaryKey

/**
 * Entidad que representa un juego J2ME (.jar) almacenado en la biblioteca local del usuario.
 *
 * Contiene tanto los metadatos extraídos del archivo MANIFEST.MF del MIDlet
 * (como nombre, desarrollador/vendor, versión y clase principal) como los detalles del
 * archivo local (URI, nombre de archivo, tamaño en bytes y ruta del icono extraído si existe).
 */
@Entity(tableName = "j2me_games")
data class J2meGame(
    @PrimaryKey(autoGenerate = true)
    val id: Long = 0,

    /**
     * Nombre del juego (MIDlet-Name del manifiesto, o el nombre del archivo sin .jar).
     */
    val title: String,

    /**
     * Desarrollador o compañía del juego (MIDlet-Vendor del manifiesto).
     */
    val vendor: String = "Desconocido",

    /**
     * Versión del juego (MIDlet-Version del manifiesto).
     */
    val version: String = "1.0",

    /**
     * Nombre de la clase principal ejecutable (definida en MIDlet-1 del manifiesto).
     */
    val mainClass: String = "",

    /**
     * URI de almacenamiento en Android (SAF) otorgada con permisos persistentes.
     */
    val fileUri: String,

    /**
     * Nombre del archivo original (ej. "SpaceShooter.jar").
     */
    val fileName: String,

    /**
     * Tamaño del archivo .jar en bytes.
     */
    val fileSizeBytes: Long = 0L,

    /**
     * Ruta absoluta en el almacenamiento interno de la app del icono extraído del .jar.
     * Es nula si el archivo .jar no incluye un icono gráfico.
     */
    val iconPath: String? = null,

    /**
     * Fecha y hora en milisegundos en la que el usuario importó este juego.
     */
    val addedTimestamp: Long = System.currentTimeMillis()
)
