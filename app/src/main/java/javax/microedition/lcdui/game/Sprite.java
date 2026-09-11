package javax.microedition.lcdui.game;

import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.Image;

/**
 * ============================================================================
 * CLASE Sprite (MIDP 2.0 Game API)
 * ============================================================================
 *
 * `Sprite` representa un actor o personaje animado compuesto por una tira de fotogramas
 * (spritesheet) procedentes de una `Image`.
 *
 * Características implementadas:
 * - Secuencias de animación y selección de fotograma actual.
 * - Transformaciones geométricas estándar (rotaciones 90/180/270 y espejos).
 * - Píxel de referencia para anclaje y rotación coherente.
 * - Detección de colisiones tanto por caja delimitadora (AABB) como por
 *   máscara precisa a nivel de píxel (pixelLevel con canal alfa).
 */
public class Sprite extends Layer {

    // Constantes de transformación de MIDP 2.0
    public static final int TRANS_NONE             = 0;
    public static final int TRANS_MIRROR_ROT180    = 1;
    public static final int TRANS_MIRROR           = 2;
    public static final int TRANS_ROT180           = 3;
    public static final int TRANS_MIRROR_ROT270    = 4;
    public static final int TRANS_ROT90            = 5;
    public static final int TRANS_ROT270           = 6;
    public static final int TRANS_MIRROR_ROT90     = 7;

    private Image image;
    private int srcFrameWidth;
    private int srcFrameHeight;
    private int cols;
    private int rows;
    private int totalFrames;

    private int[] frameSequence;
    private int currentSequenceIndex = 0;
    private int currentTransform = TRANS_NONE;

    // Píxel de referencia (relativo al fotograma original sin transformar)
    private int refX = 0;
    private int refY = 0;

    // Rectángulo de colisión personalizado (por defecto coincide con el tamaño del fotograma)
    private int colX = 0;
    private int colY = 0;
    private int colW = 0;
    private int colH = 0;
    private boolean customCollisionRect = false;

    /**
     * Crea un Sprite de un único fotograma con las dimensiones completas de la imagen.
     */
    public Sprite(Image image) {
        super(image != null ? image.getWidth() : 0, image != null ? image.getHeight() : 0);
        if (image == null) {
            throw new NullPointerException("La imagen para Sprite no puede ser nula");
        }
        initialize(image, image.getWidth(), image.getHeight());
    }

    /**
     * Crea un Sprite animado dividiendo la imagen en fotogramas de ancho y alto uniformes.
     */
    public Sprite(Image image, int frameWidth, int frameHeight) {
        super(frameWidth, frameHeight);
        if (image == null) {
            throw new NullPointerException("La imagen para Sprite no puede ser nula");
        }
        initialize(image, frameWidth, frameHeight);
    }

    /**
     * Constructor de copia.
     */
    public Sprite(Sprite s) {
        super(s != null ? s.width : 0, s != null ? s.height : 0);
        if (s == null) {
            throw new NullPointerException("El Sprite origen no puede ser nulo");
        }
        this.image = s.image;
        this.srcFrameWidth = s.srcFrameWidth;
        this.srcFrameHeight = s.srcFrameHeight;
        this.cols = s.cols;
        this.rows = s.rows;
        this.totalFrames = s.totalFrames;
        this.currentTransform = s.currentTransform;
        this.refX = s.refX;
        this.refY = s.refY;
        this.x = s.x;
        this.y = s.y;
        this.visible = s.visible;
        this.customCollisionRect = s.customCollisionRect;
        this.colX = s.colX;
        this.colY = s.colY;
        this.colW = s.colW;
        this.colH = s.colH;

        if (s.frameSequence != null) {
            this.frameSequence = s.frameSequence.clone();
        }
        this.currentSequenceIndex = s.currentSequenceIndex;
    }

