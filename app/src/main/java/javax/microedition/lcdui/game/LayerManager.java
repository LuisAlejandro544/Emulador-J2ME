package javax.microedition.lcdui.game;

import java.util.ArrayList;
import java.util.List;
import javax.microedition.lcdui.Graphics;

/**
 * ============================================================================
 * CLASE LayerManager (MIDP 2.0 Game API)
 * ============================================================================
 *
 * `LayerManager` gestiona una colección ordenada de capas (`Layer`, como `Sprite`
 * y `TiledLayer`), administrando el orden de profundidad (z-order) y la cámara
 * o ventana de visualización (View Window / scrolling del mundo del juego).
 *
 * En MIDP 2.0:
 * - El índice 0 representa la capa más cercana al espectador (primer plano).
 * - El índice `getSize() - 1` representa la capa de fondo más alejada.
 * - Al dibujar, las capas se renderizan desde el fondo hacia el primer plano.
 */
public class LayerManager {

    private final List<Layer> layers = new ArrayList<>();

    // Ventana de visualización (cámara en coordenadas del mundo)
    private int viewX = 0;
    private int viewY = 0;
    private int viewWidth = 240;
    private int viewHeight = 320;

    public LayerManager() {
    }

    /**
     * Añade una capa al final de la lista (fondo del escenario).
     */
    public void append(Layer l) {
        if (l == null) {
            throw new NullPointerException("La capa no puede ser nula");
        }
        layers.remove(l);
        layers.add(l);
    }

    /**
     * Inserta una capa en un índice de profundidad específico (0 = primer plano).
     */
    public void insert(Layer l, int index) {
        if (l == null) {
            throw new NullPointerException("La capa no puede ser nula");
        }
        if (index < 0 || index > layers.size()) {
            throw new IndexOutOfBoundsException("Índice de inserción fuera de límites: " + index);
        }
        layers.remove(l);
        layers.add(index, l);
    }

    /**
     * Remueve la capa indicada de la lista si está presente.
     */
    public void remove(Layer l) {
        if (l == null) {
            throw new NullPointerException("La capa no puede ser nula");
        }
        layers.remove(l);
    }

    /**
     * Retorna la capa ubicada en el índice de orden Z indicado.
     */
    public Layer get(int index) {
        if (index < 0 || index >= layers.size()) {
            throw new IndexOutOfBoundsException("Índice de capa fuera de límites: " + index);
        }
        return layers.get(index);
    }

    /**
     * Retorna la cantidad de capas gestionadas actualmente.
     */
    public int getSize() {
        return layers.size();
    }

    /**
     * Configura la ventana de la cámara sobre el mundo del juego.
     *
     * @param x      Coordenada X del mundo visible en la esquina superior izquierda.
     * @param y      Coordenada Y del mundo visible en la esquina superior izquierda.
     * @param width  Ancho de la ventana visible en pantalla.
     * @param height Alto de la ventana visible en pantalla.
     */
    public void setViewWindow(int x, int y, int width, int height) {
        if (width < 0 || height < 0) {
            throw new IllegalArgumentException("Dimensiones de ViewWindow inválidas");
        }
        this.viewX = x;
        this.viewY = y;
        this.viewWidth = width;
        this.viewHeight = height;
    }

    /**
     * Dibuja todas las capas visibles en el contexto gráfico provisto.
     *
     * @param g Contexto gráfico destino (usualmente `GameCanvas.getGraphics()`).
     * @param x Coordenada X en pantalla donde se sitúa la ventana de visualización.
     * @param y Coordenada Y en pantalla donde se sitúa la ventana de visualización.
     */
    public void paint(Graphics g, int x, int y) {
        if (g == null) {
            throw new NullPointerException("El contexto Graphics no puede ser nulo");
        }

        int origClipX = g.getClipX();
        int origClipY = g.getClipY();
        int origClipW = g.getClipWidth();
        int origClipH = g.getClipHeight();

        // Aplicar recorte a la ventana visible de la cámara
        g.clipRect(x, y, viewWidth, viewHeight);
        g.translate(x - viewX, y - viewY);

        // Renderizar desde el fondo (último índice) hacia el frente (índice 0)
        for (int i = layers.size() - 1; i >= 0; i--) {
            Layer layer = layers.get(i);
            if (layer.isVisible()) {
                layer.paint(g);
            }
        }

        // Restaurar traslación y recorte original
        g.translate(-(x - viewX), -(y - viewY));
        g.setClip(origClipX, origClipY, origClipW, origClipH);
    }
}
