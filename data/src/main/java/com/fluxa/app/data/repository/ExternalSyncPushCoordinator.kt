package com.fluxa.app.data.repository

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

/**
 * Fans mark-watched / watchlist actions out to whichever of Trakt, Simkl, and MAL
 * the active profile has connected. Mirrors fluxa-desktop's pushMarkWatchedExternal /
 * pushWatchlistExternal (src/core/externalSync.ts) — each provider's push is independent
 * and failures are swallowed so one provider never blocks another or the local action.
 *
 * Each provider push only counts as successful when the HTTP response actually says so —
 * a 401 clears (Simkl) or refreshes-then-retries (MAL, which has a refresh token) the
 * stored credentials instead of failing silently forever. A null response means "there
 * was nothing to push" (e.g. no IMDB id) and is skipped without stamping a sync time.
 */
class ExternalSyncPushCoordinator @Inject constructor(
    private val api: TraktApi,
    private val repository: StremioRepository,
    private val profileManager: ProfileManager
) {
    suspend fun pushMarkWatched(profile: UserProfile, meta: Meta, episodes: List<Video>, watched: Boolean) = coroutineScope {
        if (!profile.traktAccessToken.isNullOrBlank()) {
            launch { runCatching { pushTraktMarkWatched(profile.traktAccessToken!!, meta, episodes, watched) } }
        }
        if (!profile.simklAccessToken.isNullOrBlank()) {
            launch { pushSimklWithTokenHandling(profile) { token -> pushSimklMarkWatched(token, meta, episodes, watched) } }
        }
        if (watched && !profile.malAccessToken.isNullOrBlank()) {
            launch { pushMalWithTokenHandling(profile) { token -> pushMalMarkWatched(token, meta, episodes) } }
        }
    }

    suspend fun pushWatchlist(profile: UserProfile, meta: Meta, isInWatchlist: Boolean) = coroutineScope {
        if (!profile.traktAccessToken.isNullOrBlank()) {
            launch { runCatching { pushTraktWatchlist(profile.traktAccessToken!!, meta, isInWatchlist) } }
        }
        if (isInWatchlist && !profile.simklAccessToken.isNullOrBlank()) {
            launch { pushSimklWithTokenHandling(profile) { token -> pushSimklWatchlist(token, meta) } }
        }
        if (isInWatchlist && !profile.malAccessToken.isNullOrBlank()) {
            launch { pushMalWithTokenHandling(profile) { token -> pushMalWatchlist(token, meta) } }
        }
    }

    private suspend fun pushSimklWithTokenHandling(profile: UserProfile, call: suspend (String) -> Response<Unit>?) {
        val token = profile.simklAccessToken?.takeIf { it.isNotBlank() } ?: return
        val response = runCatching { call(token) }.getOrNull() ?: return
        when {
            response.isSuccessful -> profileManager.saveProfile(profile.copy(simklLastSyncAt = System.currentTimeMillis()))
            response.code() == 401 -> profileManager.saveProfile(profile.copy(simklAccessToken = null))
            else -> Unit
        }
    }

    private suspend fun pushMalWithTokenHandling(profile: UserProfile, call: suspend (String) -> Response<Unit>?) {
        val token = profile.malAccessToken?.takeIf { it.isNotBlank() } ?: return
        val firstResponse = runCatching { call(token) }.getOrNull() ?: return
        if (firstResponse.isSuccessful) {
            profileManager.saveProfile(profile.copy(malLastSyncAt = System.currentTimeMillis()))
            return
        }
        if (firstResponse.code() != 401) return

        val refreshToken = profile.malRefreshToken?.takeIf { it.isNotBlank() } ?: return
        val refreshed = runCatching { repository.refreshMalToken(refreshToken) }.getOrNull() ?: return
        val refreshedProfile = profile.copy(
            malAccessToken = refreshed.accessToken,
            malRefreshToken = refreshed.refreshToken ?: refreshToken,
            malTokenExpiresAt = refreshed.expiresIn?.let { System.currentTimeMillis() + it * 1000L }
        )

        val retryResponse = runCatching { call(refreshed.accessToken) }.getOrNull()
        when {
            retryResponse?.isSuccessful == true -> {
                profileManager.saveProfile(refreshedProfile.copy(malLastSyncAt = System.currentTimeMillis()))
            }
            retryResponse?.code() == 401 -> {
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
        if (meta.type != "series") return null
        val malId = MAL_ID_REGEX.find(meta.id)?.groupValues?.get(1)?.toIntOrNull() ?: return null
        val highestEpisode = episodes.mapNotNull { it.number }.maxOrNull() ?: return null
        val totalEpisodes = meta.episodesCount
        val status = if (totalEpisodes != null && highestEpisode >= totalEpisodes) "completed" else "watching"
        return api.malUpdateListStatus(
            malId = malId,
            token = "Bearer $token",
            numWatchedEpisodes = highestEpisode,
            status = status
        )
    }

    private suspend fun pushMalWatchlist(token: String, meta: Meta): Response<Unit>? {
        if (meta.type != "series") return null
        val malId = MAL_ID_REGEX.find(meta.id)?.groupValues?.get(1)?.toIntOrNull() ?: return null
        return api.malUpdateListStatus(
            malId = malId,
            token = "Bearer $token",
            status = "plan_to_watch"
        )
    }

    companion object {
        private val MAL_ID_REGEX = Regex("^mal:(\\d+)$")
    }
}
