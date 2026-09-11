package javax.microedition.lcdui.game;

import java.util.ArrayList;
import java.util.List;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.Image;

/**
 * ============================================================================
 * CLASE TiledLayer (MIDP 2.0 Game API)
 * ============================================================================
 *
 * `TiledLayer` representa un mapa o fondo de juego estructurado en cuadrícula (grid)
 * de celdas que reutilizan un conjunto de mosaicos o tiles tomados de una `Image`.
 *
 * Características implementadas:
 * - Mosaicos estáticos (índices 1..N; el índice 0 es celda vacía/transparente).
 * - Mosaicos animados (índices negativos: -1, -2, ...) que permiten animar agua,
 *   fuego o luces en todo el escenario cambiando una sola referencia.
 * - Operaciones por bloque `fillCells` y lectura/escritura individual `getCell`/`setCell`.
 * - Algoritmo de renderizado optimizado por ventana de recorte (frustum clipping).
 * - Detección de colisiones con Sprites y otros TiledLayers.
 */
public class TiledLayer extends Layer {

    private final int columns;
    private final int rows;
    private int tileWidth;
    private int tileHeight;
    private Image image;

    private int numStaticTiles;
    private int tilesPerRow;

    // Matriz de celdas indexada por [col + row * columns]
    private final int[] cells;

    // Lista de punteros para mosaicos animados (los índices de la lista corresponden a: -index - 1)
    private final List<Integer> animTiles = new ArrayList<>();

    public TiledLayer(int columns, int rows, Image image, int tileWidth, int tileHeight) {
        super(columns * tileWidth, rows * tileHeight);

        if (columns <= 0 || rows <= 0 || tileWidth <= 0 || tileHeight <= 0) {
            throw new IllegalArgumentException("Dimensiones de TiledLayer inválidas");
        }
        if (image == null) {
            throw new NullPointerException("La imagen para el conjunto de tiles no puede ser nula");
        }

        this.columns = columns;
        this.rows = rows;
        this.cells = new int[columns * rows];
        setStaticTileSet(image, tileWidth, tileHeight);
    }

    /**
     * Actualiza la imagen o conjunto de mosaicos estáticos del mapa.
     */
    public void setStaticTileSet(Image image, int tileWidth, int tileHeight) {
        if (image == null) {
            throw new NullPointerException("La imagen del conjunto de tiles no puede ser nula");
        }
        if (tileWidth <= 0 || tileHeight <= 0 ||
            image.getWidth() % tileWidth != 0 || image.getHeight() % tileHeight != 0) {
            throw new IllegalArgumentException("Dimensiones de tile no divisibles exactamente en la imagen");
        }

        this.image = image;
        this.tileWidth = tileWidth;
        this.tileHeight = tileHeight;
        this.tilesPerRow = image.getWidth() / tileWidth;
        int tilesPerCol = image.getHeight() / tileHeight;
        this.numStaticTiles = tilesPerRow * tilesPerCol;
        setWidth(columns * tileWidth);
        setHeight(rows * tileHeight);
    }

    /**
     * Crea un nuevo mosaico animado que apunta a un mosaico estático inicial.
     *
     * @param staticTileIndex Índice del tile estático inicial (>= 0).
     * @return Identificador del tile animado (un número negativo: -1, -2, etc.).
     */
    public int createAnimatedTile(int staticTileIndex) {
        validateStaticTileIndex(staticTileIndex);
        animTiles.add(staticTileIndex);
        return -animTiles.size();
    }

    /**
     * Modifica el mosaico estático al que apunta un mosaico animado específico.
     */
    public void setAnimatedTile(int animatedTileIndex, int staticTileIndex) {
        validateAnimatedTileIndex(animatedTileIndex);
        validateStaticTileIndex(staticTileIndex);
        int listIndex = -animatedTileIndex - 1;
        animTiles.set(listIndex, staticTileIndex);
    }

    /**
     * Obtiene el índice del mosaico estático asociado a un mosaico animado.
     */
    public int getAnimatedTile(int animatedTileIndex) {
        validateAnimatedTileIndex(animatedTileIndex);
        int listIndex = -animatedTileIndex - 1;
        return animTiles.get(listIndex);
    }

    /**
     * Asigna un tile (estático, animado o vacío = 0) a una celda específica del mapa.
     */
    public void setCell(int col, int row, int tileIndex) {
        if (col < 0 || col >= columns || row < 0 || row >= rows) {
            throw new IndexOutOfBoundsException("Coordenadas de celda fuera de límites: (" + col + ", " + row + ")");
        }
        validateTileIndex(tileIndex);
        cells[row * columns + col] = tileIndex;
    }

    /**
     * Obtiene el índice de tile asignado a la celda indicada.
     */
    public int getCell(int col, int row) {
        if (col < 0 || col >= columns || row < 0 || row >= rows) {
            throw new IndexOutOfBoundsException("Coordenadas de celda fuera de límites: (" + col + ", " + row + ")");
        }
        return cells[row * columns + col];
    }

    /**
     * Rellena un bloque rectangular de celdas con el tile indicado.
     */
    public void fillCells(int col, int row, int numCols, int numRows, int tileIndex) {
        if (col < 0 || col >= columns || row < 0 || row >= rows ||
            numCols < 0 || col + numCols > columns ||
            numRows < 0 || row + numRows > rows) {
            throw new IndexOutOfBoundsException("Rango de relleno fuera de límites del mapa");
        }
        validateTileIndex(tileIndex);

        for (int r = row; r < row + numRows; r++) {
            for (int c = col; c < col + numCols; c++) {
                cells[r * columns + c] = tileIndex;
            }
        }
    }

    public final int getCellWidth() {
        return tileWidth;
    }

