package com.fluxa.app.data.repository.library

import com.fluxa.app.data.PlatformSecrets
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.ExternalSyncApi
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.remote.TraktScrobbleEpisode
import com.fluxa.app.data.remote.TraktScrobbleRequest
import com.fluxa.app.data.remote.TraktSummary
import com.fluxa.app.data.repository.TraktIntegration
import com.fluxa.app.data.repository.TraktRepository
import javax.inject.Inject

class TraktProviderAdapter @Inject constructor(
    private val traktRepository: TraktRepository,
    private val api: ExternalSyncApi,
    private val profileManager: ProfileManager
) : ProviderAdapter {
    override val providerId = ThirdPartyProviderId.TRAKT
    override val capabilities = setOf(
        ProviderCapability.AUTHENTICATION, ProviderCapability.LIBRARY,
        ProviderCapability.CONTINUE_WATCHING, ProviderCapability.WATCH_HISTORY,
        ProviderCapability.FAVORITES, ProviderCapability.COLLECTION,
        ProviderCapability.PUSH_PROGRESS
    )

    override fun isConnected(profile: UserProfile): Boolean = !profile.traktAccessToken.isNullOrBlank()

    override suspend fun fetchContinueWatching(profile: UserProfile): List<Meta> =
        traktRepository.getTraktContinueWatching(profile, profile.safeLanguage)

    override suspend fun fetchWatchlist(profile: UserProfile): List<Meta> {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return emptyList()
        return traktRepository.getTraktWatchlist(token)
    }

    override suspend fun fetchWatched(profile: UserProfile): List<Meta>? {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return null
        return traktRepository.getTraktRecentlyWatched(token)
    }

    override suspend fun fetchFavorites(profile: UserProfile): List<Meta>? {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return null
        return traktRepository.getTraktFavorites(token)
    }

    override suspend fun fetchCollection(profile: UserProfile): List<Meta>? {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return null
        return traktRepository.getTraktCollection(token)
    }

    override suspend fun fetchWatchedEpisodeTimestamps(profile: UserProfile): Map<String, Long>? {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return null
        return traktRepository.getTraktWatchedEpisodesWithTimestamps(token)
    }

    override suspend fun pushWatchlist(profile: UserProfile, item: Meta, add: Boolean): Boolean {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return false
        val request = TraktIntegration.buildHistoryRequest(item, emptyList()) ?: return false
        val bearer = TraktIntegration.bearer(token)
        return executeWrite(profile) {
            if (add) api.addToWatchlist(bearer, PlatformSecrets.traktClientId, request).isSuccessful
            else api.removeFromWatchlist(bearer, PlatformSecrets.traktClientId, request).isSuccessful
        }
    }

    override suspend fun pushWatched(
        profile: UserProfile,
        item: Meta,
        episodes: List<Video>,
        watched: Boolean
    ): Boolean {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return false
        val request = TraktIntegration.buildHistoryRequest(item, episodes) ?: return false
        val bearer = TraktIntegration.bearer(token)
        return executeWrite(profile) {
            if (watched) api.addToHistory(bearer, PlatformSecrets.traktClientId, request).isSuccessful
            else api.removeFromHistory(bearer, PlatformSecrets.traktClientId, request).isSuccessful
        }
    }

    override suspend fun pushPlaybackProgress(
        profile: UserProfile,
        item: Meta,
        videoId: String?,
        positionMs: Long,
        durationMs: Long,
        action: PlaybackSyncAction
    ): Boolean {
        // Match Fluxa's in-app scrobble semantics: START/PAUSE/STOP are enough for
        // Trakt; periodic resume writes are handled by Nuvio/Stremio.
        if (action == PlaybackSyncAction.PROGRESS) return true
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return false
        if (durationMs <= 0L) return false
        val mediaId = TraktIntegration.scrobbleMediaId(item.id, videoId, item.type)
        val ids = TraktIntegration.idsFromContentId(mediaId) ?: return false
        val progress = (positionMs.toDouble() / durationMs.toDouble() * 100.0).toFloat().coerceIn(0f, 100f)
        val request = if (item.type == "movie") {
            TraktScrobbleRequest(movie = TraktSummary(null, null, ids), progress = progress)
        } else {
            val episode = TraktIntegration.episodeLocator(mediaId) ?: return false
            TraktScrobbleRequest(
                show = TraktSummary(null, null, ids),
                episode = TraktScrobbleEpisode(season = episode.season, number = episode.episode),
                progress = progress
            )
        }
        val bearer = TraktIntegration.bearer(token)
        return executeWrite(profile) {
            when (action) {
                PlaybackSyncAction.PAUSE -> api.scrobblePause(bearer, PlatformSecrets.traktClientId, request).isSuccessful
                PlaybackSyncAction.STOP -> api.scrobbleStop(bearer, PlatformSecrets.traktClientId, request).isSuccessful
                PlaybackSyncAction.START, PlaybackSyncAction.PROGRESS -> api.scrobbleStart(bearer, PlatformSecrets.traktClientId, request).isSuccessful
            }
        }
    }

    override suspend fun pushFavorite(profile: UserProfile, item: Meta, favorite: Boolean): Boolean {
        val token = profile.traktAccessToken?.takeIf(String::isNotBlank) ?: return false
        val request = TraktIntegration.buildHistoryRequest(item, emptyList()) ?: return false
        val bearer = TraktIntegration.bearer(token)
        return executeWrite(profile) {
            if (favorite) api.addToFavorites(bearer, PlatformSecrets.traktClientId, request).isSuccessful
            else api.removeFromFavorites(bearer, PlatformSecrets.traktClientId, request).isSuccessful
        }
    }


    override suspend fun clearPlaybackProgress(profile: UserProfile, item: Meta): Boolean =
        traktRepository.clearPlaybackProgress(profile.traktAccessToken, item)

    private suspend fun executeWrite(
        profile: UserProfile,
        block: suspend () -> Boolean
    ): Boolean = runCatching { block() }
        .onSuccess { success ->
            if (success) profileManager.clearExternalSyncFailure(profile.id, id)
            else profileManager.recordExternalSyncFailure(profile.id, id)
        }
        .onFailure { profileManager.recordExternalSyncFailure(profile.id, id) }
        .getOrDefault(false)

}
