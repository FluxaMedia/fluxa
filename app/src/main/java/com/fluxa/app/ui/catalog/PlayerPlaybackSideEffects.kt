package com.fluxa.app.ui.catalog

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import com.fluxa.app.common.Constants
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.TraktIntegration
import com.fluxa.app.shared.feature.player.withCurrentEpisodeArtwork
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

@Composable
internal fun PlayerPlaybackSideEffects(
    context: Context,
    viewModel: HomeViewModel,
    activeProfile: UserProfile?,
    meta: Meta,
    currentVideoId: String?,
    currentStreamIndex: Int,
    currentEpisodeMetaLine: String?,
    currentEpisodeArtwork: String?,
    nextEpisode: Video?,
    currentStreams: List<Stream>,
    isPlaying: Boolean,
    playWhenReadyForScrobble: Boolean,
    currentPositionMs: () -> Long,
    duration: Long,
    lastSavedTimestamp: Long,
    onLastSavedTimestampChanged: (Long) -> Unit
) {
    val scope = rememberCoroutineScope()
    var hasScrobbledStart by remember(meta.id, currentVideoId) { mutableStateOf(false) }
    var hasScrobbledStartSimkl by remember(meta.id, currentVideoId) { mutableStateOf(false) }
    var hasScrobbledStop by remember(meta.id, currentVideoId) { mutableStateOf(false) }
    var wasPlayWhenReadyForScrobble by remember(meta.id, currentVideoId) { mutableStateOf(false) }
    var isPausedForScrobble by remember(meta.id, currentVideoId) { mutableStateOf(false) }
    var scrobblePauseJob by remember(meta.id, currentVideoId) { mutableStateOf<Job?>(null) }

    val latestCurrentPositionMs = currentPositionMs
    val latestDuration by rememberUpdatedState(duration)
    val latestCurrentVideoId by rememberUpdatedState(currentVideoId)
    val latestCurrentStreamIndex by rememberUpdatedState(currentStreamIndex)
    val latestCurrentEpisodeMetaLine by rememberUpdatedState(currentEpisodeMetaLine)
    val latestCurrentEpisodeArtwork by rememberUpdatedState(currentEpisodeArtwork)
    val latestNextEpisode by rememberUpdatedState(nextEpisode)
    val latestCurrentStreams by rememberUpdatedState(currentStreams)
    val latestActiveProfile by rememberUpdatedState(activeProfile)
    val latestPlayWhenReadyForScrobble by rememberUpdatedState(playWhenReadyForScrobble)
    val latestHasScrobbledStop by rememberUpdatedState(hasScrobbledStop)

    fun enqueueDurableTraktScrobble(action: String, progress: Float) {
        val profile = latestActiveProfile ?: return
        if (!PlayerScrobbleCoordinator.shouldEnqueueDurable(action, profile.traktAccessToken, progress)) return
        TraktScrobbleWorker.enqueue(
            context = context,
            profileId = profile.id,
            mediaType = meta.type,
            mediaId = TraktIntegration.scrobbleMediaId(meta.id, latestCurrentVideoId, meta.type),
            progress = progress,
            action = action
        )
    }

    fun enqueueDurableSimklScrobble(action: String, positionMs: Long, durationMs: Long) {
        val profile = latestActiveProfile ?: return
        val token = profile.simklAccessToken
        if (token.isNullOrBlank() || durationMs <= 0L) return
        SimklScrobbleWorker.enqueue(
            context = context,
            profileId = profile.id,
            mediaType = meta.type,
            mediaId = TraktIntegration.scrobbleMediaId(meta.id, latestCurrentVideoId, meta.type),
            action = action,
            positionMs = positionMs,
            durationMs = durationMs
        )
    }

    LaunchedEffect(isPlaying, duration) {
        if (duration <= 0) return@LaunchedEffect

        suspend fun checkProgress() {
            val pos = currentPositionMs()
            val token = activeProfile?.traktAccessToken
            val progress = PlayerScrobbleCoordinator.progressPercent(pos, duration)

            if (PlayerScrobbleCoordinator.shouldSendStart(token, isPlaying, hasScrobbledStart, progress)) {
                hasScrobbledStart = true
                isPausedForScrobble = false
                viewModel.scrobblePlayback(
                    token.orEmpty(),
                    meta.type,
                    TraktIntegration.scrobbleMediaId(meta.id, currentVideoId, meta.type),
                    progress,
                    "start"
                )
            }

            val simklToken = activeProfile?.simklAccessToken
            if (PlayerScrobbleCoordinator.shouldSendStart(simklToken, isPlaying, hasScrobbledStartSimkl, progress)) {
                hasScrobbledStartSimkl = true
                enqueueDurableSimklScrobble("start", pos, duration)
            }

            if (isPlaying && isPausedForScrobble) {
                isPausedForScrobble = false
            }

            if (!hasScrobbledStop && progress >= (activeProfile?.safeWatchedThresholdPercent ?: 80f)) {
                if (token != null) enqueueDurableTraktScrobble("stop", progress)
                enqueueDurableSimklScrobble("stop", pos, duration)
                viewModel.markWatchedFromPlayback(meta, currentVideoId, currentEpisodeMetaLine, nextEpisode, duration)
                hasScrobbledStop = true
            }

            val now = System.currentTimeMillis()
            if (!hasScrobbledStop && PlayerScrobbleCoordinator.shouldSavePeriodicProgress(isPlaying, now, lastSavedTimestamp)) {
                val chosenStream = currentStreams.getOrNull(currentStreamIndex)
                viewModel.savePlaybackProgress(
                    meta.withCurrentEpisodeArtwork(currentEpisodeArtwork),
                    pos,
                    duration,
                    currentVideoId,
                    currentStreamIndex,
                    currentEpisodeMetaLine,
                    chosenStream?.playableUrl,
                    chosenStream?.title,
                    lastBingeGroup = chosenStream?.bingeGroup,
                    scrobbleTraktPause = false
                )
                onLastSavedTimestampChanged(now)
            }
        }

        checkProgress()
        if (isPlaying) {
            while (true) {
                delay(10_000L)
                checkProgress()
            }
        }
    }

    LaunchedEffect(playWhenReadyForScrobble) {
        val token = activeProfile?.traktAccessToken
        if (playWhenReadyForScrobble) {
            scrobblePauseJob?.cancel()
            scrobblePauseJob = null
        } else if (PlayerScrobbleCoordinator.shouldQueuePause(token, wasPlayWhenReadyForScrobble, hasScrobbledStart, hasScrobbledStop)) {
            scrobblePauseJob?.cancel()
            scrobblePauseJob = scope.launch {
                delay(Constants.Player.PAUSE_SCROBBLE_DELAY_MS)
                if (!latestPlayWhenReadyForScrobble && hasScrobbledStart && !hasScrobbledStop) {
                    val progress = PlayerScrobbleCoordinator.progressPercent(latestCurrentPositionMs(), latestDuration)
                    enqueueDurableTraktScrobble("pause", progress)
                    enqueueDurableSimklScrobble("pause", latestCurrentPositionMs(), latestDuration)
                    isPausedForScrobble = true
                }
            }
        }
        wasPlayWhenReadyForScrobble = playWhenReadyForScrobble
    }

    DisposableEffect(Unit) {
        onDispose {
            scrobblePauseJob?.cancel()
            if (!latestHasScrobbledStop && PlayerScrobbleCoordinator.shouldSaveOnDispose(latestCurrentPositionMs())) {
                val chosenStream = latestCurrentStreams.getOrNull(latestCurrentStreamIndex)
                viewModel.savePlaybackProgress(
                    meta.withCurrentEpisodeArtwork(latestCurrentEpisodeArtwork),
                    latestCurrentPositionMs(),
                    latestDuration,
                    latestCurrentVideoId,
                    latestCurrentStreamIndex,
                    latestCurrentEpisodeMetaLine,
                    chosenStream?.playableUrl,
                    chosenStream?.title,
                    lastBingeGroup = chosenStream?.bingeGroup,
                    scrobbleTraktPause = false
                )
            }
        }
    }

    DisposableEffect(Unit) {
        onDispose {
            scrobblePauseJob?.cancel()
            val token = latestActiveProfile?.traktAccessToken
            if (token != null && hasScrobbledStart && !hasScrobbledStop) {
                val progress = PlayerScrobbleCoordinator.progressPercent(latestCurrentPositionMs(), latestDuration)
                enqueueDurableTraktScrobble("stop", progress)
                enqueueDurableSimklScrobble("stop", latestCurrentPositionMs(), latestDuration)
                if (progress >= (latestActiveProfile?.safeWatchedThresholdPercent ?: 80f)) {
                    viewModel.markWatchedFromPlayback(meta, latestCurrentVideoId, latestCurrentEpisodeMetaLine, latestNextEpisode, latestDuration)
                }
            }
        }
    }
}
