package com.fluxa.app.player

import android.graphics.Bitmap
import android.graphics.Color
import android.graphics.Paint
import android.graphics.PorterDuff
import android.os.Handler
import android.os.HandlerThread
import android.view.Surface
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import java.nio.ByteBuffer

data class LibassVideoFrame(
    val width: Int,
    val height: Int,
    val offsetX: Int,
    val offsetY: Int
)

class LibassRenderThread(private val maxGlyphCacheBytes: Long = DEFAULT_MAX_GLYPH_CACHE_BYTES) {
    private val thread = HandlerThread("fluxa-libass").also { it.start() }
    private val handler = Handler(thread.looper)

    private var relayRenderer: NativeLibassRenderer? = null
    private var localRenderer: NativeLibassRenderer? = null
    private var activeRenderer: NativeLibassRenderer? = null

    private var surface: Surface? = null
    private var surfaceWidth = 0
    private var surfaceHeight = 0
    private var videoFrame: LibassVideoFrame? = null
    private var delayMs = 0L

    private val glyphCache = java.util.LinkedHashMap<Long, Bitmap>(128, 0.75f, true)
    private var glyphCacheBytes = 0L
    private val outMeta = IntArray(1 + MAX_ASS_IMAGES * 9)
    // The 4 MB alpha coverage scratch buffer is only needed when libass actually
    // renders. Most playback never selects ASS, so don't reserve native memory up front.
    private val outCoverage by lazy(LazyThreadSafetyMode.NONE) { ByteBuffer.allocateDirect(ASS_COVERAGE_BYTES) }
    private val paint = Paint(Paint.ANTI_ALIAS_FLAG)

    private val _activeRendererFlow = MutableStateFlow<NativeLibassRenderer?>(null)
    val activeRendererFlow: StateFlow<NativeLibassRenderer?> = _activeRendererFlow

    private val renderRunnable = Runnable {
        val pts = pendingPtsMs
        if (pts != Long.MIN_VALUE) onRender(pts)
    }
    @Volatile private var pendingPtsMs = Long.MIN_VALUE
    private var forceNextRender = true

    fun setRelayRenderer(r: NativeLibassRenderer?) {
        handler.post {
            val old = relayRenderer
            relayRenderer = r
            if (old !== r) old?.close()
            updateActive()
        }
    }

    fun setRelayRendererAsync(factory: () -> NativeLibassRenderer?) {
        handler.post {
            val new = factory()
            val old = relayRenderer
            relayRenderer = new
            if (old !== new) old?.close()
            updateActive()
        }
    }

    fun setLocalRenderer(r: NativeLibassRenderer?) {
        handler.post {
            val old = localRenderer
            localRenderer = r
            if (old !== r) old?.close()
            updateActive()
        }
    }

    fun setSurface(s: Surface?, w: Int, h: Int) {
        handler.post {
            val changed = surface !== s || surfaceWidth != w || surfaceHeight != h
            surface = s
            surfaceWidth = w
            surfaceHeight = h
            if (changed) {
                forceNextRender = true
                requestRenderLocked()
            }
        }
    }

    fun setDelay(ms: Long) {
        handler.post {
            if (delayMs != ms) {
                delayMs = ms
                forceNextRender = true
                requestRenderLocked()
            }
        }
    }

    fun setVideoFrame(frame: LibassVideoFrame?) {
        handler.post {
            if (videoFrame != frame) {
                videoFrame = frame
                forceNextRender = true
                requestRenderLocked()
            }
        }
    }

    fun onVideoFrame(ptsMs: Long) {
        pendingPtsMs = ptsMs
        handler.removeCallbacks(renderRunnable)
        handler.post(renderRunnable)
    }

    fun addRelayEvent(line: String) {
        handler.post {
            relayRenderer?.addEvent(line)
            forceNextRender = true
            requestRenderLocked()
        }
    }

    fun clearRelayEvents() {
        handler.post {
            relayRenderer?.clearEvents()
            forceNextRender = true
            requestRenderLocked()
        }
    }

    fun drainForTesting() {
        val latch = java.util.concurrent.CountDownLatch(1)
        handler.post { latch.countDown() }
        latch.await()
    }

    fun close() {
        handler.post {
            handler.removeCallbacks(renderRunnable)
            relayRenderer?.close()
            localRenderer?.close()
            relayRenderer = null
            localRenderer = null
            activeRenderer = null
            _activeRendererFlow.value = null
            surface = null
            clearGlyphCache()
            thread.quit()
        }
    }

    private fun updateActive() {
        val new = localRenderer ?: relayRenderer
        if (new !== activeRenderer) {
            activeRenderer = new
            _activeRendererFlow.value = new
            clearGlyphCache()
            forceNextRender = true
            requestRenderLocked()
        }
    }

    private fun clearGlyphCache() {
        glyphCache.values.forEach { bitmap -> if (!bitmap.isRecycled) bitmap.recycle() }
        glyphCache.clear()
        glyphCacheBytes = 0L
    }

