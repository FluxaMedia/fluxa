package com.fluxa.app.ui.rust

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Paint

object EmojiRasterizer {
    private val paint = Paint(Paint.ANTI_ALIAS_FLAG)

    @JvmStatic
    fun render(text: String, size: Int): IntArray? {
        paint.textSize = size * 0.86f
        val metrics = paint.fontMetrics
        val width = paint.measureText(text)
        if (width <= 0f) return null
        val bitmap = Bitmap.createBitmap(size, size, Bitmap.Config.ARGB_8888)
        val baseline = (size - (metrics.descent - metrics.ascent)) / 2f - metrics.ascent
        Canvas(bitmap).drawText(text, (size - width) / 2f, baseline, paint)
        val pixels = IntArray(size * size + 2)
        pixels[0] = size
        pixels[1] = size
        bitmap.getPixels(pixels, 2, size, 0, 0, size, size)
        bitmap.recycle()
        return pixels
    }
}
