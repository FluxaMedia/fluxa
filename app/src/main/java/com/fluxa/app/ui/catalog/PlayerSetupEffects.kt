@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.ui.catalog

import android.app.Activity
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.getValue
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.AspectRatioFrameLayout
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.player.DolbyVisionFallbackMode
import com.fluxa.app.player.ExoPlayerEngine
import com.fluxa.app.player.MpvEmbeddedPlayer
import com.fluxa.app.player.PlayerEngine
import com.fluxa.app.player.PlayerEngineRequest
import com.fluxa.app.player.TorrentStreamManager
import com.fluxa.app.player.TorrentStreamResult
import com.fluxa.app.player.TorrentStreamStatus
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

@Composable
internal fun PlayerOrientationLock(activity: Activity?, deviceType: DeviceType) {
    DisposableEffect(Unit) {
        val originalOrientation = activity?.requestedOrientation
            ?: android.content.pm.ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
        if (deviceType == DeviceType.Mobile) {
            activity?.requestedOrientation = android.content.pm.ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        }
        onDispose {
            activity?.requestedOrientation = originalOrientation
        }
    }
}

@Composable
internal fun PlayerMpvStateSyncEffect(
    useMpvBackend: Boolean,
    mpvPlayer: MpvEmbeddedPlayer?,
    updateEngine: (PlayerEngineSnapshot.() -> PlayerEngineSnapshot) -> Unit,
) {
    val latestUpdateEngine = rememberUpdatedState(updateEngine)
    LaunchedEffect(useMpvBackend, mpvPlayer) {
        if (!useMpvBackend || mpvPlayer == null) return@LaunchedEffect
        mpvPlayer.state.collect { s ->
            latestUpdateEngine.value {
                val newPlayback = PlaybackSnapshot(
                    isPlaying = s.isPlaying,
                    playWhenReadyForScrobble = s.isPlaying,
                    isBuffering = s.isBuffering,
                    hasStartedPlaying = s.isVideoReady || s.durationMs > 0L,
                    playbackEnded = false,
                )
                val newTimeline = TimelineSnapshot(
                    position = s.positionMs,
                    duration = if (s.durationMs > 0L) s.durationMs else 0L,
                )
                val newBuffer = BufferSnapshot(
                    bufferPercent = if (s.isVideoReady) 100 else 0,
                    loadProgress = if (s.isVideoReady) 1f else 0f,
                    seekbarBufferedProgress = if (s.isVideoReady) 1f else 0f,
                )
                val newRender = RenderSnapshot(isVideoRendered = s.isVideoReady)
                copy(
                    playback = if (playback == newPlayback) playback else newPlayback,
                    timeline = if (timeline == newTimeline) timeline else newTimeline,
                    buffer = if (buffer == newBuffer) buffer else newBuffer,
                    render = if (render == newRender) render else newRender,
                    playerError = s.error,
                )
            }
        }
    }
}

@Composable
internal fun PlayerEngineSettingsEffects(
    activeEngine: PlayerEngine?,
    audioDelayMs: Long,
    subtitleDelayMs: Long,
    resizeMode: Int
) {
    DisposableEffect(activeEngine) {
        onDispose {
            activeEngine?.setAudioDelayMs(0L)
        }
    }
    LaunchedEffect(audioDelayMs, activeEngine) {
        activeEngine?.setAudioDelayMs(audioDelayMs)
    }
    LaunchedEffect(subtitleDelayMs, activeEngine) {
        activeEngine?.setSubtitleDelayMs(subtitleDelayMs)
    }
    LaunchedEffect(resizeMode, activeEngine) {
        activeEngine?.setZoomed(resizeMode == AspectRatioFrameLayout.RESIZE_MODE_ZOOM)
    }
}

@Composable
internal fun PlayerTransientFeedbackEffects(
    showVolumeBar: Boolean,
    seekFeedbackVersion: Int,
    segmentSkipFeedbackVersion: Int,
    zoomOverlayVersion: Int,
    setShowVolumeBar: (Boolean) -> Unit,
    setShowSeekFeedback: (Boolean) -> Unit,
    resetPendingSeek: () -> Unit,
    setShowSegmentSkipFeedback: (Boolean) -> Unit,
    setShowZoomOverlay: (Boolean) -> Unit
) {
    LaunchedEffect(showVolumeBar) {
        if (showVolumeBar) {
            delay(2000)
            setShowVolumeBar(false)
        }
    }
    LaunchedEffect(seekFeedbackVersion) {
        if (seekFeedbackVersion > 0) {
            delay(900)
            setShowSeekFeedback(false)
            resetPendingSeek()
        }
    }
    LaunchedEffect(segmentSkipFeedbackVersion) {
        if (segmentSkipFeedbackVersion > 0) {
            setShowSegmentSkipFeedback(true)
            delay(760)
            setShowSegmentSkipFeedback(false)
        }
    }
    LaunchedEffect(zoomOverlayVersion) {
        if (zoomOverlayVersion > 0) {
            delay(1500)
            setShowZoomOverlay(false)
        }
    }
}

