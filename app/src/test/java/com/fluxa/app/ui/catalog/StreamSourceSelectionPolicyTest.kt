package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Stream
import org.junit.Assert.assertEquals
import org.junit.Test

class StreamSourceSelectionPolicyTest {
    @Test
    fun bingeGroupTakesPriorityForNextEpisode() {
        val streams = listOf(
            Stream(name = "720p", title = null, url = "https://example.com/720.mkv", behaviorHints = mapOf("bingeGroup" to "addon-720")),
            Stream(name = "1080p", title = null, url = "https://example.com/1080.mkv", behaviorHints = mapOf("bingeGroup" to "addon-1080"))
        )

        val selected = selectStreamIndex(
            streams = streams,
            currentVideoId = null,
            initialStreamIndex = 0,
            savedUrl = null,
            savedTitle = null,
            sourceSelectionMode = STREAM_SOURCE_MODE_FIRST,
            regexPattern = null,
            preferredBingeGroup = "addon-1080"
        )

        assertEquals(1, selected)
    }

    @Test
    fun regexModeSelectsFirstMatchingStream() {
        val streams = listOf(
            Stream(name = "720p WEBRip", title = null, url = "https://example.com/720.mkv"),
            Stream(name = "1080p WEB-DL", title = null, url = "https://example.com/1080.mkv")
        )

        val selected = selectStreamIndex(
            streams = streams,
            currentVideoId = null,
            initialStreamIndex = 0,
            savedUrl = null,
            savedTitle = null,
            sourceSelectionMode = STREAM_SOURCE_MODE_REGEX,
            regexPattern = "1080p.*WEB-DL",
            preferredBingeGroup = null
        )

        assertEquals(1, selected)
    }

    @Test
    fun invalidRegexFallsBackToManualSavedUrl() {
        val streams = listOf(
            Stream(name = "720p", title = "Old", url = "https://example.com/720.mkv"),
            Stream(name = "1080p", title = "Saved", url = "https://example.com/1080.mkv")
        )

        val selected = selectStreamIndex(
            streams = streams,
            currentVideoId = null,
            initialStreamIndex = 0,
            savedUrl = "https://example.com/1080.mkv",
            savedTitle = null,
            sourceSelectionMode = STREAM_SOURCE_MODE_REGEX,
            regexPattern = "[",
            preferredBingeGroup = null
        )

        assertEquals(1, selected)
    }

    @Test
    fun firstModeSkipsWrongEpisodeStreamsWithoutReorderingResults() {
        val streams = listOf(
            Stream(name = "wrong episode", title = "Show S01E03", url = "https://example.com/e3.mkv"),
            Stream(name = "right episode", title = "Show S01E02", url = "https://example.com/e2.mkv")
        )

        val selected = selectStreamIndex(
            streams = streams,
            currentVideoId = "tt123:1:2",
            initialStreamIndex = 0,
            savedUrl = null,
            savedTitle = null,
            sourceSelectionMode = STREAM_SOURCE_MODE_FIRST,
            regexPattern = null,
            preferredBingeGroup = null
        )

        assertEquals(1, selected)
    }

    @Test
    fun manualModeKeepsInitialIndexWhenItMatchesEpisode() {
        val streams = listOf(
            Stream(name = "first", title = "Show S01E02", url = "https://example.com/first.mkv"),
            Stream(name = "current", title = "Show S01E02", url = "https://example.com/current.mkv")
        )

        val selected = selectStreamIndex(
            streams = streams,
            currentVideoId = "tt123:1:2",
            initialStreamIndex = 1,
            savedUrl = null,
            savedTitle = null,
            sourceSelectionMode = STREAM_SOURCE_MODE_MANUAL,
            regexPattern = null,
            preferredBingeGroup = null
        )

        assertEquals(1, selected)
    }
}
