package com.fluxa.app.shared.feature.player

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

interface PlayerCommandSink {
    suspend fun setPlaying(playing: Boolean)
    suspend fun seekTo(positionMs: Long)
    suspend fun setVolume(volume: Float)
    suspend fun setSubtitleEnabled(enabled: Boolean)
    suspend fun stop()
}

class PlayerStateStore(
    private val commandSink: PlayerCommandSink,
    initialState: PlayerUiState = PlayerUiState()
) : PlaybackController {
    private val mutableState = MutableStateFlow(initialState)
    override val state: StateFlow<PlayerUiState> = mutableState.asStateFlow()

    override suspend fun dispatch(action: PlayerAction) {
        when (action) {
            PlayerAction.PlayPause -> {
                val playing = !state.value.isPlaying
                commandSink.setPlaying(playing)
                mutableState.value = state.value.copy(isPlaying = playing)
            }
            is PlayerAction.SeekTo -> {
                val position = action.positionMs.coerceIn(0L, state.value.durationMs.coerceAtLeast(0L))
                commandSink.seekTo(position)
                mutableState.value = state.value.copy(positionMs = position)
            }
            is PlayerAction.SetVolume -> {
                val volume = action.volume.coerceIn(0f, 1f)
                commandSink.setVolume(volume)
                mutableState.value = state.value.copy(volume = volume)
            }
            is PlayerAction.SetSubtitleEnabled -> {
                commandSink.setSubtitleEnabled(action.enabled)
                mutableState.value = state.value.copy(subtitleEnabled = action.enabled)
            }
            PlayerAction.Stop -> {
                commandSink.stop()
                mutableState.value = PlayerUiState()
            }
        }
    }

    fun updatePlayback(
        isPlaying: Boolean = state.value.isPlaying,
        isBuffering: Boolean = state.value.isBuffering,
        positionMs: Long = state.value.positionMs,
        durationMs: Long = state.value.durationMs,
        bufferedPositionMs: Long = state.value.bufferedPositionMs,
        errorKey: String? = state.value.errorKey
    ) {
        mutableState.value = state.value.copy(
            isPlaying = isPlaying,
            isBuffering = isBuffering,
            positionMs = positionMs.coerceAtLeast(0L),
            durationMs = durationMs.coerceAtLeast(0L),
            bufferedPositionMs = bufferedPositionMs.coerceAtLeast(0L),
            errorKey = errorKey
        )
    }

    fun setContent(content: PlayerContentUiModel?) {
        mutableState.value = PlayerUiState(content = content)
    }
}
