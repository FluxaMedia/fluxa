package com.fluxa.app.ui.catalog

import android.graphics.Bitmap
import coil3.size.Size
import coil3.transform.Transformation

internal class TrimTransparentEdgesTransformation : Transformation() {
    override val cacheKey: String = "TrimTransparentEdgesTransformation"

    override suspend fun transform(input: Bitmap, size: Size): Bitmap {
        val width = input.width
        val height = input.height
        if (width <= 0 || height <= 0) return input

        val pixels = IntArray(width * height)
        input.getPixels(pixels, 0, width, 0, 0, width, height)

        var left = width
        var right = -1
        var top = height
        var bottom = -1

        for (y in 0 until height) {
            val rowStart = y * width
            for (x in 0 until width) {
                val alpha = (pixels[rowStart + x] ushr 24) and 0xFF
                if (alpha > ALPHA_THRESHOLD) {
                    if (x < left) left = x
                    if (x > right) right = x
                    if (y < top) top = y
                    if (y > bottom) bottom = y
                }
            }
        }

        if (right < left || bottom < top) return input
        if (left == 0 && top == 0 && right == width - 1 && bottom == height - 1) return input

        return Bitmap.createBitmap(input, left, top, right - left + 1, bottom - top + 1)
    }

    private companion object {
        const val ALPHA_THRESHOLD = 8
    }
}
