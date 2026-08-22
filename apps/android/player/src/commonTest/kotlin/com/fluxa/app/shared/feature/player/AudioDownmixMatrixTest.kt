package com.fluxa.app.shared.feature.player

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class AudioDownmixMatrixTest {
    @Test
    fun pcmLayoutPreferencePreservesTheWidestSupportedSurroundLayout() {
        assertEquals(
            "7.1,6.1,5.1,5.0,quad,3.0,stereo",
            AudioChannelLayoutPolicy.preferredLayouts(8),
        )
        assertEquals(
            "6.1,5.1,5.0,quad,3.0,stereo",
            AudioChannelLayoutPolicy.preferredLayouts(7),
        )
        assertEquals("5.1,5.0,quad,3.0,stereo", AudioChannelLayoutPolicy.preferredLayouts(6))
    }

    @Test
    fun pcmLayoutPreferenceFallsBackWithoutForcingSurroundOnStereoRoutes() {
        assertEquals("quad,3.0,stereo", AudioChannelLayoutPolicy.preferredLayouts(4))
        assertEquals("stereo", AudioChannelLayoutPolicy.preferredLayouts(2))
        assertEquals("stereo", AudioChannelLayoutPolicy.preferredLayouts(0))
    }

    @Test
    fun quadDoesNotTreatRearLeftAsCenter() {
        val stereo = AudioDownmixMatrix.mix(floatArrayOf(0f, 0f, 1f, 0f), 2)

        assertTrue(stereo[0] > 0.7f)
        assertEquals(0f, stereo[1], absoluteTolerance = 0.0001f)
    }

    @Test
    fun threePointZeroUsesTheThirdChannelAsCenter() {
        val stereo = AudioDownmixMatrix.mix(floatArrayOf(0f, 0f, 1f), 2)

        assertEquals(0.70710677f, stereo[0], absoluteTolerance = 0.0001f)
        assertEquals(0.70710677f, stereo[1], absoluteTolerance = 0.0001f)
    }

    @Test
    fun threePointTargetFoldsQuadRearsInsteadOfDroppingThem() {
        val output = AudioDownmixMatrix.mix(floatArrayOf(0f, 0f, 1f, 0f), 3)

        assertTrue(output[0] > 0.7f)
        assertEquals(0f, output[1], absoluteTolerance = 0.0001f)
        assertEquals(0f, output[2], absoluteTolerance = 0.0001f)
    }

    @Test
    fun quadTargetFoldsThreePointCenterIntoFrontChannels() {
        val output = AudioDownmixMatrix.mix(floatArrayOf(0f, 0f, 1f), 4)

        assertEquals(0.70710677f, output[0], absoluteTolerance = 0.0001f)
        assertEquals(0.70710677f, output[1], absoluteTolerance = 0.0001f)
        assertEquals(0f, output[2], absoluteTolerance = 0.0001f)
        assertEquals(0f, output[3], absoluteTolerance = 0.0001f)
    }

    @Test
    fun sevenPointOneKeepsLeftAndRightSurroundsOnTheirSide() {
        val stereo = AudioDownmixMatrix.mix(
            floatArrayOf(0f, 0f, 0f, 0f, 0f, 0f, 1f, 0f),
            2
        )

        assertTrue(stereo[0] > 0.7f)
        assertEquals(0f, stereo[1], absoluteTolerance = 0.0001f)
    }

    @Test
    fun fivePointOneFallbackPreservesSixChannels() {
        val output = AudioDownmixMatrix.mix(floatArrayOf(1f, 2f, 3f, 4f, 5f, 6f, 7f, 8f), 6)

        assertEquals(6, output.size)
        assertEquals(6f, output[4])
        assertEquals(7f, output[5])
    }

    @Test
    fun sixPointOneUsesSideSurroundsAndBackCenterCorrectly() {
        val output = AudioDownmixMatrix.mix(floatArrayOf(0f, 0f, 0f, 0f, 1f, 2f, 4f), 6)

        assertEquals(2.5f, output[4], absoluteTolerance = 0.0001f)
        assertEquals(4.5f, output[5], absoluteTolerance = 0.0001f)
    }

    @Test
    fun sixPointOneStereoDownmixUsesSideChannelsOnTheirSides() {
        val stereo = AudioDownmixMatrix.mix(floatArrayOf(0f, 0f, 0f, 0f, 1f, 2f, 4f), 2)

        assertEquals(1.9142135f, stereo[0], absoluteTolerance = 0.0001f)
        assertEquals(3.328427f, stereo[1], absoluteTolerance = 0.0001f)
    }

    @Test
    fun fivePointZeroLeavesLfeEmptyWhenExpandingToFivePointOne() {
        val output = AudioDownmixMatrix.mix(floatArrayOf(1f, 2f, 3f, 4f, 5f), 6)

        assertEquals(0f, output[3])
        assertEquals(4f, output[4])
        assertEquals(5f, output[5])
    }

    @Test
    fun fivePointOneCanTargetQuadWithoutCollapsingToStereo() {
        val output = AudioDownmixMatrix.mix(floatArrayOf(1f, 2f, 3f, 4f, 5f, 6f), 4)

        assertEquals(4, output.size)
        assertTrue(output[0] > 3f)
        assertTrue(output[1] > 4f)
        assertEquals(5f, output[2])
        assertEquals(6f, output[3])
    }

    @Test
    fun sevenPointOneCanTargetFivePointZero() {
        val output = AudioDownmixMatrix.mix(floatArrayOf(1f, 2f, 3f, 4f, 5f, 6f, 7f, 8f), 5)

        assertEquals(5, output.size)
        assertEquals(6f, output[3])
        assertEquals(7f, output[4])
    }

    @Test
    fun fivePointZeroTargetFoldsLfeIntoFronts() {
        val output = AudioDownmixMatrix.mix(floatArrayOf(0f, 0f, 0f, 1f, 0f, 0f), 5)

        assertEquals(0.5f, output[0], absoluteTolerance = 0.0001f)
        assertEquals(0.5f, output[1], absoluteTolerance = 0.0001f)
    }

    @Test
    fun sixPointOneTargetKeepsFivePointOneSurroundOrder() {
        val output = AudioDownmixMatrix.mix(floatArrayOf(0f, 0f, 0f, 0f, 1f, 2f), 7)

        assertEquals(0f, output[4], absoluteTolerance = 0.0001f)
        assertEquals(1f, output[5], absoluteTolerance = 0.0001f)
        assertEquals(2f, output[6], absoluteTolerance = 0.0001f)
    }

    @Test
    fun sevenPointOneTargetMapsSixPointOneBackCenterToRearPair() {
        val output = AudioDownmixMatrix.mix(floatArrayOf(0f, 0f, 0f, 0f, 2f, 4f, 6f), 8)

        assertEquals(1f, output[4], absoluteTolerance = 0.0001f)
        assertEquals(1f, output[5], absoluteTolerance = 0.0001f)
        assertEquals(4f, output[6], absoluteTolerance = 0.0001f)
        assertEquals(6f, output[7], absoluteTolerance = 0.0001f)
    }

    @Test
    fun immersiveBedFoldsToSevenPointOneWithoutDroppingHeightEnergy() {
        val output = AudioDownmixMatrix.mix(
            floatArrayOf(0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 1f, 2f),
            8,
        )

        assertEquals(0.5f, output[0], absoluteTolerance = 0.0001f)
        assertEquals(1f, output[1], absoluteTolerance = 0.0001f)
    }

    @Test
    fun sevenPointOneTargetSynthesizesBackCenterFromRearPair() {
        val output = AudioDownmixMatrix.mix(
            floatArrayOf(0f, 0f, 0f, 0f, 1f, 3f, 2f, 4f),
            7,
        )

        assertEquals(2f, output[4], absoluteTolerance = 0.0001f)
        assertEquals(2f, output[5], absoluteTolerance = 0.0001f)
        assertEquals(4f, output[6], absoluteTolerance = 0.0001f)
    }

    @Test
    fun nativeFivePointOneLayoutIsBitPreservingWhenNoDownmixIsNeeded() {
        val input = floatArrayOf(0.1f, -0.2f, 0.3f, -0.4f, 0.5f, -0.6f)

        assertTrue(AudioDownmixMatrix.mix(input, 6).contentEquals(input))
    }

    @Test
    fun limiterNeverReturnsDigitalFullScale() {
        assertTrue(AudioDownmixMatrix.softLimit(5f) < 1f)
        assertTrue(AudioDownmixMatrix.softLimit(-5f) > -1f)
    }

    @Test
    fun limiterSilencesNonFiniteSamples() {
        assertEquals(0f, AudioDownmixMatrix.softLimit(Float.NaN))
        assertEquals(0f, AudioDownmixMatrix.softLimit(Float.POSITIVE_INFINITY))
    }

    @Test
    fun invalidTargetChannelCountsAreClampedSafely() {
        assertEquals(1, AudioDownmixMatrix.mix(floatArrayOf(1f, 1f), 0).size)
        assertEquals(8, AudioDownmixMatrix.mix(FloatArray(10), 12).size)
    }

    @Test
    fun linkedPeakLimiterCatchesAdjacentTransition() {
        val gain = AudioPeakLimiter.linkedGain(
            current = floatArrayOf(-0.98f),
            previous = floatArrayOf(0.98f),
            ceiling = 0.9f,
        )

        assertTrue(gain < 1f)
    }

    @Test
    fun linkedPeakLimiterEstimatesParabolicInterSamplePeak() {
        val gain = AudioPeakLimiter.linkedGain(
            current = floatArrayOf(1f),
            previous = floatArrayOf(1f),
            ceiling = 1f,
            next = floatArrayOf(0f),
        )

        assertTrue(gain < 1f)
    }

}
