package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.models.NativeSimklEpisodeMatch
import com.google.gson.Gson

object SimklIntegration {
    private val gson = Gson()

    fun imdbIdFrom(contentId: String): String? {
        return FluxaCoreNative.traktIdsFromContentId(contentId)?.imdb?.takeIf { it.isNotBlank() }
    }

    fun matchEpisode(episodes: List<com.fluxa.app.data.remote.SimklEpisodeInfo>, releaseDate: String?, title: String?): NativeSimklEpisodeMatch? {
        val episodesJson = gson.toJson(episodes)
        val targetJson = gson.toJson(mapOf("releaseDate" to (releaseDate.orEmpty()), "title" to title.orEmpty()))
        return FluxaCoreNative.simklMatchEpisode(episodesJson, targetJson)
    }

    fun scrobbleBody(
        imdbId: String?,
        simklId: Int?,
        isEpisode: Boolean,
        season: Int,
        episode: Int,
        timePosSec: Double,
        durationSec: Double
    ): String? {
        val ids = if (simklId != null) mapOf("simkl" to simklId) else mapOf("imdb" to imdbId)
        val idsJson = gson.toJson(ids)
        return FluxaCoreNative.simklScrobbleBody(idsJson, isEpisode, season.toLong(), episode.toLong(), timePosSec, durationSec)
    }

    fun historyBody(imdbId: String, isSeries: Boolean, episodesBySeasonNumber: Map<Int, List<Int>>): Map<String, Any> {
        return SimklSyncRequests.history(imdbId, isSeries, episodesBySeasonNumber)
    }

    fun watchlistBody(imdbId: String, isSeries: Boolean): Map<String, Any> {
        return SimklSyncRequests.watchlist(imdbId, isSeries)
    }
}
