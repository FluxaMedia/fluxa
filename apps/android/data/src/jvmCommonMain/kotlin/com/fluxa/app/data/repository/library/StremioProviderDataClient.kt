package com.fluxa.app.data.repository.library

import com.fluxa.app.common.PlatformLog
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.remote.AuthRequest
import com.fluxa.app.data.remote.DatastorePutRequest
import com.fluxa.app.data.remote.DatastoreRequest
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.StremioService
import com.fluxa.app.data.remote.Video
import javax.inject.Inject
import javax.inject.Singleton
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

@Singleton
class StremioProviderDataClient @Inject constructor(
    private val service: StremioService,
    private val profileManager: ProfileManager
) {
    suspend fun fetchAddonCount(authKey: String): Int = withContext(Dispatchers.IO) {
        if (authKey.isBlank()) return@withContext 0
        service.getAddons(AuthRequest(authKey)).body()?.result?.addons?.size ?: 0
    }

    suspend fun fetchContinueWatching(authKey: String): List<Meta> = withContext(Dispatchers.IO) {
        if (authKey.isBlank()) return@withContext emptyList()
        val items = service.getDatastore(DatastoreRequest(authKey, "library")).body()?.result.orEmpty()
        FluxaCoreNative.libraryContinueWatchingItems(items).map { it.copy(reason = "Stremio") }
    }

    suspend fun fetchWatchlist(authKey: String): List<Meta> = withContext(Dispatchers.IO) {
        if (authKey.isBlank()) return@withContext emptyList()
        val items = service.getDatastore(DatastoreRequest(authKey, "library")).body()?.result.orEmpty()
        val value = FluxaCoreNative.libraryWatchlistItems(items)
        if (value.isJsonNull) return@withContext emptyList()
        value.asJsonArray.mapNotNull { entry ->
            val item = entry.asJsonObject
            val id = item.get("id")?.takeUnless { it.isJsonNull }?.asString ?: return@mapNotNull null
            Meta(
                id = id,
                name = item.get("name")?.takeUnless { it.isJsonNull }?.asString ?: id,
                type = item.get("type")?.takeUnless { it.isJsonNull }?.asString ?: "",
                poster = item.get("poster")?.takeUnless { it.isJsonNull }?.asString,
                background = item.get("background")?.takeUnless { it.isJsonNull }?.asString,
                reason = "Stremio"
            )
        }
    }

    suspend fun pushWatchlist(
        profileId: String,
        authKey: String,
        meta: Meta,
        add: Boolean
    ): Boolean = runCatching {
        val items = FluxaCoreNative.watchlistLibraryItems(meta, add)
        if (items.isEmpty()) return@runCatching false
        service.datastorePut(DatastorePutRequest(authKey, "library", items)).isSuccessful
    }.onSuccess { success ->
        if (success) profileManager.clearExternalSyncFailure(profileId, "stremio")
        else profileManager.recordExternalSyncFailure(profileId, "stremio")
    }.onFailure {
        PlatformLog.w("StremioProvider", "Watchlist push failed", it)
        profileManager.recordExternalSyncFailure(profileId, "stremio")
    }.getOrDefault(false)

    suspend fun pushWatched(
        profileId: String,
        authKey: String,
        meta: Meta,
        episodes: List<Video>,
        watched: Boolean
    ): Boolean = runCatching {
        val watchedAt = if (watched) java.time.Instant.now().toString() else null
        val items = FluxaCoreNative.watchedStateItems(meta, episodes, watched, watchedAt)
        if (items.isEmpty()) return@runCatching false
        service.datastorePut(DatastorePutRequest(authKey, "library", items)).isSuccessful
    }.onSuccess { success ->
        if (success) profileManager.clearExternalSyncFailure(profileId, "stremio")
        else profileManager.recordExternalSyncFailure(profileId, "stremio")
    }.onFailure {
        PlatformLog.w("StremioProvider", "Watched push failed", it)
        profileManager.recordExternalSyncFailure(profileId, "stremio")
    }.getOrDefault(false)

    suspend fun pushPlaybackProgress(
        profileId: String,
        authKey: String,
        meta: Meta,
        videoId: String?,
        positionMs: Long,
        durationMs: Long
    ): Boolean = runCatching {
        if (durationMs <= 0L) return@runCatching false
        val playbackMeta = if (!videoId.isNullOrBlank()) meta.copy(lastVideoId = videoId) else meta
        val item = FluxaCoreNative.playbackProgressItem(
            playbackMeta,
            positionMs.coerceAtLeast(0L),
            durationMs,
            java.time.Instant.now().toString()
        ) ?: return@runCatching false
        service.datastorePut(DatastorePutRequest(authKey, "library", listOf(item))).isSuccessful
    }.onSuccess { success ->
        if (success) profileManager.clearExternalSyncFailure(profileId, "stremio")
        else profileManager.recordExternalSyncFailure(profileId, "stremio")
    }.onFailure {
        PlatformLog.w("StremioProvider", "Playback progress push failed", it)
        profileManager.recordExternalSyncFailure(profileId, "stremio")
    }.getOrDefault(false)

    suspend fun clearPlaybackProgress(
        profileId: String,
        authKey: String,
        meta: Meta
    ): Boolean = runCatching {
        val item = FluxaCoreNative.clearPlaybackProgressItem(meta) ?: return@runCatching false
        service.datastorePut(DatastorePutRequest(authKey, "library", listOf(item))).isSuccessful
    }.onSuccess { success ->
        if (success) profileManager.clearExternalSyncFailure(profileId, "stremio")
        else profileManager.recordExternalSyncFailure(profileId, "stremio")
    }.onFailure {
        PlatformLog.w("StremioProvider", "Playback progress clear failed", it)
        profileManager.recordExternalSyncFailure(profileId, "stremio")
    }.getOrDefault(false)

}
