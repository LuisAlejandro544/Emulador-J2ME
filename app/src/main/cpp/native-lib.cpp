/**
 * ============================================================================
 * PUENTE NATIVO J2ME EN C++ (native-lib.cpp)
 * ============================================================================
 * 
 * Este archivo implementa:
 * 1. La interfaz JNI (Java Native Interface) entre el entorno de Android
 *    y los componentes de bajo nivel.
 * 2. La capa de enlace con el núcleo de ejecución en Rust (j2me_core).
 * 3. La abstracción de hardware para gráficos (OpenGL ES) y sonido nativo.
 * 
 * Arquitectura de llamadas:
 * [Android / JVM] <---> [JNI en C++] <---> [Núcleo JVM/CLDC en Rust]
 */

#include <jni.h>
#include <string>
#include <sstream>
#include <android/log.h>
#include "framebuffer.h"

#define LOG_TAG "J2ME_Native"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

// Declaraciones de funciones exportadas por el núcleo en Rust (src/lib.rs)
extern "C" {
    int32_t j2me_core_init();
    const char* j2me_core_get_version();
    const char* j2me_core_get_architecture();
    uint64_t j2me_core_execute_cycle();
    void j2me_core_cleanup();

    int32_t j2me_core_load_jar(const uint8_t* bytes, size_t length);
    char* j2me_core_get_manifest_json();
    int32_t j2me_core_read_jar_resource(const char* filename, uint8_t** out_data, size_t* out_len);
    void j2me_core_free_string(char* ptr);
    void j2me_core_free_resource_bytes(uint8_t* ptr, size_t length);

    char* j2me_core_parse_class_bytes(const uint8_t* bytes, size_t length);
    char* j2me_core_inspect_jar_class(const char* class_name);
    int32_t j2me_core_execute_bytecode(const uint8_t* bytecode, size_t bytecode_len, uint16_t max_stack, uint16_t max_locals, int32_t* out_result);

    int32_t j2me_core_vm_reset();
    int32_t j2me_core_vm_load_class(const uint8_t* bytes, size_t length);
    int32_t j2me_core_vm_execute_method(const char* class_name, const char* method_name, const char* descriptor, int32_t* out_result);
    char* j2me_core_vm_get_stats();
}

/**
 * Retorna información del motor nativo combinando la capa C++ y el núcleo Rust.
 */
extern "C" JNIEXPORT jstring JNICALL
Java_com_example_ui_J2meNativeBridge_getNativeEngineInfo(
        JNIEnv* env,
        jobject /* this */) {

    const char* rust_version = j2me_core_get_version();
    const char* rust_arch = j2me_core_get_architecture();

    std::ostringstream oss;
    oss << "C++ NDK Bridge + "
        << (rust_version ? rust_version : "Rust Core")
        << " [" << (rust_arch ? rust_arch : "Nativo") << "]";

    LOGI("getNativeEngineInfo: %s", oss.str().c_str());
    return env->NewStringUTF(oss.str().c_str());
}

/**
 * Inicializa el núcleo nativo de emulación.
 */
extern "C" JNIEXPORT jboolean JNICALL
Java_com_example_ui_J2meNativeBridge_initializeCore(
        JNIEnv* env,
        jobject /* this */) {

    LOGI("Inicializando núcleo J2ME nativo (C++ & Rust)...");
    int32_t result = j2me_core_init();
    if (result == 0) {
        LOGI("Núcleo J2ME inicializado con éxito.");
        return JNI_TRUE;
    } else {
        LOGE("Fallo al inicializar el núcleo J2ME. Código de error: %d", result);
        return JNI_FALSE;
    }
}

/**
 * Ejecuta un ciclo de procesamiento de instrucciones en la máquina virtual.
 */
extern "C" JNIEXPORT jlong JNICALL
Java_com_example_ui_J2meNativeBridge_executeCycle(
        JNIEnv* env,
        jobject /* this */) {

    uint64_t total_cycles = j2me_core_execute_cycle();
    return static_cast<jlong>(total_cycles);
}

/**
 * Libera la memoria y recursos del núcleo nativo.
 */
extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_cleanupCore(
        JNIEnv* env,
        jobject /* this */) {

    LOGI("Liberando recursos del núcleo nativo...");
    j2me_core_cleanup();
}

