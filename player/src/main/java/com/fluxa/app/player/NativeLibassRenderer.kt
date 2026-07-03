package com.fluxa.app.player

import android.graphics.Bitmap
import java.io.Closeable

class NativeLibassRenderer private constructor(
    private var handle: Long
) : Closeable {

    fun render(timeMs: Long, bitmap: Bitmap): Int {
        if (handle == 0L || bitmap.isRecycled) return -1
        return nativeRender(handle, timeMs.coerceAtLeast(0L), bitmap)
    }

    fun renderImages(
        timeMs: Long,
        width: Int,
        height: Int,
        outMeta: IntArray,
        outCoverage: ByteArray,
        forceRender: Boolean
    ): Int {
        if (handle == 0L) return -1
        return nativeRenderImages(handle, timeMs.coerceAtLeast(0L), width, height, outMeta, outCoverage, forceRender)
    }

    fun addEvent(dialogueLine: String) {
        if (handle == 0L) return
        nativeAddEvent(handle, dialogueLine)
    }

    fun clearEvents() {
        if (handle == 0L) return
        nativeClearEvents(handle)
    }

    override fun close() {
        val current = handle
        handle = 0L
        if (current != 0L) nativeRelease(current)
    }

    private external fun nativeRender(handle: Long, timeMs: Long, bitmap: Bitmap): Int
    private external fun nativeRenderImages(
        handle: Long,
        timeMs: Long,
        width: Int,
        height: Int,
        outMeta: IntArray,
        outCoverage: ByteArray,
        forceRender: Boolean
    ): Int
    private external fun nativeRelease(handle: Long)
    private external fun nativeAddEvent(handle: Long, dialogueLine: String)
    private external fun nativeClearEvents(handle: Long)

    companion object {
        init {
            System.loadLibrary("fluxa_libass_renderer")
        }

        fun create(
            assData: ByteArray,
            fonts: List<NativeAssFont> = emptyList(),
            fontsDir: String? = null
        ): NativeLibassRenderer? {
            if (assData.isEmpty()) {
                LibassDebugLog.w("native renderer create skipped empty ASS data")
                return null
            }
            LibassDebugLog.d("native renderer create assBytes=${assData.size} fonts=${fonts.size} fontsDir=$fontsDir")
            if (fontsDir != null) {
                runCatching {
                    val cache = java.io.File(fontsDir).resolveSibling("fontconfig-cache")
                    cache.mkdirs()
                    android.system.Os.setenv("XDG_CACHE_HOME", cache.absolutePath, false)
                }
            }
            val handle = runCatching {
                nativeCreate(
                    assData,
                    fonts.map { it.name }.toTypedArray(),
                    fonts.map { it.data }.toTypedArray(),
                    fontsDir
                )
            }.onFailure { error ->
                LibassDebugLog.w("native renderer create threw", error)
            }.getOrDefault(0L)
            if (handle == 0L) {
                LibassDebugLog.w("native renderer create failed handle=0")
                return null
            }
            LibassDebugLog.d("native renderer create succeeded handle=$handle")
            return NativeLibassRenderer(handle)
        }

        fun isAssUrl(url: String): Boolean {
            val path = url.substringBefore('?').substringBefore('#').lowercase()
            return path.endsWith(".ass") || path.endsWith(".ssa")
        }

        private external fun nativeCreate(
            assData: ByteArray,
            fontNames: Array<String>,
            fontData: Array<ByteArray>,
            fontsDir: String?
        ): Long
    }
}
