package com.fluxa.app.player

import android.graphics.Bitmap
import java.io.Closeable

class NativeLibassRenderer private constructor(
    private var handle: Long
) : Closeable {
    private val lock = java.util.concurrent.locks.ReentrantLock()

    fun render(timeMs: Long, bitmap: Bitmap): Boolean {
        if (handle == 0L || bitmap.isRecycled) return false
        lock.lock()
        return try {
            nativeRender(handle, timeMs.coerceAtLeast(0L), bitmap)
        } finally {
            lock.unlock()
        }
    }

    fun addEvent(dialogueLine: String) {
        if (handle == 0L) return
        lock.lock()
        try { nativeAddEvent(handle, dialogueLine) } finally { lock.unlock() }
    }

    fun clearEvents() {
        if (handle == 0L) return
        lock.lock()
        try { nativeClearEvents(handle) } finally { lock.unlock() }
    }

    override fun close() {
        lock.lock()
        try {
            val current = handle
            handle = 0L
            if (current != 0L) nativeRelease(current)
        } finally {
            lock.unlock()
        }
    }

    private external fun nativeRender(handle: Long, timeMs: Long, bitmap: Bitmap): Boolean
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
            if (assData.isEmpty()) return null
            val handle = nativeCreate(
                assData,
                fonts.map { it.name }.toTypedArray(),
                fonts.map { it.data }.toTypedArray(),
                fontsDir
            )
            return handle.takeIf { it != 0L }?.let(::NativeLibassRenderer)
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
