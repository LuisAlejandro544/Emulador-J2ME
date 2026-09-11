package javax.microedition.lcdui;

import com.example.ui.J2meNativeBridge;

/**
 * ============================================================================
 * CONTEXTO DE DIBUJO 2D Graphics (J2ME LCDUI / MIDP 2.0)
 * ============================================================================
 *
 * `Graphics` provee todas las operaciones de trazado geométrico, texto e imágenes.
 *
 * Arquitectura híbrida de alto rendimiento:
 * - Cuando dibuja sobre la pantalla principal (`Canvas`): delega directamente
 *   a las primitivas aceleradas en C++ (`Framebuffer.cpp`) a través de JNI.
 * - Cuando dibuja sobre un búfer fuera de pantalla (`Image` mutable): manipula
 *   directamente los píxeles ARGB del mapa de bits en memoria.
 */
public class Graphics {

    // Constantes de anclaje de posición estándar de J2ME
    public static final int HCENTER  = 1;
    public static final int VCENTER  = 2;
    public static final int LEFT     = 4;
    public static final int RIGHT    = 8;
    public static final int TOP      = 16;
    public static final int BOTTOM   = 32;
    public static final int BASELINE = 64;

    // Estilos de trazo
    public static final int SOLID  = 0;
    public static final int DOTTED = 1;

    private final boolean isScreen;
    private final Image targetImage;

    private int currentColor = 0xFFFFFFFF; // Blanco por defecto
    private int transX = 0;
    private int transY = 0;
    private int clipX = 0;
    private int clipY = 0;
    private int clipWidth = 240;
    private int clipHeight = 320;
    private Font currentFont = Font.getDefaultFont();
    private int strokeStyle = SOLID;

    /**
     * Constructor para el contexto gráfico de pantalla principal.
     */
    public Graphics() {
        this.isScreen = true;
        this.targetImage = null;
        this.clipWidth = 240;
        this.clipHeight = 320;
    }

    /**
     * Constructor para dibujar sobre una imagen mutable fuera de pantalla.
     */
    public Graphics(Image targetImage) {
        this.isScreen = false;
        this.targetImage = targetImage;
        if (targetImage != null) {
            this.clipWidth = targetImage.getWidth();
            this.clipHeight = targetImage.getHeight();
        }
    }

    // ========================================================================
    // GESTIÓN DE COLORES
    // ========================================================================

