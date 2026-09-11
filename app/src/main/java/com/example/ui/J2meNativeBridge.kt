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

    /**
     * Carga y parsea un archivo JAR desde su contenido en bytes utilizando el parser seguro en Rust.
     */
    external fun loadJarFromBytes(jarBytes: ByteArray): Boolean

    /**
     * Obtiene el manifiesto parseado en formato JSON desde el núcleo Rust.
     */
    external fun getJarManifestJson(): String?

    /**
     * Extrae un recurso o clase (.class / imagen) desde el JAR en memoria.
     */
    external fun extractJarResource(resourcePath: String): ByteArray?

    /**
     * Inspecciona una clase Java (.class) en el archivo JAR cargado actualmente en memoria.
     * Retorna una cadena JSON con la superclase, métodos, tamaños de pila y firmas decodificadas.
     */
    external fun inspectJarClass(className: String): String?

    /**
     * Parsea directamente un arreglo binario de bytes de una clase Java (.class) y retorna su JSON.
     */
    external fun parseClassBytes(classBytes: ByteArray): String?

    /**
     * Ejecuta directamente un bloque de bytecode binario de JVM en el intérprete de Rust.
     * Retorna un arreglo de dos enteros: [código de estado (0=éxito, <0=error), valor retornado].
     */
    external fun executeBytecode(bytecode: ByteArray, maxStack: Int, maxLocals: Int): IntArray?

    /**
     * Reinicia o inicializa el runtime completo de la Máquina Virtual CLDC en Rust.
     */
    external fun vmReset(): Boolean

    /**
     * Carga una clase Java compilada (.class) en la Máquina Virtual global en Rust.
     */
    external fun vmLoadClass(classBytes: ByteArray): Boolean

    /**
     * Ejecuta un método de una clase en la VM global (resolviendo llamadas anidadas y memoria en Heap).
     * Retorna un arreglo de dos enteros: [código de estado (0=éxito, <0=error), valor retornado].
     */
    external fun vmExecuteMethod(className: String, methodName: String, descriptor: String): IntArray?

    /**
     * Obtiene estadísticas de diagnóstico de la VM (clases cargadas, objetos y arrays en Heap) en JSON.
     */
    external fun vmGetStats(): String?

    // ========================================================================
    // SUBSISTEMA GRÁFICO (LCDUI) Y RASTERIZADOR NATIVO C++
    // ========================================================================

    /**
     * Inicializa o redimensiona el Framebuffer nativo (por ejemplo 240x320 píxeles).
     */
    external fun graphicsInit(width: Int, height: Int)

    /**
     * Limpia la pantalla con un color ARGB completo.
     */
    external fun graphicsClear(argb: Int)

    /**
     * Establece el color de dibujo actual en formato ARGB (0xAARRGGBB).
     */
    external fun graphicsSetColor(argb: Int)

    /**
     * Obtiene el color de dibujo actual.
     */
    external fun graphicsGetColor(): Int

    /**
     * Establece el rectángulo de recorte absoluto (clipping).
     */
    external fun graphicsSetClip(x: Int, y: Int, width: Int, height: Int)

    /**
     * Interseca el rectángulo de recorte actual con las dimensiones dadas.
     */
    external fun graphicsClipRect(x: Int, y: Int, width: Int, height: Int)

    /**
     * Obtiene las coordenadas actuales del clip [x, y, w, h].
     */
    external fun graphicsGetClip(outClip: IntArray): Boolean

    /**
     * Desplaza el origen de coordenadas para las operaciones de dibujo subsiguientes.
     */
    external fun graphicsTranslate(dx: Int, dy: Int)

    /**
     * Obtiene la traslación acumulada en X.
     */
    external fun graphicsGetTranslateX(): Int

    /**
     * Obtiene la traslación acumulada en Y.
     */
    external fun graphicsGetTranslateY(): Int

    /**
     * Traza una línea recta usando el algoritmo rápido de Bresenham en C++.
     */
    external fun graphicsDrawLine(x1: Int, y1: Int, x2: Int, y2: Int)

    /**
     * Dibuja el contorno de un rectángulo.
     */
    external fun graphicsDrawRect(x: Int, y: Int, width: Int, height: Int)

    /**
     * Rellena un rectángulo con el color actual y soporte de mezcla alfa.
     */
    external fun graphicsFillRect(x: Int, y: Int, width: Int, height: Int)

    /**
     * Dibuja el contorno de un rectángulo redondeado.
     */
    external fun graphicsDrawRoundRect(x: Int, y: Int, width: Int, height: Int, arcWidth: Int, arcHeight: Int)

    /**
     * Rellena un rectángulo redondeado.
     */
    external fun graphicsFillRoundRect(x: Int, y: Int, width: Int, height: Int, arcWidth: Int, arcHeight: Int)

    /**
     * Dibuja un arco.
     */
    external fun graphicsDrawArc(x: Int, y: Int, width: Int, height: Int, startAngle: Int, arcAngle: Int)

    /**
     * Rellena un sector circular o arco.
     */
    external fun graphicsFillArc(x: Int, y: Int, width: Int, height: Int, startAngle: Int, arcAngle: Int)

    /**
     * Dibuja una cadena de texto en pantalla usando la fuente bitmap retro 8x8.
     */
    external fun graphicsDrawString(text: String, x: Int, y: Int, anchor: Int)

    /**
     * Vuelca un arreglo de píxeles ARGB crudos (sprites) sobre el Framebuffer nativo.
     */
    external fun graphicsDrawRGB(rgbData: IntArray, offset: Int, scanlength: Int, x: Int, y: Int, width: Int, height: Int, processAlpha: Boolean)

    /**
     * Copia directamente el contenido del Framebuffer nativo a un Bitmap de Android.
     */
    external fun graphicsRenderToBitmap(bitmap: android.graphics.Bitmap): Boolean

    /**
     * Copia los píxeles del Framebuffer en un arreglo de enteros.
     */
    external fun graphicsGetPixels(outPixels: IntArray): Boolean

    /**
     * Obtiene las dimensiones actuales del Framebuffer nativo [ancho, alto].
     */
    external fun graphicsGetDimensions(outDims: IntArray): Boolean
}
