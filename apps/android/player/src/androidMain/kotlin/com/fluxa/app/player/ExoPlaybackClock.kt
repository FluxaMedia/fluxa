package com.fluxa.app.player

import androidx.media3.common.PlaybackParameters
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.player.subtitle.MonotonicPlaybackClock

class ExoPlaybackClock(
    private val exoPlayer: ExoPlayer,
    val clock: MonotonicPlaybackClock
) {
    private val listener = object : Player.Listener {
        override fun onPositionDiscontinuity(
            oldPosition: Player.PositionInfo,
            newPosition: Player.PositionInfo,
            reason: Int
        ) = resync(discontinuity = true)

        override fun onIsPlayingChanged(isPlaying: Boolean) = resync(discontinuity = false)
        override fun onPlaybackParametersChanged(playbackParameters: PlaybackParameters) = resync(discontinuity = false)
        override fun onPlaybackStateChanged(playbackState: Int) = resync(discontinuity = false)
    }

    init {
        exoPlayer.addListener(listener)
        resync(discontinuity = false)
    }

    fun release() {
        exoPlayer.removeListener(listener)
    }

    private fun resync(discontinuity: Boolean) {
        val rate = if (exoPlayer.isPlaying) exoPlayer.playbackParameters.speed else 0f
        clock.resync(exoPlayer.currentPosition * 1000L, rate, discontinuity)
    }
}
