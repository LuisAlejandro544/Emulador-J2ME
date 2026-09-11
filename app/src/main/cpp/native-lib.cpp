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
