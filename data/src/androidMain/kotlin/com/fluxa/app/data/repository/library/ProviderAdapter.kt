package com.fluxa.app.data.repository.library

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video

interface ProviderAdapter {
    val id: String

    fun isConnected(profile: UserProfile): Boolean

    suspend fun fetchWatchlist(profile: UserProfile): List<Meta>

    suspend fun fetchWatched(profile: UserProfile): List<Meta>? = null

    suspend fun fetchFavorites(profile: UserProfile): List<Meta>? = null

    suspend fun fetchWatchedEpisodeTimestamps(profile: UserProfile): Map<String, Long>? = null

    suspend fun pushWatchlist(profile: UserProfile, item: Meta, add: Boolean) {}

    suspend fun pushWatched(profile: UserProfile, item: Meta, episodes: List<Video>, watched: Boolean) {}

    suspend fun pushFavorite(profile: UserProfile, item: Meta, favorite: Boolean) {}
}
