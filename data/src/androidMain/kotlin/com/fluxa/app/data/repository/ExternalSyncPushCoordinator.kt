package com.fluxa.app.data.repository

import android.util.Log
import com.fluxa.app.data.BuildConfig
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.TraktApi
import com.fluxa.app.data.remote.Video
import retrofit2.Response

import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import javax.inject.Inject

class ExternalSyncPushCoordinator @Inject constructor(
    private val api: TraktApi,
    private val repository: StremioRepository,
    private val profileManager: ProfileManager,
    private val nuvioSyncCoordinator: NuvioSyncCoordinator
) {
    suspend fun pushMarkWatched(profile: UserProfile, meta: Meta, episodes: List<Video>, watched: Boolean) = coroutineScope {
        profile.traktAccessToken?.takeIf(String::isNotBlank)?.let { token ->
            launch {
                runCatching { pushTraktMarkWatched(token, meta, episodes, watched) }
                    .onSuccess { profileManager.clearExternalSyncFailure(profile.id, "trakt") }
                    .onFailure {
                        Log.w("ExternalSyncPush", "Trakt pushMarkWatched failed for ${meta.id}", it)
                        profileManager.recordExternalSyncFailure(profile.id, "trakt")
                    }
            }
        }
        if (!profile.simklAccessToken.isNullOrBlank()) {
            launch { pushSimklWithTokenHandling(profile) { token -> pushSimklMarkWatched(token, meta, episodes, watched) } }
        }
        if (watched && !profile.malAccessToken.isNullOrBlank()) {
            launch { pushMalWithTokenHandling(profile) { token -> pushMalMarkWatched(token, meta, episodes) } }
        }
        if (!profile.nuvioAccessToken.isNullOrBlank()) {
            launch {
                runCatching { nuvioSyncCoordinator.pushWatched(profile, meta, episodes, watched) }
                    .onSuccess { profileManager.clearExternalSyncFailure(profile.id, "nuvio") }
                    .onFailure {
                        Log.w("ExternalSyncPush", "Nuvio pushWatched failed for ${meta.id}", it)
                        profileManager.recordExternalSyncFailure(profile.id, "nuvio")
                    }
            }
        }
    }

    suspend fun pushWatchlist(profile: UserProfile, meta: Meta, isInWatchlist: Boolean) = coroutineScope {
        profile.traktAccessToken?.takeIf(String::isNotBlank)?.let { token ->
            launch {
                runCatching { pushTraktWatchlist(token, meta, isInWatchlist) }
                    .onSuccess { profileManager.clearExternalSyncFailure(profile.id, "trakt") }
                    .onFailure {
                        Log.w("ExternalSyncPush", "Trakt pushWatchlist failed for ${meta.id}", it)
                        profileManager.recordExternalSyncFailure(profile.id, "trakt")
                    }
            }
        }
        if (isInWatchlist && !profile.simklAccessToken.isNullOrBlank()) {
            launch { pushSimklWithTokenHandling(profile) { token -> pushSimklWatchlist(token, meta) } }
        }
        if (isInWatchlist && !profile.malAccessToken.isNullOrBlank()) {
            launch { pushMalWithTokenHandling(profile) { token -> pushMalWatchlist(token, meta) } }
        }
        if (!profile.nuvioAccessToken.isNullOrBlank()) {
            launch {
                runCatching { nuvioSyncCoordinator.pushWatchlist(profile, meta, isInWatchlist) }
                    .onSuccess { profileManager.clearExternalSyncFailure(profile.id, "nuvio") }
                    .onFailure {
                        Log.w("ExternalSyncPush", "Nuvio pushWatchlist failed for ${meta.id}", it)
                        profileManager.recordExternalSyncFailure(profile.id, "nuvio")
                    }
            }
        }
    }

    private suspend fun pushSimklWithTokenHandling(profile: UserProfile, call: suspend (String) -> Response<Unit>?) {
        val token = profile.simklAccessToken?.takeIf { it.isNotBlank() } ?: return
        val response = runCatching { call(token) }.getOrNull()
        if (response == null) {
            Log.w("ExternalSyncPush", "Simkl push failed for profile ${profile.id}")
            profileManager.recordExternalSyncFailure(profile.id, "simkl")
            return
        }
        when (ExternalSyncPolicy.afterResponse(ExternalSyncProvider.SIMKL, response.code())) {
            ExternalSyncAction.STAMP_SUCCESS -> {
                profileManager.saveProfile(profile.copy(simklLastSyncAt = System.currentTimeMillis()))
                profileManager.clearExternalSyncFailure(profile.id, "simkl")
            }
            ExternalSyncAction.CLEAR_CREDENTIALS -> profileManager.saveProfile(profile.copy(simklAccessToken = null))
            else -> {
                Log.w("ExternalSyncPush", "Simkl push failed for profile ${profile.id} http=${response.code()}")
                profileManager.recordExternalSyncFailure(profile.id, "simkl")
            }
        }
    }

    private suspend fun pushMalWithTokenHandling(profile: UserProfile, call: suspend (String) -> Response<Unit>?) {
        val token = profile.malAccessToken?.takeIf { it.isNotBlank() } ?: return
        val firstResponse = runCatching { call(token) }.getOrNull() ?: return
        when (ExternalSyncPolicy.afterResponse(ExternalSyncProvider.MAL, firstResponse.code())) {
            ExternalSyncAction.STAMP_SUCCESS -> {
                profileManager.saveProfile(profile.copy(malLastSyncAt = System.currentTimeMillis()))
                return
            }
            ExternalSyncAction.REFRESH_CREDENTIALS -> Unit
            else -> return
        }

        val refreshToken = profile.malRefreshToken?.takeIf { it.isNotBlank() } ?: return
        val refreshed = runCatching { repository.refreshMalToken(refreshToken) }.getOrNull() ?: return
        val refreshedProfile = profile.copy(
            malAccessToken = refreshed.accessToken,
            malRefreshToken = refreshed.refreshToken ?: refreshToken,
            malTokenExpiresAt = refreshed.expiresIn?.let { System.currentTimeMillis() + it * 1000L }
        )

        val retryResponse = runCatching { call(refreshed.accessToken) }.getOrNull()
        when (ExternalSyncPolicy.afterRefreshRetry(retryResponse?.code())) {
            ExternalSyncAction.STAMP_SUCCESS -> {
                profileManager.saveProfile(refreshedProfile.copy(malLastSyncAt = System.currentTimeMillis()))
            }
            ExternalSyncAction.CLEAR_CREDENTIALS -> {
                profileManager.saveProfile(refreshedProfile.copy(malAccessToken = null, malRefreshToken = null, malTokenExpiresAt = null))
            }
            else -> profileManager.saveProfile(refreshedProfile)
        }
    }

    private suspend fun pushTraktMarkWatched(token: String, meta: Meta, episodes: List<Video>, watched: Boolean) {
        val request = TraktIntegration.buildHistoryRequest(meta, episodes) ?: return
        val bearer = TraktIntegration.bearer(token)
        if (watched) {
            api.addToHistory(bearer, BuildConfig.TRAKT_CLIENT_ID, request)
        } else {
            api.removeFromHistory(bearer, BuildConfig.TRAKT_CLIENT_ID, request)
        }
    }

    private suspend fun pushTraktWatchlist(token: String, meta: Meta, isInWatchlist: Boolean) {
        val request = TraktIntegration.buildHistoryRequest(meta, emptyList()) ?: return
        val bearer = TraktIntegration.bearer(token)
        if (isInWatchlist) {
            api.addToWatchlist(bearer, BuildConfig.TRAKT_CLIENT_ID, request)
        } else {
            api.removeFromWatchlist(bearer, BuildConfig.TRAKT_CLIENT_ID, request)
        }
    }

    private suspend fun pushSimklMarkWatched(token: String, meta: Meta, episodes: List<Video>, watched: Boolean): Response<Unit>? {
        val clientId = BuildConfig.SIMKL_CLIENT_ID
        val imdbId = SimklIntegration.imdbIdFrom(meta.id) ?: return null
        val isSeries = meta.type == "series"
        val episodesBySeason = if (isSeries) {
            episodes.mapNotNull { TraktIntegration.episodeLocator(it.id) }
                .groupBy({ it.season }, { it.episode })
        } else {
            emptyMap()
        }
        val body = SimklIntegration.historyBody(imdbId, isSeries, episodesBySeason)
        val bearer = "Bearer $token"
        return if (watched) {
            api.simklAddToHistory(clientId, bearer, body)
        } else {
            api.simklRemoveFromHistory(clientId, bearer, body)
        }
    }

    private suspend fun pushSimklWatchlist(token: String, meta: Meta): Response<Unit>? {
        val clientId = BuildConfig.SIMKL_CLIENT_ID
        val imdbId = SimklIntegration.imdbIdFrom(meta.id) ?: return null
        val body = SimklIntegration.watchlistBody(imdbId, meta.type == "series")
        return api.simklAddToList(clientId, "Bearer $token", body)
    }

    private suspend fun pushMalMarkWatched(token: String, meta: Meta, episodes: List<Video>): Response<Unit>? {
        val update = ExternalSyncPolicy.malWatchedUpdate(meta, episodes) ?: return null
        return api.malUpdateListStatus(
            malId = update.malId,
            token = "Bearer $token",
            numWatchedEpisodes = update.watchedEpisodes,
            status = update.status
        )
    }

    private suspend fun pushMalWatchlist(token: String, meta: Meta): Response<Unit>? {
        val update = ExternalSyncPolicy.malWatchlistUpdate(meta) ?: return null
        return api.malUpdateListStatus(
            malId = update.malId,
            token = "Bearer $token",
            status = update.status
        )
    }
}
