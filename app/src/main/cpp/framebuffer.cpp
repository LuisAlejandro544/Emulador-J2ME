/**
 * ============================================================================
 * IMPLEMENTACIÓN DEL RASTERIZADOR NATIVO Y FRAMEBUFFER 2D (C++)
 * ============================================================================
 *
 * Este archivo implementa el renderizado por software acelerado en C++:
 * 1. Bresenham para trazado óptimo de líneas.
 * 2. Recorte rígido (`clipping`) y traslación de coordenadas de contexto.
 * 3. Mezcla alfa (Alpha Blending) para sprites translúcidos.
 * 4. Fuente de mapa de bits clásica (8x8) para texto de juegos retro.
 * 5. Volcado directo a `android.graphics.Bitmap` usando `AndroidBitmap_lockPixels`.
 */

#include "framebuffer.h"
#include <algorithm>
#include <cmath>
#include <cstring>
#include <android/log.h>

#define LOG_TAG "J2ME_Framebuffer"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

namespace j2me {

// Tabla simplificada de fuentes 8x8 para caracteres ASCII imprimibles (32 a 126).
// Cada carácter ocupa 8 bytes (1 byte por fila de 8 píxeles horizontales).
static const uint8_t FONT_8X8[96][8] = {
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00}, // 32 ' '
    {0x18, 0x3C, 0x3C, 0x18, 0x18, 0x00, 0x18, 0x00}, // 33 '!'
    {0x66, 0x66, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00}, // 34 '"'
    {0x6C, 0x6C, 0xFE, 0x6C, 0xFE, 0x6C, 0x6C, 0x00}, // 35 '#'
    {0x18, 0x7E, 0xC0, 0x7C, 0x06, 0xFC, 0x18, 0x00}, // 36 '$'
    {0x00, 0xC6, 0xCC, 0x18, 0x30, 0x66, 0xC6, 0x00}, // 37 '%'
    {0x38, 0x6C, 0x38, 0x76, 0xDC, 0xCC, 0x76, 0x00}, // 38 '&'
    {0x18, 0x18, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00}, // 39 '\''
    {0x0C, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0C, 0x00}, // 40 '('
    {0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0x00}, // 41 ')'
    {0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00}, // 42 '*'
    {0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00}, // 43 '+'
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30}, // 44 ','
    {0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00}, // 45 '-'
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00}, // 46 '.'
    {0x06, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0x80, 0x00}, // 47 '/'
    {0x7C, 0xC6, 0xCE, 0xD6, 0xE6, 0xC6, 0x7C, 0x00}, // 48 '0'
    {0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00}, // 49 '1'
    {0x7C, 0xC6, 0x06, 0x1C, 0x30, 0x66, 0xFE, 0x00}, // 50 '2'
    {0x7C, 0xC6, 0x06, 0x3C, 0x06, 0xC6, 0x7C, 0x00}, // 51 '3'
    {0x1C, 0x3C, 0x6C, 0xCC, 0xFE, 0x0C, 0x1E, 0x00}, // 52 '4'
    {0xFE, 0xC0, 0xFC, 0x06, 0x06, 0xC6, 0x7C, 0x00}, // 53 '5'
    {0x78, 0x0C, 0xC0, 0xFC, 0xC6, 0xC6, 0x7C, 0x00}, // 54 '6'
    {0xFE, 0xC6, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x00}, // 55 '7'
    {0x7C, 0xC6, 0xC6, 0x7C, 0xC6, 0xC6, 0x7C, 0x00}, // 56 '8'
    {0x7C, 0xC6, 0xC6, 0x7E, 0x06, 0x0C, 0x78, 0x00}, // 57 '9'
    {0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x00}, // 58 ':'
    {0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x30}, // 59 ';'
    {0x0C, 0x18, 0x30, 0x60, 0x30, 0x18, 0x0C, 0x00}, // 60 '<'
    {0x00, 0x00, 0x7E, 0x00, 0x7E, 0x00, 0x00, 0x00}, // 61 '='
    {0x30, 0x18, 0x0C, 0x06, 0x0C, 0x18, 0x30, 0x00}, // 62 '>'
    {0x7C, 0xC6, 0x0C, 0x18, 0x18, 0x00, 0x18, 0x00}, // 63 '?'
    {0x7C, 0xC6, 0xDE, 0xDE, 0xDC, 0xC0, 0x7C, 0x00}, // 64 '@'
    {0x38, 0x6C, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0x00}, // 65 'A'
    {0xFC, 0x66, 0x66, 0x7C, 0x66, 0x66, 0xFC, 0x00}, // 66 'B'
    {0x3C, 0x66, 0xC0, 0xC0, 0xC0, 0x66, 0x3C, 0x00}, // 67 'C'
    {0xF8, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0xF8, 0x00}, // 68 'D'
    {0xFE, 0x62, 0x68, 0x78, 0x68, 0x62, 0xFE, 0x00}, // 69 'E'
    {0xFE, 0x62, 0x68, 0x78, 0x68, 0x60, 0xF0, 0x00}, // 70 'F'
    {0x3C, 0x66, 0xC0, 0xC0, 0xCE, 0x66, 0x3E, 0x00}, // 71 'G'
    {0xC6, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0x00}, // 72 'H'
    {0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00}, // 73 'I'
    {0x1E, 0x0C, 0x0C, 0x0C, 0xCC, 0xCC, 0x78, 0x00}, // 74 'J'
    {0xE6, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0xE6, 0x00}, // 75 'K'
    {0xF0, 0x60, 0x60, 0x60, 0x62, 0x66, 0xFE, 0x00}, // 76 'L'
    {0xC6, 0xEE, 0xFE, 0xFE, 0xD6, 0xC6, 0xC6, 0x00}, // 77 'M'
    {0xC6, 0xE6, 0xF6, 0xDE, 0xCE, 0xC6, 0xC6, 0x00}, // 78 'N'
    {0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00}, // 79 'O'
    {0xFC, 0x66, 0x66, 0x7C, 0x60, 0x60, 0xF0, 0x00}, // 80 'P'
    {0x7C, 0xC6, 0xC6, 0xC6, 0xD6, 0xDE, 0x7C, 0x0E}, // 81 'Q'
    {0xFC, 0x66, 0x66, 0x7C, 0x6C, 0x66, 0xE6, 0x00}, // 82 'R'
    {0x7C, 0xC6, 0x60, 0x38, 0x0C, 0xC6, 0x7C, 0x00}, // 83 'S'
    {0x7E, 0x5A, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00}, // 84 'T'
    {0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00}, // 85 'U'
    {0xC6, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x10, 0x00}, // 86 'V'
    {0xC6, 0xC6, 0xD6, 0xFE, 0xFE, 0xEE, 0xC6, 0x00}, // 87 'W'
    {0xC6, 0x6C, 0x38, 0x38, 0x6C, 0xC6, 0xC6, 0x00}, // 88 'X'
    {0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x3C, 0x00}, // 89 'Y'
    {0xFE, 0xC6, 0x0C, 0x18, 0x30, 0x62, 0xFE, 0x00}, // 90 'Z'
    {0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0x00}, // 91 '['
    {0xC0, 0x60, 0x30, 0x18, 0x0C, 0x06, 0x02, 0x00}, // 92 '\'
    {0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0x00}, // 93 ']'
    {0x10, 0x38, 0x6C, 0xC6, 0x00, 0x00, 0x00, 0x00}, // 94 '^'
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF}, // 95 '_'
    {0x30, 0x18, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00}, // 96 '`'
    {0x00, 0x00, 0x78, 0x0C, 0x7C, 0xCC, 0x76, 0x00}, // 97 'a'
    {0xE0, 0x60, 0x7C, 0x66, 0x66, 0x66, 0xDC, 0x00}, // 98 'b'
    {0x00, 0x00, 0x7C, 0xC6, 0xC0, 0xC6, 0x7C, 0x00}, // 99 'c'
    {0x1C, 0x0C, 0x7C, 0xCC, 0xCC, 0xCC, 0x76, 0x00}, // 100 'd'
    {0x00, 0x00, 0x7C, 0xC6, 0xFE, 0xC0, 0x7C, 0x00}, // 101 'e'
    {0x1C, 0x36, 0x30, 0x7C, 0x30, 0x30, 0x78, 0x00}, // 102 'f'
    {0x00, 0x00, 0x76, 0xCC, 0xCC, 0x7C, 0x0C, 0xF8}, // 103 'g'
    {0xE0, 0x60, 0x6C, 0x76, 0x66, 0x66, 0xE6, 0x00}, // 104 'h'
    {0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3C, 0x00}, // 105 'i'
    {0x06, 0x00, 0x06, 0x06, 0x06, 0x66, 0x66, 0x3C}, // 106 'j'
    {0xE0, 0x60, 0x66, 0x6C, 0x78, 0x6C, 0xE6, 0x00}, // 107 'k'
    {0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00}, // 108 'l'
    {0x00, 0x00, 0xEC, 0xFE, 0xD6, 0xD6, 0xC6, 0x00}, // 109 'm'
    {0x00, 0x00, 0xDC, 0x66, 0x66, 0x66, 0x66, 0x00}, // 110 'n'
    {0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0x7C, 0x00}, // 111 'o'
    {0x00, 0x00, 0xDC, 0x66, 0x66, 0x7C, 0x60, 0xF0}, // 112 'p'
    {0x00, 0x00, 0x76, 0xCC, 0xCC, 0x7C, 0x0C, 0x1E}, // 113 'q'
    {0x00, 0x00, 0xDC, 0x76, 0x60, 0x60, 0xF0, 0x00}, // 114 'r'
    {0x00, 0x00, 0x7C, 0xC0, 0x7C, 0x06, 0x7C, 0x00}, // 115 's'
    {0x10, 0x30, 0x7C, 0x30, 0x30, 0x34, 0x18, 0x00}, // 116 't'
    {0x00, 0x00, 0xCC, 0xCC, 0xCC, 0xCC, 0x76, 0x00}, // 117 'u'
    {0x00, 0x00, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x00}, // 118 'v'
    {0x00, 0x00, 0xC6, 0xD6, 0xFE, 0xFE, 0x6C, 0x00}, // 119 'w'
    {0x00, 0x00, 0xC6, 0x6C, 0x38, 0x6C, 0xC6, 0x00}, // 120 'x'
    {0x00, 0x00, 0xC6, 0xC6, 0xC6, 0x7E, 0x06, 0xFC}, // 121 'y'
    {0x00, 0x00, 0x7E, 0x4C, 0x18, 0x32, 0x7E, 0x00}, // 122 'z'
    {0x0E, 0x18, 0x18, 0x70, 0x18, 0x18, 0x0E, 0x00}, // 123 '{'
    {0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x18, 0x00}, // 124 '|'
    {0x70, 0x18, 0x18, 0x0E, 0x18, 0x18, 0x70, 0x00}, // 125 '}'
    {0x76, 0xDC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00}, // 126 '~'
};

