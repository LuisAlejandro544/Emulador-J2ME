package javax.microedition.midlet;

import java.util.HashMap;
import java.util.Map;

/**
 * ============================================================================
 * CLASE BASE ESTÁNDAR MIDlet (Java ME / MIDP 2.0)
 * ============================================================================
 *
 * El MIDlet es el punto de entrada principal para cualquier aplicación o juego
 * de J2ME. Define los métodos esenciales del ciclo de vida:
 * - `startApp()`: Llamado al iniciar o reanudar el juego.
 * - `pauseApp()`: Llamado cuando el juego entra en segundo plano.
 * - `destroyApp(boolean unconditional)`: Llamado al finalizar la ejecución.
 *
 * Esta implementación provee soporte para propiedades de la aplicación
 * (leídas del MANIFEST.MF o archivo .JAD) y notificaciones de estado.
 */
public abstract class MIDlet {

    // Mapa de propiedades de la aplicación (JAD / Manifest: MIDlet-Name, MIDlet-Version, etc.)
    private static final Map<String, String> appProperties = new HashMap<>();

    // Referencia al MIDlet activo actual
    private static MIDlet activeMIDlet = null;

    protected MIDlet() {
        activeMIDlet = this;
    }

    /**
     * Inicia la ejecución del MIDlet o lo reanuda después de una pausa.
     */
    protected abstract void startApp() throws MIDletStateChangeException;

    /**
     * Pausa el MIDlet temporalmente (ej. interrupción de llamada o cambio de app).
     */
    protected abstract void pauseApp();

    /**
     * Destruye el MIDlet y libera todos los recursos asignados.
     */
    protected abstract void destroyApp(boolean unconditional) throws MIDletStateChangeException;

    /**
     * Notifica a la plataforma que el MIDlet ha completado su ejecución y debe cerrarse.
     */
    public final void notifyDestroyed() {
        if (activeMIDlet == this) {
            activeMIDlet = null;
        }
    }

    /**
     * Notifica a la plataforma que el MIDlet ha entrado en estado de pausa.
     */
    public final void notifyPaused() {
        // Estado de pausa notificado al entorno
    }

    /**
     * Obtiene el valor de una propiedad definida en el archivo JAD o en el MANIFEST.MF.
     */
    public final String getAppProperty(String key) {
        if (key == null) return null;
        return appProperties.get(key);
    }

    /**
     * Método interno para registrar propiedades desde el parseador de Rust/Android.
     */
    public static void setAppProperty(String key, String value) {
        if (key != null && value != null) {
            appProperties.put(key, value);
        }
    }

    /**
     * Retorna la instancia activa del MIDlet si existe.
     */
    public static MIDlet getActiveMIDlet() {
        return activeMIDlet;
    }
}
