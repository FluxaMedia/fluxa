package com.fluxa.app.shared.feature.player

import kotlin.test.Test
import kotlin.test.assertEquals

class AudioPcmChannelPolicyTest {
    @Test
    fun keepsMultichannelLayoutWhenHdmiEndpointReportsStereo() {
        assertEquals(
            8,
            AudioPcmChannelPolicy.resolve(
                deviceMaxChannels = 2,
                capabilitiesMaxChannels = 2,
                speakerLayoutMaxChannels = 8,
                spatializerMaxChannels = null,
                routeSupportsMultichannel = true,
            ),
        )
    }

    @Test
    fun remainsConservativeForStereoHeadphones() {
        assertEquals(
            2,
            AudioPcmChannelPolicy.resolve(
                deviceMaxChannels = 2,
                capabilitiesMaxChannels = 8,
                speakerLayoutMaxChannels = 8,
                spatializerMaxChannels = null,
                routeSupportsMultichannel = false,
            ),
        )
    }

    @Test
    fun spatializerCanAdvertiseMultichannelInputWithoutDeviceChannelList() {
        assertEquals(
            6,
            AudioPcmChannelPolicy.resolve(
                deviceMaxChannels = null,
                capabilitiesMaxChannels = 8,
                speakerLayoutMaxChannels = null,
                spatializerMaxChannels = 6,
                routeSupportsMultichannel = false,
            ),
        )
    }
}
