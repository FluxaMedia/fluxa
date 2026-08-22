package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import org.junit.Assert.assertEquals
import org.junit.Test

class StreamSourceOrderPolicyTest {

    @Test
    fun keepsIncomingStreamsInOrder() {
        val ordered = StreamSourceOrderPolicy.keepSourceOrder(
            streams = listOf(
                Stream(name = "Bad Cam", title = "CAM 1080p 👤 12", url = "https://bad.example/video.m3u8"),
                Stream(name = "Good", title = "1080p 👤 4", url = "https://good.example/video.m3u8")
            ),
            request = StreamDiscoveryRequest(
                addons = emptyList(),
                type = "movie",
                id = "tt0000001",
                language = "tr",
                preferFastStart = true
            )
        )

        assertEquals(2, ordered.size)
        assertEquals("Bad Cam", ordered.first().name)
    }

    @Test
    fun keepsStreamsWithoutPlayableUrl() {
        val ordered = StreamSourceOrderPolicy.keepSourceOrder(
            streams = listOf(
                Stream(name = "Low", title = "720p 👤 10", url = "https://example.com/720.m3u8"),
                Stream(name = "High", title = "1080p 👤 10", url = "https://example.com/1080.m3u8"),
                Stream(name = "MetadataOnly", title = "No direct URL")
            ),
            request = StreamDiscoveryRequest(
                addons = emptyList(),
                type = "movie",
                id = "tt0000002",
                language = "tr",
                preferFastStart = false
            )
        )

        assertEquals(3, ordered.size)
        assertEquals("MetadataOnly", ordered.last().name)
    }

    @Test
    fun keepsAddonStreamsForSeriesVideoIdWithoutClientSideGuessing() {
        val ordered = StreamSourceOrderPolicy.keepSourceOrder(
            streams = listOf(
                Stream(name = "Correct", title = "Show S01E02 1080p", url = "https://example.com/s01e02.m3u8"),
                Stream(name = "Pack", title = "Show Season 1 Pack", infoHash = "abc")
            ),
            request = StreamDiscoveryRequest(
                addons = emptyList(),
                type = "series",
                id = "tt0000003:1:2",
                language = "tr"
            )
        )

        assertEquals(listOf("Correct", "Pack"), ordered.map { it.name })
    }
}