    public final int getCellHeight() {
        return tileHeight;
    }

    public final int getColumns() {
        return columns;
    }

    public final int getRows() {
        return rows;
    }

    private void validateStaticTileIndex(int staticTileIndex) {
        if (staticTileIndex < 0 || staticTileIndex > numStaticTiles) {
            throw new IndexOutOfBoundsException("Índice de tile estático inválido: " + staticTileIndex);
        }
    }

    private void validateAnimatedTileIndex(int animatedTileIndex) {
        if (animatedTileIndex >= 0 || (-animatedTileIndex - 1) >= animTiles.size()) {
            throw new IndexOutOfBoundsException("Índice de tile animado inválido: " + animatedTileIndex);
        }
    }

    private void validateTileIndex(int tileIndex) {
        if (tileIndex > 0) {
            validateStaticTileIndex(tileIndex);
        } else if (tileIndex < 0) {
            validateAnimatedTileIndex(tileIndex);
        }
    }

    private int resolveToStaticTile(int tileIndex) {
        if (tileIndex > 0) return tileIndex;
        if (tileIndex < 0) {
            int listIndex = -tileIndex - 1;
            if (listIndex < animTiles.size()) {
                return animTiles.get(listIndex);
            }
        }
        return 0;
    }

    // ========================================================================
    // DETECCIÓN DE COLISIONES
    // ========================================================================

    public final boolean collidesWith(Sprite sprite, boolean pixelLevel) {
        if (sprite == null || !this.visible || !sprite.isVisible()) return false;

        // Comprobación de solapamiento de cajas AABB
        int sx1 = sprite.getX();
        int sy1 = sprite.getY();
        int sx2 = sx1 + sprite.getWidth();
        int sy2 = sy1 + sprite.getHeight();

        int tx1 = this.x;
        int ty1 = this.y;
        int tx2 = tx1 + this.width;
        int ty2 = ty1 + this.height;

        if (sx1 >= tx2 || sx2 <= tx1 || sy1 >= ty2 || sy2 <= ty1) {
            return false;
        }

        // Determinar qué celdas del TiledLayer se solapan con el Sprite
        int startCol = Math.max(0, (sx1 - tx1) / tileWidth);
        int endCol = Math.min(columns - 1, (sx2 - 1 - tx1) / tileWidth);
        int startRow = Math.max(0, (sy1 - ty1) / tileHeight);
        int endRow = Math.min(rows - 1, (sy2 - 1 - ty1) / tileHeight);

        for (int r = startRow; r <= endRow; r++) {
            for (int c = startCol; c <= endCol; c++) {
                int cellTile = cells[r * columns + c];
                int staticIndex = resolveToStaticTile(cellTile);
                if (staticIndex > 0) {
                    if (!pixelLevel) {
                        return true;
                    }
                    // Comprobación a nivel de píxel con el tile ocupado
                    int tilePixelX = tx1 + c * tileWidth;
                    int tilePixelY = ty1 + r * tileHeight;
                    Image tileSubImage = getTileImage(staticIndex);
                    if (sprite.collidesWith(tileSubImage, tilePixelX, tilePixelY, true)) {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    public final boolean collidesWith(TiledLayer other, boolean pixelLevel) {
        if (other == null || !this.visible || !other.visible) return false;
        int ax1 = this.x;
        int ay1 = this.y;
        int ax2 = ax1 + this.width;
        int ay2 = ay1 + this.height;

        int bx1 = other.x;
        int by1 = other.y;
        int bx2 = bx1 + other.width;
        int by2 = by1 + other.height;

        return !(ax1 >= bx2 || ax2 <= bx1 || ay1 >= by2 || ay2 <= by1);
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

        return !(ax1 >= bx2 || ax2 <= bx1 || ay1 >= by2 || ay2 <= by1);
    }

    private Image getTileImage(int staticTileIndex) {
        int tilePos = staticTileIndex - 1;
        int tileCol = tilePos % tilesPerRow;
        int tileRow = tilePos / tilesPerRow;
        return Image.createImage(image, tileCol * tileWidth, tileRow * tileHeight,
                                 tileWidth, tileHeight, Sprite.TRANS_NONE);
    }

    /**
     * Dibuja los mosaicos visibles en el contexto gráfico aplicando recorte inteligente.
     */
    @Override
    public final void paint(Graphics g) {
        if (!visible || g == null) return;

        int clipX = g.getClipX();
        int clipY = g.getClipY();
        int clipW = g.getClipWidth();
        int clipH = g.getClipHeight();

        // Determinar límites de celdas visibles dentro del área de recorte
        int startCol = Math.max(0, (clipX - this.x) / tileWidth);
        int endCol = Math.min(columns - 1, (clipX + clipW - 1 - this.x) / tileWidth);
        int startRow = Math.max(0, (clipY - this.y) / tileHeight);
        int endRow = Math.min(rows - 1, (clipY + clipH - 1 - this.y) / tileHeight);

        for (int r = startRow; r <= endRow; r++) {
            for (int c = startCol; c <= endCol; c++) {
                int tileIndex = cells[r * columns + c];
                int staticIndex = resolveToStaticTile(tileIndex);
                if (staticIndex > 0) {
                    int tilePos = staticIndex - 1;
                    int tileCol = tilePos % tilesPerRow;
                    int tileRow = tilePos / tilesPerRow;
                    int srcX = tileCol * tileWidth;
                    int srcY = tileRow * tileHeight;

                    int destX = this.x + c * tileWidth;
                    int destY = this.y + r * tileHeight;

                    g.drawRegion(image, srcX, srcY, tileWidth, tileHeight,
                                 Sprite.TRANS_NONE, destX, destY, Graphics.TOP | Graphics.LEFT);
                }
            }
        }
    }
}
