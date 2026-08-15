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

    fun onSample(startUs: Long, endUs: Long, rawText: String) {
        val text = stripMarkup(rawText)
        if (text.isEmpty() || endUs <= startUs) return
        cueBuffer += TextCue(startUs, endUs, text)
        _cues.value = cueBuffer.toList()
        scheduler.setCueIndex(CueIndex(cueBuffer.toList()))
    }

    fun loadEmbeddedCues(fullFileCues: List<TextCue>) {
        cueBuffer.clear()
        cueBuffer += fullFileCues
        _cues.value = fullFileCues
        scheduler.setCueIndex(CueIndex(cueBuffer.toList()))
    }

    fun reset() {
        cueBuffer.clear()
        _cues.value = emptyList()
        scheduler.setCueIndex(CueIndex(emptyList()))
    }

    fun setDelayUs(value: Long) = scheduler.setDelayUs(value)
}
