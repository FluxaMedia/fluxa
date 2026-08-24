package com.fluxa.app.player.subtitle

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.MutableStateFlow

class EmbeddedTextEngine(
    clock: PlaybackClock,
    scope: CoroutineScope
) {
    private val scheduler = SubtitleScheduler(clock, scope)
    val frames: StateFlow<SubtitleFrame> = scheduler.frames
    private val _cues = MutableStateFlow<List<TextCue>>(emptyList())
    val cues: StateFlow<List<TextCue>> = _cues

    private val cueBuffer = mutableListOf<TextCue>()
    private val known = mutableSetOf<TextCue>()

    fun onSample(startUs: Long, endUs: Long, rawText: String) {
        val text = stripMarkup(rawText)
        if (text.isEmpty() || endUs <= startUs) return
        addCues(listOf(TextCue(startUs, endUs, text)))
    }

    fun addEmbeddedCues(cues: List<TextCue>) {
        addCues(cues)
    }

    fun reset() {
        cueBuffer.clear()
        known.clear()
        _cues.value = emptyList()
        scheduler.setCueIndex(CueIndex(emptyList()))
    }

    private fun addCues(cues: List<TextCue>) {
        val fresh = cues.filter { known.add(it) }
        if (fresh.isEmpty()) return
        cueBuffer += fresh
        val snapshot = cueBuffer.sortedBy { it.startUs }
        _cues.value = snapshot
        scheduler.setCueIndex(CueIndex(snapshot))
    }

    fun setDelayUs(value: Long) = scheduler.setDelayUs(value)
}
