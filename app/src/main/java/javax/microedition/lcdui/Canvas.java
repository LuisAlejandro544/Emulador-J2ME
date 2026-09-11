package javax.microedition.lcdui;

/**
 * ============================================================================
 * CLASE ABSTRACTA Canvas (J2ME LCDUI / MIDP 2.0)
 * ============================================================================
 *
 * `Canvas` es el lienzo gráfico base de casi la totalidad de juegos J2ME.
 * Maneja:
 * 1. El ciclo de renderizado mediante `paint(Graphics g)`.
 * 2. El bucle de eventos de teclado numérico, flechas y teclas de acción (`FIRE`).
 * 3. Eventos táctiles (`pointerPressed`, `pointerReleased`, `pointerDragged`).
 * 4. Peticiones de refresco con `repaint()` conectadas al motor de Android.
 */
public abstract class Canvas extends Displayable {

    // Constantes de acciones de juego estándar (Game Actions)
    public static final int UP     = 1;
    public static final int LEFT   = 2;
    public static final int RIGHT  = 5;
    public static final int DOWN   = 6;
    public static final int FIRE   = 8;
    public static final int GAME_A = 9;
    public static final int GAME_B = 10;
    public static final int GAME_C = 11;
    public static final int GAME_D = 12;

    // Códigos de teclas estándar (Key Codes ITU-T)
    public static final int KEY_NUM0  = 48;
    public static final int KEY_NUM1  = 49;
    public static final int KEY_NUM2  = 50;
    public static final int KEY_NUM3  = 51;
    public static final int KEY_NUM4  = 52;
    public static final int KEY_NUM5  = 53;
    public static final int KEY_NUM6  = 54;
    public static final int KEY_NUM7  = 55;
    public static final int KEY_NUM8  = 56;
    public static final int KEY_NUM9  = 57;
    public static final int KEY_STAR  = 42;
    public static final int KEY_POUND = 35;

    // Códigos virtuales típicos para SoftKeys y navegación
    public static final int KEY_SOFTKEY_LEFT  = -6;
    public static final int KEY_SOFTKEY_RIGHT = -7;
    public static final int KEY_UP            = -1;
    public static final int KEY_DOWN          = -2;
    public static final int KEY_LEFT          = -3;
    public static final int KEY_RIGHT         = -4;
    public static final int KEY_SELECT        = -5;

    // Interfaz para escuchar solicitudes de repintado desde la UI de Android
    public interface RepaintListener {
        void onRepaintRequested(Canvas canvas);
    }

    private static RepaintListener repaintListener = null;
    private boolean fullScreenMode = false;
    private final Graphics screenGraphics = new Graphics();

    protected Canvas() {
        setSize(240, 320);
    }

    /**
     * Método central de dibujo que cada juego J2ME implementa.
     */
    protected abstract void paint(Graphics g);

    /**
     * Solicita a la plataforma el repintado de todo el lienzo.
     */
    public final void repaint() {
        if (repaintListener != null) {
            repaintListener.onRepaintRequested(this);
        }
    }

    /**
     * Solicita el repintado de un área rectangular específica del lienzo.
     */
    public final void repaint(int x, int y, int width, int height) {
        repaint();
    }

    /**
     * Fuerza el procesamiento síncrono inmediato de las peticiones de repintado pendientes.
     */
    public final void serviceRepaints() {
        render();
    }

    /**
     * Ejecuta el ciclo de dibujado pasando el contexto gráfico nativo.
     */
    public void render() {
        paint(screenGraphics);
    }

    // ========================================================================
    // GESTIÓN DE EVENTOS DE TECLADO
    // ========================================================================

    public void dispatchKeyPressed(int keyCode) {
        keyPressed(keyCode);
    }

    public void dispatchKeyReleased(int keyCode) {
        keyReleased(keyCode);
    }

    public void dispatchKeyRepeated(int keyCode) {
        keyRepeated(keyCode);
    }

