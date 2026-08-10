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

    private var collectJob: Job? = null

    fun selectSidecar(source: SubtitleSource.Sidecar) {
        sidecar.load(source)
        collect(sidecar.frames)
    }

    fun selectEmbedded() {
        collect(embedded.frames)
    }

    fun clear() {
        collectJob?.cancel()
        collectJob = null
        _frames.value = SubtitleFrame(emptyList(), null)
    }

    fun setDelayUs(value: Long) {
        sidecar.setDelayUs(value)
        embedded.setDelayUs(value)
    }

    fun onEmbeddedSample(startUs: Long, endUs: Long, rawText: String) {
        embedded.onSample(startUs, endUs, rawText)
        selectEmbedded()
    }

    fun loadEmbeddedCues(fullFileCues: List<TextCue>) {
        embedded.loadEmbeddedCues(fullFileCues)
        selectEmbedded()
    }

    fun resetEmbedded() = embedded.reset()

    private fun collect(source: StateFlow<SubtitleFrame>) {
        collectJob?.cancel()
        collectJob = scope.launch { source.collect { _frames.value = it } }
    }
}
