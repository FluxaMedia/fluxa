package com.fluxa.app.data.repository

import com.fluxa.app.data.PlatformSecrets
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.platform.PlatformKeyValueStore
import com.fluxa.app.data.remote.*
import com.fluxa.app.common.AppStrings
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Named
import javax.inject.Singleton
import com.fluxa.app.core.rust.FluxaCoreNative
import com.google.gson.Gson
import com.google.gson.JsonObject

@Singleton
class TraktRepository @Inject constructor(
    @param:Named("TraktSyncDelta") private val syncCache: PlatformKeyValueStore,
    private val externalSyncApi: ExternalSyncApi,
    private val externalLibraryClient: ExternalLibraryClient,
    private val traktSyncClient: TraktSyncClient,
    private val gson: Gson,
    private val oauthClientConfig: OAuthClientConfig
) {
    private val TRAKT_KEY = PlatformSecrets.traktClientId

    private val traktCatalogClient by lazy {
        TraktCatalogClient(
            externalSyncApi = externalSyncApi,
            traktKey = TRAKT_KEY,
            unknownName = { AppStrings.t(null, "auto.unknown") }
        )
    }

    suspend fun getTraktContinueWatching(profile: UserProfile, language: String = "en"): List<Meta> = withContext(Dispatchers.IO) {
        externalLibraryClient.getTraktContinueWatching(profile, language)
    }

    suspend fun getSyncSnapshot(profile: UserProfile, language: String = profile.safeLanguage): TraktSyncSnapshot = withContext(Dispatchers.IO) {
        val token = profile.traktAccessToken ?: return@withContext TraktSyncSnapshot(0, 0)
        if (!TraktIntegration.hasClient(TRAKT_KEY)) return@withContext TraktSyncSnapshot(0, 0)
        val key = "snapshot:${profile.id}"
        val cached = syncCache.read(key)?.let { gson.fromJson(it, TraktSnapshotCache::class.java) }
        val activities = runCatching {
            externalSyncApi.getLastActivities(TraktIntegration.bearer(token), TRAKT_KEY).body()
        }.getOrNull()
        if (cached != null && activities != null) {
            val diff = FluxaCoreNative.traktActivityDiff(
                previous = cached.activities,
                current = activities,
                hasPlayback = true,
                hasWatchlistMovies = true,
                hasWatchlistShows = true,
                hasWatchedMovies = true,
                hasWatchedShows = true
            )
            if (diff["playbackChanged"] != true &&
                diff["watchlistMoviesChanged"] != true &&
                diff["watchlistShowsChanged"] != true
            ) return@withContext cached.snapshot
        }
        val snapshot = externalLibraryClient.getTraktSyncSnapshot(profile, language)
        if (activities != null) {
            syncCache.write(key, gson.toJson(TraktSnapshotCache(activities, snapshot)))
        }
        snapshot
    }

    private data class TraktSnapshotCache(val activities: JsonObject, val snapshot: TraktSyncSnapshot)

    suspend fun getWatchlist(token: String): List<Meta> = withContext(Dispatchers.IO) {
        traktSyncClient.getWatchlist(token)
    }

    suspend fun getWatchlistWithListedAt(token: String): List<Pair<Meta, Long>> = withContext(Dispatchers.IO) {
        traktSyncClient.getWatchlistWithListedAt(token)
    }

    /** Trakt-owned history only. No add-on, TMDB, local, or other-provider enrichment. */
    suspend fun getRecentlyWatched(token: String): List<Meta> = withContext(Dispatchers.IO) {
        if (!TraktIntegration.hasClient(TRAKT_KEY)) return@withContext emptyList()
        runCatching { traktSyncClient.getRecentlyWatched(token) }.getOrDefault(emptyList())
    }

    suspend fun getWatchedEpisodeIds(token: String): Map<String, Set<String>> = withContext(Dispatchers.IO) {
        traktSyncClient.getWatchedEpisodeIds(token)
    }

    suspend fun getWatchedEpisodesWithTimestamps(token: String): Map<String, Long> = withContext(Dispatchers.IO) {
        traktSyncClient.getWatchedEpisodesWithTimestamps(token)
    }

    suspend fun getWatchedState(token: String): TraktWatchedState = withContext(Dispatchers.IO) {
        traktSyncClient.getWatchedState(token)
    }

    suspend fun getCollection(token: String): List<Meta> = withContext(Dispatchers.IO) {
        traktSyncClient.getCollection(token)
    }

    suspend fun getFavorites(token: String): List<Meta> = withContext(Dispatchers.IO) {
        traktSyncClient.getFavorites(token)
    }

    suspend fun getHype(language: String = "en"): List<Meta> = traktCatalogClient.getHype(language)

    suspend fun getTrending(language: String = "en"): List<Meta> = traktCatalogClient.getTrending(language)

    suspend fun getAnticipated(language: String = "en"): List<Meta> = traktCatalogClient.getAnticipated(language)

    suspend fun clearPlaybackProgress(token: String?, meta: Meta): Boolean = withContext(Dispatchers.IO) {
        if (token.isNullOrBlank()) return@withContext false
        if (!TraktIntegration.hasClient(TRAKT_KEY)) return@withContext false
        runCatching {
            val targetKey = TraktIntegration.contentIdentityKey(meta)
            val auth = TraktIntegration.bearer(token)
            val playbackId = externalSyncApi.getPlayback(auth, TRAKT_KEY)
                .firstOrNull { item ->
                    val summary = item.movie ?: item.show ?: return@firstOrNull false
                    val type = if (item.movie != null) "movie" else "series"
                    val id = TraktIntegration.contentIdFrom(summary.ids) ?: return@firstOrNull false
                    TraktIntegration.contentIdentityKey(
                        Meta(
                            id = id,
                            name = summary.title ?: "",
                            type = type,
                            poster = null,
                            releaseInfo = summary.year?.toString()
                        )
                    ) == targetKey
                }
                ?.id
            playbackId == null || externalSyncApi.deletePlayback(playbackId, auth, TRAKT_KEY).isSuccessful
        }.getOrDefault(false)
    }

    suspend fun addToHistory(token: String, request: TraktHistorySyncRequest) = withContext(Dispatchers.IO) {
        externalSyncApi.addToHistory(TraktIntegration.bearer(token), TRAKT_KEY, request)
    }

    suspend fun removeFromHistory(token: String, request: TraktHistorySyncRequest) = withContext(Dispatchers.IO) {
        externalSyncApi.removeFromHistory(TraktIntegration.bearer(token), TRAKT_KEY, request)
    }

    suspend fun getTraktWatchlist(token: String): List<Meta> = getWatchlist(token)

    suspend fun getTraktWatchlistWithListedAt(token: String): List<Pair<Meta, Long>> = getWatchlistWithListedAt(token)

    suspend fun getTraktRecentlyWatched(token: String): List<Meta> = getRecentlyWatched(token)

    suspend fun getTraktCollection(token: String): List<Meta> = getCollection(token)

    suspend fun getTraktFavorites(token: String): List<Meta> = getFavorites(token)

    suspend fun getTraktWatchedState(token: String): TraktWatchedState = getWatchedState(token)

    suspend fun getTraktWatchedEpisodesWithTimestamps(token: String): Map<String, Long> = getWatchedEpisodesWithTimestamps(token)

    suspend fun getTraktSyncSnapshot(profile: UserProfile, language: String = profile.safeLanguage): TraktSyncSnapshot =
        getSyncSnapshot(profile, language)

    suspend fun refreshTraktToken(refreshToken: String): TraktTokenResponse = externalSyncApi.refreshToken(
        TraktRefreshTokenRequest(
            refresh_token = refreshToken,
            client_id = oauthClientConfig.traktClientId,
            client_secret = oauthClientConfig.traktClientSecret.orEmpty(),
            redirect_uri = TraktIntegration.MOBILE_REDIRECT_URI
        )
    )

    suspend fun clearTraktPlaybackProgress(token: String?, meta: Meta): Boolean = clearPlaybackProgress(token, meta)

}
