@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.media3.ui.AspectRatioFrameLayout
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.player.ExternalSubtitleTrack
import com.fluxa.app.player.NativeAssTrack
import kotlinx.coroutines.Job

internal data class PlayerRuntimeCoreState(
    val currentVideoId: String? = null,
    val positionMs: Long = 0L,
    val currentStreams: List<Stream> = emptyList(),
    val currentStreamIndex: Int = 0,
    val currentUrl: String? = null,
    val resolvedUrl: String? = null,
    val zeroSpeedTicks: Int = 0,
    val isBuffering: Boolean = false,
    val isVideoRendered: Boolean = false,
    val playbackEnded: Boolean = false,
    val hasStartedPlaying: Boolean = false,
    val playerError: String? = null
)

internal class PlayerScreenState(
    initialVideoId: String?,
    initialStreamIndex: Int,
    initialVolume: Int,
    private val onCoreTelemetry: (PlayerRuntimeCoreState) -> Unit,
    private val onCoreReset: (String) -> Unit
) {
    private companion object {
        const val CORE_PROGRESS_CHECKPOINT_MS = 3_000L
    }

    var currentUrl by mutableStateOf<String?>(null)
    var telemetryAttemptGeneration by mutableLongStateOf(0L)
    var resolvedUrl by mutableStateOf<String?>(null)
    var currentVideoId by mutableStateOf(initialVideoId)
    var currentStreams by mutableStateOf<List<Stream>>(emptyList())
    private var currentStreamIndexState by mutableIntStateOf(initialStreamIndex)
    var currentStreamIndex: Int
        get() = currentStreamIndexState
        set(value) {
            currentStreamIndexState = value
            syncPlayerCoreState()
        }
    var failedAutoFallbackUrls by mutableStateOf<Set<String>>(emptySet())
    var zeroSpeedTicks by mutableIntStateOf(0)
    var lastSavedPosition by mutableLongStateOf(0L)
    // Null means initial load; a value preserves pause/play across engine replacement.
    var lastSavedPlayWhenReady by mutableStateOf<Boolean?>(null)
    var shouldApplyInitialProgress by mutableStateOf(true)
    var controlsTimerJob by mutableStateOf<Job?>(null)
    var lastSavedTimestamp by mutableLongStateOf(0L)
    var isSwitchingAudioSource by mutableStateOf(false)

    var engine by mutableStateOf(PlayerEngineSnapshot())
        private set
    var timelinePosition by mutableLongStateOf(0L)

    var showControls by mutableStateOf(true)
    var resizeMode by mutableIntStateOf(AspectRatioFrameLayout.RESIZE_MODE_FIT)
    var videoZoomScale by mutableFloatStateOf(1.0f)
    var playbackSpeed by mutableFloatStateOf(1.0f)
    var holdSpeedVisible by mutableStateOf(false)
    var currentEpisodeMetaLine by mutableStateOf<String?>(null)
    var currentEpisodeArtwork by mutableStateOf<String?>(null)
    var skipSegments by mutableStateOf<List<IntroTimestamps>>(emptyList())
    var chapters by mutableStateOf<List<com.fluxa.app.shared.feature.player.Chapter>>(emptyList())
    var dismissedSkipSegments by mutableStateOf<Set<String>>(emptySet())
    var introAutoSkipped by mutableStateOf(false)
    var preferredBingeGroupForNextEpisode by mutableStateOf<String?>(null)
    var showParentsGuide by mutableStateOf(false)
    var parentsGuideShown by mutableStateOf(false)

    var markSegmentType by mutableStateOf<String?>(null)
    var markSegmentStartMs by mutableStateOf<Long?>(null)
    var markSegmentEndMs by mutableStateOf<Long?>(null)
    var markSegmentSubmitting by mutableStateOf(false)
    var markSegmentFeedback by mutableStateOf<String?>(null)
    var introDbCooldowns by mutableStateOf<Map<String, Long>>(emptyMap())

    var currentVolume by mutableIntStateOf(initialVolume)
    var currentVolumeExact by mutableFloatStateOf(initialVolume.toFloat())
    var showVolumeBar by mutableStateOf(false)
    var volumeBarVersion by mutableIntStateOf(0)
    var currentBrightness by mutableFloatStateOf(1f)
    var showBrightnessBar by mutableStateOf(false)
    var brightnessBarVersion by mutableIntStateOf(0)
    var showZoomOverlay by mutableStateOf(false)
    var zoomOverlayVersion by mutableIntStateOf(0)
    var showSeekFeedback by mutableStateOf(false)
    var showSegmentSkipFeedback by mutableStateOf(false)
    var segmentSkipFeedbackVersion by mutableIntStateOf(0)
    var seekDirection by mutableIntStateOf(0)
    var seekFeedbackMs by mutableLongStateOf(0L)
    var seekFeedbackVersion by mutableIntStateOf(0)
    var pendingSeekTarget by mutableStateOf<Long?>(null)

    var showSettings by mutableStateOf(false)
    var activeSettingsTab by mutableIntStateOf(0)
    var audioDelayMs by mutableLongStateOf(0L)
    var subtitleDelayMs by mutableLongStateOf(0L)

    var isScrubbing by mutableStateOf(false)
    var scrubPosition by mutableLongStateOf(0L)

    var isLocked by mutableStateOf(false)
    var showLockHint by mutableStateOf(false)

    var currentExternalSubtitleTracks by mutableStateOf<List<ExternalSubtitleTrack>>(emptyList())
    var embeddedNativeAssTracks by mutableStateOf<List<NativeAssTrack>>(emptyList())
    var externalPlayerStartedPosition by mutableLongStateOf(-1L)

    var nextEpisodePending by mutableStateOf<Video?>(null)
    var previousEpisodePending by mutableStateOf<Video?>(null)
    var terminalRecommendations by mutableStateOf<List<Meta>>(emptyList())
    var terminalRecommendationsRequested by mutableStateOf(false)
    var terminalRecommendationsLoading by mutableStateOf(false)

    private var lastCorePositionMs: Long? = null
    private var lastCoreStreamIndex: Long? = null
    private var lastCoreBuffering: Boolean? = null
    private var lastCorePlaybackEnded: Boolean? = null
    private var lastCoreStarted: Boolean? = null
    private var lastCoreRendered: Boolean? = null

    fun resetForEpisode(videoId: String) {
        terminalRecommendations = emptyList()
        terminalRecommendationsRequested = false
        terminalRecommendationsLoading = false
        currentVideoId = videoId
        currentStreamIndexState = 0
        lastSavedPosition = 0L
        shouldApplyInitialProgress = false
        engine = PlayerEngineSnapshot(
            playback = PlaybackSnapshot(
                isBuffering = true,
                hasStartedPlaying = false,
                playbackEnded = false,
            ),
            render = RenderSnapshot(isVideoRendered = false),
        )
        lastCorePositionMs = 0L
        lastCoreStreamIndex = 0L
        lastCoreBuffering = true
        lastCorePlaybackEnded = false
        lastCoreStarted = false
        lastCoreRendered = false
        onCoreReset(videoId)
    }

    fun updateEngineSnapshot(next: PlayerEngineSnapshot) {
        engine = next
        syncPlayerCoreState()
    }

    private fun syncPlayerCoreState() {
        val positionMs = engine.timeline.position
        val streamIndex = currentStreamIndex.toLong()
        val buffering = engine.playback.isBuffering
        val playbackEnded = engine.playback.playbackEnded
        val started = engine.playback.hasStartedPlaying
        val rendered = engine.render.isVideoRendered
        val positionChangedEnough = lastCorePositionMs == null ||
            kotlin.math.abs(positionMs - (lastCorePositionMs ?: 0L)) >= CORE_PROGRESS_CHECKPOINT_MS
        val stateChanged = streamIndex != lastCoreStreamIndex ||
            buffering != lastCoreBuffering ||
            playbackEnded != lastCorePlaybackEnded ||
            started != lastCoreStarted ||
            rendered != lastCoreRendered
        if (!positionChangedEnough && !stateChanged) return

        onCoreTelemetry(
            PlayerRuntimeCoreState(
                currentVideoId = currentVideoId,
                positionMs = positionMs,
                currentStreams = currentStreams,
                currentStreamIndex = currentStreamIndex,
                currentUrl = currentUrl,
                resolvedUrl = resolvedUrl,
                zeroSpeedTicks = zeroSpeedTicks,
                isBuffering = buffering,
                isVideoRendered = rendered,
                playbackEnded = playbackEnded,
                hasStartedPlaying = started,
                playerError = null,
            )
        )
        lastCorePositionMs = positionMs
        lastCoreStreamIndex = streamIndex
        lastCoreBuffering = buffering
        lastCorePlaybackEnded = playbackEnded
        lastCoreStarted = started
        lastCoreRendered = rendered
    }

}

@Composable
internal fun rememberPlayerScreenState(
    initialVideoId: String?,
    initialStreamIndex: Int,
    initialVolume: Int,
    onCoreTelemetry: (PlayerRuntimeCoreState) -> Unit,
    onCoreReset: (String) -> Unit
): PlayerScreenState {
    return remember(initialVideoId, initialStreamIndex, initialVolume) {
        PlayerScreenState(initialVideoId, initialStreamIndex, initialVolume, onCoreTelemetry, onCoreReset)
    }
}
