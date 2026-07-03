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

    fun setRelayRenderer(r: NativeLibassRenderer?) {
        handler.post {
            val old = relayRenderer
            relayRenderer = r
            if (old !== r) old?.close()
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
            surface = s
            surfaceWidth = w
            surfaceHeight = h
        }
    }

    fun setDelay(ms: Long) {
        handler.post { delayMs = ms }
    }

    fun onVideoFrame(ptsMs: Long) {
        pendingPtsMs = ptsMs
        handler.removeCallbacks(renderRunnable)
        handler.post(renderRunnable)
    }

    fun addEvent(line: String) {
        handler.post { activeRenderer?.addEvent(line) }
    }

    fun clearEvents() {
        handler.post { activeRenderer?.clearEvents() }
    }

    fun drainForTesting() {
        val latch = java.util.concurrent.CountDownLatch(1)
        handler.post { latch.countDown() }
        latch.await()
    }

    fun close() {
        handler.post {
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
        }
    }

    private fun clearGlyphCache() {
        for (i in 0 until glyphCache.size()) {
            glyphCache.valueAt(i).recycle()
        }
        glyphCache.clear()
    }

    private fun onRender(ptsMs: Long) {
        val r = activeRenderer ?: return
        val s = surface ?: return
        if (surfaceWidth <= 0 || surfaceHeight <= 0) return

        val count = r.renderImages(ptsMs + delayMs, surfaceWidth, surfaceHeight, outMeta, outCoverage)
        if (count < 0) return

        val canvas = try { s.lockHardwareCanvas() } catch (_: Exception) { null } ?: return
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
                val ptrHi = outMeta[metaIdx + 5]
                val ptrLo = outMeta[metaIdx + 6]
                val coverageOffset = outMeta[metaIdx + 7]
                val coverageLen = outMeta[metaIdx + 8]
                metaIdx += 9

                val ptr = (ptrHi.toLong() shl 32) or (ptrLo.toLong() and 0xFFFFFFFFL)

                val glyph = glyphCache[ptr]?.takeIf { !it.isRecycled && it.width == w && it.height == h }
                    ?: Bitmap.createBitmap(w, h, Bitmap.Config.ALPHA_8).also { bmp ->
                        bmp.copyPixelsFromBuffer(ByteBuffer.wrap(outCoverage, coverageOffset, coverageLen))
                        glyphCache.put(ptr, bmp)
                    }

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
}
