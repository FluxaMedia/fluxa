package com.fluxa.app.shared.feature.player

import kotlinx.coroutines.flow.StateFlow

data class PlayerContentUiModel(
    val id: String,
    val type: String,
    val title: String,
    val subtitle: String = "",
    val logoUrl: String? = null,
    val backgroundUrl: String? = null,
    val streamLabel: String = "",
    val releaseInfo: String? = null,
    val runtime: String? = null
) {
    val logo: String get() = logoUrl.orEmpty()
    val background: String get() = backgroundUrl.orEmpty()
    val isSeries: Boolean get() = type == "series"
}

data class PlayerUiState(
    val content: PlayerContentUiModel? = null,
    val isPlaying: Boolean = false,
    val isBuffering: Boolean = false,
    val positionMs: Long = 0L,
    val durationMs: Long = 0L,
    val bufferedPositionMs: Long = 0L,
    val volume: Float = 1f,
    val subtitleEnabled: Boolean = false,
    val errorKey: String? = null
)

sealed interface PlayerAction {
    data object PlayPause : PlayerAction
    data class SeekTo(val positionMs: Long) : PlayerAction
    data class SetVolume(val volume: Float) : PlayerAction
    data class SetSubtitleEnabled(val enabled: Boolean) : PlayerAction
    data object Stop : PlayerAction
}

interface PlaybackController {
    val state: StateFlow<PlayerUiState>
    suspend fun dispatch(action: PlayerAction)
}
