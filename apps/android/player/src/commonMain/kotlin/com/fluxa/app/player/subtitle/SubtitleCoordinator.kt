package com.fluxa.app.player.subtitle

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

class SubtitleCoordinator(
    private val sidecar: SidecarTextEngine,
    private val embedded: EmbeddedTextEngine,
    private val scope: CoroutineScope
) {
    private val _frames = MutableStateFlow(SubtitleFrame(emptyList(), null))
    val frames: StateFlow<SubtitleFrame> = _frames
    private val _cues = MutableStateFlow<List<TextCue>>(emptyList())
    val cues: StateFlow<List<TextCue>> = _cues

    private enum class Active { Sidecar, Embedded }

    private var framesJob: Job? = null
    private var cuesJob: Job? = null
    private var active: Active? = null

    fun selectSidecar(source: SubtitleSource.Sidecar) {
        sidecar.load(source)
        if (active == Active.Sidecar) return
        active = Active.Sidecar
        bind(sidecar.frames, sidecar.cues)
    }

    fun selectEmbedded() {
        if (active == Active.Embedded) return
        active = Active.Embedded
        bind(embedded.frames, embedded.cues)
    }

    fun clear() {
        framesJob?.cancel()
        framesJob = null
        cuesJob?.cancel()
        cuesJob = null
        active = null
        _frames.value = SubtitleFrame(emptyList(), null)
        _cues.value = emptyList()
    }

    fun setDelayUs(value: Long) {
        sidecar.setDelayUs(value)
        embedded.setDelayUs(value)
    }

    fun onEmbeddedSample(startUs: Long, endUs: Long, rawText: String) {
        embedded.onSample(startUs, endUs, rawText)
        selectEmbedded()
    }

    fun addEmbeddedCues(cues: List<TextCue>) {
        embedded.addEmbeddedCues(cues)
        selectEmbedded()
    }

    fun resetEmbedded() = embedded.reset()

    private fun bind(frames: StateFlow<SubtitleFrame>, cues: StateFlow<List<TextCue>>) {
        framesJob?.cancel()
        cuesJob?.cancel()
        framesJob = scope.launch { frames.collect { _frames.value = it } }
        cuesJob = scope.launch { cues.collect { _cues.value = it } }
    }
}
