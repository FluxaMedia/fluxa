package com.fluxa.app.ui.catalog

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
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeWatchedThresholdPercent
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.remote.withCurrentEpisodeArtwork
import com.fluxa.app.data.repository.TraktIntegration
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

@Composable
internal fun PlayerPlaybackSideEffects(
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
    onLastSavedTimestampChanged: (Long) -> Unit,
    isVideoRendered: Boolean
) {
    val scope = rememberCoroutineScope()
    var hasScrobbledStartTrakt by remember(meta.id, currentVideoId) { mutableStateOf(false) }
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
    val latestIsVideoRendered by rememberUpdatedState(isVideoRendered)

    fun enqueueDurableTraktScrobble(action: String, progress: Float) {
        val profile = latestActiveProfile ?: return
        if (!PlayerScrobbleCoordinator.shouldEnqueueDurable(action, profile.traktAccessToken, progress)) return
        viewModel.enqueueDurableTraktScrobble(
            profile = profile,
            mediaType = meta.type,
            mediaId = TraktIntegration.scrobbleMediaId(meta.id, latestCurrentVideoId, meta.type),
            progress = progress,
            action = action,
        )
    }

    fun enqueueDurableSimklScrobble(action: String, positionMs: Long, durationMs: Long) {
        val profile = latestActiveProfile ?: return
        if (profile.simklAccessToken.isNullOrBlank() || durationMs <= 0L) return
        viewModel.enqueueDurableSimklScrobble(
            profile = profile,
            meta = meta,
            videoId = latestCurrentVideoId,
            action = action,
            positionMs = positionMs,
            durationMs = durationMs,
        )
    }

    LaunchedEffect(isPlaying, duration) {
        if (duration <= 0) return@LaunchedEffect

        suspend fun checkProgress() {
            val pos = currentPositionMs()
            val progress = PlayerScrobbleCoordinator.progressPercent(pos, duration)
            val traktToken = activeProfile?.traktAccessToken
            val simklToken = activeProfile?.simklAccessToken

            if (PlayerScrobbleCoordinator.shouldSendStart(traktToken, isPlaying, hasScrobbledStartTrakt, progress)) {
                hasScrobbledStartTrakt = true
                isPausedForScrobble = false
                viewModel.scrobblePlayback(
                    traktToken.orEmpty(),
                    meta.type,
                    TraktIntegration.scrobbleMediaId(meta.id, currentVideoId, meta.type),
                    progress,
                    "start"
                )
            }

            if (PlayerScrobbleCoordinator.shouldSendStart(simklToken, isPlaying, hasScrobbledStartSimkl, progress)) {
                hasScrobbledStartSimkl = true
                enqueueDurableSimklScrobble("start", pos, duration)
            }

            if (isPlaying && isPausedForScrobble) isPausedForScrobble = false

            if (!hasScrobbledStop && PlayerScrobbleCoordinator.closeAction(pos, duration) == "stop") {
                if (hasScrobbledStartTrakt) enqueueDurableTraktScrobble("stop", progress)
                if (hasScrobbledStartSimkl) enqueueDurableSimklScrobble("stop", pos, duration)
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
        if (playWhenReadyForScrobble) {
            scrobblePauseJob?.cancel()
            scrobblePauseJob = null
        } else {
            val queueTrakt = PlayerScrobbleCoordinator.shouldQueuePause(
                activeProfile?.traktAccessToken,
                wasPlayWhenReadyForScrobble,
                hasScrobbledStartTrakt,
                hasScrobbledStop
            )
            val queueSimkl = PlayerScrobbleCoordinator.shouldQueuePause(
                activeProfile?.simklAccessToken,
                wasPlayWhenReadyForScrobble,
                hasScrobbledStartSimkl,
                hasScrobbledStop
            )
            if (queueTrakt || queueSimkl) {
                scrobblePauseJob?.cancel()
                scrobblePauseJob = scope.launch {
                    delay(Constants.Player.PAUSE_SCROBBLE_DELAY_MS)
                    if (!latestPlayWhenReadyForScrobble && !hasScrobbledStop) {
                        val position = latestCurrentPositionMs()
                        val progress = PlayerScrobbleCoordinator.progressPercent(position, latestDuration)
                        if (queueTrakt && hasScrobbledStartTrakt) enqueueDurableTraktScrobble("pause", progress)
                        if (queueSimkl && hasScrobbledStartSimkl) enqueueDurableSimklScrobble("pause", position, latestDuration)
                        isPausedForScrobble = true
                    }
                }
            }
        }
        wasPlayWhenReadyForScrobble = playWhenReadyForScrobble
    }

    DisposableEffect(Unit) {
        onDispose {
            scrobblePauseJob?.cancel()
            if (!latestHasScrobbledStop && latestIsVideoRendered && PlayerScrobbleCoordinator.shouldSaveOnDispose(latestCurrentPositionMs())) {
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
            if (!hasScrobbledStop) {
                val position = latestCurrentPositionMs()
                val progress = PlayerScrobbleCoordinator.progressPercent(position, latestDuration)
                val traktStop = latestActiveProfile?.traktAccessToken != null && hasScrobbledStartTrakt
                val simklStop = latestActiveProfile?.simklAccessToken != null && hasScrobbledStartSimkl
                if (traktStop) enqueueDurableTraktScrobble("stop", progress)
                if (simklStop) enqueueDurableSimklScrobble("stop", position, latestDuration)
                if (progress >= (latestActiveProfile?.safeWatchedThresholdPercent ?: 80f)) {
                    viewModel.markWatchedFromPlayback(meta, latestCurrentVideoId, latestCurrentEpisodeMetaLine, latestNextEpisode, latestDuration)
                }
            }
        }
    }
}
