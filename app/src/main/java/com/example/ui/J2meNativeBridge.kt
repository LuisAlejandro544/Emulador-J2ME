package com.example.ui

import android.util.Log

/**
 * Puente JNI entre la aplicación de Android y los módulos nativos en C++ y Rust.
 *
 * Carga la biblioteca nativa "j2me_native" que a su vez vincula estáticamente el
 * núcleo de ejecución en Rust ("j2me_core").
 */
object J2meNativeBridge {
    private const val TAG = "J2meNativeBridge"
    private var isLoaded = false

    init {
        try {
            System.loadLibrary("j2me_native")
            isLoaded = true
            Log.i(TAG, "Librería nativa j2me_native (C++ & Rust) cargada exitosamente.")
        } catch (e: UnsatisfiedLinkError) {
            Log.e(TAG, "No se pudo cargar la librería nativa j2me_native", e)
            isLoaded = false
        }
    }

    /**
     * Indica si la biblioteca nativa se cargó correctamente en la memoria del dispositivo.
     */
    fun isNativeLoaded(): Boolean = isLoaded

    /**
     * Obtiene la descripción detallada del motor nativo (versión de C++, versión de Rust y arquitectura de CPU).
     */
    external fun getNativeEngineInfo(): String

    /**
     * Inicializa el núcleo nativo de emulación J2ME.
     */
    external fun initializeCore(): Boolean

    /**
     * Ejecuta un ciclo de procesamiento de instrucciones en la máquina virtual.
     */
    external fun executeCycle(): Long

    /**
     * Libera la memoria y recursos del motor nativo.
     */
    external fun cleanupCore()
}
