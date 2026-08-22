package com.fluxa.app.shared.feature.player

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch

class NativePlayerCommandCallbacks(
    val setPlaying: (Boolean) -> Unit,
    val seekTo: (Long) -> Unit,
    val setVolume: (Float) -> Unit,
    val setSubtitleEnabled: (Boolean) -> Unit,
    val stop: () -> Unit
)

class NativePlayerStateBridge(
    callbacks: NativePlayerCommandCallbacks
) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private val store = PlayerStateStore(
        object : PlayerCommandSink {
            override suspend fun setPlaying(playing: Boolean) = callbacks.setPlaying(playing)
            override suspend fun seekTo(positionMs: Long) = callbacks.seekTo(positionMs)
            override suspend fun setVolume(volume: Float) = callbacks.setVolume(volume)
            override suspend fun setSubtitleEnabled(enabled: Boolean) = callbacks.setSubtitleEnabled(enabled)
            override suspend fun stop() = callbacks.stop()
        }
    )

    val state: PlayerUiState get() = store.state.value

    fun setContent(content: PlayerContentUiModel?) = store.setContent(content)

    fun playPause() = dispatch(PlayerAction.PlayPause)

    fun seekTo(positionMs: Long) = dispatch(PlayerAction.SeekTo(positionMs))

    fun setVolume(volume: Float) = dispatch(PlayerAction.SetVolume(volume))

    fun setSubtitleEnabled(enabled: Boolean) = dispatch(PlayerAction.SetSubtitleEnabled(enabled))

    fun stop() = dispatch(PlayerAction.Stop)

    fun updatePlayback(
        isPlaying: Boolean,
        isBuffering: Boolean,
        positionMs: Long,
        durationMs: Long,
        bufferedPositionMs: Long,
        errorKey: String?
    ) = store.updatePlayback(
        isPlaying,
        isBuffering,
        positionMs,
        durationMs,
        bufferedPositionMs,
        errorKey
    )

    fun close() {
        scope.cancel()
    }

    private fun dispatch(action: PlayerAction) {
        scope.launch { store.dispatch(action) }
    }
}
