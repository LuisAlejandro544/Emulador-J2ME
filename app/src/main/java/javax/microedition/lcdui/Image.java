package javax.microedition.lcdui;

import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.graphics.Matrix;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import com.example.ui.J2meNativeBridge;

/**
 * ============================================================================
 * CLASE Image (J2ME LCDUI)
 * ============================================================================
 *
 * Representa imágenes de mapa de bits tanto mutables (búferes fuera de pantalla
 * sobre los cuales se puede dibujar con `getGraphics()`) como inmutables (cargadas
 * desde recursos PNG/JPEG o transformadas).
 */
public class Image {

    private final int width;
    private final int height;
    private final boolean mutable;
    private int[] rgbPixels;
    private Graphics graphics = null;

    /**
     * Constructor privado para instanciación controlada.
     */
    private Image(int width, int height, boolean mutable, int[] pixels) {
        this.width = width;
        this.height = height;
        this.mutable = mutable;
        this.rgbPixels = pixels != null ? pixels : new int[width * height];
    }

    /**
     * Crea una imagen mutable en blanco con fondo transparente/blanco.
     */
    public static Image createImage(int width, int height) {
        if (width <= 0 || height <= 0) {
            throw new IllegalArgumentException("Dimensiones de imagen inválidas: " + width + "x" + height);
        }
        int[] pixels = new int[width * height];
        java.util.Arrays.fill(pixels, 0xFFFFFFFF);
        return new Image(width, height, true, pixels);
    }

    /**
     * Carga una imagen inmutable desde un recurso del JAR (ej. "/icon.png").
     */
    public static Image createImage(String name) throws IOException {
        if (name == null) {
            throw new NullPointerException("Nombre del recurso nulo");
        }

        // Intento de extracción directa desde el cargador JAR nativo en Rust
        byte[] data = J2meNativeBridge.INSTANCE.extractJarResource(name);
        if (data == null && name.startsWith("/")) {
            data = J2meNativeBridge.INSTANCE.extractJarResource(name.substring(1));
        }

        if (data != null && data.length > 0) {
            return createImage(data, 0, data.length);
        }

        // Intento mediante InputStream estándar de clase
        InputStream is = Image.class.getResourceAsStream(name);
        if (is != null) {
            return createImage(is);
        }

        throw new IOException("Recurso de imagen no encontrado: " + name);
    }

    /**
     * Carga una imagen inmutable desde un flujo de entrada binario (InputStream).
     */
    public static Image createImage(InputStream stream) throws IOException {
        if (stream == null) {
            throw new NullPointerException("Stream de entrada nulo");
        }
        Bitmap bitmap = BitmapFactory.decodeStream(stream);
        if (bitmap == null) {
            throw new IOException("No se pudo decodificar el formato de imagen");
        }
        return fromAndroidBitmap(bitmap);
    }

    /**
     * Carga una imagen inmutable desde un arreglo binario en memoria.
     */
    public static Image createImage(byte[] imageData, int imageOffset, int imageLength) {
        if (imageData == null) {
            throw new NullPointerException("Datos de imagen nulos");
        }
        if (imageOffset < 0 || imageLength <= 0 || imageOffset + imageLength > imageData.length) {
            throw new ArrayIndexOutOfBoundsException("Límites de búfer inválidos");
        }

        Bitmap bitmap = BitmapFactory.decodeByteArray(imageData, imageOffset, imageLength);
        if (bitmap == null) {
            throw new IllegalArgumentException("Formato de imagen no soportado");
        }
        return fromAndroidBitmap(bitmap);
    }

