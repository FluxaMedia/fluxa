package com.fluxa.app.player

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import java.util.Locale
import java.util.concurrent.atomic.AtomicInteger

class LibassEventRelay {
    private val _activeRenderer = MutableStateFlow<NativeLibassRenderer?>(null)
    val activeRenderer: StateFlow<NativeLibassRenderer?> = _activeRenderer

    @Volatile var selectedTrackId: Int? = null
        private set

    private val generation = AtomicInteger(0)

    fun headerGeneration(): Int = generation.get()

    fun setSelectedTrackId(trackId: Int?) {
        if (selectedTrackId == trackId) return
        selectedTrackId = trackId
        generation.incrementAndGet()
        clearEvents()
    }

    fun setHeader(headerData: ByteArray, fonts: List<NativeAssFont>, fontsDir: String?) {
        val new = NativeLibassRenderer.create(headerData, fonts, fontsDir)
        val old = _activeRenderer.value
        _activeRenderer.value = new
        old?.close()
    }

    fun addEvent(startMs: Long, durationMs: Long, rawMkvBody: ByteArray) {
        val renderer = _activeRenderer.value ?: return
        val endMs = startMs + durationMs
        val body = rawMkvBody.decodeToString().trim()
        val fields = body.split(',', limit = 9)
        if (fields.size < 9) return
        val line = "Dialogue: ${fields[1]},${formatAssTime(startMs)},${formatAssTime(endMs)},${fields[2]},${fields[3]},${fields[4]},${fields[5]},${fields[6]},${fields[7]},${fields[8]}"
        renderer.addEvent(line)
    }

    fun clearEvents() {
        _activeRenderer.value?.clearEvents()
    }

    fun close() {
        val old = _activeRenderer.value
        _activeRenderer.value = null
        old?.close()
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
