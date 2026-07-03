package com.fluxa.app.data.repository

import com.fluxa.app.data.local.WatchedContentDurationRecord
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.TraktSyncItem

data class TraktSyncSnapshot(
    val continueWatchingCount: Int = 0,
    val watchlistCount: Int = 0,
    val syncedItems: Int = continueWatchingCount + watchlistCount
)

data class TraktWatchedState(
    val movieKeys: Set<String> = emptySet(),
    val episodeKeys: Set<String> = emptySet(),
    val episodeIdsBySeries: Map<String, Set<String>> = emptyMap(),
    val durationRecords: List<WatchedContentDurationRecord> = emptyList()
)

internal fun TraktSyncItem.toMeta(type: String, unknownName: () -> String): Meta? {
    val traktRes = movie ?: show ?: return null
    val id = TraktIntegration.contentIdFrom(traktRes.ids) ?: return null
    return Meta(
        id = id,
        name = traktRes.title ?: unknownName(),
        type = type,
        poster = null,
        releaseInfo = traktRes.year?.toString(),
        released = traktRes.year?.let { "$it-01-01" }
    )
}

internal fun traktWatchedEpisodeKey(seriesKey: String, season: Int, episode: Int): String {
    return "$seriesKey:$season:$episode"
}

internal suspend fun fetchTraktSyncPages(
    request: suspend (page: Int, limit: Int) -> retrofit2.Response<List<TraktSyncItem>>
): List<TraktSyncItem> {
    val limit = 1000
    val first = request(1, limit)
    if (!first.isSuccessful) return emptyList()
    val results = first.body().orEmpty().toMutableList()
    val pageCount = (first.headers()["X-Pagination-Page-Count"]?.toIntOrNull() ?: 1).coerceAtLeast(1)
    val cappedPageCount = pageCount.coerceAtMost(20)
    for (page in 2..cappedPageCount) {
        val response = request(page, limit)
        if (!response.isSuccessful) break
        results += response.body().orEmpty()
    }
    return results
}
