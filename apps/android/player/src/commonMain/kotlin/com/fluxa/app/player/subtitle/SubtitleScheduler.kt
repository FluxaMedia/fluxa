package com.fluxa.app.player.subtitle

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

class SubtitleScheduler(
    private val clock: PlaybackClock,
    private val scope: CoroutineScope
) {
    private var cueIndex = CueIndex(emptyList())
    private var delayUs = 0L
    private var loopJob: Job? = null

    private val _frames = MutableStateFlow(SubtitleFrame(emptyList(), null))
    val frames: StateFlow<SubtitleFrame> = _frames

    init {
        scope.launch { clock.discontinuities.collect { restart() } }
        scope.launch { clock.playbackRate.collect { restart() } }
    }

    fun setCueIndex(index: CueIndex) {
        cueIndex = index
        restart()
    }

    fun setDelayUs(value: Long) {
        delayUs = value
        restart()
    }

    private fun restart() {
        loopJob?.cancel()
        loopJob = scope.launch { loop() }
    }

    private suspend fun loop() {
        while (true) {
            val rate = clock.playbackRate.value
            val positionUs = clock.positionUs() - delayUs
            val active = cueIndex.activeAt(positionUs)
            val boundaryUs = cueIndex.nextBoundaryUs(positionUs)
            _frames.value = SubtitleFrame(active, boundaryUs)

            if (boundaryUs == null || rate <= 0f) {
                delay(Long.MAX_VALUE)
                return
            }
            val waitMs = ((boundaryUs - positionUs) / rate).toLong() / 1000L
            delay(waitMs.coerceAtLeast(0L))
        }
    }
}