/**
 * Carga un paquete JAR en memoria a través de su búfer de bytes.
 * Retorna true si fue parseado correctamente por Rust, false si no es un ZIP/JAR válido.
 */
extern "C" JNIEXPORT jboolean JNICALL
Java_com_example_ui_J2meNativeBridge_loadJarFromBytes(
        JNIEnv* env,
        jobject /* this */,
        jbyteArray jarBytes) {

    if (jarBytes == nullptr) {
        LOGE("loadJarFromBytes: búfer de bytes nulo");
        return JNI_FALSE;
    }

    jsize length = env->GetArrayLength(jarBytes);
    if (length <= 0) {
        LOGE("loadJarFromBytes: tamaño de búfer inválido: %d", length);
        return JNI_FALSE;
    }

    jbyte* buffer = env->GetByteArrayElements(jarBytes, nullptr);
    if (buffer == nullptr) {
        LOGE("loadJarFromBytes: no se pudo obtener puntero al array de bytes");
        return JNI_FALSE;
    }

    int32_t result = j2me_core_load_jar(reinterpret_cast<const uint8_t*>(buffer), static_cast<size_t>(length));
    env->ReleaseByteArrayElements(jarBytes, buffer, JNI_ABORT);

    if (result == 0) {
        LOGI("loadJarFromBytes: Paquete JAR parseado exitosamente por Rust (%d bytes)", length);
        return JNI_TRUE;
    } else {
        LOGE("loadJarFromBytes: Fallo al parsear JAR en Rust. Código: %d", result);
        return JNI_FALSE;
    }
}

/**
 * Retorna los metadatos del manifiesto (META-INF/MANIFEST.MF) en formato JSON.
 */
extern "C" JNIEXPORT jstring JNICALL
Java_com_example_ui_J2meNativeBridge_getJarManifestJson(
        JNIEnv* env,
        jobject /* this */) {

    char* json_ptr = j2me_core_get_manifest_json();
    if (json_ptr == nullptr) {
        LOGE("getJarManifestJson: No se pudo obtener el manifiesto");
        return nullptr;
    }

    jstring result = env->NewStringUTF(json_ptr);
    j2me_core_free_string(json_ptr);
    return result;
}

/**
 * Extrae y descomprime un recurso o clase (.class / .png / .mid) desde el JAR en memoria.
 */
extern "C" JNIEXPORT jbyteArray JNICALL
Java_com_example_ui_J2meNativeBridge_extractJarResource(
        JNIEnv* env,
        jobject /* this */,
        jstring resourcePath) {

    if (resourcePath == nullptr) {
        return nullptr;
    }

    const char* c_path = env->GetStringUTFChars(resourcePath, nullptr);
    if (c_path == nullptr) {
        return nullptr;
    }

    uint8_t* out_data = nullptr;
    size_t out_len = 0;

    int32_t status = j2me_core_read_jar_resource(c_path, &out_data, &out_len);
    env->ReleaseStringUTFChars(resourcePath, c_path);

    if (status != 0 || out_data == nullptr || out_len == 0) {
        return nullptr;
    }

    jbyteArray result = env->NewByteArray(static_cast<jsize>(out_len));
    if (result != nullptr) {
        env->SetByteArrayRegion(result, 0, static_cast<jsize>(out_len), reinterpret_cast<const jbyte*>(out_data));
    }

    j2me_core_free_resource_bytes(out_data, out_len);
    return result;
}

/**
 * Inspecciona una clase Java (.class) dentro del JAR cargado actualmente en memoria.
 * Retorna una cadena JSON con la superclase, métodos, tamaños de pila y firmas.
 */
extern "C" JNIEXPORT jstring JNICALL
Java_com_example_ui_J2meNativeBridge_inspectJarClass(
        JNIEnv* env,
        jobject /* this */,
        jstring className) {

    if (className == nullptr) {
        return nullptr;
    }

    const char* c_name = env->GetStringUTFChars(className, nullptr);
    if (c_name == nullptr) {
        return nullptr;
    }

    char* json_res = j2me_core_inspect_jar_class(c_name);
    env->ReleaseStringUTFChars(className, c_name);

    if (json_res == nullptr) {
        return nullptr;
    }

    jstring result = env->NewStringUTF(json_res);
    j2me_core_free_string(json_res);
    return result;
}