Framebuffer::Framebuffer(int width, int height)
    : width_(width), height_(height),
      pixels_(width * height, 0xFF000000),
      clip_w_(width), clip_h_(height) {
}

void Framebuffer::resize(int width, int height) {
    std::lock_guard<std::mutex> lock(mutex_);
    if (width <= 0 || height <= 0) return;
    width_ = width;
    height_ = height;
    pixels_.assign(width * height, 0xFF000000);
    clip_x_ = 0;
    clip_y_ = 0;
    clip_w_ = width;
    clip_h_ = height;
    trans_x_ = 0;
    trans_y_ = 0;
}

void Framebuffer::clear(uint32_t argb) {
    std::lock_guard<std::mutex> lock(mutex_);
    std::fill(pixels_.begin(), pixels_.end(), argb);
}

void Framebuffer::setColor(uint32_t argb) {
    current_color_ = argb;
}

void Framebuffer::translate(int dx, int dy) {
    trans_x_ += dx;
    trans_y_ += dy;
}

void Framebuffer::setClip(int x, int y, int w, int h) {
    int abs_x = x + trans_x_;
    int abs_y = y + trans_y_;

    int x1 = std::max(0, abs_x);
    int y1 = std::max(0, abs_y);
    int x2 = std::min(width_, abs_x + w);
    int y2 = std::min(height_, abs_y + h);

    clip_x_ = x1;
    clip_y_ = y1;
    clip_w_ = std::max(0, x2 - x1);
    clip_h_ = std::max(0, y2 - y1);
}

