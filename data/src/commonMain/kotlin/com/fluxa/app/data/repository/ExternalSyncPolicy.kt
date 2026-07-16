package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video

enum class ExternalSyncProvider { SIMKL, MAL }

enum class ExternalSyncAction { STAMP_SUCCESS, CLEAR_CREDENTIALS, REFRESH_CREDENTIALS, KEEP_CREDENTIALS }

data class MalListUpdate(val malId: Int, val watchedEpisodes: Int?, val status: String)

object ExternalSyncPolicy {
    fun afterResponse(provider: ExternalSyncProvider, statusCode: Int): ExternalSyncAction = when {
        statusCode in 200..299 -> ExternalSyncAction.STAMP_SUCCESS
        statusCode == 401 && provider == ExternalSyncProvider.MAL -> ExternalSyncAction.REFRESH_CREDENTIALS
        statusCode == 401 -> ExternalSyncAction.CLEAR_CREDENTIALS
        else -> ExternalSyncAction.KEEP_CREDENTIALS
    }

    fun afterRefreshRetry(statusCode: Int?): ExternalSyncAction = when {
        statusCode != null && statusCode in 200..299 -> ExternalSyncAction.STAMP_SUCCESS
        statusCode == 401 -> ExternalSyncAction.CLEAR_CREDENTIALS
        else -> ExternalSyncAction.KEEP_CREDENTIALS
    }

    fun malWatchedUpdate(meta: Meta, episodes: List<Video>): MalListUpdate? {
        if (meta.type != "series") return null
        val malId = malId(meta.id) ?: return null
        val highestEpisode = episodes.mapNotNull(Video::number).maxOrNull() ?: return null
        val status = if (meta.episodesCount != null && highestEpisode >= meta.episodesCount) "completed" else "watching"
        return MalListUpdate(malId, highestEpisode, status)
    }

    fun malWatchlistUpdate(meta: Meta): MalListUpdate? {
        if (meta.type != "series") return null
        return MalListUpdate(malId(meta.id) ?: return null, null, "plan_to_watch")
    }

    private fun malId(contentId: String): Int? = Regex("^mal:(\\d+)$").find(contentId)?.groupValues?.get(1)?.toIntOrNull()
}
