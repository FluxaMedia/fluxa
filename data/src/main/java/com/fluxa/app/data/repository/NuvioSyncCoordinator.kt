package com.fluxa.app.data.repository

import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.NuvioRefreshRequest
import com.fluxa.app.data.remote.NuvioService
import com.fluxa.app.data.remote.Video
import retrofit2.Response
import javax.inject.Inject

class NuvioSyncCoordinator @Inject constructor(
    private val nuvioService: NuvioService,
    private val profileManager: ProfileManager
) {
    suspend fun pushCollections(profile: UserProfile) {
        val current = freshProfile(profile) ?: return
        val token = current.nuvioAccessToken ?: return
        val index = current.nuvioProfileIndex ?: return
        nuvioService.pushCollections(
            "Bearer $token",
            mapOf(
                "p_profile_id" to index,
                "p_collections_json" to current.safeLibraryCollections.map(LibraryUserCollection::toNuvioCollection)
            )
        ).requireSuccess()
        profileManager.saveProfile(current.copy(nuvioLastSyncAt = System.currentTimeMillis()))
    }
    suspend fun pushWatchlist(profile: UserProfile, meta: Meta, isInWatchlist: Boolean) {
        val current = freshProfile(profile) ?: return
        val token = current.nuvioAccessToken ?: return
        val index = current.nuvioProfileIndex ?: return
        val remote = nuvioService.pullLibrary("Bearer $token", mapOf("p_profile_id" to index, "p_limit" to 500, "p_offset" to 0)).bodyOrNull()
            ?.map { item -> item.toNuvioLibraryItem() }
            ?.toMutableList()
            ?: return
        val existingIndex = remote.indexOfFirst { it["content_id"] == meta.id && it["content_type"] == meta.type }
        if (isInWatchlist && existingIndex >= 0) remote.removeAt(existingIndex)
        if (isInWatchlist) remote.add(meta.toNuvioLibraryItem())
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
        val items = if (meta.type == "movie") {
            listOf(mapOf("content_id" to meta.id, "content_type" to "movie", "title" to meta.name, "watched_at" to System.currentTimeMillis()))
        } else {
            episodes.mapNotNull { episode ->
                val season = episode.season ?: return@mapNotNull null
                val number = episode.number ?: return@mapNotNull null
                mapOf("content_id" to meta.id, "content_type" to meta.type, "title" to meta.name, "season" to season, "episode" to number, "watched_at" to System.currentTimeMillis())
            }
        }
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
        val episode = videoId?.split(':')?.takeIf { it.size == 3 }
        nuvioService.pushWatchProgress(
            "Bearer $token",
            mapOf(
                "p_profile_id" to index,
                "p_entries" to listOf(
                    mapOf(
                        "content_id" to meta.id,
                        "content_type" to meta.type,
                        "video_id" to (videoId ?: meta.id),
                        "position" to position,
                        "duration" to duration,
                        "last_watched" to System.currentTimeMillis(),
                        "season" to episode?.getOrNull(1)?.toIntOrNull(),
                        "episode" to episode?.getOrNull(2)?.toIntOrNull()
                    )
                )
            )
        ).requireSuccess()
        profileManager.saveProfile(current.copy(nuvioLastSyncAt = System.currentTimeMillis()))
    }

    private suspend fun freshProfile(profile: UserProfile): UserProfile? {
        val accessToken = profile.nuvioAccessToken?.takeIf { it.isNotBlank() } ?: return null
        val expiresAt = profile.nuvioTokenExpiresAt ?: 0L
        if (expiresAt > System.currentTimeMillis() + 60_000L) return profile
        val refreshToken = profile.nuvioRefreshToken?.takeIf { it.isNotBlank() } ?: return profile
        val session = nuvioService.refreshToken(request = NuvioRefreshRequest(refreshToken)).bodyOrNull() ?: return null
        return profile.copy(
            nuvioAccessToken = session.accessToken.ifBlank { accessToken },
            nuvioRefreshToken = session.refreshToken.ifBlank { refreshToken },
            nuvioTokenExpiresAt = session.expiresIn?.let { System.currentTimeMillis() + it * 1000L },
            nuvioUserId = session.user?.id ?: profile.nuvioUserId,
            nuvioEmail = session.user?.email ?: profile.nuvioEmail
        ).also(profileManager::saveProfile)
    }
}

private fun LibraryUserCollection.toNuvioCollection(): Map<String, Any?> = mapOf(
    "id" to id,
    "title" to title,
    "backdropImageUrl" to imageUrl,
    "showOnHome" to showOnHome,
    "pinToTop" to pinToTop,
    "viewMode" to viewMode,
    "showAllTab" to showAllTab,
    "focusGlowEnabled" to focusGlowEnabled,
    "folders" to folders.orEmpty().map { folder ->
        mapOf(
            "id" to folder.id,
            "title" to folder.title,
            "coverImageUrl" to (folder.coverImageUrl ?: folder.imageUrl),
            "coverEmoji" to folder.coverEmoji,
            "focusGifUrl" to folder.focusGifUrl,
            "focusGifEnabled" to folder.focusGifEnabled,
            "titleLogoUrl" to folder.titleLogoUrl,
            "heroBackdropUrl" to folder.heroBackdropUrl,
            "tileShape" to folder.shape,
            "hideTitle" to folder.hideTitle,
            "catalogSources" to folder.catalogSources.orEmpty().map { source ->
                mapOf("provider" to "addon", "addonId" to source.addonId, "catalogId" to source.catalogId, "type" to source.type)
            }
        )
    }
)

private fun Meta.toNuvioLibraryItem(): Map<String, Any?> = mapOf(
    "content_id" to id,
    "content_type" to type,
    "name" to name,
    "poster" to poster,
    "background" to background,
    "description" to description,
    "release_info" to releaseInfo,
    "imdb_rating" to imdbRating?.toDoubleOrNull(),
    "genres" to genres,
    "added_at" to System.currentTimeMillis()
)

private fun com.fluxa.app.data.remote.NuvioLibraryItem.toNuvioLibraryItem(): Map<String, Any?> = mapOf(
    "content_id" to contentId,
    "content_type" to contentType,
    "name" to name,
    "poster" to poster,
    "background" to background,
    "description" to description,
    "release_info" to releaseInfo,
    "imdb_rating" to imdbRating,
    "genres" to genres
)

private fun <T> Response<T>.bodyOrNull(): T? = if (isSuccessful) body() else null

private fun Response<*>.requireSuccess() {
    if (!isSuccessful) throw IllegalStateException("Nuvio request failed (${code()})")
}
