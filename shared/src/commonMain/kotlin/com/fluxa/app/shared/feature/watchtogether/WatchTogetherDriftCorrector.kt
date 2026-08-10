package com.fluxa.app.shared.feature.watchtogether

import kotlin.math.abs

internal sealed interface WatchTogetherCorrection {
    data object None : WatchTogetherCorrection
    data class Seek(val positionMs: Long) : WatchTogetherCorrection
    data class Speed(val value: Float) : WatchTogetherCorrection
    data object ResetSpeed : WatchTogetherCorrection
}

/** Pure drift policy: easy to test and identical on Android, Desktop and Apple. */
internal object WatchTogetherDriftCorrector {
    const val NORMAL_SPEED = 1f
    private const val CATCH_UP_SPEED = 1.03f
    private const val SLOW_DOWN_SPEED = 0.97f
    private const val SOFT_DRIFT_MS = 250L
    private const val HARD_SEEK_DRIFT_MS = 1_000L

    fun correction(
        localPositionMs: Long,
        expectedPositionMs: Long,
        hostPlaying: Boolean,
        speedCorrectionActive: Boolean,
    ): WatchTogetherCorrection {
        val drift = expectedPositionMs - localPositionMs
        return when {
            abs(drift) > HARD_SEEK_DRIFT_MS -> WatchTogetherCorrection.Seek(expectedPositionMs.coerceAtLeast(0L))
            hostPlaying && abs(drift) > SOFT_DRIFT_MS -> WatchTogetherCorrection.Speed(
                if (drift > 0) CATCH_UP_SPEED else SLOW_DOWN_SPEED
            )
            speedCorrectionActive -> WatchTogetherCorrection.ResetSpeed
            else -> WatchTogetherCorrection.None
        }
    }
}
