package com.fluxa.app.shared.feature.player

/**
 * Applies the channel-aware dialogue lift used by Night mode.
 *
 * Channel order follows the layouts used by the PCM matrix:
 * FL, FR, FC, LFE, surrounds. Quad is FL, FR, BL, BR, so its third channel
 * is not a center channel and must never be boosted as dialogue.
 */
object AudioDialogueBoost {
    fun applyInPlace(
        samples: FloatArray,
        channels: Int,
        centerGain: Float = 1.25f,
        stereoMidGain: Float = 1.12f,
    ) {
        if (channels <= 0 || samples.size < channels) return
        when (channels) {
            3, 5, 6, 7, 8 -> samples[2] *= centerGain
            2 -> {
                val mid = (samples[0] + samples[1]) * 0.5f * stereoMidGain
                val side = (samples[0] - samples[1]) * 0.5f
                samples[0] = mid + side
                samples[1] = mid - side
            }
        }
    }
}
