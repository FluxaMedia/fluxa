@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.animation.ExperimentalAnimationApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import android.animation.Animator
import android.animation.AnimatorListenerAdapter
import android.animation.ValueAnimator
import com.fluxa.app.common.AppStrings
import android.view.LayoutInflater
import android.view.SurfaceView
import android.view.animation.DecelerateInterpolator
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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.fluxa.app.player.LibassDebugLog
import com.fluxa.app.player.MediaPlayerController
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map
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
import com.fluxa.app.player.MediaTrack
import com.fluxa.app.player.ExternalSubtitleTrack
import com.fluxa.app.player.NativeAssTrack
import com.fluxa.app.player.MpvAndroidSurfaceView
import com.fluxa.app.player.MpvEmbeddedPlayer
import com.fluxa.app.player.PlayerEngine
import com.fluxa.app.player.TorrentStreamStatus
import kotlin.math.roundToInt

private data class ExoSurfaceConfig(
    val resizeMode: Int,
    val zoomScale: Float,
    val subtitleSize: Float,
    val subtitleTextOpacity: Float,
    val subtitleBackgroundOpacity: Float,
    val subtitleOutlineOpacity: Float,
    val nativeAssOverlayActive: Boolean
)

private data class MpvSurfaceConfig(
    val player: MpvEmbeddedPlayer?,
    val zoomScale: Float
)

