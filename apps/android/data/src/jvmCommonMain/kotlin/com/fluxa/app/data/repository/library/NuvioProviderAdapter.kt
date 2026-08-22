package com.fluxa.app.data.repository.library

import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.NuvioSyncCoordinator
import javax.inject.Inject

class NuvioProviderAdapter @Inject constructor(
    private val nuvioSyncCoordinator: NuvioSyncCoordinator,
    private val profileManager: ProfileManager
) : ProviderAdapter {
    override val providerId = ThirdPartyProviderId.NUVIO
    override val capabilities = setOf(
        ProviderCapability.AUTHENTICATION, ProviderCapability.PROFILES,
        ProviderCapability.ADDONS, ProviderCapability.LIBRARY,
        ProviderCapability.CONTINUE_WATCHING, ProviderCapability.WATCH_HISTORY,
        ProviderCapability.PUSH_PROGRESS
    )

    override fun isConnected(profile: UserProfile): Boolean = !profile.nuvioAccessToken.isNullOrBlank()

    override suspend fun fetchAddonCount(profile: UserProfile): Int =
        nuvioSyncCoordinator.fetchAddonCount(profile)

    override suspend fun fetchContinueWatching(profile: UserProfile): List<Meta> =
        nuvioSyncCoordinator.fetchContinueWatching(profile)

    override suspend fun fetchWatchlist(profile: UserProfile): List<Meta> =
        nuvioSyncCoordinator.fetchLibrary(profile)

    override suspend fun fetchWatched(profile: UserProfile): List<Meta> =
        nuvioSyncCoordinator.fetchWatched(profile)

    override suspend fun fetchWatchedEpisodeTimestamps(profile: UserProfile): Map<String, Long> =
        nuvioSyncCoordinator.fetchWatchedEpisodeTimestamps(profile)

    override suspend fun clearPlaybackProgress(profile: UserProfile, item: Meta): Boolean =
        runCatching { nuvioSyncCoordinator.clearPlaybackProgress(profile, item) }
            .onSuccess { success ->
                if (success) profileManager.clearExternalSyncFailure(profile.id, id)
                else profileManager.recordExternalSyncFailure(profile.id, id)
            }
            .onFailure { profileManager.recordExternalSyncFailure(profile.id, id) }
            .getOrDefault(false)

    override suspend fun pushWatchlist(profile: UserProfile, item: Meta, add: Boolean): Boolean =
        runCatching { nuvioSyncCoordinator.pushWatchlist(profile, item, add) }
            .onSuccess { success ->
                if (success) profileManager.clearExternalSyncFailure(profile.id, id)
                else profileManager.recordExternalSyncFailure(profile.id, id)
            }
            .onFailure { profileManager.recordExternalSyncFailure(profile.id, id) }
            .getOrDefault(false)

    override suspend fun pushPlaybackProgress(
        profile: UserProfile,
        item: Meta,
        videoId: String?,
        positionMs: Long,
        durationMs: Long,
        action: PlaybackSyncAction
    ): Boolean = runCatching {
        if (durationMs <= 0L) return@runCatching false
        nuvioSyncCoordinator.pushPlaybackProgress(profile, item, videoId, positionMs, durationMs)
        true
    }.onSuccess { success ->
        if (success) profileManager.clearExternalSyncFailure(profile.id, id)
        else profileManager.recordExternalSyncFailure(profile.id, id)
    }.onFailure { profileManager.recordExternalSyncFailure(profile.id, id) }
        .getOrDefault(false)

    override suspend fun pushWatched(
        profile: UserProfile,
        item: Meta,
        episodes: List<Video>,
        watched: Boolean
    ): Boolean = runCatching { nuvioSyncCoordinator.pushWatched(profile, item, episodes, watched) }
        .onSuccess { success ->
            if (success) profileManager.clearExternalSyncFailure(profile.id, id)
            else profileManager.recordExternalSyncFailure(profile.id, id)
        }
        .onFailure { profileManager.recordExternalSyncFailure(profile.id, id) }
        .getOrDefault(false)
}
