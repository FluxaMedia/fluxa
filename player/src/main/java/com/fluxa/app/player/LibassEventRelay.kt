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
        LibassDebugLog.d("relay creating renderer headerBytes=${headerData.size} fonts=${fonts.size} fontsDir=$fontsDir")
        val new = NativeLibassRenderer.create(headerData, fonts, fontsDir)
        if (new == null) {
            LibassDebugLog.w("relay renderer creation returned null")
            return
        }
        renderThread.setRelayRenderer(new)
        LibassDebugLog.d("relay renderer active generation=${generation.get()}")
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
        renderThread.addEvent(line)
        val count = eventCount.incrementAndGet()
        if (count <= 8 || count % 50 == 0) {
            LibassDebugLog.d("relay added ASS event count=$count startMs=$startMs durationMs=$durationMs sourceBytes=${rawMkvBody.size}")
        }
    }

    fun clearEvents() {
        renderThread.clearEvents()
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
