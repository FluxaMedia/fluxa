package com.fluxa.app.core.rust

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.continueWatchingEpisodeLabelFromCore
import com.fluxa.app.data.repository.isUpNextContinueItemFromCore
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ContinueWatchingCoreContractTest {
    @Test
    fun episodePresentationUsesCoreParsing() {
        val meta = Meta(
            id = "tt123",
            type = "series",
            name = "Example",
            lastVideoId = "tt123:2:4",
            lastEpisodeName = "The Return",
        )

        assertEquals("S2 E4 · The Return", meta.continueWatchingEpisodeLabelFromCore())
    }

    @Test
    fun upNextClassificationUsesCoreProgressPolicy() {
        val meta = Meta(
            id = "tt123",
            type = "series",
            name = "Example",
            lastVideoId = "tt123:1:2",
        )

        assertTrue(meta.isUpNextContinueItemFromCore())
        assertFalse(meta.copy(type = "movie").isUpNextContinueItemFromCore())
    }
}
