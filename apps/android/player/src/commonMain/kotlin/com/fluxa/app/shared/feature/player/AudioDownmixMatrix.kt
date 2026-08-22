package com.fluxa.app.shared.feature.player

import kotlin.math.exp

/** Pure channel matrix used by every PCM fallback path. */
object AudioDownmixMatrix {
    // Media3's AudioTrack channel masks map the supported counts to the
    // conventional orders used below: 3.0 = L,R,C; 4.0 = L,R,BL,BR;
    // 5.1/6.1/7.1 = L,R,C,LFE, surrounds (+ rear center/sides).
    fun mix(samples: FloatArray, targetChannels: Int): FloatArray {
        val normalizedTarget = targetChannels.coerceIn(1, 8)
        val output = FloatArray(normalizedTarget)
        mixInto(samples, normalizedTarget, output)
        return output
    }

    /** Writes into a caller-owned buffer so realtime processors do not allocate per frame. */
    fun mixInto(samples: FloatArray, targetChannels: Int, output: FloatArray) {
        if (output.isEmpty()) return
        // Keep this boundary defensive: a bad route report must never crash
        // the realtime audio thread with an invalid channel count.
        val normalizedTarget = targetChannels.coerceIn(1, 8).coerceAtMost(output.size)
        when (normalizedTarget) {
            1 -> output[0] = mono(samples)
            2 -> stereoInto(samples, output)
            3 -> threePointZeroInto(samples, output)
            4 -> fourPointZeroInto(samples, output)
            5 -> fivePointZeroInto(samples, output)
            6 -> fivePointOneInto(samples, output)
            7 -> sixPointOneInto(samples, output)
            else -> sevenPointOneInto(samples, output)
        }
    }

    /** Soft-knee safety limiter that avoids hard digital clipping. */
    fun softLimit(value: Float): Float {
        if (!value.isFinite()) return 0f
        val sign = if (value < 0f) -1f else 1f
        val magnitude = kotlin.math.abs(value)
        if (magnitude <= 0.88f) return value
        val excess = magnitude - 0.88f
        val compressed = 0.88f + 0.10f * (1f - exp(-excess / 0.10f))
        return (compressed * sign).coerceIn(-0.98f, 0.98f)
    }

    private fun stereoInto(samples: FloatArray, output: FloatArray) {
        if (samples.size <= 2) {
            output[0] = samples.getOrElse(0) { 0f }
            output[1] = samples.getOrElse(1) { 0f }
            return
        }
        var left = samples[0]
        var right = samples[1]
        when (samples.size) {
            3, 5, 6, 7, 8 -> {
                val center = samples[2] * 0.70710677f
                left += center
                right += center
            }
        }
        when (samples.size) {
            4 -> {
                left += samples[2] * 0.70710677f
                right += samples[3] * 0.70710677f
            }
            5 -> {
                left += samples[3] * 0.70710677f
                right += samples[4] * 0.70710677f
            }
            // 5.1: FL, FR, FC, LFE, BL, BR.
            6 -> {
                left += samples[4] * 0.70710677f
                right += samples[5] * 0.70710677f
                left += samples[3] * 0.5f
                right += samples[3] * 0.5f
            }
            // 6.1: FL, FR, FC, LFE, BC, SL, SR.
            7 -> {
                left += samples[5] * 0.70710677f
                right += samples[6] * 0.70710677f
                left += samples[3] * 0.5f + samples[4] * 0.5f
                right += samples[3] * 0.5f + samples[4] * 0.5f
            }
            // 7.1: FL, FR, FC, LFE, BL, BR, SL, SR.
            8 -> {
                left += (samples[4] + samples[6]) * 0.70710677f
                right += (samples[5] + samples[7]) * 0.70710677f
                left += samples[3] * 0.5f
                right += samples[3] * 0.5f
            }
        }
        output[0] = left
        output[1] = right
    }

    private fun mono(samples: FloatArray): Float {
        if (samples.isEmpty()) return 0f
        if (samples.size == 1) return samples[0]
        var value = (samples[0] + samples.getOrElse(1) { 0f }) * 0.5f
        if (samples.size >= 3 && samples.size != 4) value += samples[2] * 0.70710677f
        when (samples.size) {
            4 -> value += (samples[2] + samples[3]) * 0.35355338f
            5 -> value += (samples[3] + samples[4]) * 0.35355338f
            6 -> {
                value += (samples[4] + samples[5]) * 0.35355338f
                value += samples[3] * 0.5f
            }
            7 -> {
                value += (samples[5] + samples[6]) * 0.35355338f
                value += samples[3] * 0.5f + samples[4] * 0.5f
            }
            8 -> {
                value += (samples[4] + samples[5] + samples[6] + samples[7]) * 0.35355338f
                value += samples[3] * 0.5f
            }
        }
        return value
    }

