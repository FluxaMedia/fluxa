package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.ContentIdentity
import com.fluxa.app.domain.discovery.*

import org.junit.Assert.assertTrue
import org.junit.Test

class ContinueWatchingListMergerTest {

    @Test
    fun aliasesEpisodeIdToSeriesBaseId() {
        val keys = ContentIdentity.mergeKeys(
            meta(id = "tt7654321:2:4", name = "Episode", type = "series")
        )

        assertTrue("tt7654321:2:4" in keys)
        assertTrue("tt7654321" in keys)
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
