package com.fluxa.app.player

import kotlinx.coroutines.flow.StateFlow
import java.util.Locale
import java.util.concurrent.atomic.AtomicInteger

class LibassEventRelay {
    val renderThread = LibassRenderThread()

    val activeRenderer: StateFlow<NativeLibassRenderer?> = renderThread.activeRendererFlow

    @Volatile var selectedTrackId: Int? = null
        private set

    private val generation = AtomicInteger(0)
    private val eventCount = AtomicInteger(0)

    private val lock = Any()
    private var lastHeader: ByteArray? = null
    private var lastFontsDir: String? = null
    private var lastFonts: List<NativeAssFont> = emptyList()
    private var pendingFonts: List<NativeAssFont> = emptyList()
    private val eventLines = mutableListOf<String>()

    fun headerGeneration(): Int = generation.get()

    fun setSelectedTrackId(trackId: Int?) {
        if (selectedTrackId == trackId) return
        LibassDebugLog.d("relay selected track changed from $selectedTrackId to $trackId")
        selectedTrackId = trackId
        generation.incrementAndGet()
        eventCount.set(0)
        clearEvents()
    }

    fun setHeader(headerData: ByteArray, fonts: List<NativeAssFont>, fontsDir: String?) {
        val effectiveFonts = synchronized(lock) {
            val chosen = if (fonts.isEmpty()) pendingFonts else fonts
            lastHeader = headerData
            lastFontsDir = fontsDir
            lastFonts = chosen
            chosen
        }
        LibassDebugLog.d("relay creating renderer headerBytes=${headerData.size} fonts=${effectiveFonts.size} fontsDir=$fontsDir")
        renderThread.setRelayRendererAsync {
            val new = NativeLibassRenderer.create(headerData, effectiveFonts, fontsDir)
            if (new == null) {
                LibassDebugLog.w("relay renderer creation returned null")
            } else {
                LibassDebugLog.d("relay renderer active generation=${generation.get()}")
            }
            new
        }
    }

    fun hasFonts(): Boolean = synchronized(lock) {
        lastFonts.isNotEmpty() || pendingFonts.isNotEmpty()
    }

    fun resetFonts() {
        synchronized(lock) {
            lastHeader = null
            lastFontsDir = null
            lastFonts = emptyList()
            pendingFonts = emptyList()
            eventLines.clear()
        }
        LibassDebugLog.d("relay font state reset for new stream")
    }

    fun updateFonts(fonts: List<NativeAssFont>) {
        if (fonts.isEmpty()) return
        val header: ByteArray?
        val dir: String?
        synchronized(lock) {
            if (lastFonts.isNotEmpty()) return
            pendingFonts = fonts
            header = lastHeader
            dir = lastFontsDir
        }
        if (header == null) {
            LibassDebugLog.d("relay stored late fonts=${fonts.size}, renderer not created yet")
            return
        }
        LibassDebugLog.d("relay recreating renderer with late fonts=${fonts.size}")
        val replay = synchronized(lock) { eventLines.toList() }
        setHeader(header, fonts, dir)
        replay.forEach { renderThread.addRelayEvent(it) }
        if (replay.isNotEmpty()) LibassDebugLog.d("relay replayed events=${replay.size} after font update")
    }

    fun addEvent(startMs: Long, durationMs: Long, rawMkvBody: ByteArray) {
        val endMs = startMs + durationMs
        val body = rawMkvBody.decodeToString().trim()
        val line = if (body.startsWith("Dialogue:", ignoreCase = true)) {
            val fields = body.substringAfter(':').trim().split(',', limit = 11)
            if (fields.size < 11) {
                LibassDebugLog.w("relay could not parse Media3 Dialogue sample fields=${fields.size} bodyPrefix=${body.take(96)}")
                return
            }
            "Dialogue: ${fields[3]},${formatAssTime(startMs)},${formatAssTime(endMs)},${fields[4]},${fields[5]},${fields[6]},${fields[7]},${fields[8]},${fields[9]},${fields[10]}"
        } else {
            val fields = body.split(',', limit = 9)
            if (fields.size < 9) {
                LibassDebugLog.w("relay could not parse raw MKV ASS sample fields=${fields.size} bodyPrefix=${body.take(96)}")
                return
            }
            "Dialogue: ${fields[1]},${formatAssTime(startMs)},${formatAssTime(endMs)},${fields[2]},${fields[3]},${fields[4]},${fields[5]},${fields[6]},${fields[7]},${fields[8]}"
        }
        synchronized(lock) { eventLines += line }
        renderThread.addRelayEvent(line)
        val count = eventCount.incrementAndGet()
        if (count <= 8 || count % 50 == 0) {
            LibassDebugLog.d("relay added ASS event count=$count startMs=$startMs durationMs=$durationMs sourceBytes=${rawMkvBody.size}")
        }
    }

    fun clearEvents() {
        synchronized(lock) { eventLines.clear() }
        renderThread.clearRelayEvents()
        LibassDebugLog.d("relay cleared events")
    }

    fun close() {
        renderThread.close()
        LibassDebugLog.d("relay closed renderer")
    }

    private fun formatAssTime(ms: Long): String {
        val totalCentis = ms.coerceAtLeast(0L) / 10L
        val centis = totalCentis % 100
        val totalSeconds = totalCentis / 100
        val seconds = totalSeconds % 60
        val totalMinutes = totalSeconds / 60
        val minutes = totalMinutes % 60
        val hours = totalMinutes / 60
        return "%d:%02d:%02d.%02d".format(Locale.US, hours, minutes, seconds, centis)
    }
}
