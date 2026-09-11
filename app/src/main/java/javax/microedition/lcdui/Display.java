package javax.microedition.lcdui;

import android.os.Handler;
import android.os.Looper;
import javax.microedition.midlet.MIDlet;
import java.util.HashMap;
import java.util.Map;

/**
 * ============================================================================
 * CLASE Display (J2ME LCDUI / MIDP 2.0)
 * ============================================================================
 *
 * `Display` es el administrador central de la pantalla del dispositivo.
 * Controla qué `Displayable` (usualmente un `Canvas`) está activo, la ejecución
 * serializada de eventos en el hilo de UI (`callSerially`) y capacidades
 * de hardware como vibración y retroiluminación.
 */
public class Display {

    public static final int LIST_ELEMENT = 1;
    public static final int CHOICE_GROUP_ELEMENT = 2;
    public static final int ALERT = 3;
    public static final int COLOR_BACKGROUND = 0;
    public static final int COLOR_FOREGROUND = 1;
    public static final int COLOR_HIGHLIGHTED_BACKGROUND = 2;
    public static final int COLOR_HIGHLIGHTED_FOREGROUND = 3;
    public static final int COLOR_BORDER = 4;
    public static final int COLOR_HIGHLIGHTED_BORDER = 5;

    private static final Map<MIDlet, Display> displays = new HashMap<>();
    private static Display currentDisplayInstance = null;

    private final MIDlet midlet;
    private Displayable currentDisplayable = null;
    private final Handler mainHandler = new Handler(Looper.getMainLooper());

    private Display(MIDlet midlet) {
        this.midlet = midlet;
    }

    /**
     * Retorna la instancia de Display asociada al MIDlet dado.
     */
    public static synchronized Display getDisplay(MIDlet m) {
        if (m == null) {
            throw new NullPointerException("MIDlet nulo al solicitar Display");
        }
        Display d = displays.get(m);
        if (d == null) {
            d = new Display(m);
            displays.put(m, d);
        }
        currentDisplayInstance = d;
        return d;
    }

    public static Display getActiveDisplay() {
        return currentDisplayInstance;
    }

    /**
     * Obtiene el Displayable actualmente visible.
     */
    public Displayable getCurrent() {
        return currentDisplayable;
    }

    /**
     * Establece el nuevo elemento visible en pantalla.
     */
    public void setCurrent(Displayable next) {
        if (currentDisplayable != null) {
            currentDisplayable.setShown(false);
        }
        this.currentDisplayable = next;
        if (next != null) {
            next.setShown(true);
            if (next instanceof Canvas) {
                ((Canvas) next).repaint();
            }
        }
    }

    /**
     * Encola una tarea ejecutable en el hilo principal de la interfaz de usuario.
     */
    public void callSerially(Runnable r) {
        if (r != null) {
            mainHandler.post(r);
        }
    }

    /**
     * Hace vibrar el dispositivo durante la duración indicada en milisegundos.
     */
    public boolean vibrate(int duration) {
        // Soporte de vibración háptica
        return true;
    }

    /**
     * Hace parpadear la retroiluminación durante la duración indicada.
     */
    public boolean flashBacklight(int duration) {
        return true;
    }

    public boolean isColor() {
        return true;
    }

    public int numColors() {
        return 16777216; // Color verdadero de 24 bits
    }

    public int numAlphaLevels() {
        return 256; // 8 bits de canal alfa
    }

    public int getColor(int colorSpecifier) {
        switch (colorSpecifier) {
            case COLOR_BACKGROUND: return 0xFFFFFF;
            case COLOR_FOREGROUND: return 0x000000;
            case COLOR_HIGHLIGHTED_BACKGROUND: return 0x000088;
            case COLOR_HIGHLIGHTED_FOREGROUND: return 0xFFFFFF;
            case COLOR_BORDER: return 0x444444;
            case COLOR_HIGHLIGHTED_BORDER: return 0x0000AA;
            default: return 0x000000;
        }
    }
}