/**
 * Parsea directamente un arreglo de bytes de una clase Java (.class) y retorna su JSON.
 */
extern "C" JNIEXPORT jstring JNICALL
Java_com_example_ui_J2meNativeBridge_parseClassBytes(
        JNIEnv* env,
        jobject /* this */,
        jbyteArray classBytes) {

    if (classBytes == nullptr) {
        return nullptr;
    }

    jsize len = env->GetArrayLength(classBytes);
    if (len == 0) {
        return nullptr;
    }

    jbyte* bytes_ptr = env->GetByteArrayElements(classBytes, nullptr);
    if (bytes_ptr == nullptr) {
        return nullptr;
    }

    char* json_res = j2me_core_parse_class_bytes(
            reinterpret_cast<const uint8_t*>(bytes_ptr),
            static_cast<size_t>(len)
    );

    env->ReleaseByteArrayElements(classBytes, bytes_ptr, JNI_ABORT);

    if (json_res == nullptr) {
        return nullptr;
    }

    jstring result = env->NewStringUTF(json_res);
    j2me_core_free_string(json_res);
    return result;
}

/**
 * Ejecuta directamente un flujo de bytecode binario de JVM en el intérprete de Rust.
 * Retorna un arreglo de dos enteros: [código de estado (0=éxito, <0=error), valor retornado].
 */
extern "C" JNIEXPORT jintArray JNICALL
Java_com_example_ui_J2meNativeBridge_executeBytecode(
        JNIEnv* env,
        jobject /* this */,
        jbyteArray bytecode,
        jint maxStack,
        jint maxLocals) {

    if (bytecode == nullptr) {
        return nullptr;
    }

    jsize len = env->GetArrayLength(bytecode);
    if (len == 0) {
        return nullptr;
    }

    jbyte* bytes_ptr = env->GetByteArrayElements(bytecode, nullptr);
    if (bytes_ptr == nullptr) {
        return nullptr;
    }

    int32_t out_result = 0;
    int32_t status = j2me_core_execute_bytecode(
            reinterpret_cast<const uint8_t*>(bytes_ptr),
            static_cast<size_t>(len),
            static_cast<uint16_t>(maxStack),
            static_cast<uint16_t>(maxLocals),
            &out_result
    );

    env->ReleaseByteArrayElements(bytecode, bytes_ptr, JNI_ABORT);

    jintArray result = env->NewIntArray(2);
    if (result != nullptr) {
        jint elems[2] = { status, out_result };
        env->SetIntArrayRegion(result, 0, 2, elems);
    }
    return result;
}

/**
 * Reinicia o inicializa el runtime completo de la Máquina Virtual CLDC en Rust.
 */
extern "C" JNIEXPORT jboolean JNICALL
Java_com_example_ui_J2meNativeBridge_vmReset(
        JNIEnv* /* env */,
        jobject /* this */) {
    int32_t status = j2me_core_vm_reset();
    return (status == 0) ? JNI_TRUE : JNI_FALSE;
}

/**
 * Carga una clase binaria en la Máquina Virtual global en Rust.
 */
extern "C" JNIEXPORT jboolean JNICALL
Java_com_example_ui_J2meNativeBridge_vmLoadClass(
        JNIEnv* env,
        jobject /* this */,
        jbyteArray classBytes) {
    if (classBytes == nullptr) {
        return JNI_FALSE;
    }
    jsize len = env->GetArrayLength(classBytes);
    if (len == 0) {
        return JNI_FALSE;
    }
    jbyte* bytes_ptr = env->GetByteArrayElements(classBytes, nullptr);
    if (bytes_ptr == nullptr) {
        return JNI_FALSE;
    }

    int32_t status = j2me_core_vm_load_class(reinterpret_cast<const uint8_t*>(bytes_ptr), static_cast<size_t>(len));
    env->ReleaseByteArrayElements(classBytes, bytes_ptr, JNI_ABORT);

    return (status == 0) ? JNI_TRUE : JNI_FALSE;
}

/**
 * Ejecuta un método de una clase en la VM global resolviendo llamadas e instancias en el Heap.
 */
