package com.fluxa.app.data.repository

import kotlin.test.Test
import kotlin.test.assertEquals

class StremioStreamParserTest {
    @Test
    fun parsesDirectAndTorrentStreams() {
        val streams = StremioStreamParser.parse(
            """{"streams":[{"title":"Direct","url":"https://video","behaviorHints":{"proxyHeaders":{"request":{"Authorization":"token"}}}},{"infoHash":"abc","fileIdx":2}]}"""
        )

        assertEquals("https://video", streams?.first()?.url)
        assertEquals("token", streams?.first()?.requestHeaders?.get("Authorization"))
        assertEquals("stremio://torrent/abc/2", streams?.last()?.url)
    }
}
