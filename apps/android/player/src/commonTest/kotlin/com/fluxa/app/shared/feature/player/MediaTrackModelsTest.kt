package com.fluxa.app.shared.feature.player

import kotlin.test.Test
import kotlin.test.assertEquals

class MediaTrackModelsTest {
    @Test
    fun threeChannelAudioUsesTheSameLayoutNameAsThePcmMatrix() {
        val track = MediaTrack(
            id = "three-channel",
            label = "three-channel",
            type = 1,
            groupIndex = 0,
            trackIndex = 0,
            isSelected = false,
            channelCount = 3,
        )

        assertEquals("3.0", track.audioChannelLabel)
    }
}
