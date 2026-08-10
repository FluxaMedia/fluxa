package com.fluxa.app.player

import com.fluxa.app.player.subtitle.MonotonicPlaybackClock
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

class MpvPlaybackClock(
    private val player: MpvEmbeddedPlayer,
    scope: CoroutineScope,
    val clock: MonotonicPlaybackClock
) {
    @Volatile private var currentSpeed = 1f
    private var lastPositionMs = -1L

    init {
        scope.launch {
            player.state.collect { state ->
                val jumped = lastPositionMs >= 0 &&
                    kotlin.math.abs(state.positionMs - lastPositionMs) > RESYNC_THRESHOLD_MS
                lastPositionMs = state.positionMs
                val rate = if (state.isPlaying) currentSpeed else 0f
                clock.resync(state.positionMs * 1000L, rate, discontinuity = jumped)
            }
        }
    }

    fun setSpeed(speed: Float) {
        currentSpeed = speed
        val rate = if (player.state.value.isPlaying) speed else 0f
        clock.resync(clock.positionUs(), rate, discontinuity = false)
    }

    private companion object {
        const val RESYNC_THRESHOLD_MS = 750L
    }
}
