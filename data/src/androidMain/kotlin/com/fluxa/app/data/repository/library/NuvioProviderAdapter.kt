package com.fluxa.app.data.repository.library

import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.NuvioSyncCoordinator
import javax.inject.Inject

class NuvioProviderAdapter @Inject constructor(
    private val nuvioSyncCoordinator: NuvioSyncCoordinator,
    private val profileManager: ProfileManager
) : ProviderAdapter {
    override val id = "nuvio"

    override fun isConnected(profile: UserProfile): Boolean = !profile.nuvioAccessToken.isNullOrBlank()

    override suspend fun fetchWatchlist(profile: UserProfile): List<Meta> = profile.nuvioLibrarySnapshot.orEmpty()

    override suspend fun pushWatchlist(profile: UserProfile, item: Meta, add: Boolean) {
        runCatching { nuvioSyncCoordinator.pushWatchlist(profile, item, add) }
            .onSuccess { profileManager.clearExternalSyncFailure(profile.id, id) }
            .onFailure { profileManager.recordExternalSyncFailure(profile.id, id) }
    }

    override suspend fun pushWatched(profile: UserProfile, item: Meta, episodes: List<Video>, watched: Boolean) {
        runCatching { nuvioSyncCoordinator.pushWatched(profile, item, episodes, watched) }
            .onSuccess { profileManager.clearExternalSyncFailure(profile.id, id) }
            .onFailure { profileManager.recordExternalSyncFailure(profile.id, id) }
    }
}
