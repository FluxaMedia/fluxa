package com.fluxa.app.player

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class TorrentCorePolicyTest {
    @Test
    fun normalizesStremioTorrentLinkToMagnetWithTrackers() {
        val info = TorrentCorePolicy.plan(
            link = "stremio://torrent/ABCDEF1234567890ABCDEF1234567890ABCDEF12/4",
            title = "tt123:1:2",
            requestedFileIdx = null,
            preferredFilename = null,
            sources = listOf(
                "tracker:udp://tracker.example:1337/announce",
                "tracker:https://tracker.example/announce",
                "dht:ignored",
                "tracker:udp://tracker.example:1337/announce"
            ),
            fileStats = emptyList()
        )

        val normalized = info.normalizedLink
        assertTrue(
            "unexpected magnet prefix: $normalized",
            normalized.startsWith("magnet:?xt=urn:btih:abcdef1234567890abcdef1234567890abcdef12")
        )
        // Addon-provided trackers survive dedupe (each appears once).
        assertEquals(
            1,
            "tracker.example%3A1337".toRegex().findAll(normalized).count()
        )
        assertTrue(normalized.contains("tr=https%3A%2F%2Ftracker.example%2Fannounce"))
        // Fallback trackers are always appended for peer discovery.
        assertTrue(
            "fallback tracker missing: $normalized",
            normalized.contains("opentrackr.org")
        )
    }

    @Test
    fun resolvesTorrentFileIndexByPreferredFilenameThenEpisodeThenLargestVideo() {
        val stats = listOf(
            TorrFileStat(id = 1, path = "Show/Extras/sample.txt", length = 10),
            TorrFileStat(id = 2, path = "Show/Show.S01E02.1080p.mkv", length = 200),
            TorrFileStat(id = 3, path = "Show/Show.S01E03.1080p.mkv", length = 300),
            TorrFileStat(id = 4, path = "Show/Movie.mp4", length = 500)
        )

        assertEquals(
            3,
            TorrentCorePolicy.plan(
                link = "infohash:abcdef",
                title = "tt123:1:2",
                requestedFileIdx = null,
                preferredFilename = "Show.S01E03.1080p.mkv",
                sources = emptyList(),
                fileStats = stats
            ).selectedFileIdx
        )
        assertEquals(
            2,
            TorrentCorePolicy.plan(
                link = "infohash:abcdef",
                title = "tt123:1:2",
                requestedFileIdx = null,
                preferredFilename = null,
                sources = emptyList(),
                fileStats = stats
            ).selectedFileIdx
        )
        assertEquals(
            9,
            TorrentCorePolicy.plan(
                link = "infohash:abcdef",
                title = "movie-title",
                requestedFileIdx = 9,
                preferredFilename = null,
                sources = emptyList(),
                fileStats = emptyList()
            ).selectedFileIdx
        )
    }

    @Test
    fun ordersFallbackCandidatesByEpisodeThenLargestVideoAndExcludesRejected() {
        val info = TorrentCorePolicy.plan(
            link = "infohash:abcdef",
            title = "tt123:1:2",
            requestedFileIdx = null,
            preferredFilename = null,
            sources = emptyList(),
            rejectedIndex = 2,
            fileStats = listOf(
                TorrFileStat(id = 1, path = "Show.S01E02.mkv", length = 100),
                TorrFileStat(id = 2, path = "Show.S01E02.1080p.mkv", length = 400),
                TorrFileStat(id = 3, path = "Show.S01E03.1080p.mkv", length = 500),
                TorrFileStat(id = 4, path = "readme.txt", length = 1000)
            )
        )

        assertEquals(listOf(1, 3), info.fallbackFileIndexes)
    }

    @Test
    fun computesTorrentStatusRuntimeState() {
        val preloaded = TorrentCorePolicy.statusInfo(
            TorrStatus(
                hash = "hash",
                title = "title",
                downloadSpeed = 0.0,
                activePeers = 0,
                totalPeers = 0,
                progress = 4.0,
                stat = 1,
                preload = 0,
                loadedSize = 256L * 1024L,
                preloadSize = 512L * 1024L,
                fileStats = null
            )
        )
        assertEquals(50, preloaded.bufferProgress)
        assertFalse(preloaded.isPlayableEnough)

        val ready = TorrentCorePolicy.statusInfo(
            TorrStatus(
                hash = "hash",
                title = "title",
                downloadSpeed = 0.0,
                activePeers = 0,
                totalPeers = 0,
                progress = 20.0,
                stat = 3,
                fileStats = null
            )
        )
        assertEquals(100, ready.bufferProgress)
        assertTrue(ready.isPlayableEnough)
    }
}
