package com.fluxa.app.data.repository.library

import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.AnilistGraphQlRequest
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.ExternalSyncApi
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.ExternalLibraryClient
import com.fluxa.app.data.repository.TraktIntegration
import com.google.gson.Gson
import com.google.gson.JsonObject
import com.google.gson.reflect.TypeToken
import javax.inject.Inject

private data class AnilistGraphqlQueries(
    val saveMediaListEntry: String,
    val mediaListEntryLookup: String,
    val deleteMediaListEntry: String
)

class AnilistProviderAdapter @Inject constructor(
    private val externalLibraryClient: ExternalLibraryClient,
    private val api: ExternalSyncApi,
    private val profileManager: ProfileManager,
    private val gson: Gson
) : ProviderAdapter {
    override val providerId = ThirdPartyProviderId.ANILIST
    override val capabilities = setOf(
        ProviderCapability.AUTHENTICATION, ProviderCapability.LIBRARY,
        ProviderCapability.CONTINUE_WATCHING, ProviderCapability.WATCH_HISTORY
    )

    private val queries: AnilistGraphqlQueries by lazy {
        gson.fromJson(FluxaCoreUniFfi.coreInvokeValue("anilistGraphqlQueries", "{}"), AnilistGraphqlQueries::class.java)
    }

    override fun isConnected(profile: UserProfile): Boolean = !profile.anilistAccessToken.isNullOrBlank()

    override suspend fun fetchContinueWatching(profile: UserProfile): List<Meta> =
        externalLibraryClient.getAnilistContinueWatching(profile)

    override suspend fun fetchWatchlist(profile: UserProfile): List<Meta> {
        val token = profile.anilistAccessToken?.takeIf(String::isNotBlank) ?: return emptyList()
        val snapshot = externalLibraryClient.getAnilistLibrarySnapshot(token)
        return (snapshot.watchlist.map { it.first } + snapshot.watching).distinctBy { it.id }
    }

    override suspend fun fetchWatching(profile: UserProfile): List<Meta>? {
        val token = profile.anilistAccessToken?.takeIf(String::isNotBlank) ?: return null
        return externalLibraryClient.getAnilistLibrarySnapshot(token).watching
    }

    override suspend fun fetchWatched(profile: UserProfile): List<Meta>? {
        val token = profile.anilistAccessToken?.takeIf(String::isNotBlank) ?: return null
        return externalLibraryClient.getAnilistLibrarySnapshot(token).completed
    }

    override suspend fun pushWatchlist(profile: UserProfile, item: Meta, add: Boolean): Boolean {
        val token = profile.anilistAccessToken?.takeIf(String::isNotBlank) ?: return false
        return runCatching {
            if (add) pushListEntry(token, item, "PLANNING", progress = null)
            else pushWatchlistRemoval(token, item)
        }.onSuccess { success ->
            if (success) profileManager.clearExternalSyncFailure(profile.id, id)
            else profileManager.recordExternalSyncFailure(profile.id, id)
        }.onFailure { profileManager.recordExternalSyncFailure(profile.id, id) }
            .getOrDefault(false)
    }

    override suspend fun pushWatched(
        profile: UserProfile,
        item: Meta,
        episodes: List<Video>,
        watched: Boolean
    ): Boolean {
        if (!watched) return false
        val token = profile.anilistAccessToken?.takeIf(String::isNotBlank) ?: return false
        val progress = episodes.mapNotNull { TraktIntegration.episodeLocator(it.id)?.episode }.maxOrNull()
            ?: episodes.size.takeIf { it > 0 }
        val status = when {
            progress == null -> "COMPLETED"
            item.episodesCount != null && progress >= item.episodesCount -> "COMPLETED"
            else -> "CURRENT"
        }
        return runCatching { pushListEntry(token, item, status, progress) }
            .onSuccess { success ->
                if (success) profileManager.clearExternalSyncFailure(profile.id, id)
                else profileManager.recordExternalSyncFailure(profile.id, id)
            }
            .onFailure { profileManager.recordExternalSyncFailure(profile.id, id) }
            .getOrDefault(false)
    }

    private suspend fun pushListEntry(token: String, meta: Meta, status: String, progress: Int?): Boolean {
        val args = JsonObject().apply {
            addProperty("contentId", meta.id)
            addProperty("status", status)
            progress?.let { addProperty("progress", it) }
        }
        val variablesJson = FluxaCoreUniFfi.coreInvokeValue("anilistSaveMediaListEntryVariables", args.toString())
        if (variablesJson.isJsonNull) return false

        val variablesType = object : TypeToken<Map<String, Any?>>() {}.type
        val variables: Map<String, Any?> = gson.fromJson(variablesJson, variablesType)
        val response = api.anilistGraphQl(
            "Bearer $token",
            AnilistGraphQlRequest(query = queries.saveMediaListEntry, variables = variables)
        )
        return response.get("errors") == null
    }

    private suspend fun pushWatchlistRemoval(token: String, meta: Meta): Boolean {
        val mediaId = meta.id.removePrefix("anilist:").toIntOrNull() ?: return false
        val lookupResponse = api.anilistGraphQl(
            "Bearer $token",
            AnilistGraphQlRequest(query = queries.mediaListEntryLookup, variables = mapOf("mediaId" to mediaId))
        )
        val entryId = lookupResponse.getAsJsonObject("data")
            ?.getAsJsonObject("Media")
            ?.getAsJsonObject("mediaListEntry")
            ?.get("id")
            ?.takeUnless { it.isJsonNull }
            ?.asInt ?: return true

        val deleteResponse = api.anilistGraphQl(
            "Bearer $token",
            AnilistGraphQlRequest(query = queries.deleteMediaListEntry, variables = mapOf("id" to entryId))
        )
        return deleteResponse.get("errors") == null
    }
}
