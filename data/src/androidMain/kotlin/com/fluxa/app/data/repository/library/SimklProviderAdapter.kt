package com.fluxa.app.data.repository.library

import com.fluxa.app.data.BuildConfig
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.TraktApi
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.ExternalSyncAction
import com.fluxa.app.data.repository.ExternalSyncPolicy
import com.fluxa.app.data.repository.ExternalSyncProvider
import com.fluxa.app.data.repository.SimklIntegration
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.TraktIntegration
import javax.inject.Inject
import retrofit2.Response

class SimklProviderAdapter @Inject constructor(
    private val repository: StremioRepository,
    private val api: TraktApi,
    private val profileManager: ProfileManager
) : ProviderAdapter {
    override val id = "simkl"

    override fun isConnected(profile: UserProfile): Boolean = !profile.simklAccessToken.isNullOrBlank()

    override suspend fun fetchWatchlist(profile: UserProfile): List<Meta> {
        if (!isConnected(profile)) return emptyList()
        return repository.getSimklLibraryItems(profile, "plantowatch")
    }

    override suspend fun fetchWatched(profile: UserProfile): List<Meta>? {
        if (!isConnected(profile)) return null
        return repository.getSimklLibraryItems(profile, "completed")
    }

    override suspend fun fetchWatchedEpisodeTimestamps(profile: UserProfile): Map<String, Long>? {
        if (!isConnected(profile)) return null
        return repository.getSimklWatchedEpisodesWithTimestamps(profile)
    }

    override suspend fun pushWatchlist(profile: UserProfile, item: Meta, add: Boolean) {
        withTokenHandling(profile) { token ->
            val clientId = BuildConfig.SIMKL_CLIENT_ID
            val imdbId = SimklIntegration.imdbIdFrom(item.id) ?: return@withTokenHandling null
            if (add) {
                val body = SimklIntegration.watchlistBody(imdbId, item.type == "series")
                api.simklAddToList(clientId, "Bearer $token", body)
            } else {
                val body = SimklIntegration.watchlistRemovalBody(imdbId, item.type == "series")
                api.simklRemoveFromList(clientId, "Bearer $token", body)
            }
        }
    }

    override suspend fun pushWatched(profile: UserProfile, item: Meta, episodes: List<Video>, watched: Boolean) {
        withTokenHandling(profile) { token ->
            val clientId = BuildConfig.SIMKL_CLIENT_ID
            val imdbId = SimklIntegration.imdbIdFrom(item.id) ?: return@withTokenHandling null
            val isSeries = item.type == "series"
            val episodesBySeason = if (isSeries) {
                episodes.mapNotNull { TraktIntegration.episodeLocator(it.id) }
                    .groupBy({ it.season }, { it.episode })
            } else {
                emptyMap()
            }
            val body = SimklIntegration.historyBody(imdbId, isSeries, episodesBySeason)
            if (watched) api.simklAddToHistory(clientId, "Bearer $token", body)
            else api.simklRemoveFromHistory(clientId, "Bearer $token", body)
        }
    }

    private suspend fun withTokenHandling(profile: UserProfile, call: suspend (String) -> Response<Unit>?) {
        val token = profile.simklAccessToken?.takeIf { it.isNotBlank() } ?: return
        val response = runCatching { call(token) }.getOrNull()
        if (response == null) {
            profileManager.recordExternalSyncFailure(profile.id, id)
            return
        }
        when (ExternalSyncPolicy.afterResponse(ExternalSyncProvider.SIMKL, response.code())) {
            ExternalSyncAction.STAMP_SUCCESS -> {
                profileManager.updateProfile(profile.id) { it.copy(simklLastSyncAt = System.currentTimeMillis()) }
                profileManager.clearExternalSyncFailure(profile.id, id)
            }
            ExternalSyncAction.CLEAR_CREDENTIALS -> profileManager.updateProfile(profile.id) { it.copy(simklAccessToken = null) }
            else -> profileManager.recordExternalSyncFailure(profile.id, id)
        }
    }
}
