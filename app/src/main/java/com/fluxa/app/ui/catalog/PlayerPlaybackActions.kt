@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.ui.catalog

import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeAutoPlayNextEpisode
import com.fluxa.app.data.local.safeTryBingeGroup
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.player.PlayerEngine
import com.fluxa.app.shared.feature.player.MediaTrack
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

internal class PlayerPlaybackActions(
    private val state: PlayerScreenState,
    private val scope: CoroutineScope,
    private val useMpvBackend: Boolean,
    private val exoPlayer: ExoPlayer,
    private val activeEngine: PlayerEngine?,
    private val meta: Meta,
    private val videoId: String?,
    private val viewModel: HomeViewModel,
    private val activeProfile: UserProfile?,
    private val availableSubtitles: List<MediaTrack>,
    private val currentSubtitle: MediaTrack?,
    private val seekForwardMs: Long,
    private val seekBackwardMs: Long,
    private val requestPlayPauseFocus: () -> Unit,
    private val onSelectSource: (PlayerSourceSelectionRequest) -> Unit,
    private val onBack: () -> Unit,
) {
    fun seekSafely(targetPosition: Long) {
        if (!state.engine.playback.hasStartedPlaying) return
        val maxDuration = playbackDurationLimit()
        val target = targetPosition.coerceIn(0L, maxDuration)
        state.pendingSeekTarget = target
        state.engine = state.engine.copy(timeline = state.engine.timeline.copy(position = target))

        if (useMpvBackend) {
            activeEngine?.seekTo(target)
            return
        }

        val before = exoPlayer.currentPosition
        state.timelinePosition = target
        exoPlayer.seekTo(target)
        if (!state.resolvedUrl.isTorrentPlaybackUrl()) {
            scope.launch {
                delay(SEEK_RETRY_DELAY_MS)
                val after = exoPlayer.currentPosition
                if (
                    target > before + SEEK_FORWARD_TOLERANCE_MS &&
                    after < before + SEEK_RETRY_PROGRESS_MS &&
                    exoPlayer.isCurrentMediaItemSeekable
                ) {
                    exoPlayer.seekTo(target)
                    state.timelinePosition = target
                    state.engine = state.engine.copy(
                        timeline = state.engine.timeline.copy(position = target),
                    )
                }
            }
        }
    }

    fun showControlsTemporarily() {
        state.showControls = true
        state.controlsTimerJob?.cancel()
        state.controlsTimerJob = scope.launch {
            delay(CONTROLS_HIDE_DELAY_MS)
            if (!state.isScrubbing) state.showControls = false
        }
        scope.launch {
            delay(FOCUS_REQUEST_DELAY_MS)
            runCatching(requestPlayPauseFocus)
        }
    }

    fun switchToStream(streamIndex: Int) {
        val nextStream = state.currentStreams.getOrNull(streamIndex) ?: return
        if (streamIndex == state.currentStreamIndex || nextStream.playableUrl.isNullOrEmpty()) return

        android.util.Log.w(
            TAG,
            "Switching stream ${state.currentStreamIndex} -> $streamIndex: " +
                "fileIdx=${nextStream.fileIdx} url=${nextStream.playableUrl} " +
                "name=${nextStream.name} title=${nextStream.title}",
        )
        state.lastSavedPosition = currentPosition()
        state.shouldApplyInitialProgress = false
        state.isSwitchingAudioSource = true
        state.engine = state.engine.copy(
            playerError = null,
            playback = state.engine.playback.copy(
                isBuffering = true,
                hasStartedPlaying = false,
            ),
            render = RenderSnapshot(),
        )
        state.currentStreamIndex = streamIndex
        state.currentUrl = nextStream.playableUrl
        scope.launch {
            delay(STREAM_SWITCH_SETTLE_MS)
            state.isSwitchingAudioSource = false
        }
    }

    fun fallbackToNextCloudstreamStream(): Boolean {
        if (!isCloudstreamPlayback()) return false
        val failedUrl = state.currentStreams.getOrNull(state.currentStreamIndex)?.playableUrl
            ?: state.currentUrl
        if (!failedUrl.isNullOrBlank()) {
            state.failedAutoFallbackUrls = state.failedAutoFallbackUrls + failedUrl
        }
        val indices = state.currentStreams.indices.drop(state.currentStreamIndex + 1) +
            state.currentStreams.indices.take(state.currentStreamIndex)
        val nextIndex = indices.firstOrNull { index ->
            val candidateUrl = state.currentStreams[index].playableUrl
            !candidateUrl.isNullOrBlank() && candidateUrl !in state.failedAutoFallbackUrls
        } ?: return false
        switchToStream(nextIndex)
        return true
    }

    fun openSourceSelection() {
        val selectedStream = state.currentStreams.getOrNull(state.currentStreamIndex)
        onSelectSource(
            PlayerSourceSelectionRequest(
                meta = meta,
                videoId = state.currentVideoId ?: videoId,
                progress = currentPosition().takeIf { it > 0L } ?: state.engine.timeline.position,
                streams = state.currentStreams,
                streamIndex = state.currentStreamIndex.takeIf { it >= 0 },
                streamUrl = selectedStream?.playableUrl ?: state.currentUrl,
                streamTitle = selectedStream?.title,
            ),
        )
    }

    fun playNext() {
        state.nextEpisodePending?.let { next ->
            if (activeProfile?.safeTryBingeGroup == true) {
                state.preferredBingeGroupForNextEpisode =
                    state.currentStreams.getOrNull(state.currentStreamIndex)?.bingeGroup
            }
            state.resetForEpisode(next.id)
        } ?: onBack()
    }

    fun playPrevious() {
        state.previousEpisodePending?.let { previous -> state.resetForEpisode(previous.id) }
    }

    fun toggleSubtitleSelection() {
        if (currentSubtitle != null) {
            activeEngine?.disableSubtitles()
        } else {
            TrackSelectionState.findPreferredSubtitle(
                availableSubtitles = availableSubtitles,
                profile = activeProfile,
                meta = meta,
            )?.let { track -> activeEngine?.enableSubtitle(track) }
        }
        showControlsTemporarily()
    }

    fun closePlayer() {
        state.showSettings = false
        state.showControls = false
        onBack()
    }

    fun performRelativeSeek(direction: Int) {
        if (!state.engine.playback.hasStartedPlaying) return
        val step = if (direction > 0) seekForwardMs else seekBackwardMs
        val base = state.pendingSeekTarget ?: currentPosition()
        val rawTarget = if (direction > 0) base + step else base - step
        seekSafely(rawTarget.coerceIn(0L, playbackDurationLimit()))
        if (state.seekDirection == direction) {
            state.seekFeedbackMs += step
        } else {
            state.seekDirection = direction
            state.seekFeedbackMs = step
        }
        state.seekFeedbackVersion += 1
        state.showSeekFeedback = true
    }

    fun autoPlayNextWhenEnded() {
        if (
            state.engine.playback.playbackEnded &&
            state.engine.playback.hasStartedPlaying &&
            activeProfile?.safeAutoPlayNextEpisode == true
        ) {
            playNext()
        }
    }

    fun isCloudstreamPlayback(): Boolean =
        meta.id.startsWith(CLOUDSTREAM_ID_PREFIX) ||
            state.currentVideoId?.startsWith(CLOUDSTREAM_ID_PREFIX) == true ||
            state.currentStreams.getOrNull(state.currentStreamIndex)?.addonName?.trim() in
            viewModel.loadedCs3ApiNames.value

    private fun currentPosition(): Long =
        if (useMpvBackend) state.engine.timeline.position else exoPlayer.currentPosition

    private fun playbackDurationLimit(): Long = when {
        useMpvBackend && state.engine.timeline.duration > 0L -> state.engine.timeline.duration
        exoPlayer.duration > 0L -> exoPlayer.duration
        state.engine.timeline.duration > 0L -> state.engine.timeline.duration
        else -> Long.MAX_VALUE
    }

    private companion object {
        const val TAG = "PlayerScreen"
        const val CLOUDSTREAM_ID_PREFIX = "cs3:"
        const val SEEK_RETRY_DELAY_MS = 900L
        const val SEEK_FORWARD_TOLERANCE_MS = 2_000L
        const val SEEK_RETRY_PROGRESS_MS = 1_000L
        const val CONTROLS_HIDE_DELAY_MS = 5_000L
        const val FOCUS_REQUEST_DELAY_MS = 100L
        const val STREAM_SWITCH_SETTLE_MS = 1_400L
    }
}
