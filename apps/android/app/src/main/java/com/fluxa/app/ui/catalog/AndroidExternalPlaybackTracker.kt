package com.fluxa.app.ui.catalog

import android.content.ComponentName
import android.content.Context
import android.media.MediaMetadata
import android.media.session.MediaController
import android.media.session.MediaSessionManager
import android.media.session.PlaybackState
import android.os.SystemClock
import android.service.notification.NotificationListenerService
import kotlinx.coroutines.delay

/**
 * Notification-listener bridge used only to obtain MediaSession state from an external player.
 * Android does not expose another generic API for reading another app's playback position.
 */
class FluxaExternalMediaSessionListener : NotificationListenerService()

internal enum class ExternalPlaybackState {
    PLAYING,
    PAUSED,
    STOPPED,
}

internal data class ExternalPlaybackSample(
    val packageName: String,
    val state: ExternalPlaybackState,
    val positionMs: Long,
    val durationMs: Long,
)

internal object AndroidExternalPlaybackTracker {
    fun hasMediaSessionAccess(context: Context): Boolean = runCatching {
        mediaSessionManager(context).getActiveSessions(listenerComponent(context))
        true
    }.getOrDefault(false)

    suspend fun monitor(
        context: Context,
        targetPackage: String?,
        expectedTitle: String?,
        onSample: suspend (ExternalPlaybackSample) -> Unit,
    ) {
        val manager = mediaSessionManager(context)
        val listener = listenerComponent(context)
        var selectedPackage = targetPackage?.trim()?.takeIf(String::isNotEmpty)
        var sawPlayback = false
        var idlePollsAfterPlayback = 0
        var lastEmittedState: ExternalPlaybackState? = null
        var lastEmittedPosition = -1L
        var lastPeriodicEmitAt = 0L

        while (true) {
            val controllers = runCatching { manager.getActiveSessions(listener) }.getOrElse { return }
                .filter { it.packageName != context.packageName }
            val controller = selectController(
                controllers = controllers,
                targetPackage = selectedPackage,
                expectedTitle = expectedTitle,
            )

            if (controller == null) {
                if (sawPlayback) {
                    idlePollsAfterPlayback += 1
                    if (idlePollsAfterPlayback >= 4) return
                }
                delay(1_000L)
                continue
            }

            selectedPackage = controller.packageName
            idlePollsAfterPlayback = 0
            val sample = controller.toSample()
            if (sample == null) {
                delay(1_000L)
                continue
            }
            if (sample.state == ExternalPlaybackState.PLAYING || sample.state == ExternalPlaybackState.PAUSED) {
                sawPlayback = true
            } else if (!sawPlayback) {
                // Ignore a stale stopped MediaSession from the target app. Many players keep a
                // stopped session registered before the newly launched item actually starts.
                delay(1_000L)
                continue
            }

            val now = SystemClock.elapsedRealtime()
            val stateChanged = sample.state != lastEmittedState
            val positionMoved = kotlin.math.abs(sample.positionMs - lastEmittedPosition) >= 5_000L
            val periodic = now - lastPeriodicEmitAt >= 10_000L
            if (stateChanged || positionMoved && periodic || (sample.state == ExternalPlaybackState.STOPPED && sawPlayback)) {
                onSample(sample)
                lastEmittedState = sample.state
                lastEmittedPosition = sample.positionMs
                lastPeriodicEmitAt = now
            }

            if (sample.state == ExternalPlaybackState.STOPPED && sawPlayback) return
            delay(1_000L)
        }
    }

    private fun selectController(
        controllers: List<MediaController>,
        targetPackage: String?,
        expectedTitle: String?,
    ): MediaController? {
        if (!targetPackage.isNullOrBlank()) {
            controllers.firstOrNull { it.packageName == targetPackage }?.let { return it }
        }
        val expected = expectedTitle?.trim()?.takeIf(String::isNotEmpty)
        if (expected != null) {
            controllers.firstOrNull { controller ->
                val title = controller.metadata?.getText(MediaMetadata.METADATA_KEY_TITLE)?.toString()
                !title.isNullOrBlank() && (
                    title.equals(expected, ignoreCase = true) ||
                        expected.contains(title, ignoreCase = true) ||
                        title.contains(expected, ignoreCase = true)
                    )
            }?.let { return it }
        }
        return controllers.firstOrNull { controller ->
            controller.playbackState?.state in setOf(
                PlaybackState.STATE_PLAYING,
                PlaybackState.STATE_BUFFERING,
                PlaybackState.STATE_PAUSED,
            )
        }
    }

    private fun MediaController.toSample(): ExternalPlaybackSample? {
        val playback = playbackState ?: return null
        val mappedState = when (playback.state) {
            PlaybackState.STATE_PLAYING,
            PlaybackState.STATE_BUFFERING,
            PlaybackState.STATE_CONNECTING -> ExternalPlaybackState.PLAYING
            PlaybackState.STATE_PAUSED -> ExternalPlaybackState.PAUSED
            PlaybackState.STATE_STOPPED,
            PlaybackState.STATE_NONE,
            PlaybackState.STATE_ERROR -> ExternalPlaybackState.STOPPED
            else -> return null
        }
        val rawPosition = playback.position.coerceAtLeast(0L)
        val position = if (
            mappedState == ExternalPlaybackState.PLAYING &&
            playback.lastPositionUpdateTime > 0L &&
            playback.playbackSpeed != 0f
        ) {
            val elapsed = (SystemClock.elapsedRealtime() - playback.lastPositionUpdateTime).coerceAtLeast(0L)
            (rawPosition + elapsed * playback.playbackSpeed).toLong().coerceAtLeast(0L)
        } else {
            rawPosition
        }
        val duration = metadata?.getLong(MediaMetadata.METADATA_KEY_DURATION)?.coerceAtLeast(0L) ?: 0L
        return ExternalPlaybackSample(
            packageName = packageName,
            state = mappedState,
            positionMs = position,
            durationMs = duration,
        )
    }

    private fun mediaSessionManager(context: Context): MediaSessionManager =
        context.getSystemService(Context.MEDIA_SESSION_SERVICE) as MediaSessionManager

    private fun listenerComponent(context: Context): ComponentName =
        ComponentName(context, FluxaExternalMediaSessionListener::class.java)
}