    private void initialize(Image image, int frameWidth, int frameHeight) {
        if (frameWidth <= 0 || frameHeight <= 0 ||
            image.getWidth() % frameWidth != 0 || image.getHeight() % frameHeight != 0) {
            throw new IllegalArgumentException("Dimensiones de fotograma inválidas o no divisibles exactamente");
        }

        this.image = image;
        this.srcFrameWidth = frameWidth;
        this.srcFrameHeight = frameHeight;
        this.cols = image.getWidth() / frameWidth;
        this.rows = image.getHeight() / frameHeight;
        this.totalFrames = cols * rows;

        this.frameSequence = new int[totalFrames];
        for (int i = 0; i < totalFrames; i++) {
            this.frameSequence[i] = i;
        }
        this.currentSequenceIndex = 0;
        this.colW = frameWidth;
        this.colH = frameHeight;
        updateDimensions();
    }

    private void updateDimensions() {
        boolean swap = (currentTransform == TRANS_ROT90 || currentTransform == TRANS_ROT270 ||
                        currentTransform == TRANS_MIRROR_ROT90 || currentTransform == TRANS_MIRROR_ROT270);
        this.width = swap ? srcFrameHeight : srcFrameWidth;
        this.height = swap ? srcFrameWidth : srcFrameHeight;
    }

    /**
     * Modifica la imagen origen y las dimensiones de sus fotogramas manteniendo la posición.
     */
    public void setImage(Image img, int frameWidth, int frameHeight) {
        int oldRefX = getRefPixelX();
        int oldRefY = getRefPixelY();
        initialize(img, frameWidth, frameHeight);
        setRefPixelPosition(oldRefX, oldRefY);
    }

    /**
     * Define la secuencia de reproducción de los fotogramas (animación).
     */
    public void setFrameSequence(int[] sequence) {
        if (sequence == null) {
            this.frameSequence = new int[totalFrames];
            for (int i = 0; i < totalFrames; i++) {
                this.frameSequence[i] = i;
            }
            this.currentSequenceIndex = 0;
            return;
        }

        if (sequence.length == 0) {
            throw new IllegalArgumentException("Secuencia de fotogramas vacía");
        }

        for (int frame : sequence) {
            if (frame < 0 || frame >= totalFrames) {
                throw new ArrayIndexOutOfBoundsException("Índice de fotograma fuera de rango: " + frame);
            }
        }
        this.frameSequence = sequence.clone();
        this.currentSequenceIndex = 0;
    }

    /**
     * Establece el índice actual dentro de la secuencia de fotogramas.
     */
    public void setFrame(int sequenceIndex) {
        if (sequenceIndex < 0 || sequenceIndex >= frameSequence.length) {
            throw new IndexOutOfBoundsException("Índice de secuencia inválido: " + sequenceIndex);
        }
        this.currentSequenceIndex = sequenceIndex;
    }

    /**
     * Retorna el índice del fotograma activo dentro de la secuencia.
     */
    public int getFrame() {
        return currentSequenceIndex;
    }

    /**
     * Avanza al siguiente fotograma en la secuencia de animación con ciclo continuo.
     */
    public void nextFrame() {
        currentSequenceIndex = (currentSequenceIndex + 1) % frameSequence.length;
    }

    /**
     * Retrocede al fotograma anterior en la secuencia de animación con ciclo continuo.
     */
    public void prevFrame() {
        if (currentSequenceIndex == 0) {
            currentSequenceIndex = frameSequence.length - 1;
        } else {
            currentSequenceIndex--;
        }
    }

    /**
     * Retorna la longitud total de la secuencia de animación activa.
     */
    public int getFrameSequenceLength() {
        return frameSequence.length;
    }

    /**
     * Retorna la cantidad de fotogramas individuales que contiene la imagen origen.
     */
    public int getRawFrameCount() {
        return totalFrames;
    }

    /**
     * Define el punto o píxel de referencia dentro del fotograma original (sin transformar).
     */
    public void defineReferencePixel(int x, int y) {
        this.refX = x;
        this.refY = y;
    }

    /**
     * Posiciona el Sprite de forma que su píxel de referencia coincida con (x, y) absoluto.
     */
    public void setRefPixelPosition(int x, int y) {
        this.x = x - getTransformedRefX();
        this.y = y - getTransformedRefY();
    }

