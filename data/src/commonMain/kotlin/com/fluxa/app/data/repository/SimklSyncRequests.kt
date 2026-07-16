package com.fluxa.app.data.repository

object SimklSyncRequests {
    fun history(imdbId: String, isSeries: Boolean, episodesBySeasonNumber: Map<Int, List<Int>>): Map<String, Any> {
        val ids = mapOf("imdb" to imdbId)
        if (!isSeries) return mapOf("movies" to listOf(mapOf("ids" to ids)))
        val seasons = episodesBySeasonNumber.map { (season, episodes) ->
            mapOf("number" to season, "episodes" to episodes.map { mapOf("number" to it) })
        }
        return mapOf("shows" to listOf(mapOf("ids" to ids, "seasons" to seasons)))
    }

    fun watchlist(imdbId: String, isSeries: Boolean): Map<String, Any> {
        val ids = mapOf("imdb" to imdbId)
        return if (isSeries) {
            mapOf("shows" to listOf(mapOf("ids" to ids, "to" to "plantowatch")))
        } else {
            mapOf("movies" to listOf(mapOf("ids" to ids, "to" to "plantowatch")))
        }
    }
}
