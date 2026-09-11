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