extern "C" JNIEXPORT jintArray JNICALL
Java_com_example_ui_J2meNativeBridge_vmExecuteMethod(
        JNIEnv* env,
        jobject /* this */,
        jstring className,
        jstring methodName,
        jstring descriptor) {
    if (className == nullptr || methodName == nullptr || descriptor == nullptr) {
        return nullptr;
    }

    const char* c_class = env->GetStringUTFChars(className, nullptr);
    const char* c_method = env->GetStringUTFChars(methodName, nullptr);
    const char* c_desc = env->GetStringUTFChars(descriptor, nullptr);

    int32_t out_result = 0;
    int32_t status = j2me_core_vm_execute_method(c_class, c_method, c_desc, &out_result);

    env->ReleaseStringUTFChars(className, c_class);
    env->ReleaseStringUTFChars(methodName, c_method);
    env->ReleaseStringUTFChars(descriptor, c_desc);

    jintArray result = env->NewIntArray(2);
    if (result != nullptr) {
        jint elems[2] = { status, out_result };
        env->SetIntArrayRegion(result, 0, 2, elems);
    }
    return result;
}

/**
 * Obtiene las estadísticas diagnósticas de la VM en formato JSON.
 */
extern "C" JNIEXPORT jstring JNICALL
Java_com_example_ui_J2meNativeBridge_vmGetStats(
        JNIEnv* env,
        jobject /* this */) {
    char* json_ptr = j2me_core_vm_get_stats();
    if (json_ptr == nullptr) {
        return nullptr;
    }
    jstring result = env->NewStringUTF(json_ptr);
    j2me_core_free_string(json_ptr);
    return result;
}