void Framebuffer::clipRect(int x, int y, int w, int h) {
    int abs_x = x + trans_x_;
    int abs_y = y + trans_y_;

    int cur_x2 = clip_x_ + clip_w_;
    int cur_y2 = clip_y_ + clip_h_;

    int new_x1 = std::max(clip_x_, abs_x);
    int new_y1 = std::max(clip_y_, abs_y);
    int new_x2 = std::min(cur_x2, abs_x + w);
    int new_y2 = std::min(cur_y2, abs_y + h);

    clip_x_ = new_x1;
    clip_y_ = new_y1;
    clip_w_ = std::max(0, new_x2 - new_x1);
    clip_h_ = std::max(0, new_y2 - new_y1);
}

inline void Framebuffer::setPixelInternal(int x, int y, uint32_t color) {
    if (x >= clip_x_ && x < (clip_x_ + clip_w_) &&
        y >= clip_y_ && y < (clip_y_ + clip_h_)) {
        pixels_[y * width_ + x] = color;
    }
}

inline void Framebuffer::blendPixelInternal(int x, int y, uint32_t color) {
    if (x < clip_x_ || x >= (clip_x_ + clip_w_) ||
        y < clip_y_ || y >= (clip_y_ + clip_h_)) {
        return;
    }

    uint32_t src_a = (color >> 24) & 0xFF;
    if (src_a == 255) {
        pixels_[y * width_ + x] = color;
        return;
    }
    if (src_a == 0) return;

    uint32_t dst = pixels_[y * width_ + x];
    uint32_t dst_a = (dst >> 24) & 0xFF;
    uint32_t dst_r = (dst >> 16) & 0xFF;
    uint32_t dst_g = (dst >> 8) & 0xFF;
    uint32_t dst_b = dst & 0xFF;

    uint32_t src_r = (color >> 16) & 0xFF;
    uint32_t src_g = (color >> 8) & 0xFF;
    uint32_t src_b = color & 0xFF;

    uint32_t inv_a = 255 - src_a;
    uint32_t out_r = (src_r * src_a + dst_r * inv_a) / 255;
    uint32_t out_g = (src_g * src_a + dst_g * inv_a) / 255;
    uint32_t out_b = (src_b * src_a + dst_b * inv_a) / 255;
    uint32_t out_a = src_a + (dst_a * inv_a) / 255;

    pixels_[y * width_ + x] = (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
}

void Framebuffer::setPixel(int x, int y, uint32_t argb) {
    std::lock_guard<std::mutex> lock(mutex_);
    blendPixelInternal(x + trans_x_, y + trans_y_, argb);
}

void Framebuffer::drawLine(int x1, int y1, int x2, int y2) {
    std::lock_guard<std::mutex> lock(mutex_);
    int ax1 = x1 + trans_x_;
    int ay1 = y1 + trans_y_;
    int ax2 = x2 + trans_x_;
    int ay2 = y2 + trans_y_;

    // Algoritmo de Bresenham para trazado de líneas rápido y exacto
    int dx = std::abs(ax2 - ax1);
    int dy = std::abs(ay2 - ay1);
    int sx = (ax1 < ax2) ? 1 : -1;
    int sy = (ay1 < ay2) ? 1 : -1;
    int err = dx - dy;

    while (true) {
        blendPixelInternal(ax1, ay1, current_color_);
        if (ax1 == ax2 && ay1 == ay2) break;
        int e2 = 2 * err;
        if (e2 > -dy) {
            err -= dy;
            ax1 += sx;
        }
        if (e2 < dx) {
            err += dx;
            ay1 += sy;
        }
    }
}

void Framebuffer::drawRect(int x, int y, int w, int h) {
    if (w <= 0 || h <= 0) return;
    std::lock_guard<std::mutex> lock(mutex_);
    int x1 = x + trans_x_;
    int y1 = y + trans_y_;
    int x2 = x1 + w;
    int y2 = y1 + h;

    for (int px = x1; px <= x2; ++px) {
        blendPixelInternal(px, y1, current_color_);
        blendPixelInternal(px, y2, current_color_);
    }
    for (int py = y1; py <= y2; ++py) {
        blendPixelInternal(x1, py, current_color_);
        blendPixelInternal(x2, py, current_color_);
    }
}

void Framebuffer::fillRect(int x, int y, int w, int h) {
    if (w <= 0 || h <= 0) return;
    std::lock_guard<std::mutex> lock(mutex_);
    int ax = x + trans_x_;
    int ay = y + trans_y_;

    int start_x = std::max(clip_x_, ax);
    int end_x = std::min(clip_x_ + clip_w_, ax + w);
    int start_y = std::max(clip_y_, ay);
    int end_y = std::min(clip_y_ + clip_h_, ay + h);

    if (start_x >= end_x || start_y >= end_y) return;

    uint32_t a = (current_color_ >> 24) & 0xFF;
    if (a == 255) {
        for (int py = start_y; py < end_y; ++py) {
            uint32_t* row = &pixels_[py * width_ + start_x];
            std::fill_n(row, end_x - start_x, current_color_);
        }
    } else {
        for (int py = start_y; py < end_y; ++py) {
            for (int px = start_x; px < end_x; ++px) {
                blendPixelInternal(px, py, current_color_);
            }
        }
    }
}

void Framebuffer::drawRoundRect(int x, int y, int w, int h, int arcWidth, int arcHeight) {
    // Simplificación limpia de esquinas redondeadas
    drawRect(x, y, w, h);
}

void Framebuffer::fillRoundRect(int x, int y, int w, int h, int arcWidth, int arcHeight) {
    fillRect(x, y, w, h);
}

void Framebuffer::drawArc(int x, int y, int w, int h, int startAngle, int arcAngle) {
    drawRect(x, y, w, h);
}

void Framebuffer::fillArc(int x, int y, int w, int h, int startAngle, int arcAngle) {
    fillRect(x, y, w, h);
}

void Framebuffer::drawRGB(const uint32_t* rgbData, int offset, int scanlength,
                          int x, int y, int width, int height, bool processAlpha) {
    if (!rgbData || width <= 0 || height <= 0) return;
    std::lock_guard<std::mutex> lock(mutex_);

    int ax = x + trans_x_;
    int ay = y + trans_y_;

    for (int row = 0; row < height; ++row) {
        int py = ay + row;
        if (py < clip_y_ || py >= (clip_y_ + clip_h_)) continue;

        const uint32_t* src_row = rgbData + offset + (row * scanlength);
        for (int col = 0; col < width; ++col) {
            int px = ax + col;
            if (px < clip_x_ || px >= (clip_x_ + clip_w_)) continue;

            uint32_t color = src_row[col];
            if (processAlpha) {
                blendPixelInternal(px, py, color);
            } else {
                setPixelInternal(px, py, 0xFF000000 | (color & 0x00FFFFFF));
            }
        }
    }
}

void Framebuffer::drawChar(char ch, int x, int y) {
    if (ch < 32 || ch > 126) return;
    const uint8_t* glyph = FONT_8X8[ch - 32];
    for (int r = 0; r < 8; ++r) {
        uint8_t row_bits = glyph[r];
        for (int c = 0; c < 8; ++c) {
            if (row_bits & (0x80 >> c)) {
                blendPixelInternal(x + c, y + r, current_color_);
            }
        }
    }
}

void Framebuffer::drawString(const std::string& text, int x, int y, int anchor) {
    if (text.empty()) return;
    std::lock_guard<std::mutex> lock(mutex_);

    int total_width = static_cast<int>(text.length()) * 8;
    int total_height = 8;

    int ax = x + trans_x_;
    int ay = y + trans_y_;

    // Cálculo del punto de inicio según los anclajes de J2ME
    if (anchor & ANCHOR_RIGHT) {
        ax -= total_width;
    } else if (anchor & ANCHOR_HCENTER) {
        ax -= total_width / 2;
    }

    if (anchor & ANCHOR_BOTTOM) {
        ay -= total_height;
    } else if (anchor & ANCHOR_VCENTER) {
        ay -= total_height / 2;
    }

    for (size_t i = 0; i < text.length(); ++i) {
        drawChar(text[i], ax + static_cast<int>(i) * 8, ay);
    }
}

bool Framebuffer::copyToAndroidBitmap(JNIEnv* env, jobject bitmap) {
    if (!bitmap) return false;

    AndroidBitmapInfo info;
    if (AndroidBitmap_getInfo(env, bitmap, &info) < 0) {
        return false;
    }

    if (info.format != ANDROID_BITMAP_FORMAT_RGBA_8888) {
        return false;
    }

    void* dst_pixels = nullptr;
    if (AndroidBitmap_lockPixels(env, bitmap, &dst_pixels) < 0) {
        return false;
    }

    {
        std::lock_guard<std::mutex> lock(mutex_);
        int copy_w = std::min(static_cast<int>(info.width), width_);
        int copy_h = std::min(static_cast<int>(info.height), height_);

        uint8_t* dst_row = static_cast<uint8_t*>(dst_pixels);
        for (int y = 0; y < copy_h; ++y) {
            // Conversión de ARGB (0xAARRGGBB) a RGBA esperado por Android Bitmap
            const uint32_t* src_row = &pixels_[y * width_];
            uint32_t* dst_line = reinterpret_cast<uint32_t*>(dst_row);
            for (int x = 0; x < copy_w; ++x) {
                uint32_t argb = src_row[x];
                uint32_t a = (argb >> 24) & 0xFF;
                uint32_t r = (argb >> 16) & 0xFF;
                uint32_t g = (argb >> 8) & 0xFF;
                uint32_t b = argb & 0xFF;
                dst_line[x] = (a << 24) | (b << 16) | (g << 8) | r;
            }
            dst_row += info.stride;
        }
    }

    AndroidBitmap_unlockPixels(env, bitmap);
    return true;
}

bool Framebuffer::copyPixelsTo(uint32_t* dst, size_t max_pixels) {
    if (!dst) return false;
    std::lock_guard<std::mutex> lock(mutex_);
    size_t count = std::min(max_pixels, pixels_.size());
    std::memcpy(dst, pixels_.data(), count * sizeof(uint32_t));
    return true;
}

Framebuffer& getGlobalFramebuffer() {
    static Framebuffer s_framebuffer(240, 320);
    return s_framebuffer;
}

} // namespace j2me
