package com.fluxa.app.shared.feature.player

import kotlin.math.abs

/**
 * Lightweight linked inter-sample peak detector for realtime PCM processing.
 * When a look-ahead frame is available, a three-point parabolic reconstruction
 * samples six points around the current sample (a conservative 4x estimate).
 * It is not a resampler and allocates nothing on the audio thread.
 */
object AudioPeakLimiter {
    fun linkedGain(
        current: FloatArray,
        previous: FloatArray,
        ceiling: Float = 0.98f,
        next: FloatArray? = null,
    ): Float {
        if (current.isEmpty() || ceiling <= 0f) return 1f
        var peak = 0f
        current.forEachIndexed { index, sample ->
            if (!sample.isFinite()) return@forEachIndexed
            peak = maxOf(peak, abs(sample))
            val prior = previous.getOrElse(index) { 0f }
            if (prior.isFinite()) {
                peak = maxOf(peak, abs((prior + sample) * 0.5f))
            }
            val following = next?.getOrNull(index)
            if (following != null && following.isFinite() && prior.isFinite()) {
                // Lagrange interpolation through samples at t=-1, 0, +1.
                // Evaluate six quarter-ish points around the current sample;
                // this catches a reconstructed peak that is not present in the
                // discrete samples without introducing a resampling stage.
                QUARTER_POINTS.forEach { t ->
                    val reconstructed =
                        0.5f * t * (t - 1f) * prior +
                            (1f - t * t) * sample +
                            0.5f * t * (t + 1f) * following
                    if (reconstructed.isFinite()) peak = maxOf(peak, abs(reconstructed))
                }
            }
        }
        return if (peak > ceiling) (ceiling / peak).coerceIn(0f, 1f) else 1f
    }

    private val QUARTER_POINTS = floatArrayOf(-0.75f, -0.5f, -0.25f, 0.25f, 0.5f, 0.75f)
}
