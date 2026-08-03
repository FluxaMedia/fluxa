package com.fluxa.app.data.repository.library

import com.fluxa.app.data.BuildConfig
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.TraktApi
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.TraktIntegration
import com.fluxa.app.data.repository.TraktRepository
import javax.inject.Inject

class TraktProviderAdapter @Inject constructor(
    private val traktRepository: TraktRepository,
    private val api: TraktApi,
    private val profileManager: ProfileManager
) : ProviderAdapter {
    override val id = "trakt"

    override fun isConnected(profile: UserProfile): Boolean = !profile.traktAccessToken.isNullOrBlank()

    override suspend fun fetchWatchlist(profile: UserProfile): List<Meta> {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return emptyList()
        return traktRepository.getTraktWatchlist(token)
    }

    override suspend fun fetchWatched(profile: UserProfile): List<Meta>? {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return null
        return traktRepository.getTraktRecentlyWatched(token, profile.safeLanguage, profile)
    }

    override suspend fun fetchFavorites(profile: UserProfile): List<Meta>? {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return null
        return traktRepository.getTraktFavorites(token)
    }

    override suspend fun fetchWatchedEpisodeTimestamps(profile: UserProfile): Map<String, Long>? {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return null
        return traktRepository.getTraktWatchedEpisodesWithTimestamps(token)
    }

    override suspend fun pushWatchlist(profile: UserProfile, item: Meta, add: Boolean) {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return
        val request = TraktIntegration.buildHistoryRequest(item, emptyList()) ?: return
        val bearer = TraktIntegration.bearer(token)
        runCatching {
            if (add) api.addToWatchlist(bearer, BuildConfig.TRAKT_CLIENT_ID, request)
            else api.removeFromWatchlist(bearer, BuildConfig.TRAKT_CLIENT_ID, request)
        }.onSuccess { profileManager.clearExternalSyncFailure(profile.id, id) }
            .onFailure { profileManager.recordExternalSyncFailure(profile.id, id) }
    }

    override suspend fun pushWatched(profile: UserProfile, item: Meta, episodes: List<Video>, watched: Boolean) {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return
        val request = TraktIntegration.buildHistoryRequest(item, episodes) ?: return
        val bearer = TraktIntegration.bearer(token)
        runCatching {
            if (watched) api.addToHistory(bearer, BuildConfig.TRAKT_CLIENT_ID, request)
            else api.removeFromHistory(bearer, BuildConfig.TRAKT_CLIENT_ID, request)
        }.onSuccess { profileManager.clearExternalSyncFailure(profile.id, id) }
            .onFailure { profileManager.recordExternalSyncFailure(profile.id, id) }
    }

    override suspend fun pushFavorite(profile: UserProfile, item: Meta, favorite: Boolean) {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return
        val request = TraktIntegration.buildHistoryRequest(item, emptyList()) ?: return
        val bearer = TraktIntegration.bearer(token)
        runCatching {
            if (favorite) api.addToFavorites(bearer, BuildConfig.TRAKT_CLIENT_ID, request)
            else api.removeFromFavorites(bearer, BuildConfig.TRAKT_CLIENT_ID, request)
        }.onSuccess { profileManager.clearExternalSyncFailure(profile.id, id) }
            .onFailure { profileManager.recordExternalSyncFailure(profile.id, id) }
    }
}
