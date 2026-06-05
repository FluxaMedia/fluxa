@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)

package com.fluxa.app.ui.catalog

import android.content.Context
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.AspectRatioFrameLayout
import kotlinx.coroutines.delay
import kotlin.math.abs
import java.util.concurrent.atomic.AtomicInteger
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.player.MediaTrack
import com.fluxa.app.player.MpvEmbeddedPlayer
import com.fluxa.app.player.PlayerEngine
import com.fluxa.app.player.TorrentStreamStatus
import com.fluxa.app.player.VideoFormatBadge

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
    activeVideoFormatBadge: VideoFormatBadge?,
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
    val gestureState = remember { object { var startedInOriginal = false; var pinchBelowFitAccum = 1.0f } }
    val playerContent = remember(meta) { meta.toPlayerContentUiModel() }
    val currentStreamDetailLine = remember(state.currentStreams, state.currentStreamIndex) {
        state.currentStreams
            .getOrNull(state.currentStreamIndex)
            ?.cloudstreamPlaybackDetailLine()
    }

    val showFpsCounter = activeProfile?.safeShowFpsCounter ?: false
    val videoFrameCount = remember { AtomicInteger(0) }
    var fpsCounterValue by remember { mutableIntStateOf(0) }
    DisposableEffect(exoPlayer, showFpsCounter, useMpvBackend) {
        val listener = if (showFpsCounter && !useMpvBackend) {
            androidx.media3.exoplayer.video.VideoFrameMetadataListener { _, _, _, _ ->
                videoFrameCount.incrementAndGet()
            }.also { exoPlayer.setVideoFrameMetadataListener(it) }
        } else null
        onDispose { listener?.let { exoPlayer.clearVideoFrameMetadataListener(it) } }
    }
    LaunchedEffect(showFpsCounter, useMpvBackend) {
        if (showFpsCounter) {
            while (true) {
                delay(1000L)
                fpsCounterValue = if (useMpvBackend) {
                    mpvPlayer?.currentFps()?.toInt() ?: 0
                } else {
                    videoFrameCount.getAndSet(0)
                }
            }
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
            .focusRequester(mainFocusRequester)
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
                    gestureState.pinchBelowFitAccum = 1.0f
                },
                onPinchZoom = { zoomDelta ->
                    if (zoomDelta > 1.0f) {
                        gestureState.pinchBelowFitAccum = 1.0f
                        if (gestureState.startedInOriginal && state.resizeMode == AspectRatioFrameLayout.RESIZE_MODE_FIT) {
                            state.resizeMode = AspectRatioFrameLayout.RESIZE_MODE_ZOOM
                            state.videoZoomScale = 1.0f
                            gestureState.startedInOriginal = false
                        }
                        state.videoZoomScale = (state.videoZoomScale * zoomDelta).coerceAtMost(4.0f)
                    } else {
                        if (state.videoZoomScale > 1.05f) {
                            state.videoZoomScale = (state.videoZoomScale * zoomDelta).coerceAtLeast(1.0f)
                            gestureState.pinchBelowFitAccum = 1.0f
                        } else if (state.resizeMode == AspectRatioFrameLayout.RESIZE_MODE_ZOOM && !gestureState.startedInOriginal) {
                            // Keep pinch-out at Fill instead of falling back to letterboxed Original.
                            gestureState.pinchBelowFitAccum = (gestureState.pinchBelowFitAccum * zoomDelta).coerceAtMost(1.0f)
                            if (gestureState.pinchBelowFitAccum < 0.75f) {
                                state.resizeMode = AspectRatioFrameLayout.RESIZE_MODE_ZOOM
                                state.videoZoomScale = 1.0f
                                gestureState.pinchBelowFitAccum = 1.0f
                            }
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
                state.resizeMode = if (state.resizeMode == AspectRatioFrameLayout.RESIZE_MODE_FIT) {
                    AspectRatioFrameLayout.RESIZE_MODE_ZOOM
                } else {
                    AspectRatioFrameLayout.RESIZE_MODE_FIT
                }
                state.videoZoomScale = 1.0f
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
            showZoomOverlay = state.showZoomOverlay,
            audioCodecBadge = currentAudio?.audioCodecBadge,
            activeVideoFormatBadge = activeVideoFormatBadge,
            fpsCounterValue = if (showFpsCounter) fpsCounterValue else -1
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
