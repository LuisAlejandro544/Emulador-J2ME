package javax.microedition.lcdui;

/**
 * ============================================================================
 * CLASE Font (J2ME LCDUI)
 * ============================================================================
 *
 * Provee fuentes tipográficas para la medición y renderizado de texto.
 * Compatible con los estilos, tamaños y familias estándar de MIDP 2.0.
 */
public final class Font {

    public static final int STYLE_PLAIN = 0;
    public static final int STYLE_BOLD = 1;
    public static final int STYLE_ITALIC = 2;
    public static final int STYLE_UNDERLINED = 4;

    public static final int SIZE_SMALL = 8;
    public static final int SIZE_MEDIUM = 0;
    public static final int SIZE_LARGE = 16;

    public static final int FACE_SYSTEM = 0;
    public static final int FACE_MONOSPACE = 32;
    public static final int FACE_PROPORTIONAL = 64;

    public static final int FONT_STATIC_TEXT = 0;
    public static final int FONT_INPUT_TEXT = 1;

    private static final Font DEFAULT_FONT = new Font(FACE_SYSTEM, STYLE_PLAIN, SIZE_MEDIUM);

    private final int face;
    private final int style;
    private final int size;

    private Font(int face, int style, int size) {
        this.face = face;
        this.style = style;
        this.size = size;
    }

    public static Font getDefaultFont() {
        return DEFAULT_FONT;
    }

    public static Font getFont(int face, int style, int size) {
        return new Font(face, style, size);
    }

    public static Font getFont(int fontSpecifier) {
        return DEFAULT_FONT;
    }

    public int getStyle() {
        return style;
    }

    public int getSize() {
        return size;
    }

    public int getFace() {
        return face;
    }

    public boolean isPlain() {
        return style == STYLE_PLAIN;
    }

    public boolean isBold() {
        return (style & STYLE_BOLD) != 0;
    }

    public boolean isItalic() {
        return (style & STYLE_ITALIC) != 0;
    }

    public boolean isUnderlined() {
        return (style & STYLE_UNDERLINED) != 0;
    }

    /**
     * Retorna el ancho en píxeles de una cadena de texto (8 píxeles por carácter en fuente bitmap).
     */
    public int stringWidth(String str) {
        if (str == null) return 0;
        return str.length() * 8;
    }

    public int substringWidth(String str, int offset, int len) {
        if (str == null || offset < 0 || len < 0 || offset + len > str.length()) return 0;
        return len * 8;
    }

    public int charWidth(char ch) {
        return 8;
    }

    public int charsWidth(char[] ch, int offset, int length) {
        if (ch == null || offset < 0 || length < 0 || offset + length > ch.length) return 0;
        return length * 8;
    }

    public int getHeight() {
        return 8;
    }

    public int getBaselinePosition() {
        return 7;
    }
}
