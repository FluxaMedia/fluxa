package com.fluxa.app.player

import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.shared.feature.player.PlaybackController
import com.fluxa.app.shared.feature.player.PlayerAction
import com.fluxa.app.shared.feature.player.PlayerContentUiModel
import com.fluxa.app.shared.feature.player.PlayerUiState
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class AndroidPlaybackController(
    private val player: ExoPlayer,
    private val content: () -> PlayerContentUiModel?
) : PlaybackController, Player.Listener {
    private val mutableState = MutableStateFlow(snapshot())
    override val state: StateFlow<PlayerUiState> = mutableState.asStateFlow()

    init {
        player.addListener(this)
    }

    override suspend fun dispatch(action: PlayerAction) {
        when (action) {
            PlayerAction.PlayPause -> if (player.isPlaying) player.pause() else player.play()
            is PlayerAction.SeekTo -> player.seekTo(action.positionMs)
            is PlayerAction.SetVolume -> player.volume = action.volume.coerceIn(0f, 1f)
            is PlayerAction.SetSubtitleEnabled -> Unit
            PlayerAction.Stop -> player.stop()
        }
        mutableState.value = snapshot()
    }

    override fun onEvents(player: Player, events: Player.Events) {
        mutableState.value = snapshot()
    }

    fun close() {
        player.removeListener(this)
    }

    private fun snapshot(): PlayerUiState {
        return PlayerUiState(
            content = content(),
            isPlaying = player.isPlaying,
            isBuffering = player.playbackState == Player.STATE_BUFFERING,
            positionMs = player.currentPosition.coerceAtLeast(0L),
            durationMs = player.duration.coerceAtLeast(0L),
            bufferedPositionMs = player.bufferedPosition.coerceAtLeast(0L),
            volume = player.volume
        )
    }
}
