package com.fluxa.app.player

import androidx.media3.common.C
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.shared.feature.player.PlaybackController
import com.fluxa.app.shared.feature.player.PlayerAction
import com.fluxa.app.shared.feature.player.PlayerContentUiModel
import com.fluxa.app.shared.feature.player.PlayerUiState
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class AndroidExoPlaybackController(
    private val exoPlayer: ExoPlayer,
    content: PlayerContentUiModel? = null
) : PlaybackController, Player.Listener {
    private val mutableState = MutableStateFlow(PlayerUiState(content = content))

    override val state: StateFlow<PlayerUiState> = mutableState.asStateFlow()

    init {
        exoPlayer.addListener(this)
        publish()
    }

    fun updateContent(content: PlayerContentUiModel?) {
        mutableState.value = mutableState.value.copy(content = content)
        publish()
    }

    override suspend fun dispatch(action: PlayerAction) {
        when (action) {
            PlayerAction.PlayPause -> exoPlayer.playWhenReady = !exoPlayer.isPlaying
            is PlayerAction.SeekTo -> exoPlayer.seekTo(action.positionMs.coerceAtLeast(0L))
            is PlayerAction.SetVolume -> exoPlayer.volume = action.volume.coerceIn(0f, 1f)
            is PlayerAction.SetSubtitleEnabled -> {
                exoPlayer.trackSelectionParameters = exoPlayer.trackSelectionParameters
                    .buildUpon()
                    .setTrackTypeDisabled(C.TRACK_TYPE_TEXT, !action.enabled)
                    .build()
                mutableState.value = mutableState.value.copy(subtitleEnabled = action.enabled)
            }
            PlayerAction.Stop -> exoPlayer.stop()
        }
        publish()
    }

    override fun onEvents(player: Player, events: Player.Events) {
        publish()
    }

    fun close() {
        exoPlayer.removeListener(this)
    }

    private fun publish() {
        val duration = exoPlayer.duration.takeUnless { it == C.TIME_UNSET }?.coerceAtLeast(0L) ?: 0L
        mutableState.value = mutableState.value.copy(
            isPlaying = exoPlayer.isPlaying,
            isBuffering = exoPlayer.playbackState == Player.STATE_BUFFERING,
            positionMs = exoPlayer.currentPosition.coerceAtLeast(0L),
            durationMs = duration,
            bufferedPositionMs = exoPlayer.bufferedPosition.coerceAtLeast(0L),
            volume = exoPlayer.volume
        )
    }
}
