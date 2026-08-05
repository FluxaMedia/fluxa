package com.fluxa.app.data.repository

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.domain.ContentIdentity

import com.google.gson.GsonBuilder

object TraktIntegration {
    const val MOBILE_REDIRECT_URI = "fluxa://oauth/trakt"
    private const val TRAKT_API_BASE_URL = "https://api.trakt.tv"
    private val logGson = GsonBuilder().disableHtmlEscaping().create()

    fun hasClient(apiKey: String): Boolean = FluxaCoreNative.traktHasClient(apiKey)

    fun bearer(token: String): String = FluxaCoreNative.traktBearer(token)

    fun scrobbleUrl(action: String): String = FluxaCoreNative.traktScrobbleUrl(action)

    fun playbackUrl(type: String? = null): String =
        FluxaCoreNative.traktPlaybackUrl(type)

    fun toLogJson(value: Any?): String = runCatching { logGson.toJson(value) }.getOrElse { "<json-error>" }

    fun redactedBearer(token: String): String {
        val clean = token.trim()
        if (clean.isBlank()) return "Bearer <blank>"
        return "Bearer ${clean.take(6)}...${clean.takeLast(4)}"
    }

    fun redactedApiKey(apiKey: String): String {
        val clean = apiKey.trim()
        if (clean.isBlank()) return "<blank>"
        return "${clean.take(6)}...${clean.takeLast(4)}"
    }

    fun traktHeadersForLog(token: String, apiKey: String): String = mapOf(
        "Authorization" to redactedBearer(token),
        "trakt-api-key" to redactedApiKey(apiKey),
        "trakt-api-version" to "2",
        "Content-Type" to "application/json"
    ).toString()

    fun tokenExpiresAt(createdAtSeconds: Long, expiresInSeconds: Long): Long {
        return FluxaCoreNative.traktTokenExpiresAt(createdAtSeconds, expiresInSeconds)
    }

    fun contentIdFrom(ids: TraktIds): String? {
        return FluxaCoreNative.traktContentIdFrom(ids)
    }

    fun idsFromContentId(rawId: String): TraktIds? {
        return FluxaCoreNative.traktIdsFromContentId(rawId)
    }

    fun episodeLocator(videoId: String): EpisodeLocator? {
        return FluxaCoreNative.traktEpisodeLocator(videoId)?.let { EpisodeLocator(it.season, it.episode) }
    }

    fun showIdFromEpisodeId(videoId: String): String {
        return FluxaCoreNative.traktShowIdFromEpisodeId(videoId)
    }

    fun scrobbleMediaId(parentId: String, videoId: String?, mediaType: String): String {
        return FluxaCoreNative.traktScrobbleMediaId(parentId, videoId, mediaType)
    }

    fun buildHistoryRequest(meta: Meta, episodes: List<Video>): TraktHistorySyncRequest? {
        return FluxaCoreNative.traktHistoryRequest(meta, episodes)
    }

    fun contentIdentityKey(meta: Meta): String {
        return ContentIdentity.traktKey(meta)
    }
}

data class EpisodeLocator(val season: Int, val episode: Int)