    private fun trimGlyphCache() {
        while (glyphCache.size > MAX_GLYPH_CACHE_ENTRIES || glyphCacheBytes > maxGlyphCacheBytes) {
            val iterator = glyphCache.entries.iterator()
            if (!iterator.hasNext()) return
            val eldest = iterator.next()
            glyphCacheBytes = (glyphCacheBytes - eldest.value.allocationByteCount.toLong()).coerceAtLeast(0L)
            if (!eldest.value.isRecycled) eldest.value.recycle()
            iterator.remove()
        }
    }

    private fun requestRenderLocked() {
        if (pendingPtsMs == Long.MIN_VALUE) return
        handler.removeCallbacks(renderRunnable)
        handler.post(renderRunnable)
    }

    private fun onRender(ptsMs: Long) {
        val s = surface ?: return
        if (surfaceWidth <= 0 || surfaceHeight <= 0) return

        val r = activeRenderer
        if (r == null) {
            if (!forceNextRender) return
            val canvas = try { s.lockHardwareCanvas() } catch (_: Exception) { null } ?: return
            forceNextRender = false
            canvas.drawColor(Color.TRANSPARENT, PorterDuff.Mode.CLEAR)
            s.unlockCanvasAndPost(canvas)
            return
        }

        val frame = videoFrame ?: LibassVideoFrame(surfaceWidth, surfaceHeight, 0, 0)
        val forceRender = forceNextRender
        val count = r.renderImages(ptsMs + delayMs, frame.width, frame.height, outMeta, outCoverage, forceRender)
        if (count < 0) return

        val canvas = try { s.lockHardwareCanvas() } catch (_: Exception) { null } ?: return
        forceNextRender = false
        try {
            canvas.drawColor(Color.TRANSPARENT, PorterDuff.Mode.CLEAR)
            if (count == 0) return

            var metaIdx = 1
            for (i in 0 until count) {
                val x = outMeta[metaIdx]
                val y = outMeta[metaIdx + 1]
                val w = outMeta[metaIdx + 2]
                val h = outMeta[metaIdx + 3]
                val assColor = outMeta[metaIdx + 4]
                val coverageHashHigh = outMeta[metaIdx + 5]
                val coverageHashLow = outMeta[metaIdx + 6]
                val coverageOffset = outMeta[metaIdx + 7]
                val coverageLen = outMeta[metaIdx + 8]
                metaIdx += 9

                val key = (coverageHashHigh.toLong() shl 32) or (coverageHashLow.toLong() and 0xFFFF_FFFFL)
                val cachedGlyph = glyphCache[key]
                val glyph = cachedGlyph?.takeIf { !it.isRecycled && it.width == w && it.height == h }
                    ?: createGlyph(w, h, coverageOffset, coverageLen).also { created ->
                        if (cachedGlyph != null) {
                            glyphCache.remove(key)
                            glyphCacheBytes = (glyphCacheBytes - cachedGlyph.allocationByteCount.toLong()).coerceAtLeast(0L)
                            if (!cachedGlyph.isRecycled) cachedGlyph.recycle()
                        }
                        glyphCache[key] = created
                        glyphCacheBytes += created.allocationByteCount.toLong()
                        trimGlyphCache()
                    }

                val r_ch = (assColor ushr 24) and 0xFF
                val g_ch = (assColor ushr 16) and 0xFF
                val b_ch = (assColor ushr 8) and 0xFF
                val alpha = 255 - (assColor and 0xFF)
                paint.color = (alpha shl 24) or (r_ch shl 16) or (g_ch shl 8) or b_ch
                canvas.drawBitmap(glyph, (x + frame.offsetX).toFloat(), (y + frame.offsetY).toFloat(), paint)
            }
        } finally {
            s.unlockCanvasAndPost(canvas)
        }
    }

    private fun createGlyph(w: Int, h: Int, offset: Int, len: Int): Bitmap {
        val srcStride = (w + 3) and 3.inv()
        val bmp = Bitmap.createBitmap(w, h, Bitmap.Config.ALPHA_8)
        val rowBytes = bmp.rowBytes
        if (rowBytes == srcStride) {
            val source = outCoverage.duplicate().apply {
                position(offset)
                limit(offset + len)
            }.slice()
            bmp.copyPixelsFromBuffer(source)
        } else {
            val packed = ByteArray(rowBytes * h)
            val source = outCoverage.duplicate()
            for (y in 0 until h) {
                source.position(offset + y * srcStride)
                source.get(packed, y * rowBytes, w)
            }
            bmp.copyPixelsFromBuffer(ByteBuffer.wrap(packed))
        }
        return bmp
    }

    companion object {
        const val DEFAULT_MAX_GLYPH_CACHE_BYTES = 12L * 1024L * 1024L
        const val MAX_ASS_IMAGES = 200
        const val ASS_COVERAGE_BYTES = 4 * 1024 * 1024
        const val MAX_GLYPH_CACHE_ENTRIES = 192
    }
}