    private fun fivePointOneInto(samples: FloatArray, output: FloatArray) {
        output.fill(0f, 0, 6)
        if (samples.isEmpty()) return
        output[0] = samples[0]
        if (samples.size > 1) output[1] = samples[1]
        if (samples.size > 2) output[2] = samples[2]

        when (samples.size) {
            // 4.0: FL, FR, BL, BR (no center/LFE).
            4 -> {
                output[4] = samples[2]
                output[5] = samples[3]
            }
            // 5.0: FL, FR, FC, BL, BR (insert an empty LFE slot).
            5 -> {
                output[4] = samples[3]
                output[5] = samples[4]
            }
            // 5.1: FL, FR, FC, LFE, BL, BR.
            6 -> for (index in 3..5) output[index] = samples[index]
            // 6.1: FL, FR, FC, LFE, BC, SL, SR.
            // Fold the back-center equally into the two 5.1 surrounds.
            7 -> {
                output[3] = samples[3]
                output[4] = samples[5] + samples[4] * 0.5f
                output[5] = samples[6] + samples[4] * 0.5f
            }
            // 7.1: FL, FR, FC, LFE, BL, BR, SL, SR.
            // Keep the side/rear energy on its corresponding side.
            else -> {
                output[3] = samples.getOrElse(3) { 0f }
                output[4] = (samples.getOrElse(4) { 0f } + samples.getOrElse(6) { 0f }) * 0.5f
                output[5] = (samples.getOrElse(5) { 0f } + samples.getOrElse(7) { 0f }) * 0.5f
            }
        }
    }

    /** 6.1 output: FL, FR, FC, LFE, BC, SL, SR. */
    private fun sixPointOneInto(samples: FloatArray, output: FloatArray) {
        output.fill(0f, 0, 7)
        if (samples.isEmpty()) return
        output[0] = samples.getOrElse(0) { 0f }
        output[1] = samples.getOrElse(1) { 0f }
        output[2] = samples.getOrElse(2) { 0f }
        output[3] = samples.getOrElse(3) { 0f }
        when (samples.size) {
            4 -> {
                output[5] = samples[2]
                output[6] = samples[3]
            }
            5 -> {
                output[5] = samples[3]
                output[6] = samples[4]
            }
            6 -> {
                output[5] = samples[4]
                output[6] = samples[5]
            }
            7 -> for (index in 4..6) output[index] = samples[index]
            8 -> {
                // 7.1 has rear left/right followed by side left/right;
                // synthesize the 6.1 back-center without discarding energy.
                output[4] = (samples[4] + samples[5]) * 0.5f
                output[5] = samples[6]
                output[6] = samples[7]
            }
            else -> {
                output[4] = samples.getOrElse(4) { 0f }
                output[5] = samples.getOrElse(5) { 0f }
                output[6] = samples.getOrElse(6) { 0f }
            }
        }
    }

    /** 7.1 output: FL, FR, FC, LFE, BL, BR, SL, SR. */
    private fun sevenPointOneInto(samples: FloatArray, output: FloatArray) {
        output.fill(0f, 0, 8)
        if (samples.isEmpty()) return
        when (samples.size) {
            3 -> {
                output[0] = samples[0]
                output[1] = samples[1]
                output[2] = samples[2]
            }
            4 -> {
                output[0] = samples[0]
                output[1] = samples[1]
                output[4] = samples[2]
                output[5] = samples[3]
            }
            5 -> {
                output[0] = samples[0]
                output[1] = samples[1]
                output[2] = samples[2]
                output[4] = samples[3]
                output[5] = samples[4]
            }
            6 -> for (index in 0..5) output[index] = samples[index]
            7 -> {
                for (index in 0..3) output[index] = samples[index]
                output[4] = samples[4] * 0.5f
                output[5] = samples[4] * 0.5f
                output[6] = samples[5]
                output[7] = samples[6]
            }
            else -> for (index in 0..7) output[index] = samples[index]
        }
        if (samples.size > 8) {
            // 7.1.x immersive beds append height channels. Fold their energy
            // into the nearest bed channels when the sink cannot represent
            // the height layout, retaining the 7.1 bed and avoiding silence.
            val extraGain = 0.5f
            output[0] += samples[8] * extraGain
            output[1] += samples.getOrElse(9) { 0f } * extraGain
            if (samples.size > 10) {
                output[6] += samples[10] * extraGain
                output[7] += samples.getOrElse(11) { 0f } * extraGain
            }
        }
    }

