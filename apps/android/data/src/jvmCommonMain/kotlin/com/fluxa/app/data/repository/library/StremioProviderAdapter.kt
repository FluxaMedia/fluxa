package com.fluxa.app.data.repository.library

import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import javax.inject.Inject

class StremioProviderAdapter @Inject constructor(
    private val client: StremioProviderDataClient
) : ProviderAdapter {
    override val providerId = ThirdPartyProviderId.STREMIO
    override val capabilities = setOf(
        ProviderCapability.AUTHENTICATION, ProviderCapability.PROFILES,
        ProviderCapability.ADDONS, ProviderCapability.LIBRARY,
        ProviderCapability.CONTINUE_WATCHING, ProviderCapability.WATCH_HISTORY,
        ProviderCapability.PUSH_PROGRESS
    )

    override fun isConnected(profile: UserProfile): Boolean = profile.authKey.isNotBlank()

    override suspend fun fetchAddonCount(profile: UserProfile): Int =
        client.fetchAddonCount(profile.authKey)

    override suspend fun fetchContinueWatching(profile: UserProfile): List<Meta> =
        client.fetchContinueWatching(profile.authKey)

    override suspend fun fetchWatchlist(profile: UserProfile): List<Meta> =
        client.fetchWatchlist(profile.authKey)

    override suspend fun pushWatchlist(profile: UserProfile, item: Meta, add: Boolean): Boolean =
        profile.authKey.takeIf(String::isNotBlank)?.let { authKey ->
            client.pushWatchlist(profile.id, authKey, item, add)
        } ?: false

    override suspend fun pushWatched(
        profile: UserProfile,
        item: Meta,
        episodes: List<Video>,
        watched: Boolean
    ): Boolean = profile.authKey.takeIf(String::isNotBlank)?.let { authKey ->
        client.pushWatched(profile.id, authKey, item, episodes, watched)
    } ?: false

    override suspend fun pushPlaybackProgress(
        profile: UserProfile,
        item: Meta,
        videoId: String?,
        positionMs: Long,
        durationMs: Long,
        action: PlaybackSyncAction
    ): Boolean = profile.authKey.takeIf(String::isNotBlank)?.let { authKey ->
        client.pushPlaybackProgress(profile.id, authKey, item, videoId, positionMs, durationMs)
    } ?: false

    override suspend fun clearPlaybackProgress(profile: UserProfile, item: Meta): Boolean =
        profile.authKey.takeIf(String::isNotBlank)?.let { authKey ->
            client.clearPlaybackProgress(profile.id, authKey, item)
        } ?: false
}
