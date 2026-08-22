package com.fluxa.app.player.subtitle

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

interface SubtitleFetcher {
    suspend fun fetchProgressive(
        url: String,
        headers: Map<String, String>,
        onChunk: suspend (String) -> Unit
    )
}

interface SubtitleCueDecoder {
    fun decode(content: String): List<TextCue>
}

interface SidecarTextEngine {
    val frames: StateFlow<SubtitleFrame>
    val cues: StateFlow<List<TextCue>>
    fun load(source: SubtitleSource.Sidecar)
    fun setDelayUs(value: Long)
}

class SidecarTextEngineImpl(
    private val clock: PlaybackClock,
    private val scope: CoroutineScope,
    private val fetcher: SubtitleFetcher,
    private val decoder: SubtitleCueDecoder,
) : SidecarTextEngine {
    private val scheduler = SubtitleScheduler(clock, scope)
    override val frames: StateFlow<SubtitleFrame> = scheduler.frames
    private val _cues = kotlinx.coroutines.flow.MutableStateFlow<List<TextCue>>(emptyList())
    override val cues: StateFlow<List<TextCue>> = _cues

    private var loadJob: Job? = null

    override fun load(source: SubtitleSource.Sidecar) {
        loadJob?.cancel()
        loadJob = scope.launch {
            val buffer = StringBuilder()
            fetcher.fetchProgressive(source.url, source.headers) { chunk ->
                buffer.append(chunk)
                val cues = decoder.decode(buffer.toString())
                if (cues.isNotEmpty()) {
                    _cues.value = cues
                    scheduler.setCueIndex(CueIndex(cues))
                }
            }
        }
    }

    override fun setDelayUs(value: Long) = scheduler.setDelayUs(value)
}
