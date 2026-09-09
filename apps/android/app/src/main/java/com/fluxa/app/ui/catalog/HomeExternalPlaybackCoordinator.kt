package com.fluxa.app.ui.catalog

import android.content.Context
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeWatchedThresholdPercent
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.domain.playback.PlaybackSyncCoordinator
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

internal class HomeExternalPlaybackCoordinator(
    private val scope: CoroutineScope,
    private val context: Context,
    private val activeProfile: () -> UserProfile?,
    private val playbackSyncCoordinator: PlaybackSyncCoordinator,
    private val saveProgress: (Meta, Long, Long, String?, Int, String?, String?, String?) -> Unit,
    private val markWatched: (Meta, String?, String?, Long) -> Unit
) {
    private var trackingJob: Job? = null
    private var session: Session? = null

    fun start(
        meta: Meta,
        videoId: String?,
        initialPositionMs: Long,
        initialDurationMs: Long,
        streamIndex: Int,
        episodeName: String?,
        streamUrl: String?,
        streamTitle: String?,
        targetPackage: String?
    ) {
        val profile = activeProfile() ?: return
        trackingJob?.cancel()
        val next = Session(
            profile = profile,
            meta = meta,
            videoId = videoId,
            streamIndex = streamIndex,
            episodeName = episodeName,
            streamUrl = streamUrl,
            streamTitle = streamTitle,
            positionMs = initialPositionMs.coerceAtLeast(0L),
            durationMs = initialDurationMs.coerceAtLeast(0L)
        )
        session = next
        trackingJob = scope.launch {
            AndroidExternalPlaybackTracker.monitor(
                context = context,
                targetPackage = targetPackage,
                expectedTitle = episodeName ?: meta.name
            ) { sample -> handleSample(next, sample) }
        }
    }

    fun finish(returnedPositionMs: Long?, returnedDurationMs: Long?) {
        val current = session ?: return
        trackingJob?.cancel()
        trackingJob = null
        returnedPositionMs?.takeIf { it >= 0L }?.let { current.positionMs = it }
        returnedDurationMs?.takeIf { it > 0L }?.let { current.durationMs = it }
        scope.launch { finishSession(current) }
    }

    fun hasMediaSessionAccess(): Boolean = AndroidExternalPlaybackTracker.hasMediaSessionAccess(context)

    private suspend fun handleSample(current: Session, sample: ExternalPlaybackSample) {
        if (current.finished || session !== current) return
        current.positionMs = sample.positionMs.coerceAtLeast(0L)
        if (sample.durationMs > 0L) current.durationMs = sample.durationMs
        val duration = current.durationMs
        val position = current.positionMs
        when (sample.state) {
            ExternalPlaybackState.PLAYING -> {
                if (duration > 0L) {
                    if (!current.traktStarted || current.wasPaused) {
                        current.traktStarted = playbackSyncCoordinator.scheduleTraktScrobble(
                            current.profile, current.meta, current.videoId, position, duration, "start"
                        ) || current.traktStarted
                    }
                    if (!current.simklStarted || current.wasPaused) {
                        current.simklStarted = playbackSyncCoordinator.scheduleSimklScrobble(
                            current.profile, current.meta, current.videoId, position, duration, "start"
                        ) || current.simklStarted
                    }
                }
                current.wasPaused = false
                val now = System.currentTimeMillis()
                if (now - current.lastProgressSavedAt >= 10_000L) {
                    save(current)
                    current.lastProgressSavedAt = now
                }
            }
            ExternalPlaybackState.PAUSED -> {
                current.wasPaused = true
                save(current)
                if (duration > 0L) {
                    if (current.traktStarted) playbackSyncCoordinator.scheduleTraktScrobble(
                        current.profile, current.meta, current.videoId, position, duration, "pause"
                    )
                    if (current.simklStarted) playbackSyncCoordinator.scheduleSimklScrobble(
                        current.profile, current.meta, current.videoId, position, duration, "pause"
                    )
                }
            }
            ExternalPlaybackState.STOPPED -> finishSession(current)
        }
    }

    private fun save(current: Session) {
        saveProgress(
            current.meta,
            current.positionMs,
            current.durationMs,
            current.videoId,
            current.streamIndex,
            current.episodeName,
            current.streamUrl,
            current.streamTitle
        )
    }

    private suspend fun finishSession(current: Session) {
        if (current.finished || session !== current) return
        current.finished = true
        session = null
        save(current)
        val duration = current.durationMs
        val position = current.positionMs
        if (duration <= 0L) return
        if (current.traktStarted) playbackSyncCoordinator.scheduleTraktScrobble(
            current.profile, current.meta, current.videoId, position, duration, "stop"
        )
        if (current.simklStarted) playbackSyncCoordinator.scheduleSimklScrobble(
            current.profile, current.meta, current.videoId, position, duration, "stop"
        )
        val progress = FluxaCoreNative.playerProgressPercent(position, duration).toDouble()
        if (progress >= current.profile.safeWatchedThresholdPercent.toDouble()) {
            markWatched(current.meta, current.videoId, current.episodeName, duration)
        }
    }

    private data class Session(
        val profile: UserProfile,
        val meta: Meta,
        val videoId: String?,
        val streamIndex: Int,
        val episodeName: String?,
        val streamUrl: String?,
        val streamTitle: String?,
        var positionMs: Long,
        var durationMs: Long,
        var lastProgressSavedAt: Long = 0L,
        var traktStarted: Boolean = false,
        var simklStarted: Boolean = false,
        var wasPaused: Boolean = false,
        var finished: Boolean = false
    )
}
