@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.animation.ExperimentalAnimationApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import android.view.LayoutInflater
import android.view.SurfaceView
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.AspectRatioFrameLayout
import androidx.media3.ui.PlayerView
import com.fluxa.app.R
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video
import com.fluxa.app.player.AudioCodecBadge
import com.fluxa.app.player.MediaTrack
import com.fluxa.app.player.VideoFormatBadge
import com.fluxa.app.player.MpvAndroidSurfaceView
import com.fluxa.app.player.MpvEmbeddedPlayer
import com.fluxa.app.player.PlayerEngine
import com.fluxa.app.player.TorrentStreamStatus

private data class ExoSurfaceConfig(
    val resizeMode: Int,
    val zoomScale: Float,
    val subtitleSize: Float,
    val subtitleTextOpacity: Float,
    val subtitleBackgroundOpacity: Float,
    val subtitleOutlineOpacity: Float
)

private data class MpvSurfaceConfig(
    val player: MpvEmbeddedPlayer?,
    val zoomScale: Float
)

@Composable
internal fun BoxScope.PlayerPlaybackSurface(
    content: PlayerContentUiModel,
    currentUrl: String?,
    resolvedUrl: String?,
    useMpvBackend: Boolean,
    mpvPlayer: MpvEmbeddedPlayer?,
    exoPlayer: ExoPlayer,
    activeProfile: UserProfile?,
    resizeMode: Int,
    playback: PlaybackSnapshot,
    timeline: TimelineSnapshot,
    buffer: BufferSnapshot,
    render: RenderSnapshot,
    playerError: String?,
    torrentStatus: TorrentStreamStatus,
    deviceType: DeviceType,
    isSwitchingAudioSource: Boolean,
    currentStreamIndex: Int,
    currentStreamDetailLine: String?,
    currentStreamsSize: Int,
    lang: String,
    showControls: Boolean,
    activeEngine: PlayerEngine?,
    showControlsTemp: () -> Unit,
    seekSafely: (Long) -> Unit,
    toggleSubtitleSelection: () -> Unit,
    onToggleAspect: () -> Unit,
    playbackSpeed: Float,
    onPlaybackSpeedChange: (Float) -> Unit,
    playPauseFocusRequester: androidx.compose.ui.focus.FocusRequester,
    seekbarFocusRequester: androidx.compose.ui.focus.FocusRequester,
    isScrubbing: Boolean,
    scrubPosition: Long,
    onScrubbingChange: (Boolean, Long) -> Unit,
    currentEpisodeMetaLine: String?,
    currentSubtitle: MediaTrack?,
    effectiveTechnicalInfo: String?,
    seekForwardMs: Long,
    seekBackwardMs: Long,
    hasPreviousEpisode: Boolean,
    hasNextEpisode: Boolean,
    nextEpisode: Video?,
    onPlayPrevious: () -> Unit,
    onPlayNext: () -> Unit,
    onCast: () -> Unit,
    onOpenInExternalPlayer: () -> Unit,
    onPictureInPicture: () -> Unit,
    onShowSettingsTab: (Int) -> Unit,
    onClose: () -> Unit,
    onNextEpisodeCardShown: () -> Unit,
    timelinePosition: () -> Long,
    skipSegments: List<IntroTimestamps>,
    dismissedSkipSegments: Set<String>,
    onSkipSegment: (IntroTimestamps) -> Unit,
    onDismissSegment: (IntroTimestamps) -> Unit,
    showSegmentSkipFeedback: Boolean,
    holdSpeedVisible: Boolean,
    showVolumeBar: Boolean,
    currentVolume: Int,
    maxVolume: Int,
    showSeekFeedback: Boolean,
    seekDirection: Int,
    seekFeedbackMs: Long,
    videoZoomScale: Float = 1.0f,
    showZoomOverlay: Boolean = false,
    audioCodecBadge: AudioCodecBadge? = null,
    activeVideoFormatBadge: VideoFormatBadge? = null,
    fpsCounterValue: Int = -1
) {
    val seekSurfaceViewRef = remember { mutableStateOf<SurfaceView?>(null) }

    if (!resolvedUrl.isNullOrEmpty()) {
        if (useMpvBackend) {
            val mpvSurfaceConfig = remember(mpvPlayer, videoZoomScale) {
                MpvSurfaceConfig(mpvPlayer, videoZoomScale)
            }
            AndroidView(
                factory = { ctx ->
                    MpvAndroidSurfaceView(ctx).apply {
                        applyMpvSurfaceConfig(mpvSurfaceConfig)
                    }
                },
                update = { view ->
                    view.applyMpvSurfaceConfig(mpvSurfaceConfig)
                },
                modifier = Modifier.fillMaxSize()
            )
        } else {
            val exoSurfaceConfig = remember(
                resizeMode, videoZoomScale,
                activeProfile?.safeSubtitleSize,
                activeProfile?.safeSubtitleTextOpacity,
                activeProfile?.safeSubtitleBackgroundOpacity,
                activeProfile?.safeSubtitleOutlineOpacity
            ) {
                ExoSurfaceConfig(
                    resizeMode = resizeMode,
                    zoomScale = videoZoomScale,
                    subtitleSize = activeProfile?.safeSubtitleSize ?: 20f,
                    subtitleTextOpacity = activeProfile?.safeSubtitleTextOpacity ?: 1f,
                    subtitleBackgroundOpacity = activeProfile?.safeSubtitleBackgroundOpacity ?: 0.75f,
                    subtitleOutlineOpacity = activeProfile?.safeSubtitleOutlineOpacity ?: 0f
                )
            }
            AndroidView(
                factory = { ctx ->
                    (LayoutInflater.from(ctx).inflate(R.layout.player_view_surface, android.widget.FrameLayout(ctx), false) as PlayerView).apply {
                        player = exoPlayer
                        useController = false
                        setBackgroundColor(0xFF000000.toInt())
                        layoutParams = android.widget.FrameLayout.LayoutParams(
                            android.widget.FrameLayout.LayoutParams.MATCH_PARENT,
                            android.widget.FrameLayout.LayoutParams.MATCH_PARENT,
                            android.view.Gravity.CENTER
                        )
                        applyExoSurfaceConfig(exoSurfaceConfig, activeProfile)
                    }
                },
                modifier = Modifier.fillMaxSize(),
                update = { view ->
                    if (seekSurfaceViewRef.value == null) {
                        seekSurfaceViewRef.value = view.videoSurfaceView as? SurfaceView
                    }
                    view.applyExoSurfaceConfig(exoSurfaceConfig, activeProfile)
                }
            )
        }
    }
    val currentPosition = timelinePosition()
    val showLoadingOverlay = playerError != null || (!render.isVideoRendered && !(useMpvBackend && playback.hasStartedPlaying))
    if (showLoadingOverlay) {
        ArtisticLoadingOverlay(content.background, content.logo, content.title, torrentStatus, deviceType, buffer, playerError, currentUrl, isSwitchingAudioSource, currentSourceIdx = currentStreamIndex + 1, totalSources = currentStreamsSize, playback, lang = lang)
        Box(
            modifier = Modifier
                .align(Alignment.TopStart)
                .windowInsetsPadding(WindowInsets.safeDrawing)
                .padding(8.dp)
        ) {
            PlayerTopIconButton(FluxaIcons.ArrowBack, onClose)
        }
    }

    val controlsAlpha by animateFloatAsState(if (showControls && render.isVideoRendered) 1f else 0f)
    CompositionLocalProvider(LocalSeekSurfaceView provides seekSurfaceViewRef.value) {
    Box(modifier = Modifier.fillMaxSize().alpha(controlsAlpha).windowInsetsPadding(WindowInsets.safeDrawing)) {
        if (showControls && render.isVideoRendered) {
            PlayerUIContent(
                content = content,
                lang = lang,
                duration = timeline.duration,
                position = currentPosition,
                bufferedFraction = buffer.seekbarBufferedProgress,
                isPlaying = playback.isPlaying,
                isBuffering = playback.isBuffering,
                hasStartedPlaying = playback.hasStartedPlaying,
                deviceType = deviceType,
                onPlayPause = {
                    activeEngine?.setPaused(playback.isPlaying)
                    showControlsTemp()
                },
                onSeek = { seekSafely(it); showControlsTemp() },
                onToggleSubtitles = { toggleSubtitleSelection() },
                onToggleAspect = onToggleAspect,
                onSpeedChange = onPlaybackSpeedChange,
                playbackSpeed = playbackSpeed,
                playPauseFocusRequester = playPauseFocusRequester,
                seekbarFocusRequester = seekbarFocusRequester,
                isScrubbing = isScrubbing,
                scrubPosition = scrubPosition,
                onScrubbingChange = onScrubbingChange,
                isSwitchingAudioSource = isSwitchingAudioSource,
                detailedStatus = torrentStatus.detailedStatus,
                episodeMetaLine = currentEpisodeMetaLine,
                streamDetailLine = currentStreamDetailLine,
                subtitlesEnabled = currentSubtitle != null,
                supportsTrackSettings = true,
                technicalInfo = effectiveTechnicalInfo,
                seekForwardMs = seekForwardMs,
                seekBackwardMs = seekBackwardMs,
                hasPreviousEpisode = hasPreviousEpisode,
                hasNextEpisode = hasNextEpisode,
                showSourcesButton = false,
                showEpisodesButton = content.isSeries,
                onPlayPrevious = onPlayPrevious,
                onPlayNext = onPlayNext,
                onCast = onCast,
                onOpenInExternalPlayer = onOpenInExternalPlayer,
                onPictureInPicture = onPictureInPicture,
                onShowSettings = onShowSettingsTab,
                onClose = onClose,
                accentColor = Color(activeProfile?.safeAccentColorArgb ?: 0xFFE53935.toInt()),
                audioCodecBadge = audioCodecBadge,
                videoFormatBadge = activeVideoFormatBadge
            )
        }
    }

    PlayerSkipSegmentOverlay(
        currentPosition = currentPosition,
        skipSegments = skipSegments,
        dismissedSkipSegments = dismissedSkipSegments,
        hasStartedPlaying = playback.hasStartedPlaying,
        showControls = showControls,
        deviceType = deviceType,
        nextEpisode = nextEpisode,
        nextEpisodeThresholdReached = timeline.duration > 0L &&
            currentPosition >= (timeline.duration * ((activeProfile?.safeNextEpisodeThresholdPercent ?: 90f) / 100f)).toLong(),
        autoSkipSegments = activeProfile?.safeAutoSkipIntro == true,
        lang = lang,
        onSkipSegment = onSkipSegment,
        onPlayNextEpisode = onPlayNext,
        onDismissSegment = onDismissSegment,
        onNextEpisodeCardShown = {
            onNextEpisodeCardShown()
        },
        modifier = Modifier.align(Alignment.BottomEnd)
    )

    val zoomOverlayMode = when {
        resizeMode == AspectRatioFrameLayout.RESIZE_MODE_FIT -> ZoomOverlayMode.Original
        videoZoomScale <= 1.05f -> ZoomOverlayMode.Fit
        else -> ZoomOverlayMode.Zoom
    }
    val zoomLabelText = when (zoomOverlayMode) {
        ZoomOverlayMode.Original -> AppStrings.t(lang, "player.zoom_original")
        ZoomOverlayMode.Fit -> AppStrings.t(lang, "player.zoom_fill")
        ZoomOverlayMode.Zoom -> "${"%.1f".format(videoZoomScale)}x"
    }

    PlayerTransientOverlays(
        showSegmentSkipFeedback = showSegmentSkipFeedback,
        holdSpeedVisible = holdSpeedVisible,
        activeProfile = activeProfile,
        deviceType = deviceType,
        showVolumeBar = showVolumeBar,
        currentVolume = currentVolume,
        maxVolume = maxVolume,
        showSeekFeedback = showSeekFeedback,
        seekDirection = seekDirection,
        seekFeedbackMs = seekFeedbackMs,
        seekForwardMs = seekForwardMs,
        seekBackwardMs = seekBackwardMs,
        showZoomOverlay = showZoomOverlay,
        zoomOverlayMode = zoomOverlayMode,
        zoomLabelText = zoomLabelText,
        fpsCounterValue = fpsCounterValue
    )
    } // CompositionLocalProvider
}

