package com.fluxa.app.data.repository

import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeDisabledLocalAddonIds
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.NuvioRefreshRequest
import com.fluxa.app.data.remote.toDto
import com.fluxa.app.data.remote.NuvioService
import com.fluxa.app.data.remote.Video
import retrofit2.Response
import javax.inject.Inject

class NuvioSyncCoordinator @Inject constructor(
    private val nuvioService: NuvioService,
    private val profileManager: ProfileManager
) {
    suspend fun isHealthy(): Boolean {
        return runCatching { nuvioService.healthCheck().bodyOrNull()?.status?.lowercase() in setOf("healthy", "ok") }.getOrDefault(false)
    }

    suspend fun pushCollections(profile: UserProfile) {
        val current = freshProfile(profile) ?: return
        val token = current.nuvioAccessToken ?: return
        val index = current.nuvioProfileIndex ?: return
        nuvioService.pushCollections(
            "Bearer $token",
            mapOf(
                "p_profile_id" to index,
                "p_collections_json" to current.safeLibraryCollections.map(NuvioSyncRequests::collection)
            )
        ).requireSuccess()
        profileManager.saveProfile(current.copy(nuvioLastSyncAt = System.currentTimeMillis()))
    }

    suspend fun pushAddons(profile: UserProfile) {
        val current = freshProfile(profile) ?: return
        val token = current.nuvioAccessToken ?: return
        val index = current.nuvioProfileIndex ?: return
        val disabled = current.safeDisabledLocalAddonIds
        val desired = current.safeInstalledLocalAddons.mapIndexed { sortOrder, url ->
            mapOf(
                "url" to url,
                "name" to url,
                "enabled" to (com.fluxa.app.domain.discovery.StremioAddonUrls.identity(url) !in disabled),
                "sort_order" to sortOrder
            )
        }
        val authorization = "Bearer $token"
        val existing = nuvioService.pullAddons(authorization, profileId = "eq.$index").body()?.map { it.toDomain() }
            ?: throw IllegalStateException("Unable to load Nuvio add-ons")
        val desiredByUrl = desired.associateBy { it["url"] as String }
        nuvioService.pushAddons(
            authorization,
            mapOf(
                "p_profile_id" to index,
                "p_addons" to desired
            )
        ).requireSuccess()
        existing.forEach { addon ->
            val desiredAddon = desiredByUrl[addon.url]
            if (desiredAddon == null) {
                addon.id?.let { id ->
                    nuvioService.deleteAddon(authorization, "eq.$id", "eq.$index").requireSuccess()
                }
            } else {
                addon.id?.let { id ->
                    nuvioService.updateAddon(authorization, "eq.$id", "eq.$index", desiredAddon).requireSuccess()
                }
            }
        }
        profileManager.saveProfile(current.copy(nuvioLastSyncAt = System.currentTimeMillis()))
    }
    suspend fun pushWatchlist(profile: UserProfile, meta: Meta, isInWatchlist: Boolean) {
        val current = freshProfile(profile) ?: return
        val token = current.nuvioAccessToken ?: return
        val index = current.nuvioProfileIndex ?: return
        val remote = nuvioService.pullLibrary("Bearer $token", mapOf("p_profile_id" to index, "p_limit" to 500, "p_offset" to 0)).bodyOrNull()?.map { it.toDomain() }
            ?.map(NuvioSyncRequests::libraryItem)
            ?.toMutableList()
            ?: return
        val existingIndex = remote.indexOfFirst { it["content_id"] == meta.id && it["content_type"] == meta.type }
        if (isInWatchlist && existingIndex >= 0) remote.removeAt(existingIndex)
        if (isInWatchlist) remote.add(NuvioSyncRequests.libraryItem(meta, System.currentTimeMillis()))
        else if (existingIndex >= 0) remote.removeAt(existingIndex)
        nuvioService.pushLibrary(
            "Bearer $token",
            mapOf(
                "p_profile_id" to index,
                "p_items" to remote
            )
        ).requireSuccess()
        profileManager.saveProfile(current.copy(nuvioLastSyncAt = System.currentTimeMillis()))
    }

    suspend fun pushWatched(profile: UserProfile, meta: Meta, episodes: List<Video>, watched: Boolean) {
        val current = freshProfile(profile) ?: return
        val token = current.nuvioAccessToken ?: return
        val index = current.nuvioProfileIndex ?: return
        val items = NuvioSyncRequests.watchedItems(meta, episodes, System.currentTimeMillis())
        if (items.isEmpty()) return
        val keys = items.map { item ->
            mapOf(
                "content_id" to item["content_id"],
                "season" to item["season"],
                "episode" to item["episode"]
            )
        }
        if (watched) {
            nuvioService.pushWatchedItems("Bearer $token", mapOf("p_profile_id" to index, "p_items" to items)).requireSuccess()
        } else {
            nuvioService.deleteWatchedItems("Bearer $token", mapOf("p_profile_id" to index, "p_keys" to keys)).requireSuccess()
        }
        profileManager.saveProfile(current.copy(nuvioLastSyncAt = System.currentTimeMillis()))
    }

    suspend fun pushPlaybackProgress(profile: UserProfile, meta: Meta, videoId: String?, position: Long, duration: Long) {
        if (duration <= 0L) return
        val current = freshProfile(profile) ?: return
        val token = current.nuvioAccessToken ?: return
        val index = current.nuvioProfileIndex ?: return
        val entry = NuvioSyncRequests.playbackProgress(meta, videoId, position, duration, System.currentTimeMillis())
        nuvioService.pushWatchProgress(
            "Bearer $token",
            mapOf(
                "p_profile_id" to index,
                "p_entries" to listOf(entry)
            )
        ).requireSuccess()
        profileManager.saveProfile(current.copy(nuvioLastSyncAt = System.currentTimeMillis()))
    }

    private suspend fun freshProfile(profile: UserProfile): UserProfile? {
        val accessToken = profile.nuvioAccessToken?.takeIf { it.isNotBlank() } ?: return null
        val expiresAt = profile.nuvioTokenExpiresAt ?: 0L
        if (expiresAt > System.currentTimeMillis() + 60_000L) return profile
        val refreshToken = profile.nuvioRefreshToken?.takeIf { it.isNotBlank() } ?: return profile
        val session = nuvioService.refreshToken(request = NuvioRefreshRequest(refreshToken).toDto()).bodyOrNull()?.toDomain() ?: return null
        return profile.copy(
            nuvioAccessToken = session.accessToken.ifBlank { accessToken },
            nuvioRefreshToken = session.refreshToken.ifBlank { refreshToken },
            nuvioTokenExpiresAt = session.expiresIn?.let { System.currentTimeMillis() + it * 1000L },
            nuvioUserId = session.user?.id ?: profile.nuvioUserId,
            nuvioEmail = session.user?.email ?: profile.nuvioEmail
        ).also(profileManager::saveProfile)
    }
}

private fun <T> Response<T>.bodyOrNull(): T? = if (isSuccessful) body() else null

private fun Response<*>.requireSuccess() {
    if (!isSuccessful) throw IllegalStateException("Nuvio request failed (${code()})")
}
