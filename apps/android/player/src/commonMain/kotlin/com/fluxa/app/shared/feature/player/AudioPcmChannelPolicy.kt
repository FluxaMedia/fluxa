package com.fluxa.app.shared.feature.player

/** Pure route-channel decision shared by capability reporting and playback. */
object AudioPcmChannelPolicy {
    fun resolve(
        deviceMaxChannels: Int?,
        capabilitiesMaxChannels: Int,
        speakerLayoutMaxChannels: Int?,
        spatializerMaxChannels: Int?,
        routeSupportsMultichannel: Boolean,
    ): Int {
        val advertisedLayoutMax = maxOf(speakerLayoutMaxChannels ?: 0, spatializerMaxChannels ?: 0)
        val routePcmMax = maxOf(capabilitiesMaxChannels, advertisedLayoutMax).coerceIn(2, 8)
        return when {
            deviceMaxChannels != null -> maxOf(
                deviceMaxChannels,
                if (routeSupportsMultichannel) speakerLayoutMaxChannels ?: 0 else 0,
                if (routeSupportsMultichannel) spatializerMaxChannels ?: 0 else 0,
            ).coerceAtMost(routePcmMax).coerceIn(2, 8)
            routeSupportsMultichannel -> maxOf(
                capabilitiesMaxChannels,
                speakerLayoutMaxChannels ?: 0,
                spatializerMaxChannels ?: 0,
            ).coerceIn(2, 8)
            spatializerMaxChannels != null -> spatializerMaxChannels
                .coerceAtMost(capabilitiesMaxChannels)
                .coerceIn(2, 8)
            else -> 2
        }
    }
}
