package com.fluxa.app.data.repository.library

import com.fluxa.app.data.PlatformSecrets
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.ExternalSyncApi
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.ExternalLibraryClient
import com.fluxa.app.data.repository.ExternalSyncAction
import com.fluxa.app.data.repository.ExternalSyncPolicy
import com.fluxa.app.data.repository.ExternalSyncProvider
import com.fluxa.app.data.repository.SimklIntegration
import com.fluxa.app.data.repository.TraktIntegration
import javax.inject.Inject
import retrofit2.Response
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.RequestBody.Companion.toRequestBody

class SimklProviderAdapter @Inject constructor(
    private val externalLibraryClient: ExternalLibraryClient,
    private val api: ExternalSyncApi,
    private val profileManager: ProfileManager
) : ProviderAdapter {
    override val providerId = ThirdPartyProviderId.SIMKL
    override val capabilities = setOf(
        ProviderCapability.AUTHENTICATION, ProviderCapability.LIBRARY,
        ProviderCapability.CONTINUE_WATCHING, ProviderCapability.WATCH_HISTORY,
        ProviderCapability.PUSH_PROGRESS
    )

    override fun isConnected(profile: UserProfile): Boolean = !profile.simklAccessToken.isNullOrBlank()

    override suspend fun fetchContinueWatching(profile: UserProfile): List<Meta> {
        if (!isConnected(profile)) return emptyList()
        return externalLibraryClient.getSimklContinueWatching(profile, profile.language ?: "en")
    }

    override suspend fun fetchWatchlist(profile: UserProfile): List<Meta> {
        if (!isConnected(profile)) return emptyList()
        return externalLibraryClient.getSimklLibraryItems(profile, "plantowatch")
    }

    override suspend fun fetchWatching(profile: UserProfile): List<Meta>? {
        if (!isConnected(profile)) return null
        return externalLibraryClient.getSimklLibraryItems(profile, "watching")
    }

    override suspend fun fetchWatched(profile: UserProfile): List<Meta>? {
        if (!isConnected(profile)) return null
        return externalLibraryClient.getSimklLibraryItems(profile, "completed")
    }

    override suspend fun fetchWatchedEpisodeTimestamps(profile: UserProfile): Map<String, Long>? {
        if (!isConnected(profile)) return null
        return externalLibraryClient.getSimklWatchedEpisodesWithTimestamps(profile)
    }

    override suspend fun pushWatchlist(profile: UserProfile, item: Meta, add: Boolean): Boolean =
        withTokenHandling(profile) { token ->
            val clientId = PlatformSecrets.simklClientId
            val imdbId = SimklIntegration.imdbIdFrom(item.id) ?: return@withTokenHandling null
            if (add) {
                val body = SimklIntegration.watchlistBody(imdbId, item.type == "series")
                api.simklAddToList(clientId, "Bearer $token", body)
            } else {
                val body = SimklIntegration.watchlistRemovalBody(imdbId, item.type == "series")
                api.simklRemoveFromList(clientId, "Bearer $token", body)
            }
        }

    override suspend fun pushWatched(
        profile: UserProfile,
        item: Meta,
        episodes: List<Video>,
        watched: Boolean
    ): Boolean = withTokenHandling(profile) { token ->
            val clientId = PlatformSecrets.simklClientId
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

    override suspend fun pushPlaybackProgress(
        profile: UserProfile,
        item: Meta,
        videoId: String?,
        positionMs: Long,
        durationMs: Long,
        action: PlaybackSyncAction
    ): Boolean {
        // Simkl scrobbling is event based; don't issue an API/search request every
        // telemetry tick. START/PAUSE/STOP carry the meaningful state changes.
        if (action == PlaybackSyncAction.PROGRESS) return true
        return withTokenHandling(profile) { token ->
            if (durationMs <= 0L) return@withTokenHandling null
            val mediaId = TraktIntegration.scrobbleMediaId(item.id, videoId, item.type)
            val imdbId = SimklIntegration.imdbIdFrom(mediaId) ?: return@withTokenHandling null
            val isEpisode = item.type == "series"
            val episode = if (isEpisode) TraktIntegration.episodeLocator(mediaId) else null
            if (isEpisode && episode == null) return@withTokenHandling null
            val clientId = PlatformSecrets.simklClientId
            if (clientId.isBlank()) return@withTokenHandling null
            val bearer = "Bearer $token"
            val lookup = api.simklSearchById(imdbId, clientId, bearer)
            val wantType = if (isEpisode) "tv" else "movie"
            val simklId = if (lookup.isSuccessful) {
                lookup.body()?.firstOrNull { it.type == wantType }?.ids?.simkl
            } else {
                null
            }
            val body = SimklIntegration.scrobbleBody(
                imdbId = imdbId,
                simklId = simklId,
                isEpisode = isEpisode,
                season = episode?.season ?: 1,
                episode = episode?.episode ?: 1,
                timePosSec = positionMs.coerceAtLeast(0L) / 1000.0,
                durationSec = durationMs / 1000.0
            ) ?: return@withTokenHandling null
            val requestBody = body.toRequestBody("application/json".toMediaType())
            when (action) {
                PlaybackSyncAction.PAUSE -> api.simklScrobblePause(clientId, bearer, body = requestBody)
                PlaybackSyncAction.STOP -> api.simklScrobbleStop(clientId, bearer, body = requestBody)
                PlaybackSyncAction.START, PlaybackSyncAction.PROGRESS ->
                    api.simklScrobbleStart(clientId, bearer, body = requestBody)
            }
        }
    }

    private suspend fun withTokenHandling(
        profile: UserProfile,
        call: suspend (String) -> Response<Unit>?
    ): Boolean {
        val token = profile.simklAccessToken?.takeIf { it.isNotBlank() } ?: return false
        val response = runCatching { call(token) }.getOrNull()
        if (response == null) {
            profileManager.recordExternalSyncFailure(profile.id, id)
            return false
        }
        return when (ExternalSyncPolicy.afterResponse(ExternalSyncProvider.SIMKL, response.code())) {
            ExternalSyncAction.STAMP_SUCCESS -> {
                profileManager.updateProfile(profile.id) { it.copy(simklLastSyncAt = System.currentTimeMillis()) }
                profileManager.clearExternalSyncFailure(profile.id, id)
                true
            }
            ExternalSyncAction.CLEAR_CREDENTIALS -> {
                profileManager.updateProfile(profile.id) { it.copy(simklAccessToken = null) }
                false
            }
            else -> {
                profileManager.recordExternalSyncFailure(profile.id, id)
                false
            }
        }
    }
}
