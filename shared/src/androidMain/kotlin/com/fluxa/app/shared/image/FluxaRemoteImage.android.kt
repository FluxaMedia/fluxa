package com.fluxa.app.shared.image

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import coil3.Bitmap
import coil3.compose.AsyncImage
import coil3.compose.LocalPlatformContext
import coil3.request.ImageRequest
import coil3.request.crossfade
import coil3.request.transformations
import coil3.size.Size
import coil3.transform.Transformation

@Composable
actual fun FluxaRemoteImage(
    imageUrl: String?,
    cacheKey: String?,
    diskCacheKey: String?,
    requestWidthPx: Int?,
    requestHeightPx: Int?,
    contentDescription: String?,
    modifier: Modifier,
    contentScale: ContentScale,
    alignment: Alignment,
    onError: (() -> Unit)?,
    trimTransparentPadding: Boolean
) {
    val context = LocalPlatformContext.current
    val request = remember(context, imageUrl, cacheKey, diskCacheKey, requestWidthPx, requestHeightPx, trimTransparentPadding) {
        ImageRequest.Builder(context)
            .data(imageUrl?.let(::sanitizeImageUrl))
            .crossfade(false)
            .memoryCacheKey(cacheKey?.let { if (trimTransparentPadding) "$it|alpha-trim-v1" else it })
            .diskCacheKey(diskCacheKey ?: imageUrl)
            .apply {
                if (trimTransparentPadding) transformations(AndroidAlphaTrimTransformation)
                if (requestWidthPx != null && requestHeightPx != null &&
                    requestWidthPx > 0 && requestHeightPx > 0
                ) {
                    size(requestWidthPx, requestHeightPx)
                }
            }
            .build()
    }
    AsyncImage(
        model = request,
        contentDescription = contentDescription,
        modifier = modifier,
        contentScale = contentScale,
        alignment = alignment,
        onError = { onError?.invoke() }
    )
}


private object AndroidAlphaTrimTransformation : Transformation() {
    override val cacheKey: String = "fluxa-alpha-trim-v1"

    override suspend fun transform(input: Bitmap, size: Size): Bitmap {
        val width = input.width
        val height = input.height
        if (width <= 1 || height <= 1 || !input.hasAlpha()) return input
        val pixels = IntArray(width * height)
        input.getPixels(pixels, 0, width, 0, 0, width, height)
        val threshold = 8
        var left = width
        var top = height
        var right = -1
        var bottom = -1
        pixels.forEachIndexed { index, color ->
            if ((color ushr 24) > threshold) {
                val x = index % width
                val y = index / width
                if (x < left) left = x
                if (x > right) right = x
                if (y < top) top = y
                if (y > bottom) bottom = y
            }
        }
        if (right < left || bottom < top) return input
        val paddingX = ((right - left + 1) * 0.025f).toInt().coerceAtLeast(2)
        val paddingY = ((bottom - top + 1) * 0.04f).toInt().coerceAtLeast(2)
        left = (left - paddingX).coerceAtLeast(0)
        top = (top - paddingY).coerceAtLeast(0)
        right = (right + paddingX).coerceAtMost(width - 1)
        bottom = (bottom + paddingY).coerceAtMost(height - 1)
        if (left == 0 && top == 0 && right == width - 1 && bottom == height - 1) return input
        return android.graphics.Bitmap.createBitmap(input, left, top, right - left + 1, bottom - top + 1)
    }
}
