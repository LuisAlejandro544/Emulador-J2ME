package javax.microedition.lcdui.game;

import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.Image;
import com.example.ui.J2meNativeBridge;

/**
 * ============================================================================
 * CLASE ABSTRACTA GameCanvas (MIDP 2.0 Game API)
 * ============================================================================
 *
 * `GameCanvas` es la base para bucles de juego de alto rendimiento en J2ME.
 * A diferencia del `Canvas` tradicional, proporciona:
 * - Un búfer secundario fuera de pantalla (Off-screen buffer) con `getGraphics()`.
 * - Volcado síncrono del búfer al Framebuffer nativo con `flushGraphics()`.
 * - Consulta directa del estado de teclas sin interrupciones con `getKeyStates()`.
 * - Opción de supresión de eventos de teclado convencionales (`suppressKeyEvents`).
 */
public abstract class GameCanvas extends Canvas {

    // Máscaras de bits para estados de teclas
    public static final int UP_PRESSED     = 1 << Canvas.UP;      // 1 << 1 = 2
    public static final int LEFT_PRESSED   = 1 << Canvas.LEFT;    // 1 << 2 = 4
    public static final int RIGHT_PRESSED  = 1 << Canvas.RIGHT;   // 1 << 5 = 32
    public static final int DOWN_PRESSED   = 1 << Canvas.DOWN;    // 1 << 6 = 64
    public static final int FIRE_PRESSED   = 1 << Canvas.FIRE;    // 1 << 8 = 256
    public static final int GAME_A_PRESSED = 1 << Canvas.GAME_A;  // 1 << 9 = 512
    public static final int GAME_B_PRESSED = 1 << Canvas.GAME_B;  // 1 << 10 = 1024
    public static final int GAME_C_PRESSED = 1 << Canvas.GAME_C;  // 1 << 11 = 2048
    public static final int GAME_D_PRESSED = 1 << Canvas.GAME_D;  // 1 << 12 = 4096

    private final boolean suppressKeyEvents;
    private final Image offscreenImage;
    private final Graphics offscreenGraphics;

    // Máscara atómica con las teclas actualmente presionadas y las presionadas desde la última lectura
    private int keyStates = 0;
    private int latchedKeyStates = 0;

    /**
     * Constructor para GameCanvas con opción de suprimir el despacho a keyPressed/keyReleased.
     *
     * @param suppressKeyEvents Si es true, el juego consulta el teclado exclusivamente con `getKeyStates()`.
     */
    protected GameCanvas(boolean suppressKeyEvents) {
        super();
        this.suppressKeyEvents = suppressKeyEvents;
        // Búfer off-screen mutable de resolución nativa 240x320
        this.offscreenImage = Image.createImage(getWidth(), getHeight());
        this.offscreenGraphics = offscreenImage.getGraphics();
    }

    /**
     * Retorna el contexto gráfico asociado al búfer secundario fuera de pantalla.
     */
    protected Graphics getGraphics() {
        return offscreenGraphics;
    }

    /**
     * Vuelca el búfer secundario completo a la pantalla / Framebuffer físico de C++.
     */
    public void flushGraphics() {
        flushGraphics(0, 0, getWidth(), getHeight());
    }

    /**
     * Vuelca una región rectangular del búfer secundario al Framebuffer físico de C++.
     */
    public void flushGraphics(int x, int y, int width, int height) {
        if (width <= 0 || height <= 0) return;
        int[] pixels = offscreenImage.getInternalPixels();
        J2meNativeBridge.INSTANCE.graphicsDrawRGB(pixels, 0, getWidth(), x, y, width, height, false);
    }

    /**
     * Retorna el estado consolidado de las teclas de juego desde la última llamada.
     *
     * @return Máscara de bits con las combinaciones de `UP_PRESSED`, `FIRE_PRESSED`, etc.
     */
    public synchronized int getKeyStates() {
        int result = keyStates | latchedKeyStates;
        latchedKeyStates = keyStates; // Conservar solo las teclas que sigan sostenidas actualmente
        return result;
    }

    private int gameActionToMask(int gameAction) {
        switch (gameAction) {
            case UP:     return UP_PRESSED;
            case DOWN:   return DOWN_PRESSED;
            case LEFT:   return LEFT_PRESSED;
            case RIGHT:  return RIGHT_PRESSED;
            case FIRE:   return FIRE_PRESSED;
            case GAME_A: return GAME_A_PRESSED;
            case GAME_B: return GAME_B_PRESSED;
            case GAME_C: return GAME_C_PRESSED;
            case GAME_D: return GAME_D_PRESSED;
            default:     return 0;
        }
    }

    @Override
    public synchronized void dispatchKeyPressed(int keyCode) {
        int action = getGameAction(keyCode);
        int mask = gameActionToMask(action);
        keyStates |= mask;
        latchedKeyStates |= mask;

        if (!suppressKeyEvents) {
            super.dispatchKeyPressed(keyCode);
        }
    }

    @Override
    public synchronized void dispatchKeyReleased(int keyCode) {
        int action = getGameAction(keyCode);
        int mask = gameActionToMask(action);
        keyStates &= ~mask;

        if (!suppressKeyEvents) {
            super.dispatchKeyReleased(keyCode);
        }
    }

    /**
     * Dibuja por defecto el contenido del búfer secundario si el sistema solicita repintado.
     */
    @Override
    public void paint(Graphics g) {
        if (g != null && offscreenImage != null) {
            g.drawImage(offscreenImage, 0, 0, Graphics.TOP | Graphics.LEFT);
        }
    }
}
