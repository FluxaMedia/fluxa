package com.fluxa.app.shared.feature.player

import kotlin.test.Test
import kotlin.test.assertEquals

class AudioDialogueBoostTest {
    @Test
    fun boostsTheActualCenterInSurroundLayouts() {
        val samples = floatArrayOf(1f, 2f, 3f, 4f, 5f, 6f)

        AudioDialogueBoost.applyInPlace(samples, channels = 6)

        assertEquals(1f, samples[0])
        assertEquals(2f, samples[1])
        assertEquals(3.75f, samples[2])
        assertEquals(4f, samples[3])
    }

    @Test
    fun doesNotTreatQuadRearLeftAsDialogue() {
        val samples = floatArrayOf(1f, 2f, 3f, 4f)

        AudioDialogueBoost.applyInPlace(samples, channels = 4)

        assertEquals(3f, samples[2])
    }

    @Test
    fun stereoBoostRaisesMidWithoutChangingSide() {
        val samples = floatArrayOf(0.8f, 0.2f)

        AudioDialogueBoost.applyInPlace(samples, channels = 2)

        assertEquals(0.86f, samples[0], absoluteTolerance = 0.0001f)
        assertEquals(0.26f, samples[1], absoluteTolerance = 0.0001f)
    }
}
