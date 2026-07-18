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
import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.core.rust.FluxaUniFfiCoreStateHandle
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video
import com.fluxa.app.player.ExternalSubtitleTrack
import com.fluxa.app.player.NativeAssTrack
import com.google.gson.Gson
import kotlinx.coroutines.Job

internal data class PlayerRuntimeCoreState(
    val currentVideoId: String? = null,
    val currentStreams: List<Stream> = emptyList(),
    val currentStreamIndex: Int = 0,
    val currentUrl: String? = null,
    val resolvedUrl: String? = null,
    val zeroSpeedTicks: Int = 0,
    val isBuffering: Boolean = false,
    val isVideoRendered: Boolean = false,
    val playerError: String? = null
)

internal class PlayerScreenState(
    initialVideoId: String?,
    initialStreamIndex: Int,
    initialVolume: Int
) {
    private val gson = Gson()
    private val coreState: FluxaUniFfiCoreStateHandle = FluxaCoreUniFfi.createAppCoreState(
        mapOf(
            "player" to mapOf(
                "currentVideoId" to initialVideoId,
                "currentStreamIndex" to initialStreamIndex,
                "lastSavedPosition" to 0L,
                "shouldApplyInitialProgress" to true,
                "playbackEnded" to false,
                "hasStartedPlaying" to false,
                "isVideoRendered" to false,
                "isBuffering" to true
            )
        )
    )

    var currentUrl by mutableStateOf<String?>(null)
    var resolvedUrl by mutableStateOf<String?>(null)
    var currentVideoId by mutableStateOf(initialVideoId)
    var currentStreams by mutableStateOf<List<Stream>>(emptyList())
    var currentStreamIndex by mutableIntStateOf(initialStreamIndex)
    var failedAutoFallbackUrls by mutableStateOf<Set<String>>(emptySet())
    var zeroSpeedTicks by mutableIntStateOf(0)
    var lastSavedPosition by mutableLongStateOf(0L)
    var shouldApplyInitialProgress by mutableStateOf(true)
    var controlsTimerJob by mutableStateOf<Job?>(null)
    var lastSavedTimestamp by mutableLongStateOf(0L)
    var isSwitchingAudioSource by mutableStateOf(false)

    var engine by mutableStateOf(PlayerEngineSnapshot())
    var timelinePosition by mutableLongStateOf(0L)

    var showControls by mutableStateOf(true)
    var resizeMode by mutableIntStateOf(AspectRatioFrameLayout.RESIZE_MODE_FIT)
    var videoZoomScale by mutableFloatStateOf(1.0f)
    var playbackSpeed by mutableFloatStateOf(1.0f)
    var holdSpeedVisible by mutableStateOf(false)
    var currentEpisodeMetaLine by mutableStateOf<String?>(null)
    var currentEpisodeArtwork by mutableStateOf<String?>(null)
    var skipSegments by mutableStateOf<List<IntroTimestamps>>(emptyList())
    var chapters by mutableStateOf<List<com.fluxa.app.player.Chapter>>(emptyList())
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

    fun resetForEpisode(videoId: String) {
        val snapshot = coreState.dispatch(
            CoreAction(
                type = "playerResetForEpisode",
                videoId = videoId
            )
        )
        val player = gson.fromJson(snapshot, CoreStateSnapshot::class.java)?.player ?: return
        currentVideoId = player.currentVideoId
        currentStreamIndex = player.currentStreamIndex
        lastSavedPosition = player.lastSavedPosition
        shouldApplyInitialProgress = player.shouldApplyInitialProgress
        engine = PlayerEngineSnapshot(
            playback = PlaybackSnapshot(
                isBuffering = player.isBuffering,
                hasStartedPlaying = player.hasStartedPlaying,
                playbackEnded = player.playbackEnded,
            ),
            render = RenderSnapshot(isVideoRendered = player.isVideoRendered),
        )
    }

    private data class CoreAction(
        val type: String,
        val videoId: String
    )

    private data class CoreStateSnapshot(
        val player: CorePlayerSnapshot = CorePlayerSnapshot()
    )

    private data class CorePlayerSnapshot(
        val currentVideoId: String? = null,
        val currentStreamIndex: Int = 0,
        val lastSavedPosition: Long = 0L,
        val shouldApplyInitialProgress: Boolean = false,
        val playbackEnded: Boolean = false,
        val hasStartedPlaying: Boolean = false,
        val isVideoRendered: Boolean = false,
        val isBuffering: Boolean = true
    )
}

@Composable
internal fun rememberPlayerScreenState(
    initialVideoId: String?,
    initialStreamIndex: Int,
    initialVolume: Int
): PlayerScreenState {
    return remember { PlayerScreenState(initialVideoId, initialStreamIndex, initialVolume) }
}