    /**
     * Crea una sub-imagen con soporte para transformaciones estándar de rotación y espejo de MIDP 2.0.
     */
    public static Image createImage(Image source, int x, int y, int width, int height, int transform) {
        if (source == null) {
            throw new NullPointerException("Imagen origen nula");
        }
        if (x < 0 || y < 0 || width <= 0 || height <= 0 ||
            x + width > source.width || y + height > source.height) {
            throw new IllegalArgumentException("Área de corte fuera de los límites de la imagen");
        }

        int[] subPixels = new int[width * height];
        source.getRGB(subPixels, 0, width, x, y, width, height);

        // Si no hay transformación (TRANS_NONE = 0), retorna copia directa
        if (transform == 0) {
            return new Image(width, height, false, subPixels);
        }

        // Aplicar transformaciones J2ME (rotaciones 90, 180, 270 y espejados)
        boolean swapDimensions = (transform == 5 || transform == 6 || transform == 4 || transform == 7);
        int outW = swapDimensions ? height : width;
        int outH = swapDimensions ? width : height;
        int[] transformed = new int[outW * outH];

        for (int r = 0; r < height; ++r) {
            for (int c = 0; c < width; ++c) {
                int srcCol = c;
                int srcRow = r;
                int dstCol = c;
                int dstRow = r;

                switch (transform) {
                    case 2: // MIRROR
                        dstCol = width - 1 - srcCol;
                        dstRow = srcRow;
                        break;
                    case 1: // MIRROR_ROT180
                        dstCol = srcCol;
                        dstRow = height - 1 - srcRow;
                        break;
                    case 3: // ROT180
                        dstCol = width - 1 - srcCol;
                        dstRow = height - 1 - srcRow;
                        break;
                    case 5: // ROT90
                        dstCol = height - 1 - srcRow;
                        dstRow = srcCol;
                        break;
                    case 6: // ROT270
                        dstCol = srcRow;
                        dstRow = width - 1 - srcCol;
                        break;
                    default:
                        break;
                }
                transformed[dstRow * outW + dstCol] = subPixels[srcRow * width + srcCol];
            }
        }

        return new Image(outW, outH, false, transformed);
    }

    /**
     * Crea una imagen inmutable a partir de un arreglo directo de píxeles ARGB.
     */
    public static Image createRGBImage(int[] rgb, int width, int height, boolean processAlpha) {
        if (rgb == null || width <= 0 || height <= 0 || rgb.length < width * height) {
            throw new IllegalArgumentException("Parámetros de datos RGB inválidos");
        }
        int[] copy = new int[width * height];
        if (processAlpha) {
            System.arraycopy(rgb, 0, copy, 0, width * height);
        } else {
            for (int i = 0; i < width * height; ++i) {
                copy[i] = 0xFF000000 | (rgb[i] & 0x00FFFFFF);
            }
        }
        return new Image(width, height, false, copy);
    }

    private static Image fromAndroidBitmap(Bitmap bitmap) {
        int w = bitmap.getWidth();
        int h = bitmap.getHeight();
        int[] pixels = new int[w * h];
        bitmap.getPixels(pixels, 0, w, 0, 0, w, h);
        bitmap.recycle();
        return new Image(w, h, false, pixels);
    }

    public int getWidth() {
        return width;
    }

    public int getHeight() {
        return height;
    }

    public boolean isMutable() {
        return mutable;
    }

    /**
     * Retorna el contexto de dibujo Graphics para imágenes mutables.
     */
    public Graphics getGraphics() {
        if (!mutable) {
            throw new IllegalStateException("No se puede obtener Graphics de una imagen inmutable");
        }
        if (graphics == null) {
            graphics = new Graphics(this);
        }
        return graphics;
    }

    /**
     * Copia píxeles en formato ARGB desde la imagen hacia un arreglo de enteros.
     */
    public void getRGB(int[] rgbData, int offset, int scanlength, int x, int y, int width, int height) {
        if (rgbData == null) {
            throw new NullPointerException("Arreglo de destino rgbData nulo");
        }
        if (width <= 0 || height <= 0) return;
        if (x < 0 || y < 0 || x + width > this.width || y + height > this.height) {
            throw new IllegalArgumentException("Región fuera de los límites de la imagen");
        }

        for (int r = 0; r < height; ++r) {
            int srcPos = (y + r) * this.width + x;
            int dstPos = offset + (r * scanlength);
            System.arraycopy(rgbPixels, srcPos, rgbData, dstPos, width);
        }
    }

    public int[] getInternalPixels() {
        return rgbPixels;
    }
}