    public int getRefPixelX() {
        return this.x + getTransformedRefX();
    }

    public int getRefPixelY() {
        return this.y + getTransformedRefY();
    }

    private int getTransformedRefX() {
        switch (currentTransform) {
            case TRANS_NONE: return refX;
            case TRANS_MIRROR: return srcFrameWidth - 1 - refX;
            case TRANS_ROT180: return srcFrameWidth - 1 - refX;
            case TRANS_MIRROR_ROT180: return refX;
            case TRANS_ROT90: return srcFrameHeight - 1 - refY;
            case TRANS_ROT270: return refY;
            case TRANS_MIRROR_ROT90: return refY;
            case TRANS_MIRROR_ROT270: return srcFrameHeight - 1 - refY;
            default: return refX;
        }
    }

    private int getTransformedRefY() {
        switch (currentTransform) {
            case TRANS_NONE: return refY;
            case TRANS_MIRROR: return refY;
            case TRANS_ROT180: return srcFrameHeight - 1 - refY;
            case TRANS_MIRROR_ROT180: return srcFrameHeight - 1 - refY;
            case TRANS_ROT90: return refX;
            case TRANS_ROT270: return srcFrameWidth - 1 - refX;
            case TRANS_MIRROR_ROT90: return srcFrameWidth - 1 - refX;
            case TRANS_MIRROR_ROT270: return refX;
            default: return refY;
        }
    }

    /**
     * Aplica una transformación de rotación o reflejo sobre el Sprite manteniendo anclado el punto de referencia.
     */
    public void setTransform(int transform) {
        if (transform < 0 || transform > 7) {
            throw new IllegalArgumentException("Transformación inválida: " + transform);
        }
        if (this.currentTransform == transform) return;

        int oldRefAbsX = getRefPixelX();
        int oldRefAbsY = getRefPixelY();

        this.currentTransform = transform;
        updateDimensions();
        setRefPixelPosition(oldRefAbsX, oldRefAbsY);
    }

    /**
     * Define el rectángulo relativo para la detección de colisiones.
     */
    public void defineCollisionRectangle(int x, int y, int width, int height) {
        if (width < 0 || height < 0) {
            throw new IllegalArgumentException("Dimensiones de colisión inválidas");
        }
        this.customCollisionRect = true;
        this.colX = x;
        this.colY = y;
        this.colW = width;
        this.colH = height;
    }

    // ========================================================================
    // DETECCIÓN DE COLISIONES
    // ========================================================================

    public final boolean collidesWith(Sprite other, boolean pixelLevel) {
        if (other == null || !this.visible || !other.visible) return false;

        // 1. Verificación preliminar de AABB (Bounding Box)
        int ax1 = this.x + (customCollisionRect ? colX : 0);
        int ay1 = this.y + (customCollisionRect ? colY : 0);
        int ax2 = ax1 + (customCollisionRect ? colW : this.width);
        int ay2 = ay1 + (customCollisionRect ? colH : this.height);

        int bx1 = other.x + (other.customCollisionRect ? other.colX : 0);
        int by1 = other.y + (other.customCollisionRect ? other.colY : 0);
        int bx2 = bx1 + (other.customCollisionRect ? other.colW : other.width);
        int by2 = by1 + (other.customCollisionRect ? other.colH : other.height);

        if (ax1 >= bx2 || ax2 <= bx1 || ay1 >= by2 || ay2 <= by1) {
            return false;
        }

        if (!pixelLevel) {
            return true;
        }

        // 2. Colisión precisa a nivel de píxel (compara intersección y canal alfa)
        int intersectX1 = Math.max(this.x, other.x);
        int intersectY1 = Math.max(this.y, other.y);
        int intersectX2 = Math.min(this.x + this.width, other.x + other.width);
        int intersectY2 = Math.min(this.y + this.height, other.y + other.height);

        for (int py = intersectY1; py < intersectY2; py++) {
            for (int px = intersectX1; px < intersectX2; px++) {
                int pixelA = this.getTransformedPixel(px - this.x, py - this.y);
                int pixelB = other.getTransformedPixel(px - other.x, py - other.y);
                if (((pixelA >>> 24) != 0) && ((pixelB >>> 24) != 0)) {
                    return true;
                }
            }
        }
        return false;
    }

