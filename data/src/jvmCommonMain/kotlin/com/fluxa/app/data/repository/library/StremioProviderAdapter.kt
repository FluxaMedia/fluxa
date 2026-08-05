package com.fluxa.app.data.repository.library

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.StremioRepository
import javax.inject.Inject

class StremioProviderAdapter @Inject constructor(
    private val repository: StremioRepository
) : ProviderAdapter {
    override val id = "stremio"

    override fun isConnected(profile: UserProfile): Boolean = profile.authKey.isNotBlank()

    override suspend fun fetchWatchlist(profile: UserProfile): List<Meta> {
        if (profile.authKey.isBlank()) return emptyList()
        return repository.getLibraryWatchlistWithTimestamps(profile.authKey).map { it.first }
    }

    override suspend fun pushWatchlist(profile: UserProfile, item: Meta, add: Boolean) {
        if (profile.authKey.isBlank()) return
        repository.pushWatchlist(profile.authKey, item, add)
    }

    override suspend fun pushWatched(profile: UserProfile, item: Meta, episodes: List<Video>, watched: Boolean) {
        if (profile.authKey.isBlank()) return
        repository.syncWatchedState(profile.authKey, traktToken = null, meta = item, episodes = episodes, watched = watched)
    }
}
