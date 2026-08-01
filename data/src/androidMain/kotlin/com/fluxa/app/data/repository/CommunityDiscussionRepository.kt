package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.BuildConfig
import com.fluxa.app.data.remote.StremioService
import com.fluxa.app.ui.catalog.CommunityComment
import com.google.gson.Gson
import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.Request
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class CommunityDiscussionRepository @Inject constructor(private val gson: Gson) {
    suspend fun traktComments(contentId: String, contentType: String): List<CommunityComment> = withContext(Dispatchers.IO) {
        val request = FluxaCoreNative.traktCommentsRequest(contentId, contentType) ?: return@withContext emptyList()
        if (BuildConfig.TRAKT_CLIENT_ID.isBlank()) return@withContext emptyList()
        val url = "https://api.trakt.tv/${request.resource}/${request.id}/comments/likes?extended=full&limit=100"
        getJson(url, mapOf("trakt-api-version" to "2", "trakt-api-key" to BuildConfig.TRAKT_CLIENT_ID))
            ?.asJsonArrayOrNull()
            ?.mapNotNull(::traktComment)
            .orEmpty()
    }

    suspend fun mdblistDiscussion(contentId: String, contentType: String, apiKey: String): List<CommunityComment> = withContext(Dispatchers.IO) {
        if (apiKey.isBlank()) return@withContext emptyList()
        val tmdbId = contentId.removePrefix("tmdb:").substringBefore(':').toLongOrNull() ?: return@withContext emptyList()
        val url = FluxaCoreNative.mdblistDiscussionUrl("tmdb", if (contentType == "series") "show" else "movie", tmdbId)
        val root = getJson("$url?apikey=$apiKey") ?: return@withContext emptyList()
        val comments = when {
            root.isJsonArray -> root.asJsonArray
            root.isJsonObject -> root.asJsonObject.getAsJsonArray("comments") ?: root.asJsonObject.getAsJsonArray("data")
            else -> null
        } ?: return@withContext emptyList()
        comments.mapNotNull(::mdblistComment)
    }

    private fun getJson(url: String, headers: Map<String, String> = emptyMap()): JsonElement? = runCatching {
        val request = Request.Builder().url(url).get().apply { headers.forEach { (key, value) -> header(key, value) } }.build()
        StremioService.sharedClient.newCall(request).execute().use { response ->
            if (!response.isSuccessful) null else JsonParser.parseString(response.body.string())
        }
    }.getOrNull()

    private fun JsonElement.asJsonArrayOrNull(): JsonArray? = takeIf { it.isJsonArray }?.asJsonArray

    private fun traktComment(value: JsonElement): CommunityComment? {
        val item = value.takeIf { it.isJsonObject }?.asJsonObject ?: return null
        val user = item.getAsJsonObject("user")
        val body = item.string("comment") ?: return null
        return CommunityComment(user?.string("name") ?: user?.string("username") ?: "Trakt", body, item.int("likes"), item.bool("spoiler"))
    }

    private fun mdblistComment(value: JsonElement): CommunityComment? {
        val item = value.takeIf { it.isJsonObject }?.asJsonObject ?: return null
        val body = item.string("content") ?: item.string("comment") ?: return null
        val user = item.getAsJsonObject("user")
        return CommunityComment(user?.string("name") ?: user?.string("username") ?: item.string("author") ?: "MDBList", body, item.int("likes"), item.bool("spoiler"))
    }

    private fun JsonObject.string(key: String): String? = get(key)?.takeIf { !it.isJsonNull }?.asString?.takeIf { it.isNotBlank() }
    private fun JsonObject.int(key: String): Int = get(key)?.takeIf { !it.isJsonNull }?.asInt ?: 0
    private fun JsonObject.bool(key: String): Boolean = get(key)?.takeIf { !it.isJsonNull }?.asBoolean ?: false
}
