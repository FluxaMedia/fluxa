package com.fluxa.app.ui.catalog

import android.graphics.Bitmap
import android.graphics.BitmapFactory
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
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.media3.exoplayer.ExoPlayer
import kotlinx.coroutines.delay
import kotlinx.coroutines.suspendCancellableCoroutine
import java.io.ByteArrayOutputStream
import kotlin.coroutines.resume

internal val LocalSeekSurfaceView = compositionLocalOf<SurfaceView?> { null }
internal val LocalSeekExoPlayer = compositionLocalOf<ExoPlayer?> { null }

private const val BucketMs = 10_000L
private const val BackgroundCaptureIntervalMs = 60_000L
private const val ThumbWidth = 480
private const val ThumbHeight = 270
private const val CacheMaxBytes = 32 * 1024 * 1024

@Composable
internal fun rememberSeekThumbnail(
    surfaceView: SurfaceView?,
    scrubPosition: Long,
    isScrubbing: Boolean,
    livePosition: Long,
    isPlaying: Boolean
): ImageBitmap? {
    var thumbnail by remember { mutableStateOf<ImageBitmap?>(null) }
    val cache = remember {
        object : LruCache<Long, ByteArray>(CacheMaxBytes) {
            override fun sizeOf(key: Long, value: ByteArray) = value.size
        }
    }
    val bucket = (scrubPosition / BucketMs) * BucketMs

    LaunchedEffect(bucket, isScrubbing) {
        if (!isScrubbing) { thumbnail = null; return@LaunchedEffect }
        cache[bucket]?.let { thumbnail = decodeJpeg(it) }
    }

    val latestIsScrubbing by rememberUpdatedState(isScrubbing)
    val latestIsPlaying by rememberUpdatedState(isPlaying)
    val latestLivePosition by rememberUpdatedState(livePosition)
    LaunchedEffect(surfaceView) {
        val sv = surfaceView ?: return@LaunchedEffect
        if (android.os.Build.VERSION.SDK_INT < 26) return@LaunchedEffect
        var lastCapturedAt = -1L
        while (true) {
            delay(2000)
            if (latestIsScrubbing || !latestIsPlaying) continue
            val pos = latestLivePosition
            if (lastCapturedAt >= 0 && pos - lastCapturedAt < BackgroundCaptureIntervalMs) continue
            val posBucket = (pos / BucketMs) * BucketMs
            if (cache[posBucket] != null) { lastCapturedAt = pos; continue }
            val captured = capturePixelCopy(sv)
            if (captured != null) {
                cache.put(posBucket, compressToJpeg(captured))
                lastCapturedAt = pos
            }
        }
    }

    return thumbnail
}

private fun compressToJpeg(captured: Bitmap): ByteArray {
    val composed = Bitmap.createBitmap(ThumbWidth, ThumbHeight, Bitmap.Config.ARGB_8888)
    val canvas = android.graphics.Canvas(composed)
    canvas.drawColor(android.graphics.Color.BLACK)
    canvas.drawBitmap(captured, 0f, 0f, null)
    captured.recycle()
    val out = ByteArrayOutputStream()
    composed.compress(Bitmap.CompressFormat.JPEG, 75, out)
    composed.recycle()
    return out.toByteArray()
}

private fun decodeJpeg(bytes: ByteArray): ImageBitmap? =
    runCatching { BitmapFactory.decodeByteArray(bytes, 0, bytes.size)?.asImageBitmap() }.getOrNull()

@RequiresApi(26)
private suspend fun capturePixelCopy(surfaceView: SurfaceView): Bitmap? =
    suspendCancellableCoroutine { cont ->
        val dest = Bitmap.createBitmap(ThumbWidth, ThumbHeight, Bitmap.Config.ARGB_8888)
        runCatching {
            PixelCopy.request(surfaceView, null, dest, { result ->
                if (result == PixelCopy.SUCCESS) cont.resume(dest)
                else { dest.recycle(); cont.resume(null) }
            }, Handler(Looper.getMainLooper()))
        }.onFailure { dest.recycle(); cont.resume(null) }
    }