private fun PlayerView.applyExoSurfaceConfig(config: ExoSurfaceConfig, profile: UserProfile?) {
    if (getTag(R.id.player_surface_config_tag) == config) return
    resizeMode = config.resizeMode
    scaleX = config.zoomScale
    scaleY = config.zoomScale
    subtitleView?.let { sv ->
        sv.setApplyEmbeddedStyles(false)
        sv.setApplyEmbeddedFontSizes(false)
        sv.setFixedTextSize(android.util.TypedValue.COMPLEX_UNIT_SP, config.subtitleSize)
        sv.setStyle(subtitleCaptionStyle(profile))
    }
    setTag(R.id.player_surface_config_tag, config)
}

private fun MpvAndroidSurfaceView.applyMpvSurfaceConfig(config: MpvSurfaceConfig) {
    if (getTag(R.id.player_surface_config_tag) == config) return
    bind(config.player)
    scaleX = config.zoomScale
    scaleY = config.zoomScale
    setTag(R.id.player_surface_config_tag, config)
}

@Composable
internal fun PlayerSettingsPanel(
    meta: Meta,
    currentVideoId: String?,
    deviceType: DeviceType,
    viewModel: HomeViewModel,
    activeProfile: UserProfile?,
    lang: String,
    activeSettingsTab: Int,
    currentStreams: List<Stream>,
    currentUrl: String?,
    currentStreamIndex: Int,
    availableAudios: List<MediaTrack>,
    currentAudio: MediaTrack?,
    availableSubtitles: List<MediaTrack>,
    currentSubtitle: MediaTrack?,
    playbackSpeed: Float,
    audioDelayMs: Long,
    subtitleDelayMs: Long,
    onEpisodeSelected: (String, String?) -> Unit,
    onCloseSettings: () -> Unit,
    onSelectStreamIndex: (Int) -> Unit,
    onSelectAudio: (MediaTrack) -> Unit,
    onSelectSubtitle: (MediaTrack) -> Unit,
    onDisableSubtitle: () -> Unit,
    onSpeedChange: (Float) -> Unit,
    onAudioDelayChange: (Long) -> Unit,
    onSubtitleDelayChange: (Long) -> Unit,
    onSubtitleTextOpacityChange: (Float) -> Unit,
    onSubtitleBackgroundOpacityChange: (Float) -> Unit,
    onSubtitleOutlineOpacityChange: (Float) -> Unit
) {
    if (activeSettingsTab == 3 && meta.type == "series") {
        EpisodeSidebar(
            meta = meta,
            currentId = currentVideoId ?: meta.id,
            deviceType = deviceType,
            viewModel = viewModel,
            activeProfile = activeProfile,
            onSelect = onEpisodeSelected,
            onClose = onCloseSettings
        )
    } else if (activeSettingsTab == 4) {
        SourceSidebar(
            streams = currentStreams,
            currentUrl = currentUrl.orEmpty(),
            deviceType = deviceType,
            lang = lang,
            onSelect = { selectedUrl ->
                val selectedIndex = currentStreams.indexOfFirst { it.playableUrl == selectedUrl }
                if (selectedIndex >= 0) onSelectStreamIndex(selectedIndex)
                onCloseSettings()
            },
            onClose = onCloseSettings
        )
    } else {
        UniversalSettingsSidebar(
            activeTab = activeSettingsTab,
            audioTracks = availableAudios,
            currentAudio = currentAudio,
            subtitleTracks = availableSubtitles,
            currentSubtitle = currentSubtitle,
            playbackSpeed = playbackSpeed,
            audioDelayMs = audioDelayMs,
            subtitleDelayMs = subtitleDelayMs,
            subtitleTextOpacity = activeProfile?.safeSubtitleTextOpacity ?: 1f,
            subtitleBackgroundOpacity = activeProfile?.safeSubtitleBackgroundOpacity ?: 0.5f,
            subtitleOutlineOpacity = activeProfile?.safeSubtitleOutlineOpacity ?: 1f,
            onSelectAudio = {
                onSelectAudio(it)
                onCloseSettings()
            },
            onSelectSubtitle = {
                onSelectSubtitle(it)
                onCloseSettings()
            },
            onDisableSubtitle = {
                onDisableSubtitle()
                onCloseSettings()
            },
            onSpeedChange = {
                onSpeedChange(it)
                onCloseSettings()
            },
            onAudioDelayChange = onAudioDelayChange,
            onSubtitleDelayChange = onSubtitleDelayChange,
            onSubtitleTextOpacityChange = onSubtitleTextOpacityChange,
            onSubtitleBackgroundOpacityChange = onSubtitleBackgroundOpacityChange,
            onSubtitleOutlineOpacityChange = onSubtitleOutlineOpacityChange,
            deviceType = deviceType,
            lang = lang,
            onClose = onCloseSettings
        )
    }
}
