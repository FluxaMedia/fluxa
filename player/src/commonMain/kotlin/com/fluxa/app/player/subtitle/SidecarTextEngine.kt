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

interface SidecarTextEngine {
    val frames: StateFlow<SubtitleFrame>
    fun load(source: SubtitleSource.Sidecar)
    fun setDelayUs(value: Long)
}

class SidecarTextEngineImpl(
    private val clock: PlaybackClock,
    private val scope: CoroutineScope,
    private val fetcher: SubtitleFetcher
) : SidecarTextEngine {
    private val scheduler = SubtitleScheduler(clock, scope)
    override val frames: StateFlow<SubtitleFrame> = scheduler.frames

    private var loadJob: Job? = null

    override fun load(source: SubtitleSource.Sidecar) {
        loadJob?.cancel()
        loadJob = scope.launch {
            val parser = cueParserFor(source.format)
            val buffer = StringBuilder()
            fetcher.fetchProgressive(source.url, source.headers) { chunk ->
                buffer.append(chunk)
                val cues = parser.parse(buffer.toString())
                if (cues.isNotEmpty()) scheduler.setCueIndex(CueIndex(cues))
            }
        }
    }

    override fun setDelayUs(value: Long) = scheduler.setDelayUs(value)
}
