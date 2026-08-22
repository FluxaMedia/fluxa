package com.fluxa.app.data.repository.library

import com.fluxa.app.data.local.ProviderDataOwner
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.providerDataOwner
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video

enum class PlaybackSyncAction { START, PROGRESS, PAUSE, STOP }

enum class ProviderCapability {
    AUTHENTICATION,
    PROFILES,
    ADDONS,
    LIBRARY,
    CONTINUE_WATCHING,
    WATCH_HISTORY,
    FAVORITES,
    COLLECTION,
    PUSH_PROGRESS
}

interface ProviderAdapter {
    val providerId: ThirdPartyProviderId
    val id: String get() = providerId.key
    val capabilities: Set<ProviderCapability>

    fun isConnected(profile: UserProfile): Boolean

    fun dataOwner(profile: UserProfile): ProviderDataOwner? = profile.providerDataOwner(providerId)

    suspend fun fetchAddonCount(profile: UserProfile): Int? = null

    suspend fun fetchContinueWatching(profile: UserProfile): List<Meta>? = null

    suspend fun fetchWatchlist(profile: UserProfile): List<Meta>

    suspend fun fetchWatching(profile: UserProfile): List<Meta>? = null

    suspend fun fetchWatched(profile: UserProfile): List<Meta>? = null

    suspend fun fetchFavorites(profile: UserProfile): List<Meta>? = null

    suspend fun fetchCollection(profile: UserProfile): List<Meta>? = null

    suspend fun fetchWatchedEpisodeTimestamps(profile: UserProfile): Map<String, Long>? = null

    suspend fun pushWatchlist(profile: UserProfile, item: Meta, add: Boolean): Boolean = false

    suspend fun pushWatched(
        profile: UserProfile,
        item: Meta,
        episodes: List<Video>,
        watched: Boolean
    ): Boolean = false

    suspend fun pushPlaybackProgress(
        profile: UserProfile,
        item: Meta,
        videoId: String?,
        positionMs: Long,
        durationMs: Long,
        action: PlaybackSyncAction
    ): Boolean = false

    suspend fun pushFavorite(profile: UserProfile, item: Meta, favorite: Boolean): Boolean = false

    /** Clears remote resume progress. Unsupported providers must return false. */
    suspend fun clearPlaybackProgress(profile: UserProfile, item: Meta): Boolean = false
}
