package com.fluxa.app.player

import android.graphics.Bitmap
import android.graphics.Color
import android.graphics.Paint
import android.graphics.PorterDuff
import android.os.Handler
import android.os.HandlerThread
import android.util.LongSparseArray
import android.view.Surface
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import java.nio.ByteBuffer

class LibassRenderThread {
    private val thread = HandlerThread("fluxa-libass").also { it.start() }
    private val handler = Handler(thread.looper)

    private var relayRenderer: NativeLibassRenderer? = null
    private var localRenderer: NativeLibassRenderer? = null
    private var activeRenderer: NativeLibassRenderer? = null

    private var surface: Surface? = null
    private var surfaceWidth = 0
    private var surfaceHeight = 0
    private var delayMs = 0L

    private val glyphCache = LongSparseArray<Bitmap>()
    private val outMeta = IntArray(1 + 200 * 9)
    private val outCoverage = ByteArray(4 * 1024 * 1024)
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
            val new = factory() ?: return@post
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
        for (i in 0 until glyphCache.size()) {
            glyphCache.valueAt(i).recycle()
        }
        glyphCache.clear()
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

        val forceRender = forceNextRender
        val count = r.renderImages(ptsMs + delayMs, surfaceWidth, surfaceHeight, outMeta, outCoverage, forceRender)
        if (count < 0) return

        if (glyphCache.size() > 256) clearGlyphCache()

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
                val coverageOffset = outMeta[metaIdx + 7]
                val coverageLen = outMeta[metaIdx + 8]
                metaIdx += 9

                val key = glyphKey(coverageOffset, coverageLen, w, h)
                val glyph = glyphCache[key]?.takeIf { !it.isRecycled && it.width == w && it.height == h }
                    ?: createGlyph(w, h, coverageOffset, coverageLen).also { glyphCache.put(key, it) }

                val r_ch = (assColor ushr 24) and 0xFF
                val g_ch = (assColor ushr 16) and 0xFF
                val b_ch = (assColor ushr 8) and 0xFF
                val alpha = 255 - (assColor and 0xFF)
                paint.color = (alpha shl 24) or (r_ch shl 16) or (g_ch shl 8) or b_ch
                canvas.drawBitmap(glyph, x.toFloat(), y.toFloat(), paint)
            }
        } finally {
            s.unlockCanvasAndPost(canvas)
        }
    }

    private fun glyphKey(offset: Int, len: Int, w: Int, h: Int): Long {
        var hash = -0x340d631b7bdddcdbL
        hash = (hash xor w.toLong()) * 0x100000001b3L
        hash = (hash xor h.toLong()) * 0x100000001b3L
        for (i in offset until offset + len) {
            hash = (hash xor (outCoverage[i].toLong() and 0xff)) * 0x100000001b3L
        }
        return hash
    }

    private fun createGlyph(w: Int, h: Int, offset: Int, len: Int): Bitmap {
        val srcStride = (w + 3) and 3.inv()
        val bmp = Bitmap.createBitmap(w, h, Bitmap.Config.ALPHA_8)
        val rowBytes = bmp.rowBytes
        if (rowBytes == srcStride) {
            bmp.copyPixelsFromBuffer(ByteBuffer.wrap(outCoverage, offset, len))
        } else {
            val packed = ByteArray(rowBytes * h)
            for (y in 0 until h) {
                System.arraycopy(outCoverage, offset + y * srcStride, packed, y * rowBytes, w)
            }
            bmp.copyPixelsFromBuffer(ByteBuffer.wrap(packed))
        }
        return bmp
    }
}