    public void setColor(int RGB) {
        // En J2ME, setColor(int) recibe 0x00RRGGBB; se asume opaco (0xFF...)
        this.currentColor = 0xFF000000 | (RGB & 0x00FFFFFF);
        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsSetColor(this.currentColor);
        }
    }

    public void setColor(int red, int green, int blue) {
        int r = Math.max(0, Math.min(255, red));
        int g = Math.max(0, Math.min(255, green));
        int b = Math.max(0, Math.min(255, blue));
        setColor((r << 16) | (g << 8) | b);
    }

    public void setARGBColor(int argbColor) {
        this.currentColor = argbColor;
        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsSetColor(this.currentColor);
        }
    }

    public int getColor() {
        return currentColor & 0x00FFFFFF;
    }

    public int getRedComponent() {
        return (currentColor >> 16) & 0xFF;
    }

    public int getGreenComponent() {
        return (currentColor >> 8) & 0xFF;
    }

    public int getBlueComponent() {
        return currentColor & 0xFF;
    }

    // ========================================================================
    // TRASLACIÓN Y RECORTE (CLIPPING)
    // ========================================================================

    public void translate(int x, int y) {
        this.transX += x;
        this.transY += y;
        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsTranslate(x, y);
        }
    }

    public int getTranslateX() {
        return transX;
    }

    public int getTranslateY() {
        return transY;
    }

    public void setClip(int x, int y, int width, int height) {
        this.clipX = x;
        this.clipY = y;
        this.clipWidth = Math.max(0, width);
        this.clipHeight = Math.max(0, height);
        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsSetClip(x, y, width, height);
        }
    }

    public void clipRect(int x, int y, int width, int height) {
        int newX1 = Math.max(this.clipX, x);
        int newY1 = Math.max(this.clipY, y);
        int newX2 = Math.min(this.clipX + this.clipWidth, x + width);
        int newY2 = Math.min(this.clipY + this.clipHeight, y + height);

        this.clipX = newX1;
        this.clipY = newY1;
        this.clipWidth = Math.max(0, newX2 - newX1);
        this.clipHeight = Math.max(0, newY2 - newY1);

        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsClipRect(x, y, width, height);
        }
    }

    public int getClipX() {
        return clipX;
    }

    public int getClipY() {
        return clipY;
    }

    public int getClipWidth() {
        return clipWidth;
    }

    public int getClipHeight() {
        return clipHeight;
    }

    // ========================================================================
    // PRIMITIVAS GEOMÉTRICAS 2D
    // ========================================================================

    public void drawLine(int x1, int y1, int x2, int y2) {
        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsDrawLine(x1, y1, x2, y2);
        } else if (targetImage != null) {
            drawBresenhamLine(targetImage.getInternalPixels(), targetImage.getWidth(), targetImage.getHeight(),
                    x1 + transX, y1 + transY, x2 + transX, y2 + transY, currentColor);
        }
    }

    public void drawRect(int x, int y, int width, int height) {
        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsDrawRect(x, y, width, height);
        } else {
            drawLine(x, y, x + width, y);
            drawLine(x, y + height, x + width, y + height);
            drawLine(x, y, x, y + height);
            drawLine(x + width, y, x + width, y + height);
        }
    }

    public void fillRect(int x, int y, int width, int height) {
        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsFillRect(x, y, width, height);
        } else if (targetImage != null) {
            fillImageRect(targetImage.getInternalPixels(), targetImage.getWidth(), targetImage.getHeight(),
                    x + transX, y + transY, width, height, currentColor);
        }
    }

    public void drawRoundRect(int x, int y, int width, int height, int arcWidth, int arcHeight) {
        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsDrawRoundRect(x, y, width, height, arcWidth, arcHeight);
        } else {
            drawRect(x, y, width, height);
        }
    }

    public void fillRoundRect(int x, int y, int width, int height, int arcWidth, int arcHeight) {
        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsFillRoundRect(x, y, width, height, arcWidth, arcHeight);
        } else {
            fillRect(x, y, width, height);
        }
    }

    public void drawArc(int x, int y, int width, int height, int startAngle, int arcAngle) {
        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsDrawArc(x, y, width, height, startAngle, arcAngle);
        } else {
            drawRect(x, y, width, height);
        }
    }

    public void fillArc(int x, int y, int width, int height, int startAngle, int arcAngle) {
        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsFillArc(x, y, width, height, startAngle, arcAngle);
        } else {
            fillRect(x, y, width, height);
        }
    }

    // ========================================================================
    // DIBUJADO DE TEXTO
    // ========================================================================

    public void setFont(Font font) {
        this.currentFont = font != null ? font : Font.getDefaultFont();
    }

    public Font getFont() {
        return currentFont;
    }

    public void drawString(String str, int x, int y, int anchor) {
        if (str == null || str.isEmpty()) return;
        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsDrawString(str, x, y, anchor);
        }
    }

    public void drawSubstring(String str, int offset, int len, int x, int y, int anchor) {
        if (str == null || offset < 0 || len < 0 || offset + len > str.length()) return;
        drawString(str.substring(offset, offset + len), x, y, anchor);
    }

    public void drawChar(char character, int x, int y, int anchor) {
        drawString(String.valueOf(character), x, y, anchor);
    }

    // ========================================================================
    // IMÁGENES Y ARREGLOS RGB (SPRITES)
    // ========================================================================

    public void drawImage(Image img, int x, int y, int anchor) {
        if (img == null) {
            throw new NullPointerException("Imagen a dibujar nula");
        }

        int w = img.getWidth();
        int h = img.getHeight();

        int posX = x;
        int posY = y;

        if ((anchor & RIGHT) != 0) {
            posX -= w;
        } else if ((anchor & HCENTER) != 0) {
            posX -= w / 2;
        }

        if ((anchor & BOTTOM) != 0) {
            posY -= h;
        } else if ((anchor & VCENTER) != 0) {
            posY -= h / 2;
        }

        drawRGB(img.getInternalPixels(), 0, w, posX, posY, w, h, true);
    }

    public void drawRGB(int[] rgbData, int offset, int scanlength,
                        int x, int y, int width, int height, boolean processAlpha) {
        if (rgbData == null || width <= 0 || height <= 0) return;
        if (isScreen) {
            J2meNativeBridge.INSTANCE.graphicsDrawRGB(rgbData, offset, scanlength, x, y, width, height, processAlpha);
        } else if (targetImage != null) {
            int[] dst = targetImage.getInternalPixels();
            int dstW = targetImage.getWidth();
            int dstH = targetImage.getHeight();

            int startX = Math.max(0, x + transX);
            int endX = Math.min(dstW, x + transX + width);
            int startY = Math.max(0, y + transY);
            int endY = Math.min(dstH, y + transY + height);

            for (int row = startY; row < endY; ++row) {
                int srcRow = row - (y + transY);
                for (int col = startX; col < endX; ++col) {
                    int srcCol = col - (x + transX);
                    int srcPix = rgbData[offset + (srcRow * scanlength) + srcCol];
                    dst[row * dstW + col] = srcPix;
                }
            }
        }
    }

    public void setStrokeStyle(int style) {
        this.strokeStyle = style;
    }

    public int getStrokeStyle() {
        return strokeStyle;
    }

    // ========================================================================
    // AUXILIARES PARA BÚFER OFF-SCREEN
    // ========================================================================

    private static void fillImageRect(int[] pixels, int imgW, int imgH,
                                      int x, int y, int w, int h, int color) {
        int x1 = Math.max(0, x);
        int y1 = Math.max(0, y);
        int x2 = Math.min(imgW, x + w);
        int y2 = Math.min(imgH, y + h);

        for (int r = y1; r < y2; ++r) {
            int start = r * imgW + x1;
            int count = x2 - x1;
            java.util.Arrays.fill(pixels, start, start + count, color);
        }
    }

    private static void drawBresenhamLine(int[] pixels, int imgW, int imgH,
                                          int x1, int y1, int x2, int y2, int color) {
        int dx = Math.abs(x2 - x1);
        int dy = Math.abs(y2 - y1);
        int sx = x1 < x2 ? 1 : -1;
        int sy = y1 < y2 ? 1 : -1;
        int err = dx - dy;

        while (true) {
            if (x1 >= 0 && x1 < imgW && y1 >= 0 && y1 < imgH) {
                pixels[y1 * imgW + x1] = color;
            }
            if (x1 == x2 && y1 == y2) break;
            int e2 = 2 * err;
            if (e2 > -dy) {
                err -= dy;
                x1 += sx;
            }
            if (e2 < dx) {
                err += dx;
                y1 += sy;
            }
        }
    }
}