private data class SurfaceCropTarget(
    val width: Int,
    val height: Int
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
    currentExternalSubtitles: List<ExternalSubtitleTrack>,
    embeddedNativeAssTracks: List<NativeAssTrack>,
    subtitleDelayMs: Long,
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
    chapters: List<com.fluxa.app.player.Chapter> = emptyList(),
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
    fillScale: Float = 1.0f,
    showZoomOverlay: Boolean = false,
    parentsGuide: List<com.fluxa.app.data.remote.ParentsGuideCategory> = emptyList(),
    showParentsGuide: Boolean = false,
    onParentsGuideAnimationComplete: () -> Unit = {}
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
            val libassRelay = remember(exoPlayer) { MediaPlayerController.getLibassRelay(exoPlayer) }
            val relayRendererActive by (libassRelay?.activeRenderer?.let { it.map { r -> r != null } }
                ?: MutableStateFlow(false)).collectAsStateWithLifecycle(false)

            val nativeAssActive = relayRendererActive && currentSubtitle != null ||
                selectedNativeAssSubtitle(currentSubtitle, currentExternalSubtitles) != null ||
                selectedEmbeddedNativeAssTrack(currentSubtitle, embeddedNativeAssTracks) != null
            LaunchedEffect(
                nativeAssActive,
                relayRendererActive,
                currentSubtitle?.id,
                currentSubtitle?.sampleMimeType,
                currentExternalSubtitles.size,
                embeddedNativeAssTracks.size
            ) {
                LibassDebugLog.d(
                    "surface nativeAssActive=$nativeAssActive relayRendererActive=$relayRendererActive " +
                        "currentSubtitle=${currentSubtitle?.id} mime=${currentSubtitle?.sampleMimeType} label=${currentSubtitle?.label} lang=${currentSubtitle?.language} " +
                        "externalCount=${currentExternalSubtitles.size} embeddedAssCount=${embeddedNativeAssTracks.size} " +
                        "selectedExternal=${selectedNativeAssSubtitle(currentSubtitle, currentExternalSubtitles)?.let { LibassDebugLog.urlSummary(it.url) } ?: "<none>"} " +
                        "selectedEmbedded=${selectedEmbeddedNativeAssTrack(currentSubtitle, embeddedNativeAssTracks)?.id ?: "<none>"}"
                )
            }

            val exoSurfaceConfig = remember(
                resizeMode, videoZoomScale,
                activeProfile?.safeSubtitleSize,
                activeProfile?.safeSubtitleTextOpacity,
                activeProfile?.safeSubtitleBackgroundOpacity,
                activeProfile?.safeSubtitleOutlineOpacity,
                nativeAssActive
            ) {
                ExoSurfaceConfig(
                    resizeMode = resizeMode,
                    zoomScale = videoZoomScale,
                    subtitleSize = activeProfile?.safeSubtitleSize ?: 20f,
                    subtitleTextOpacity = activeProfile?.safeSubtitleTextOpacity ?: 1f,
                    subtitleBackgroundOpacity = activeProfile?.safeSubtitleBackgroundOpacity ?: 0.75f,
                    subtitleOutlineOpacity = activeProfile?.safeSubtitleOutlineOpacity ?: 0f,
                    nativeAssOverlayActive = nativeAssActive
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
            NativeLibassSubtitleOverlay(
                exoPlayer = exoPlayer,
                externalSubtitle = selectedNativeAssSubtitle(currentSubtitle, currentExternalSubtitles),
                embeddedSubtitle = selectedEmbeddedNativeAssTrack(currentSubtitle, embeddedNativeAssTracks),
                subtitleDelayMs = subtitleDelayMs,
                modifier = Modifier.fillMaxSize()
            )
        }
    }
    val currentPosition = timelinePosition()
    val showLoadingOverlay = playerError != null ||
        (!render.isVideoRendered && !(useMpvBackend && playback.hasStartedPlaying)) ||
        (playback.hasStartedPlaying && playback.isBuffering)
    if (showLoadingOverlay) {
        ArtisticLoadingOverlay(content.background, content.logo, content.title, torrentStatus, deviceType, buffer, playerError, currentUrl, isSwitchingAudioSource, currentSourceIdx = currentStreamIndex + 1, totalSources = currentStreamsSize, playback, hasRenderedFirstFrame = render.isVideoRendered, lang = lang)
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
    CompositionLocalProvider(
        LocalSeekSurfaceView provides seekSurfaceViewRef.value,
        LocalSeekExoPlayer provides exoPlayer
    ) {
    Box(modifier = Modifier.fillMaxSize().alpha(controlsAlpha)) {
        if (showControls && render.isVideoRendered) {
            PlayerUIContent(
                content = content,
                lang = lang,
                duration = timeline.duration,
                position = currentPosition,
                bufferedFraction = buffer.seekbarBufferedProgress,
                chapters = chapters,
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
                onScrubSeek = { activeEngine?.seekTo(it, exact = false) },
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
                introDbMarkingEnabled = content.isSeries && activeProfile?.safeIntroDbApiKey?.isNotBlank() == true,
                onPlayPrevious = onPlayPrevious,
                onPlayNext = onPlayNext,
                onCast = onCast,
                onOpenInExternalPlayer = onOpenInExternalPlayer,
                onPictureInPicture = onPictureInPicture,
                onShowSettings = onShowSettingsTab,
                onClose = onClose,
                accentColor = Color(activeProfile?.safeAccentColorArgb ?: FluxaColors.accentArgb)
            )
        }
    }

    PlayerParentsGuideOverlay(
        categories = parentsGuide,
        lang = lang,
        isVisible = showParentsGuide,
        onAnimationComplete = onParentsGuideAnimationComplete,
        accentColor = Color(activeProfile?.safeAccentColorArgb ?: FluxaColors.accentArgb),
        modifier = Modifier
            .align(Alignment.TopStart)
            .windowInsetsPadding(WindowInsets.safeDrawing)
            .padding(start = 16.dp, top = 64.dp)
    )

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
        videoZoomScale <= fillScale * 1.05f -> ZoomOverlayMode.Fit
        else -> ZoomOverlayMode.Zoom
    }
    val zoomLabelText = when (zoomOverlayMode) {
        ZoomOverlayMode.Original -> AppStrings.t(lang, "player.zoom_original")
        ZoomOverlayMode.Fit -> AppStrings.t(lang, "player.zoom_fill")
        ZoomOverlayMode.Zoom -> "${"%.1f".format(videoZoomScale / fillScale)}x"
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
        zoomLabelText = zoomLabelText
    )
    }
}

private fun PlayerView.applyExoSurfaceConfig(config: ExoSurfaceConfig, profile: UserProfile?) {
    resizeMode = config.resizeMode
    scaleX = 1.0f
    scaleY = 1.0f
    videoSurfaceView?.let { surface ->
        surface.scaleX = 1.0f
        surface.scaleY = 1.0f
        applySurfaceCropLayout(surface, config)
    }
    subtitleView?.let { sv ->
        sv.setApplyEmbeddedStyles(true)
        sv.setApplyEmbeddedFontSizes(true)
        sv.setFixedTextSize(android.util.TypedValue.COMPLEX_UNIT_SP, config.subtitleSize)
        sv.setStyle(subtitleCaptionStyle(profile))
        sv.visibility = if (config.nativeAssOverlayActive) android.view.View.GONE else android.view.View.VISIBLE
    }
    setTag(R.id.player_surface_config_tag, config)
}