    protected void keyPressed(int keyCode) {
        // Implementación por defecto vacía, sobreescrita por el juego
    }

    protected void keyReleased(int keyCode) {
        // Implementación por defecto vacía, sobreescrita por el juego
    }

    protected void keyRepeated(int keyCode) {
        // Implementación por defecto vacía, sobreescrita por el juego
    }

    // ========================================================================
    // GESTIÓN DE EVENTOS TÁCTILES (POINTER)
    // ========================================================================

    public void dispatchPointerPressed(int x, int y) {
        pointerPressed(x, y);
    }

    public void dispatchPointerReleased(int x, int y) {
        pointerReleased(x, y);
    }

    public void dispatchPointerDragged(int x, int y) {
        pointerDragged(x, y);
    }

    protected void pointerPressed(int x, int y) {
        // Implementación por defecto vacía, sobreescrita por el juego
    }

    protected void pointerReleased(int x, int y) {
        // Implementación por defecto vacía, sobreescrita por el juego
    }

    protected void pointerDragged(int x, int y) {
        // Implementación por defecto vacía, sobreescrita por el juego
    }

    // ========================================================================
    // MAPEO DE ACCIONES DE JUEGO (GAME ACTIONS)
    // ========================================================================

    public int getGameAction(int keyCode) {
        switch (keyCode) {
            case KEY_UP:
            case KEY_NUM2:
                return UP;
            case KEY_DOWN:
            case KEY_NUM8:
                return DOWN;
            case KEY_LEFT:
            case KEY_NUM4:
                return LEFT;
            case KEY_RIGHT:
            case KEY_NUM6:
                return RIGHT;
            case KEY_SELECT:
            case KEY_NUM5:
                return FIRE;
            case KEY_NUM1:
                return GAME_A;
            case KEY_NUM3:
                return GAME_B;
            case KEY_NUM7:
                return GAME_C;
            case KEY_NUM9:
                return GAME_D;
            default:
                return 0;
        }
    }

    public int getKeyCode(int gameAction) {
        switch (gameAction) {
            case UP:     return KEY_UP;
            case DOWN:   return KEY_DOWN;
            case LEFT:   return KEY_LEFT;
            case RIGHT:  return KEY_RIGHT;
            case FIRE:   return KEY_SELECT;
            case GAME_A: return KEY_NUM1;
            case GAME_B: return KEY_NUM3;
            case GAME_C: return KEY_NUM7;
            case GAME_D: return KEY_NUM9;
            default:     return 0;
        }
    }

    public String getKeyName(int keyCode) {
        switch (keyCode) {
            case KEY_NUM0: return "0";
            case KEY_NUM1: return "1";
            case KEY_NUM2: return "2";
            case KEY_NUM3: return "3";
            case KEY_NUM4: return "4";
            case KEY_NUM5: return "5";
            case KEY_NUM6: return "6";
            case KEY_NUM7: return "7";
            case KEY_NUM8: return "8";
            case KEY_NUM9: return "9";
            case KEY_STAR: return "*";
            case KEY_POUND: return "#";
            case KEY_UP: return "UP";
            case KEY_DOWN: return "DOWN";
            case KEY_LEFT: return "LEFT";
            case KEY_RIGHT: return "RIGHT";
            case KEY_SELECT: return "FIRE";
            case KEY_SOFTKEY_LEFT: return "SOFT1";
            case KEY_SOFTKEY_RIGHT: return "SOFT2";
            default: return "KEY_" + keyCode;
        }
    }

    public boolean hasPointerEvents() {
        return true;
    }

    public boolean hasPointerMotionEvents() {
        return true;
    }

    public boolean isDoubleBuffered() {
        return true;
    }

    public void setFullScreenMode(boolean mode) {
        this.fullScreenMode = mode;
    }

    public boolean isFullScreenMode() {
        return fullScreenMode;
    }

    public static void setRepaintListener(RepaintListener listener) {
        repaintListener = listener;
    }
}