    public final boolean collidesWith(TiledLayer other, boolean pixelLevel) {
        if (other == null || !this.visible || !other.visible) return false;
        return other.collidesWith(this, pixelLevel);
    }

    public final boolean collidesWith(Image other, int x, int y, boolean pixelLevel) {
        if (other == null || !this.visible) return false;
        int ax1 = this.x;
        int ay1 = this.y;
        int ax2 = ax1 + this.width;
        int ay2 = ay1 + this.height;

        int bx1 = x;
        int by1 = y;
        int bx2 = x + other.getWidth();
        int by2 = y + other.getHeight();

        if (ax1 >= bx2 || ax2 <= bx1 || ay1 >= by2 || ay2 <= by1) {
            return false;
        }
        if (!pixelLevel) return true;

        int intersectX1 = Math.max(ax1, bx1);
        int intersectY1 = Math.max(ay1, by1);
        int intersectX2 = Math.min(ax2, bx2);
        int intersectY2 = Math.min(ay2, by2);

        int[] otherPixels = other.getInternalPixels();
        int otherW = other.getWidth();

        for (int py = intersectY1; py < intersectY2; py++) {
            for (int px = intersectX1; px < intersectX2; px++) {
                int pixelA = this.getTransformedPixel(px - this.x, py - this.y);
                int pixelB = otherPixels[(py - y) * otherW + (px - x)];
                if (((pixelA >>> 24) != 0) && ((pixelB >>> 24) != 0)) {
                    return true;
                }
            }
        }
        return false;
    }

    private int getTransformedPixel(int x, int y) {
        if (x < 0 || x >= width || y < 0 || y >= height) return 0;
        int rawFrame = frameSequence[currentSequenceIndex];
        int frameCol = rawFrame % cols;
        int frameRow = rawFrame / cols;
        int srcOriginX = frameCol * srcFrameWidth;
        int srcOriginY = frameRow * srcFrameHeight;

        int srcX;
        int srcY;

        switch (currentTransform) {
            case TRANS_NONE:
                srcX = x;
                srcY = y;
                break;
            case TRANS_MIRROR:
                srcX = srcFrameWidth - 1 - x;
                srcY = y;
                break;
            case TRANS_ROT180:
                srcX = srcFrameWidth - 1 - x;
                srcY = srcFrameHeight - 1 - y;
                break;
            case TRANS_MIRROR_ROT180:
                srcX = x;
                srcY = srcFrameHeight - 1 - y;
                break;
            case TRANS_ROT90:
                srcX = y;
                srcY = srcFrameHeight - 1 - x;
                break;
            case TRANS_ROT270:
                srcX = srcFrameWidth - 1 - y;
                srcY = x;
                break;
            case TRANS_MIRROR_ROT90:
                srcX = y;
                srcY = x;
                break;
            case TRANS_MIRROR_ROT270:
                srcX = srcFrameWidth - 1 - y;
                srcY = srcFrameHeight - 1 - x;
                break;
            default:
                srcX = x;
                srcY = y;
        }

        int[] pixels = image.getInternalPixels();
        return pixels[(srcOriginY + srcY) * image.getWidth() + (srcOriginX + srcX)];
    }

    /**
     * Dibuja el fotograma actual del Sprite en el contexto gráfico provisto.
     */
    @Override
    public final void paint(Graphics g) {
        if (!visible || g == null) return;

        int rawFrame = frameSequence[currentSequenceIndex];
        int frameCol = rawFrame % cols;
        int frameRow = rawFrame / cols;
        int srcX = frameCol * srcFrameWidth;
        int srcY = frameRow * srcFrameHeight;

        g.drawRegion(image, srcX, srcY, srcFrameWidth, srcFrameHeight,
                     currentTransform, this.x, this.y, Graphics.TOP | Graphics.LEFT);
    }
}
