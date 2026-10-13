package com.fluxa.app.ui.rust

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import java.nio.ByteBuffer

object ArtworkDecoder {
    @JvmStatic
    fun decode(bytes: ByteArray, targetWidth: Int, targetHeight: Int): ByteArray? {
        val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        BitmapFactory.decodeByteArray(bytes, 0, bytes.size, bounds)
        if (bounds.outWidth <= 0 || bounds.outHeight <= 0) return null
        val fit = minOf(
            targetWidth.toFloat() / bounds.outWidth,
            targetHeight.toFloat() / bounds.outHeight,
        )
        val options = BitmapFactory.Options().apply {
            inSampleSize = if (fit >= 1f) 1 else Integer.highestOneBit((1f / fit).toInt().coerceAtLeast(1))
            inPreferredConfig = Bitmap.Config.ARGB_8888
        }
        val bitmap = BitmapFactory.decodeByteArray(bytes, 0, bytes.size, options) ?: return null
        if (bitmap.config != Bitmap.Config.ARGB_8888) {
            bitmap.recycle()
            return null
        }
        val out = ByteArray(8 + bitmap.width * bitmap.height * 4)
        val buffer = ByteBuffer.wrap(out)
        buffer.putInt(bitmap.width)
        buffer.putInt(bitmap.height)
        bitmap.copyPixelsToBuffer(buffer)
        bitmap.recycle()
        return out
    }
}
