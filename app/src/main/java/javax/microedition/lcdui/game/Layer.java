package javax.microedition.lcdui.game;

import javax.microedition.lcdui.Graphics;

/**
 * ============================================================================
 * CLASE ABSTRACTA Layer (MIDP 2.0 Game API)
 * ============================================================================
 *
 * `Layer` es la clase base para todos los elementos visuales interactivos en
 * la Game API de J2ME, como `Sprite` y `TiledLayer`.
 *
 * Provee:
 * - Posicionamiento bidimensional en pantalla (coordenadas x, y).
 * - Dimensiones del plano visual (ancho, alto).
 * - Visibilidad conmutada para optimizar el ciclo de renderizado.
 * - Método abstracto `paint(Graphics g)` implementado por cada subclase.
 */
public abstract class Layer {

    int x;
    int y;
    int width;
    int height;
    boolean visible = true;

    /**
     * Constructor protegido para capas con dimensiones iniciales.
     *
     * @param width  Ancho de la capa en píxeles (debe ser >= 0).
     * @param height Alto de la capa en píxeles (debe ser >= 0).
     */
    protected Layer(int width, int height) {
        setWidth(width);
        setHeight(height);
    }

    /**
     * Establece la posición absoluta de la capa en el lienzo de juego.
     */
    public void setPosition(int x, int y) {
        this.x = x;
        this.y = y;
    }

    /**
     * Desplaza la capa una cantidad relativa de píxeles respecto a su posición actual.
     */
    public void move(int dx, int dy) {
        this.x += dx;
        this.y += dy;
    }

    /**
     * Retorna la coordenada horizontal actual de la esquina superior izquierda.
     */
    public final int getX() {
        return x;
    }

    /**
     * Retorna la coordenada vertical actual de la esquina superior izquierda.
     */
    public final int getY() {
        return y;
    }

    /**
     * Retorna el ancho total de la capa en píxeles.
     */
    public final int getWidth() {
        return width;
    }

    /**
     * Retorna el alto total de la capa en píxeles.
     */
    public final int getHeight() {
        return height;
    }

    /**
     * Configura la visibilidad de la capa. Si es falsa, no se dibuja en `paint()`.
     */
    public void setVisible(boolean visible) {
        this.visible = visible;
    }

    /**
     * Retorna true si la capa está configurada como visible.
     */
    public final boolean isVisible() {
        return visible;
    }

    /**
     * Asigna el ancho de la capa validando que no sea negativo.
     */
    protected void setWidth(int width) {
        if (width < 0) {
            throw new IllegalArgumentException("El ancho de Layer debe ser >= 0");
        }
        this.width = width;
    }

    /**
     * Asigna el alto de la capa validando que no sea negativo.
     */
    protected void setHeight(int height) {
        if (height < 0) {
            throw new IllegalArgumentException("El alto de Layer debe ser >= 0");
        }
        this.height = height;
    }

    /**
     * Dibuja la capa sobre el contexto gráfico provisto respetando su visibilidad y posición.
     */
    public abstract void paint(Graphics g);
}
