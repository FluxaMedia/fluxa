package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.ContentIdentity
import com.fluxa.app.domain.discovery.*

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ContinueWatchingListMergerTest {

    @Test
    fun mergesSameTitleAndYearAcrossProviders() {
        val local = meta(
            id = "tmdb:movie:1",
            name = "Same Movie",
            releaseInfo = "2024",
            timeOffset = 1_000L,
            duration = 10_000L,
            poster = "poster"
        )
        val trakt = meta(
            id = "tt1234567",
            name = "Same Movie",
            releaseInfo = "2024",
            timeOffset = 2_000L,
            duration = 10_000L,
            reason = "Trakt.tv"
        )

        val merged = ContinueWatchingListMerger.mergeDuplicates(listOf(local, trakt))

        assertEquals(1, merged.size)
        assertEquals("tt1234567", merged.first().id)
        assertNull(merged.first().poster)
        assertEquals(2_000L, merged.first().timeOffset)
    }

    @Test
    fun aliasesEpisodeIdToSeriesBaseId() {
        val keys = ContentIdentity.mergeKeys(
            meta(id = "tt7654321:2:4", name = "Episode", type = "series")
        )

        assertTrue("tt7654321:2:4" in keys)
        assertTrue("tt7654321" in keys)
    }

    @Test
    fun preservesDistinctItemsWhenIdentityDoesNotMatch() {
        val merged = ContinueWatchingListMerger.mergeDuplicates(
            listOf(
                meta(id = "tt1", name = "One", releaseInfo = "2020"),
                meta(id = "tt2", name = "Two", releaseInfo = "2020")
            )
        )

        assertEquals(listOf("tt1", "tt2"), merged.map { it.id })
    }

    private fun meta(
        id: String,
        name: String,
        type: String = "movie",
        releaseInfo: String? = null,
        timeOffset: Long? = null,
        duration: Long? = null,
        reason: String? = null,
        poster: String? = null
    ): Meta {
        return Meta(
            id = id,
            name = name,
            type = type,
            poster = poster,
            releaseInfo = releaseInfo,
            timeOffset = timeOffset,
            duration = duration,
            reason = reason
        )
    }
}