private fun applySurfaceCropLayout(surface: android.view.View, config: ExoSurfaceConfig) {
    val parent = surface.parent as? android.view.View ?: return
    val scale = if (config.resizeMode == AspectRatioFrameLayout.RESIZE_MODE_ZOOM) {
        config.zoomScale.coerceAtLeast(1.0f)
    } else {
        1.0f
    }
    val baseWidth = parent.width.takeIf { it > 0 } ?: surface.width
    val baseHeight = parent.height.takeIf { it > 0 } ?: surface.height
    if (baseWidth <= 0 || baseHeight <= 0) return
    val targetWidth = (baseWidth * scale).roundToInt().coerceAtLeast(baseWidth)
    val targetHeight = (baseHeight * scale).roundToInt().coerceAtLeast(baseHeight)
    val target = SurfaceCropTarget(targetWidth, targetHeight)
    if (surface.getTag(R.id.player_surface_crop_target_tag) == target) return
    surface.setTag(R.id.player_surface_crop_target_tag, target)
    (surface.getTag(R.id.player_surface_crop_animator_tag) as? ValueAnimator)?.cancel()
    val current = surface.layoutParams as? android.widget.FrameLayout.LayoutParams
    val startWidth = current?.width?.takeIf { it > 0 } ?: surface.width.takeIf { it > 0 } ?: baseWidth
    val startHeight = current?.height?.takeIf { it > 0 } ?: surface.height.takeIf { it > 0 } ?: baseHeight
    if (startWidth == targetWidth && startHeight == targetHeight) {
        surface.layoutParams = android.widget.FrameLayout.LayoutParams(
            targetWidth,
            targetHeight,
            android.view.Gravity.CENTER
        )
        return
    }
    ValueAnimator.ofFloat(0f, 1f).apply {
        duration = 140L
        interpolator = DecelerateInterpolator()
        addUpdateListener { animator ->
            val fraction = animator.animatedFraction
            val animatedWidth = (startWidth + ((targetWidth - startWidth) * fraction)).roundToInt()
            val animatedHeight = (startHeight + ((targetHeight - startHeight) * fraction)).roundToInt()
            surface.layoutParams = android.widget.FrameLayout.LayoutParams(
                animatedWidth,
                animatedHeight,
                android.view.Gravity.CENTER
            )
        }
        addListener(object : AnimatorListenerAdapter() {
            override fun onAnimationEnd(animation: Animator) {
                surface.layoutParams = android.widget.FrameLayout.LayoutParams(
                    targetWidth,
                    targetHeight,
                    android.view.Gravity.CENTER
                )
                if (surface.getTag(R.id.player_surface_crop_animator_tag) == this@apply) {
                    surface.setTag(R.id.player_surface_crop_animator_tag, null)
                }
            }
        })
        surface.setTag(R.id.player_surface_crop_animator_tag, this)
        start()
    }
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
    onSubtitleOutlineOpacityChange: (Float) -> Unit,
    currentPositionMs: Long = 0L,
    markSegmentType: String? = null,
    markSegmentStartMs: Long? = null,
    markSegmentEndMs: Long? = null,
    markSegmentSubmitting: Boolean = false,
    markSegmentFeedback: String? = null,
    markSegmentCooldownRemainingSec: Long? = null,
    onSelectMarkSegmentType: (String) -> Unit = {},
    onMarkSegmentStart: () -> Unit = {},
    onMarkSegmentEnd: () -> Unit = {},
    onAdjustMarkSegmentStart: (Long) -> Unit = {},
    onAdjustMarkSegmentEnd: (Long) -> Unit = {},
    onSubmitMarkSegment: () -> Unit = {}
) {
    if (activeSettingsTab == 5) {
        MarkSegmentSidebar(
            deviceType = deviceType,
            lang = lang,
            selectedType = markSegmentType,
            startMs = markSegmentStartMs,
            endMs = markSegmentEndMs,
            currentPositionMs = currentPositionMs,
            submitting = markSegmentSubmitting,
            cooldownRemainingSec = markSegmentCooldownRemainingSec,
            feedback = markSegmentFeedback,
            onSelectType = onSelectMarkSegmentType,
            onMarkStart = onMarkSegmentStart,
            onMarkEnd = onMarkSegmentEnd,
            onAdjustStart = onAdjustMarkSegmentStart,
            onAdjustEnd = onAdjustMarkSegmentEnd,
            onSubmit = onSubmitMarkSegment,
            onClose = onCloseSettings
        )
    } else if (activeSettingsTab == 3 && meta.type == "series") {
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
