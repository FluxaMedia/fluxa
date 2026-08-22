package com.fluxa.app.data.repository

import com.fluxa.app.data.local.WatchedContentDurationRecord

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

fun traktWatchedEpisodeKey(seriesKey: String, season: Int, episode: Int): String {
    return "$seriesKey:$season:$episode"
}
