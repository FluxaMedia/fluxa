package com.fluxa.app.ui.catalog

import android.graphics.Bitmap
import android.os.Handler
import android.os.Looper
import android.util.LruCache
import android.view.PixelCopy
import android.view.SurfaceView
import androidx.annotation.RequiresApi
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import kotlinx.coroutines.delay
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlin.coroutines.resume

internal val LocalSeekSurfaceView = compositionLocalOf<SurfaceView?> { null }

@Composable
internal fun rememberSeekThumbnail(
    surfaceView: SurfaceView?,
    scrubPosition: Long,
    isScrubbing: Boolean
): ImageBitmap? {
    var thumbnail by remember { mutableStateOf<ImageBitmap?>(null) }
    val cache = remember { LruCache<Long, ImageBitmap>(30) }

    LaunchedEffect(scrubPosition, isScrubbing) {
        if (!isScrubbing) { thumbnail = null; return@LaunchedEffect }
        val sv = surfaceView ?: return@LaunchedEffect
        if (android.os.Build.VERSION.SDK_INT < 26) return@LaunchedEffect

        val quantized = (scrubPosition / 1000L) * 1000L
        cache[quantized]?.let { thumbnail = it; return@LaunchedEffect }

        // Wait for the existing scrub seek (80ms debounce) to render the frame.
        delay(220)
        if (!isScrubbing) return@LaunchedEffect

        val bitmap = Bitmap.createBitmap(320, 180, Bitmap.Config.ARGB_8888)
        val captured = capturePixelCopy(sv, bitmap) ?: return@LaunchedEffect

        val imgBitmap = captured.asImageBitmap()
        cache.put(quantized, imgBitmap)
        thumbnail = imgBitmap
    }

    return thumbnail
}

@RequiresApi(26)
private suspend fun capturePixelCopy(surfaceView: SurfaceView, dest: Bitmap): Bitmap? =
    suspendCancellableCoroutine { cont ->
        runCatching {
            PixelCopy.request(surfaceView, null, dest, { result ->
                cont.resume(if (result == PixelCopy.SUCCESS) dest else null)
            }, Handler(Looper.getMainLooper()))
        }.onFailure { cont.resume(null) }
    }
