package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.TraktIds
import com.fluxa.app.data.remote.TraktSummary
import com.fluxa.app.data.remote.TraktSyncItem
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import kotlin.test.Test
import kotlin.test.assertEquals

class AccountSyncPoliciesTest {
    @Test
    fun mapsTraktSummaryWithInjectedIdentityPolicy() {
        val item = TraktSyncItem(show = TraktSummary("Show", 2025, TraktIds(imdb = "tt1")))
        val meta = TraktSyncMapper.toMeta(item, "series", { "Unknown" }) { it.imdb }
        assertEquals("tt1", meta?.id)
        assertEquals("2025-01-01", meta?.released)
    }

    @Test
    fun buildsSimklSeriesHistoryRequest() {
        val request = SimklSyncRequests.history("tt1", true, mapOf(2 to listOf(3, 4)))
        val shows = request["shows"] as List<*>
        val show = shows.single() as Map<*, *>
        val seasons = show["seasons"] as List<*>
        val season = seasons.single() as Map<*, *>
        assertEquals(2, season["number"])
        assertEquals(2, (season["episodes"] as List<*>).size)
    }

    @Test
    fun buildsNuvioProgressAndWatchedRequests() {
        val meta = Meta(id = "tt1", name = "Show", type = "series")
        val progress = NuvioSyncRequests.playbackProgress(meta, "tt1:2:3", 400, 1000, 42)
        assertEquals("tt1_s2e3", progress["progress_key"])
        assertEquals(2, progress["season"])
        val watched = NuvioSyncRequests.watchedItems(
            meta,
            listOf(Video(id = "tt1:2:3", season = 2, number = 3)),
            watchedAt = 42
        )
        assertEquals(3, watched.single()["episode"])
    }

    @Test
    fun choosesExternalRefreshAndCredentialActions() {
        assertEquals(
            ExternalSyncAction.REFRESH_CREDENTIALS,
            ExternalSyncPolicy.afterResponse(ExternalSyncProvider.MAL, 401)
        )
        assertEquals(
            ExternalSyncAction.CLEAR_CREDENTIALS,
            ExternalSyncPolicy.afterResponse(ExternalSyncProvider.SIMKL, 401)
        )
        assertEquals(
            "completed",
            ExternalSyncPolicy.malWatchedUpdate(
                Meta(id = "mal:42", name = "Anime", type = "series", episodesCount = 12),
                listOf(Video(id = "episode", number = 12))
            )?.status
        )
    }
}
