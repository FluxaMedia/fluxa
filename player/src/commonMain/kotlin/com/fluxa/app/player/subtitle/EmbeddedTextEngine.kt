package com.fluxa.app.player.subtitle

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.StateFlow

class EmbeddedTextEngine(
    clock: PlaybackClock,
    scope: CoroutineScope
) {
    private val scheduler = SubtitleScheduler(clock, scope)
    val frames: StateFlow<SubtitleFrame> = scheduler.frames

    private val cues = mutableListOf<TextCue>()

    fun onSample(startUs: Long, endUs: Long, rawText: String) {
        val text = stripMarkup(rawText)
        if (text.isEmpty() || endUs <= startUs) return
        cues += TextCue(startUs, endUs, text)
        scheduler.setCueIndex(CueIndex(cues.toList()))
    }

    fun loadEmbeddedCues(fullFileCues: List<TextCue>) {
        cues.clear()
        cues += fullFileCues
        scheduler.setCueIndex(CueIndex(cues.toList()))
    }

    fun reset() {
        cues.clear()
        scheduler.setCueIndex(CueIndex(emptyList()))
    }

    fun setDelayUs(value: Long) = scheduler.setDelayUs(value)
}
