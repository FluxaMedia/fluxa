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

    /** Builds the {movies:[...]} or {shows:[{ids, seasons:[{number, episodes:[{number}]}]}]} body Simkl's
     * /sync/history (and /sync/history/remove) endpoints expect. */
    fun historyBody(imdbId: String, isSeries: Boolean, episodesBySeasonNumber: Map<Int, List<Int>>): Map<String, Any> {
        val ids = mapOf("imdb" to imdbId)
        return if (!isSeries) {
            mapOf("movies" to listOf(mapOf("ids" to ids)))
        } else {
            val seasons = episodesBySeasonNumber.map { (season, episodes) ->
                mapOf("number" to season, "episodes" to episodes.map { mapOf("number" to it) })
            }
            mapOf("shows" to listOf(mapOf("ids" to ids, "seasons" to seasons)))
        }
    }

    fun watchlistBody(imdbId: String, isSeries: Boolean): Map<String, Any> {
        val ids = mapOf("imdb" to imdbId)
        return if (isSeries) {
            mapOf("shows" to listOf(mapOf("ids" to ids, "to" to "plantowatch")))
        } else {
            mapOf("movies" to listOf(mapOf("ids" to ids, "to" to "plantowatch")))
        }
    }
}
