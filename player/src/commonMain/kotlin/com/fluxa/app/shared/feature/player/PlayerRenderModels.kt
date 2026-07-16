package com.fluxa.app.shared.feature.player

data class PlayerSourceUiModel(
    val id: String,
    val provider: String,
    val title: String,
    val description: String? = null,
    val selected: Boolean = false
)

data class PlayerTrackUiModel(
    val id: String,
    val label: String,
    val language: String? = null,
    val selected: Boolean = false,
    val supported: Boolean = true
)

data class PlayerRenderState(
    val content: PlayerContentUiModel? = null,
    val isPlaying: Boolean = false,
    val isBuffering: Boolean = false,
    val positionMs: Long = 0L,
    val durationMs: Long = 0L,
    val bufferedPositionMs: Long = 0L,
    val controlsVisible: Boolean = true,
    val sources: List<PlayerSourceUiModel> = emptyList(),
    val audioTracks: List<PlayerTrackUiModel> = emptyList(),
    val subtitleTracks: List<PlayerTrackUiModel> = emptyList(),
    val errorKey: String? = null
) {
    val progress: Float get() = if (durationMs <= 0L) 0f else (positionMs.toFloat() / durationMs).coerceIn(0f, 1f)
}

sealed interface PlayerRenderAction {
    data object PlayPause : PlayerRenderAction
    data class SeekTo(val positionMs: Long) : PlayerRenderAction
    data class SourceSelected(val id: String) : PlayerRenderAction
    data class AudioTrackSelected(val id: String) : PlayerRenderAction
    data class SubtitleTrackSelected(val id: String?) : PlayerRenderAction
    data object Retry : PlayerRenderAction
    data object Back : PlayerRenderAction
}
