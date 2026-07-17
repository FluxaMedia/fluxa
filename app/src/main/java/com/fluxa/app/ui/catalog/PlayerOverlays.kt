package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.shared.feature.player.MobilePlayerUIContent
import com.fluxa.app.shared.feature.player.PlayerContentUiModel
import com.fluxa.app.shared.feature.player.TvPlayerUIContent

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale

internal suspend fun resolveIntroImdbId(
    viewModel: HomeViewModel,
    meta: Meta,
    videoId: String?,
    language: String
): String? {
    return viewModel.resolvePlaybackIntroImdbId(meta, videoId, language)
}

internal fun extractSeasonEpisode(videoId: String?): Pair<Int, Int>? {
    return FluxaCoreNative.parseEpisodeLocator(videoId)?.let { it.season to it.episode }
}

@Composable
internal fun PlayerUIContent(
    content: PlayerContentUiModel, lang: String, duration: Long, position: Long, bufferedFraction: Float, chapters: List<com.fluxa.app.player.Chapter> = emptyList(), isPlaying: Boolean, isBuffering: Boolean, hasStartedPlaying: Boolean, deviceType: DeviceType,
    onPlayPause: () -> Unit, onSeek: (Long) -> Unit, onToggleSubtitles: () -> Unit, onToggleAspect: () -> Unit, onSpeedChange: (Float) -> Unit, playbackSpeed: Float, playPauseFocusRequester: FocusRequester, seekbarFocusRequester: FocusRequester,
    isScrubbing: Boolean, scrubPosition: Long, onScrubbingChange: (Boolean, Long) -> Unit, onScrubSeek: (Long) -> Unit = {},
    isSwitchingAudioSource: Boolean = false, detailedStatus: String = "", episodeMetaLine: String? = null, streamDetailLine: String? = null, subtitlesEnabled: Boolean = false, technicalInfo: String? = null,
    supportsTrackSettings: Boolean = true,
    seekForwardMs: Long = 10_000L, seekBackwardMs: Long = 10_000L,
    hasPreviousEpisode: Boolean = false,
    hasNextEpisode: Boolean = false,
    showSourcesButton: Boolean = false,
    showEpisodesButton: Boolean = false,
    introDbMarkingEnabled: Boolean = false,
    onPlayPrevious: () -> Unit = {},
    onPlayNext: () -> Unit = {},
    onCast: () -> Unit = {},
    onOpenInExternalPlayer: () -> Unit = {},
    onPictureInPicture: () -> Unit = {},
    onShowSettings: (Int) -> Unit,
    onClose: () -> Unit,
    accentColor: Color = FluxaColors.accent
) {
    val seekSurfaceView = LocalSeekSurfaceView.current
    val seekPreviewBitmap = rememberSeekThumbnail(
        seekSurfaceView, scrubPosition, isScrubbing, position, isPlaying, onScrubSeek
    )
    val scrubFreezeFrame = rememberScrubFreezeFrame(seekSurfaceView, isScrubbing)
    scrubFreezeFrame?.let { frozen ->
        Image(
            bitmap = frozen,
            contentDescription = null,
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Fit
        )
    }

    if (deviceType == DeviceType.Mobile) {
        MobilePlayerUIContent(
            title = content.title,
            content = content,
            lang = lang,
            duration = duration,
            position = position,
            bufferedFraction = bufferedFraction,
            chapters = chapters,
            isPlaying = isPlaying,
            isBuffering = isBuffering,
            hasStartedPlaying = hasStartedPlaying,
            onPlayPause = onPlayPause,
            onSeek = onSeek,
            playbackSpeed = playbackSpeed,
            subtitlesEnabled = subtitlesEnabled,
            supportsTrackSettings = supportsTrackSettings,
            technicalInfo = technicalInfo,
            episodeMetaLine = episodeMetaLine,
            streamDetailLine = streamDetailLine,
            seekForwardMs = seekForwardMs,
            seekBackwardMs = seekBackwardMs,
            hasPreviousEpisode = hasPreviousEpisode,
            hasNextEpisode = hasNextEpisode,
            showSourcesButton = showSourcesButton,
            showEpisodesButton = showEpisodesButton,
            introDbMarkingEnabled = introDbMarkingEnabled,
            onPlayPrevious = onPlayPrevious,
            onPlayNext = onPlayNext,
            onCast = onCast,
            onOpenInExternalPlayer = onOpenInExternalPlayer,
            onPictureInPicture = onPictureInPicture,
            onToggleAspect = onToggleAspect,
            onShowSettings = onShowSettings,
            onClose = onClose,
            isScrubbing = isScrubbing,
            scrubPosition = scrubPosition,
            onScrubbingChange = onScrubbingChange,
            seekPreviewBitmap = seekPreviewBitmap,
            accentColor = accentColor
        )
        return
    }

    TvPlayerUIContent(
        title = content.title,
        content = content,
        lang = lang,
        duration = duration,
        position = position,
        bufferedFraction = bufferedFraction,
        chapters = chapters,
        isPlaying = isPlaying,
        isBuffering = isBuffering,
        hasStartedPlaying = hasStartedPlaying,
        deviceType = deviceType,
        onPlayPause = onPlayPause,
        onSeek = onSeek,
        onToggleSubtitles = onToggleSubtitles,
        onToggleAspect = onToggleAspect,
        onSpeedChange = onSpeedChange,
        playbackSpeed = playbackSpeed,
        playPauseFocusRequester = playPauseFocusRequester,
        seekbarFocusRequester = seekbarFocusRequester,
        isScrubbing = isScrubbing,
        scrubPosition = scrubPosition,
        onScrubbingChange = onScrubbingChange,
        seekPreviewBitmap = seekPreviewBitmap,
        isSwitchingAudioSource = isSwitchingAudioSource,
        detailedStatus = detailedStatus,
        episodeMetaLine = episodeMetaLine,
        streamDetailLine = streamDetailLine,
        subtitlesEnabled = subtitlesEnabled,
        supportsTrackSettings = supportsTrackSettings,
        technicalInfo = technicalInfo,
        seekForwardMs = seekForwardMs,
        seekBackwardMs = seekBackwardMs,
        hasPreviousEpisode = hasPreviousEpisode,
        hasNextEpisode = hasNextEpisode,
        showSourcesButton = showSourcesButton,
        showEpisodesButton = showEpisodesButton,
        introDbMarkingEnabled = introDbMarkingEnabled,
        onPlayPrevious = onPlayPrevious,
        onPlayNext = onPlayNext,
        onCast = onCast,
        onOpenInExternalPlayer = onOpenInExternalPlayer,
        onPictureInPicture = onPictureInPicture,
        onShowSettings = onShowSettings,
        onClose = onClose
    )
}