// ============================================================================
// SUBSISTEMA GRÁFICO (LCDUI) Y RASTERIZADOR 2D NATIVO
// ============================================================================

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsInit(
        JNIEnv* /* env */,
        jobject /* this */,
        jint width,
        jint height) {
    j2me::getGlobalFramebuffer().resize(width, height);
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsClear(
        JNIEnv* /* env */,
        jobject /* this */,
        jint argb) {
    j2me::getGlobalFramebuffer().clear(static_cast<uint32_t>(argb));
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsSetColor(
        JNIEnv* /* env */,
        jobject /* this */,
        jint argb) {
    j2me::getGlobalFramebuffer().setColor(static_cast<uint32_t>(argb));
}

extern "C" JNIEXPORT jint JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsGetColor(
        JNIEnv* /* env */,
        jobject /* this */) {
    return static_cast<jint>(j2me::getGlobalFramebuffer().getColor());
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsSetClip(
        JNIEnv* /* env */,
        jobject /* this */,
        jint x,
        jint y,
        jint width,
        jint height) {
    j2me::getGlobalFramebuffer().setClip(x, y, width, height);
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsClipRect(
        JNIEnv* /* env */,
        jobject /* this */,
        jint x,
        jint y,
        jint width,
        jint height) {
    j2me::getGlobalFramebuffer().clipRect(x, y, width, height);
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsGetClip(
        JNIEnv* env,
        jobject /* this */,
        jintArray outClip) {
    if (!outClip || env->GetArrayLength(outClip) < 4) return JNI_FALSE;
    auto& fb = j2me::getGlobalFramebuffer();
    jint clip[4] = {
        fb.getClipX(),
        fb.getClipY(),
        fb.getClipWidth(),
        fb.getClipHeight()
    };
    env->SetIntArrayRegion(outClip, 0, 4, clip);
    return JNI_TRUE;
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsTranslate(
        JNIEnv* /* env */,
        jobject /* this */,
        jint dx,
        jint dy) {
    j2me::getGlobalFramebuffer().translate(dx, dy);
}

extern "C" JNIEXPORT jint JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsGetTranslateX(
        JNIEnv* /* env */,
        jobject /* this */) {
    return j2me::getGlobalFramebuffer().getTranslateX();
}

extern "C" JNIEXPORT jint JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsGetTranslateY(
        JNIEnv* /* env */,
        jobject /* this */) {
    return j2me::getGlobalFramebuffer().getTranslateY();
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsDrawLine(
        JNIEnv* /* env */,
        jobject /* this */,
        jint x1,
        jint y1,
        jint x2,
        jint y2) {
    j2me::getGlobalFramebuffer().drawLine(x1, y1, x2, y2);
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsDrawRect(
        JNIEnv* /* env */,
        jobject /* this */,
        jint x,
        jint y,
        jint width,
        jint height) {
    j2me::getGlobalFramebuffer().drawRect(x, y, width, height);
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsFillRect(
        JNIEnv* /* env */,
        jobject /* this */,
        jint x,
        jint y,
        jint width,
        jint height) {
    j2me::getGlobalFramebuffer().fillRect(x, y, width, height);
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsDrawRoundRect(
        JNIEnv* /* env */,
        jobject /* this */,
        jint x,
        jint y,
        jint width,
        jint height,
        jint arcWidth,
        jint arcHeight) {
    j2me::getGlobalFramebuffer().drawRoundRect(x, y, width, height, arcWidth, arcHeight);
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsFillRoundRect(
        JNIEnv* /* env */,
        jobject /* this */,
        jint x,
        jint y,
        jint width,
        jint height,
        jint arcWidth,
        jint arcHeight) {
    j2me::getGlobalFramebuffer().fillRoundRect(x, y, width, height, arcWidth, arcHeight);
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsDrawArc(
        JNIEnv* /* env */,
        jobject /* this */,
        jint x,
        jint y,
        jint width,
        jint height,
        jint startAngle,
        jint arcAngle) {
    j2me::getGlobalFramebuffer().drawArc(x, y, width, height, startAngle, arcAngle);
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsFillArc(
        JNIEnv* /* env */,
        jobject /* this */,
        jint x,
        jint y,
        jint width,
        jint height,
        jint startAngle,
        jint arcAngle) {
    j2me::getGlobalFramebuffer().fillArc(x, y, width, height, startAngle, arcAngle);
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsDrawString(
        JNIEnv* env,
        jobject /* this */,
        jstring text,
        jint x,
        jint y,
        jint anchor) {
    if (!text) return;
    const char* c_str = env->GetStringUTFChars(text, nullptr);
    if (c_str) {
        j2me::getGlobalFramebuffer().drawString(c_str, x, y, anchor);
        env->ReleaseStringUTFChars(text, c_str);
    }
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsDrawRGB(
        JNIEnv* env,
        jobject /* this */,
        jintArray rgbData,
        jint offset,
        jint scanlength,
        jint x,
        jint y,
        jint width,
        jint height,
        jboolean processAlpha) {
    if (!rgbData || width <= 0 || height <= 0) return;
    jint* data = env->GetIntArrayElements(rgbData, nullptr);
    if (data) {
        j2me::getGlobalFramebuffer().drawRGB(
            reinterpret_cast<const uint32_t*>(data),
            offset, scanlength, x, y, width, height, processAlpha == JNI_TRUE
        );
        env->ReleaseIntArrayElements(rgbData, data, JNI_ABORT);
    }
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsRenderToBitmap(
        JNIEnv* env,
        jobject /* this */,
        jobject bitmap) {
    return j2me::getGlobalFramebuffer().copyToAndroidBitmap(env, bitmap) ? JNI_TRUE : JNI_FALSE;
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsGetPixels(
        JNIEnv* env,
        jobject /* this */,
        jintArray outPixels) {
    if (!outPixels) return JNI_FALSE;
    jsize len = env->GetArrayLength(outPixels);
    jint* data = env->GetIntArrayElements(outPixels, nullptr);
    if (!data) return JNI_FALSE;

    bool ok = j2me::getGlobalFramebuffer().copyPixelsTo(reinterpret_cast<uint32_t*>(data), len);
    env->ReleaseIntArrayElements(outPixels, data, 0);
    return ok ? JNI_TRUE : JNI_FALSE;
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_example_ui_J2meNativeBridge_graphicsGetDimensions(
        JNIEnv* env,
        jobject /* this */,
        jintArray outDims) {
    if (!outDims || env->GetArrayLength(outDims) < 2) return JNI_FALSE;
    auto& fb = j2me::getGlobalFramebuffer();
    jint dims[2] = { fb.getWidth(), fb.getHeight() };
    env->SetIntArrayRegion(outDims, 0, 2, dims);
    return JNI_TRUE;
}