    private fun threePointZeroInto(samples: FloatArray, output: FloatArray) {
        output.fill(0f, 0, 3)
        if (samples.isEmpty()) return
        output[0] = samples[0]
        if (samples.size > 1) output[1] = samples[1]
        when (samples.size) {
            3 -> output[2] = samples[2]
            // 4.0: FL, FR, BL, BR. Fold the rear pair into the front pair;
            // dropping them would make a three-channel sink lose surround data.
            4 -> {
                output[0] += samples[2] * 0.70710677f
                output[1] += samples[3] * 0.70710677f
            }
            // 5.0: FL, FR, FC, BL, BR.
            5 -> {
                output[2] = samples[2]
                output[0] += samples[3] * 0.70710677f
                output[1] += samples[4] * 0.70710677f
            }
            // 5.1: FL, FR, FC, LFE, BL, BR.
            6 -> {
                output[2] = samples[2]
                output[0] += samples[3] * 0.5f + samples[4] * 0.70710677f
                output[1] += samples[3] * 0.5f + samples[5] * 0.70710677f
            }
            // 6.1: FL, FR, FC, LFE, BC, SL, SR.
            7 -> {
                output[2] = samples[2]
                output[0] += samples[3] * 0.5f + samples[4] * 0.5f + samples[5] * 0.70710677f
                output[1] += samples[3] * 0.5f + samples[4] * 0.5f + samples[6] * 0.70710677f
            }
            // 7.1: FL, FR, FC, LFE, BL, BR, SL, SR.
            else -> {
                output[2] = samples[2]
                output[0] += samples[3] * 0.5f + (samples[4] + samples[6]) * 0.70710677f
                output[1] += samples[3] * 0.5f + (samples[5] + samples[7]) * 0.70710677f
            }
        }
    }

    /** Quad output: FL, FR, BL, BR. */
    private fun fourPointZeroInto(samples: FloatArray, output: FloatArray) {
        output.fill(0f, 0, 4)
        if (samples.isEmpty()) return
        output[0] = samples[0]
        if (samples.size > 1) output[1] = samples[1]
        when (samples.size) {
            3 -> {
                // 3.0 has a center where quad has a rear pair; preserve the
                // dialogue energy in the front channels instead of dropping it.
                output[0] += samples[2] * 0.70710677f
                output[1] += samples[2] * 0.70710677f
            }
            4 -> {
                output[2] = samples[2]
                output[3] = samples[3]
            }
            5 -> {
                output[0] += samples[2] * 0.70710677f
                output[1] += samples[2] * 0.70710677f
                output[2] = samples[3]
                output[3] = samples[4]
            }
            6 -> {
                output[0] += samples[2] * 0.70710677f + samples[3] * 0.5f
                output[1] += samples[2] * 0.70710677f + samples[3] * 0.5f
                output[2] = samples[4]
                output[3] = samples[5]
            }
            7 -> {
                output[0] += samples[2] * 0.70710677f + samples[3] * 0.5f + samples[4] * 0.5f
                output[1] += samples[2] * 0.70710677f + samples[3] * 0.5f + samples[4] * 0.5f
                output[2] = samples[5]
                output[3] = samples[6]
            }
            else -> {
                output[0] += samples[2] * 0.70710677f + samples[3] * 0.5f
                output[1] += samples[2] * 0.70710677f + samples[3] * 0.5f
                output[2] = (samples[4] + samples[6]) * 0.5f
                output[3] = (samples[5] + samples[7]) * 0.5f
            }
        }
    }

    /** 5.0 output: FL, FR, FC, BL, BR. LFE is folded into the fronts instead of discarded. */
    private fun fivePointZeroInto(samples: FloatArray, output: FloatArray) {
        output.fill(0f, 0, 5)
        if (samples.isEmpty()) return
        output[0] = samples[0]
        if (samples.size > 1) output[1] = samples[1]
        if (samples.size > 2 && samples.size != 4) output[2] = samples[2]
        when (samples.size) {
            4 -> {
                output[3] = samples[2]
                output[4] = samples[3]
            }
            5 -> {
                output[3] = samples[3]
                output[4] = samples[4]
            }
            6 -> {
                output[0] += samples[3] * 0.5f
                output[1] += samples[3] * 0.5f
                output[3] = samples[4]
                output[4] = samples[5]
            }
            7 -> {
                output[0] += samples[3] * 0.5f
                output[1] += samples[3] * 0.5f
                output[3] = samples[5] + samples[4] * 0.5f
                output[4] = samples[6] + samples[4] * 0.5f
            }
            else -> {
                output[0] += samples.getOrElse(3) { 0f } * 0.5f
                output[1] += samples.getOrElse(3) { 0f } * 0.5f
                output[3] = (samples[4] + samples[6]) * 0.5f
                output[4] = (samples[5] + samples[7]) * 0.5f
            }
        }
    }
}
