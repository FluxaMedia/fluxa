@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)

package com.fluxa.app.ui.catalog

import android.content.Context
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.unit.IntSize
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.AspectRatioFrameLayout
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import kotlin.math.abs
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.IntroDbSubmitResult
import com.fluxa.app.player.MediaTrack
import com.fluxa.app.player.MpvEmbeddedPlayer
import com.fluxa.app.player.PlayerEngine
import com.fluxa.app.player.TorrentStreamStatus

private fun markSegmentCooldownRemainingSec(state: PlayerScreenState, meta: Meta): Long? {
    val type = state.markSegmentType ?: return null
    val seasonEpisode = extractSeasonEpisode(state.currentVideoId) ?: extractSeasonEpisode(meta.id) ?: return null
    val cooldownUntil = state.introDbCooldowns["$type:${seasonEpisode.first}:${seasonEpisode.second}"] ?: return null
    val remainingMs = cooldownUntil - System.currentTimeMillis()
    return if (remainingMs > 0) remainingMs / 1000 else null
}

private const val MIN_MANUAL_FILL_SCALE = 1.34f

@Composable
internal fun PlayerScreenContent(
    meta: Meta,
    state: PlayerScreenState,
    context: Context,
    lang: String,
    deviceType: DeviceType,
    exoPlayer: ExoPlayer,
    mpvPlayer: MpvEmbeddedPlayer?,
    useMpvBackend: Boolean,
    activeProfile: UserProfile?,
    onUpdateProfile: (UserProfile) -> Unit,
    activeEngine: PlayerEngine?,
    torrentStatus: TorrentStreamStatus,
    mainFocusRequester: FocusRequester,
    playPauseFocusRequester: FocusRequester,
    seekbarFocusRequester: FocusRequester,
    audioManager: android.media.AudioManager,
    maxVolume: Int,
    seekForwardMs: Long,
    seekBackwardMs: Long,
    viewModel: HomeViewModel,
    availableAudios: List<MediaTrack>,
    currentAudio: MediaTrack?,
    availableSubtitles: List<MediaTrack>,
    currentSubtitle: MediaTrack?,
    effectiveTechnicalInfo: String?,
    parentsGuide: List<com.fluxa.app.data.remote.ParentsGuideCategory> = emptyList(),
    showControlsTemp: () -> Unit,
    seekSafely: (Long) -> Unit,
    toggleSubtitleSelection: () -> Unit,
    performRelativeSeek: (Int) -> Unit,
    playPrevious: () -> Unit,
    playNext: () -> Unit,
    smartCast: () -> Unit,
    openInExternalPlayer: () -> Unit,
    openSourceSelectionScreen: () -> Unit,
    closePlayer: () -> Unit,
    switchToStream: (Int) -> Unit
) {
    val coroutineScope = rememberCoroutineScope()
    val gestureState = remember { object { var startedInOriginal = false; var snappedToFillThisGesture = false } }
    var containerSize by remember { mutableStateOf(IntSize.Zero) }
    val emptyVideoAspectRatio = remember { MutableStateFlow<Float?>(null) }
    val mpvVideoAspectRatio by (mpvPlayer?.videoAspectRatio ?: emptyVideoAspectRatio).collectAsStateWithLifecycle()
    var exoVideoAspectRatio by remember { mutableStateOf<Float?>(null) }
    DisposableEffect(exoPlayer) {
        fun applyVideoSize(videoSize: androidx.media3.common.VideoSize) {
            if (videoSize.width > 0 && videoSize.height > 0) {
                val pixelWidthHeightRatio = videoSize.pixelWidthHeightRatio.takeIf { it > 0f } ?: 1f
                exoVideoAspectRatio = (videoSize.width.toFloat() * pixelWidthHeightRatio) / videoSize.height.toFloat()
            }
        }
        applyVideoSize(exoPlayer.videoSize)
        val listener = object : androidx.media3.common.Player.Listener {
            override fun onVideoSizeChanged(videoSize: androidx.media3.common.VideoSize) {
                applyVideoSize(videoSize)
            }
        }
        exoPlayer.addListener(listener)
        onDispose { exoPlayer.removeListener(listener) }
    }
    val activeVideoAspectRatio = if (useMpvBackend) mpvVideoAspectRatio else exoVideoAspectRatio
    val fillScale = remember(activeVideoAspectRatio, containerSize) {
        val aspectRatio = activeVideoAspectRatio
        val measuredFillScale = if (aspectRatio == null || aspectRatio <= 0f || containerSize.width <= 0 || containerSize.height <= 0) {
            1.0f
        } else {
            val containerAspectRatio = containerSize.width.toFloat() / containerSize.height.toFloat()
            maxOf(containerAspectRatio / aspectRatio, aspectRatio / containerAspectRatio)
        }
        maxOf(measuredFillScale, MIN_MANUAL_FILL_SCALE)
    }
    val playerContent = remember(meta) { meta.toPlayerContentUiModel() }
    val currentStreamDetailLine = remember(state.currentStreams, state.currentStreamIndex) {
        state.currentStreams
            .getOrNull(state.currentStreamIndex)
            ?.cloudstreamPlaybackDetailLine()
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
            .focusRequester(mainFocusRequester)
            .onSizeChanged { containerSize = it }
            .playerInputControls(
                deviceType = deviceType,
                hasStartedPlaying = state.engine.playback.hasStartedPlaying,
                showControls = state.showControls,
                activeProfile = activeProfile,
                activeEngine = activeEngine,
                playbackSpeed = state.playbackSpeed,
                onRaiseVolume = {
                    audioManager.adjustStreamVolume(android.media.AudioManager.STREAM_MUSIC, android.media.AudioManager.ADJUST_RAISE, 0)
                    state.currentVolume = audioManager.getStreamVolume(android.media.AudioManager.STREAM_MUSIC)
                    state.showVolumeBar = true
                },
                onLowerVolume = {
                    audioManager.adjustStreamVolume(android.media.AudioManager.STREAM_MUSIC, android.media.AudioManager.ADJUST_LOWER, 0)
                    state.currentVolume = audioManager.getStreamVolume(android.media.AudioManager.STREAM_MUSIC)
                    state.showVolumeBar = true
                },
                onShowControlsTemp = showControlsTemp,
                onHideControls = {
                    state.showControls = false
                    state.controlsTimerJob?.cancel()
                },
                onHoldSpeedVisibleChanged = { state.holdSpeedVisible = it },
                onRelativeSeek = performRelativeSeek,
                onClosePlayer = closePlayer,
                onPinchZoomGestureStart = {
                    gestureState.startedInOriginal = (state.resizeMode == AspectRatioFrameLayout.RESIZE_MODE_FIT)
                    gestureState.snappedToFillThisGesture = false
                },
                onPinchZoom = pinch@ { zoomDelta ->
                    if (gestureState.snappedToFillThisGesture) {
                        state.showZoomOverlay = true
                        state.zoomOverlayVersion += 1
                        return@pinch
                    }
                    if (zoomDelta > 1.0f) {
                        if (gestureState.startedInOriginal && state.resizeMode == AspectRatioFrameLayout.RESIZE_MODE_FIT) {
                            state.resizeMode = AspectRatioFrameLayout.RESIZE_MODE_ZOOM
                            state.videoZoomScale = fillScale
                            gestureState.startedInOriginal = false
                            gestureState.snappedToFillThisGesture = true
                        } else {
                            if (state.resizeMode == AspectRatioFrameLayout.RESIZE_MODE_FIT) {
                                state.resizeMode = AspectRatioFrameLayout.RESIZE_MODE_ZOOM
                                state.videoZoomScale = fillScale
                            } else {
                                state.videoZoomScale = (state.videoZoomScale * zoomDelta)
                                    .coerceIn(fillScale, fillScale * 4.0f)
                            }
                        }
                    } else if (zoomDelta < 1.0f && state.resizeMode == AspectRatioFrameLayout.RESIZE_MODE_ZOOM) {
                        val nextScale = state.videoZoomScale * zoomDelta
                        if (nextScale < fillScale) {
                            state.resizeMode = AspectRatioFrameLayout.RESIZE_MODE_FIT
                            state.videoZoomScale = 1.0f
                        } else {
                            state.videoZoomScale = nextScale.coerceAtMost(fillScale * 4.0f)
                        }
                    }
                    state.showZoomOverlay = true
                    state.zoomOverlayVersion += 1
                }
            )
    ) {
        PlayerPlaybackSurface(
            content = playerContent,
            currentUrl = state.currentUrl,
            resolvedUrl = state.resolvedUrl,
            useMpvBackend = useMpvBackend,
            mpvPlayer = mpvPlayer,
            exoPlayer = exoPlayer,
            activeProfile = activeProfile,
            resizeMode = state.resizeMode,
            playback = state.engine.playback,
            timeline = state.engine.timeline,
            buffer = state.engine.buffer,
            render = state.engine.render,
            playerError = state.engine.playerError,
            torrentStatus = torrentStatus,
            deviceType = deviceType,
            isSwitchingAudioSource = state.isSwitchingAudioSource,
            currentStreamIndex = state.currentStreamIndex,
            currentStreamDetailLine = currentStreamDetailLine,
            currentStreamsSize = state.currentStreams.size,
            lang = lang,
            showControls = state.showControls,
            activeEngine = activeEngine,
            showControlsTemp = showControlsTemp,
            seekSafely = seekSafely,
            toggleSubtitleSelection = toggleSubtitleSelection,
            onToggleAspect = {
                val switchingToZoom = state.resizeMode == AspectRatioFrameLayout.RESIZE_MODE_FIT
                state.resizeMode = if (switchingToZoom) {
                    AspectRatioFrameLayout.RESIZE_MODE_ZOOM
                } else {
                    AspectRatioFrameLayout.RESIZE_MODE_FIT
                }
                state.videoZoomScale = if (switchingToZoom) fillScale else 1.0f
                showControlsTemp()
            },
            playbackSpeed = state.playbackSpeed,
            onPlaybackSpeedChange = {
                state.playbackSpeed = it
                activeEngine?.setSpeed(it)
                showControlsTemp()
            },
            playPauseFocusRequester = playPauseFocusRequester,
            seekbarFocusRequester = seekbarFocusRequester,
            isScrubbing = state.isScrubbing,
            scrubPosition = state.scrubPosition,
            onScrubbingChange = { s, p ->
                state.isScrubbing = s
                state.scrubPosition = p
            },
            currentEpisodeMetaLine = state.currentEpisodeMetaLine,
            currentSubtitle = currentSubtitle,
            currentExternalSubtitles = state.currentExternalSubtitleTracks,
            embeddedNativeAssTracks = state.embeddedNativeAssTracks,
            subtitleDelayMs = state.subtitleDelayMs,
            effectiveTechnicalInfo = effectiveTechnicalInfo,
            seekForwardMs = seekForwardMs,
            seekBackwardMs = seekBackwardMs,
            hasPreviousEpisode = state.previousEpisodePending != null,
            hasNextEpisode = state.nextEpisodePending != null,
            nextEpisode = state.nextEpisodePending,
            onPlayPrevious = playPrevious,
            onPlayNext = playNext,
            onCast = smartCast,
            onOpenInExternalPlayer = openInExternalPlayer,
            onPictureInPicture = { enterPlayerPipMode(context, lang, state.engine.playback.isPlaying, state.nextEpisodePending != null) },
            onShowSettingsTab = { tab ->
                if (tab == 4) {
                    openSourceSelectionScreen()
                } else if (tab == 3 && meta.type != "series") {
                    showControlsTemp()
                } else if ((tab == 0 || tab == 1) && deviceType == DeviceType.Mobile) {
                    state.showMobileTrackPicker = true
                    state.showControls = false
                } else {
                    state.activeSettingsTab = tab
                    state.showSettings = true
                    state.showControls = false
                }
            },
            onClose = closePlayer,
            onNextEpisodeCardShown = {
                state.nextEpisodePending?.let { viewModel.onNextEpisodeCardShown(meta, it.id, activeProfile) }
            },
            timelinePosition = { if (useMpvBackend) state.engine.timeline.position else state.timelinePosition },
            skipSegments = state.skipSegments,
            chapters = state.chapters,
            dismissedSkipSegments = state.dismissedSkipSegments,
            onSkipSegment = { segment ->
                if (segment.type == "outro") playNext() else seekSafely(segment.endTime)
                state.dismissedSkipSegments = state.dismissedSkipSegments + segment.dismissKey()
                state.introAutoSkipped = true
                state.segmentSkipFeedbackVersion += 1
                showControlsTemp()
            },
            onDismissSegment = { segment ->
                state.dismissedSkipSegments = state.dismissedSkipSegments + segment.dismissKey()
            },
            showSegmentSkipFeedback = state.showSegmentSkipFeedback,
            holdSpeedVisible = state.holdSpeedVisible,
            showVolumeBar = state.showVolumeBar,
            currentVolume = state.currentVolume,
            maxVolume = maxVolume,
            showSeekFeedback = state.showSeekFeedback,
            seekDirection = state.seekDirection,
            seekFeedbackMs = state.seekFeedbackMs,
            videoZoomScale = state.videoZoomScale,
            fillScale = fillScale,
            showZoomOverlay = state.showZoomOverlay,
            parentsGuide = parentsGuide,
            showParentsGuide = state.showParentsGuide,
            onParentsGuideAnimationComplete = { state.showParentsGuide = false }
        )

        if (state.showSettings) {
            PlayerSettingsPanel(
                meta = meta,
                currentVideoId = state.currentVideoId,
                deviceType = deviceType,
                viewModel = viewModel,
                activeProfile = activeProfile,
                lang = lang,
                activeSettingsTab = state.activeSettingsTab,
                currentStreams = state.currentStreams,
                currentUrl = state.currentUrl,
                currentStreamIndex = state.currentStreamIndex,
                availableAudios = availableAudios,
                currentAudio = currentAudio,
                availableSubtitles = availableSubtitles,
                currentSubtitle = currentSubtitle,
                playbackSpeed = state.playbackSpeed,
                audioDelayMs = state.audioDelayMs,
                subtitleDelayMs = state.subtitleDelayMs,
                onEpisodeSelected = { nextId, episodeName ->
                    state.currentVideoId = nextId
                    state.currentEpisodeMetaLine = episodeName
                    state.currentStreamIndex = 0
                    state.lastSavedPosition = 0L
                    state.shouldApplyInitialProgress = false
                    state.showSettings = false
                    showControlsTemp()
                },
                onCloseSettings = {
                    state.showSettings = false
                    showControlsTemp()
                },
                onSelectStreamIndex = switchToStream,
                onSelectAudio = { activeEngine?.selectAudio(it) },
                onSelectSubtitle = { activeEngine?.enableSubtitle(it) },
                onDisableSubtitle = { activeEngine?.disableSubtitles() },
                onSpeedChange = {
                    state.playbackSpeed = it
                    activeEngine?.setSpeed(it)
                    showControlsTemp()
                },
                onAudioDelayChange = { state.audioDelayMs = it.coerceIn(-5_000L, 5_000L) },
                onSubtitleDelayChange = { state.subtitleDelayMs = it.coerceIn(-5_000L, 5_000L) },
                onSubtitleTextOpacityChange = { value ->
                    activeProfile?.let { onUpdateProfile(it.copy(subtitleTextOpacity = value.coerceIn(0f, 1f))) }
                },
                onSubtitleBackgroundOpacityChange = { value ->
                    activeProfile?.let { onUpdateProfile(it.copy(subtitleBackgroundOpacity = value.coerceIn(0f, 1f))) }
                },
                onSubtitleOutlineOpacityChange = { value ->
                    activeProfile?.let { onUpdateProfile(it.copy(subtitleOutlineOpacity = value.coerceIn(0f, 1f))) }
                },
                currentPositionMs = if (useMpvBackend) state.engine.timeline.position else state.timelinePosition,
                markSegmentType = state.markSegmentType,
                markSegmentStartMs = state.markSegmentStartMs,
                markSegmentEndMs = state.markSegmentEndMs,
                markSegmentSubmitting = state.markSegmentSubmitting,
                markSegmentFeedback = state.markSegmentFeedback,
                markSegmentCooldownRemainingSec = markSegmentCooldownRemainingSec(state, meta),
                onSelectMarkSegmentType = { state.markSegmentType = it },
                onMarkSegmentStart = { state.markSegmentStartMs = if (useMpvBackend) state.engine.timeline.position else state.timelinePosition },
                onMarkSegmentEnd = { state.markSegmentEndMs = if (useMpvBackend) state.engine.timeline.position else state.timelinePosition },
                onAdjustMarkSegmentStart = { delta -> state.markSegmentStartMs = (state.markSegmentStartMs ?: 0L).plus(delta).coerceAtLeast(0L) },
                onAdjustMarkSegmentEnd = { delta -> state.markSegmentEndMs = (state.markSegmentEndMs ?: 0L).plus(delta).coerceAtLeast(0L) },
                onSubmitMarkSegment = {
                    val type = state.markSegmentType
                    val start = state.markSegmentStartMs
                    val end = state.markSegmentEndMs
                    val apiKey = activeProfile?.safeIntroDbApiKey
                    if (type != null && start != null && end != null && !apiKey.isNullOrBlank()) {
                        coroutineScope.launch {
                            state.markSegmentSubmitting = true
                            val imdbId = resolveIntroImdbId(viewModel, meta, state.currentVideoId, lang)
                            val seasonEpisode = imdbId?.let { extractSeasonEpisode(state.currentVideoId) ?: extractSeasonEpisode(meta.id) }
                            if (imdbId == null || seasonEpisode == null) {
                                state.markSegmentFeedback = playerText(lang, "mark_segment_unknown_episode")
                            } else {
                                val (season, episode) = seasonEpisode
                                val result = viewModel.submitIntroSegment(apiKey, type, imdbId, season, episode, start / 1000.0, end / 1000.0)
                                when (result) {
                                    is IntroDbSubmitResult.Success -> {
                                        val cooldownKey = "$type:$season:$episode"
                                        state.introDbCooldowns = state.introDbCooldowns + (cooldownKey to System.currentTimeMillis() + 5 * 60_000L)
                                        state.markSegmentStartMs = null
                                        state.markSegmentEndMs = null
                                        state.markSegmentFeedback = playerText(lang, "mark_segment_status_${result.status}")
                                    }
                                    is IntroDbSubmitResult.Error -> {
                                        state.markSegmentFeedback = playerText(lang, "mark_segment_error_${result.reason}")
                                    }
                                }
                            }
                            state.markSegmentSubmitting = false
                        }
                    }
                }
            )
        }

        if (state.showMobileTrackPicker && deviceType == DeviceType.Mobile) {
            MobileTrackPickerOverlay(
                lang = lang,
                availableAudios = availableAudios,
                currentAudio = currentAudio,
                availableSubtitles = availableSubtitles,
                currentSubtitle = currentSubtitle,
                audioDelayMs = state.audioDelayMs,
                subtitleDelayMs = state.subtitleDelayMs,
                subtitleTextOpacity = activeProfile?.safeSubtitleTextOpacity ?: 1f,
                subtitleBackgroundOpacity = activeProfile?.safeSubtitleBackgroundOpacity ?: 0.5f,
                subtitleOutlineOpacity = activeProfile?.safeSubtitleOutlineOpacity ?: 1f,
                onApply = { selectedAudio, selectedSubtitle ->
                    selectedAudio?.let { activeEngine?.selectAudio(it) }
                    if (selectedSubtitle != null) {
                        activeEngine?.enableSubtitle(selectedSubtitle)
                    } else {
                        activeEngine?.disableSubtitles()
                    }
                    state.showMobileTrackPicker = false
                    showControlsTemp()
                },
                onAudioDelayChange = { state.audioDelayMs = it.coerceIn(-5_000L, 5_000L) },
                onSubtitleDelayChange = { state.subtitleDelayMs = it.coerceIn(-5_000L, 5_000L) },
                onSubtitleTextOpacityChange = { value ->
                    activeProfile?.let { onUpdateProfile(it.copy(subtitleTextOpacity = value.coerceIn(0f, 1f))) }
                },
                onSubtitleBackgroundOpacityChange = { value ->
                    activeProfile?.let { onUpdateProfile(it.copy(subtitleBackgroundOpacity = value.coerceIn(0f, 1f))) }
                },
                onSubtitleOutlineOpacityChange = { value ->
                    activeProfile?.let { onUpdateProfile(it.copy(subtitleOutlineOpacity = value.coerceIn(0f, 1f))) }
                },
                onDismiss = {
                    state.showMobileTrackPicker = false
                    showControlsTemp()
                }
            )
        }
    }
}
