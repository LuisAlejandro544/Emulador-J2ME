/**
 * ============================================================================
 * RASTERIZADOR NATIVO Y FRAMEBUFFER 2D DE ALTO RENDIMIENTO (C++)
 * ============================================================================
 *
 * Este archivo de cabecera define la clase `Framebuffer`:
 * - Manejo de búfer de píxeles ARGB8888 en memoria contigua.
 * - Primitivas 2D aceleradas para la especificación LCDUI de Java ME:
 *   líneas (Bresenham), rectángulos rellenos/huecos, arcos, clipping y traslación.
 * - Fuente bitmap 8x8 integrada para renderizado ultra rápido de texto (`drawString`).
 * - Sincronización segura entre hilos de emulación y dibujado en Android.
 */

#ifndef J2ME_FRAMEBUFFER_H
#define J2ME_FRAMEBUFFER_H

#include <cstdint>
#include <vector>
#include <mutex>
#include <string>
#include <jni.h>
#include <android/bitmap.h>

namespace j2me {

// Constantes de anclaje estándar de J2ME LCDUI Graphics
constexpr int ANCHOR_HCENTER  = 1;
constexpr int ANCHOR_VCENTER  = 2;
constexpr int ANCHOR_LEFT     = 4;
constexpr int ANCHOR_RIGHT    = 8;
constexpr int ANCHOR_TOP      = 16;
constexpr int ANCHOR_BOTTOM   = 32;
constexpr int ANCHOR_BASELINE = 64;

class Framebuffer {
public:
    Framebuffer(int width = 240, int height = 320);
    ~Framebuffer() = default;

    // Inicialización y configuración del búfer
    void resize(int width, int height);
    void clear(uint32_t argb);
    int getWidth() const { return width_; }
    int getHeight() const { return height_; }

    // Estado gráfico de contexto (Graphics context)
    void setColor(uint32_t argb);
    uint32_t getColor() const { return current_color_; }

    void translate(int dx, int dy);
    int getTranslateX() const { return trans_x_; }
    int getTranslateY() const { return trans_y_; }

    void setClip(int x, int y, int w, int h);
    void clipRect(int x, int y, int w, int h);
    int getClipX() const { return clip_x_ - trans_x_; }
    int getClipY() const { return clip_y_ - trans_y_; }
    int getClipWidth() const { return clip_w_; }
    int getClipHeight() const { return clip_h_; }

    // Primitivas de dibujo 2D (coordenadas locales sujetas a traslación y clipping)
    void setPixel(int x, int y, uint32_t argb);
    void drawLine(int x1, int y1, int x2, int y2);
    void drawRect(int x, int y, int w, int h);
    void fillRect(int x, int y, int w, int h);
    void drawRoundRect(int x, int y, int w, int h, int arcWidth, int arcHeight);
    void fillRoundRect(int x, int y, int w, int h, int arcWidth, int arcHeight);
    void drawArc(int x, int y, int w, int h, int startAngle, int arcAngle);
    void fillArc(int x, int y, int w, int h, int startAngle, int arcAngle);

    // Dibujado de arreglos de píxeles (RGB / Sprites)
    void drawRGB(const uint32_t* rgbData, int offset, int scanlength,
                 int x, int y, int width, int height, bool processAlpha);

    // Dibujado de texto usando la fuente bitmap de 8x8 píxeles
    void drawString(const std::string& text, int x, int y, int anchor);
    void drawChar(char ch, int x, int y);

    // Transferencia y copia hacia Android UI
    bool copyToAndroidBitmap(JNIEnv* env, jobject bitmap);
    bool copyPixelsTo(uint32_t* dst, size_t max_pixels);

private:
    int width_;
    int height_;
    std::vector<uint32_t> pixels_;
    mutable std::mutex mutex_;

    // Transformaciones y límites de recorte
    int trans_x_{0};
    int trans_y_{0};
    int clip_x_{0};
    int clip_y_{0};
    int clip_w_{240};
    int clip_h_{320};

    uint32_t current_color_{0xFFFFFFFF}; // Blanco por defecto

    // Funciones internas auxiliares sin lock
    void setPixelInternal(int x, int y, uint32_t color);
    void blendPixelInternal(int x, int y, uint32_t color);
};

// Instancia global del Framebuffer para la sesión actual del emulador
Framebuffer& getGlobalFramebuffer();

} // namespace j2me

#endif // J2ME_FRAMEBUFFER_H
