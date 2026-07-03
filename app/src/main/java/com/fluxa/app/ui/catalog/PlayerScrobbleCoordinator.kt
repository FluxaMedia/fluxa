package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import com.fluxa.app.core.rust.FluxaCoreNative

object PlayerScrobbleCoordinator {
    fun progressPercent(positionMs: Long, durationMs: Long): Float {
        return FluxaCoreNative.playerProgressPercent(positionMs, durationMs)
    }

    fun shouldSendStart(
        token: String?,
        isPlaying: Boolean,
        hasScrobbledStart: Boolean,
        progress: Float
    ): Boolean {
        return FluxaCoreNative.playerShouldSendScrobbleStart(token, isPlaying, hasScrobbledStart, progress)
    }

    fun shouldMarkStopped(hasScrobbledStop: Boolean, progress: Float): Boolean {
        return FluxaCoreNative.playerShouldMarkScrobbleStopped(hasScrobbledStop, progress)
    }

    fun shouldQueuePause(
        token: String?,
        wasPlayWhenReady: Boolean,
        hasScrobbledStart: Boolean,
        hasScrobbledStop: Boolean
    ): Boolean {
        return FluxaCoreNative.playerShouldQueueScrobblePause(
            token,
            wasPlayWhenReady,
            hasScrobbledStart,
            hasScrobbledStop
        )
    }

    fun shouldEnqueueDurable(action: String, token: String?, progress: Float): Boolean {
        return FluxaCoreNative.playerShouldEnqueueDurableScrobble(action, token, progress)
    }

    fun shouldSavePeriodicProgress(isPlaying: Boolean, nowMs: Long, lastSavedAtMs: Long): Boolean {
        return FluxaCoreNative.playerShouldSavePeriodicProgress(isPlaying, nowMs, lastSavedAtMs)
    }

    fun shouldSaveOnDispose(positionMs: Long): Boolean {
        return FluxaCoreNative.playerShouldSaveOnDispose(positionMs)
    }
}
