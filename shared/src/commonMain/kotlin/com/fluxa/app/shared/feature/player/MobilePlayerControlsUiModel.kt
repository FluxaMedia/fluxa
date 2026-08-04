package com.fluxa.app.shared.feature.player

data class MobilePlayerControlsUiModel(
    val duration: Long,
    val position: Long,
    val bufferedFraction: Float,
    val isPlaying: Boolean,
    val isBuffering: Boolean,
    val hasStartedPlaying: Boolean,
    val playbackSpeed: Float,
    val subtitlesEnabled: Boolean,
    val supportsTrackSettings: Boolean,
    val technicalInfo: String?,
    val episodeMetaLine: String?,
    val streamDetailLine: String?,
    val seekForwardMs: Long,
    val seekBackwardMs: Long,
    val hasPreviousEpisode: Boolean,
    val hasNextEpisode: Boolean,
    val showSourcesButton: Boolean,
    val showEpisodesButton: Boolean,
    val introDbMarkingEnabled: Boolean = false
)

data class MobilePlayerControlsCallbacks(
    val onPlayPause: () -> Unit,
    val onSeek: (Long) -> Unit,
    val onPlayPrevious: () -> Unit,
    val onPlayNext: () -> Unit,
    val onCast: () -> Unit,
    val onOpenInExternalPlayer: () -> Unit,
    val onPictureInPicture: () -> Unit,
    val onToggleAspect: () -> Unit,
    val onShowSettings: (Int) -> Unit,
    val onClose: () -> Unit
)
