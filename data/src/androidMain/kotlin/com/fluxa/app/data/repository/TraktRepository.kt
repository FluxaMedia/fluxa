package com.fluxa.app.data.repository

import com.fluxa.app.data.BuildConfig
import android.content.Context
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.local.safeLocalAddons
import com.fluxa.app.data.remote.*
import com.fluxa.app.common.AppStrings
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton
import dagger.hilt.android.qualifiers.ApplicationContext
import com.fluxa.app.core.rust.FluxaCoreNative
import com.google.gson.Gson
import com.google.gson.JsonObject

@Singleton
class TraktRepository @Inject constructor(
    @ApplicationContext context: Context,
    private val externalSyncApi: ExternalSyncApi,
    private val addonRepository: AddonRepository,
    private val externalLibraryClient: ExternalLibraryClient,
    private val traktSyncClient: TraktSyncClient,
    private val gson: Gson,
    private val oauthClientConfig: OAuthClientConfig
) {
    private val TRAKT_KEY = BuildConfig.TRAKT_CLIENT_ID
    private val MAX_CONCURRENT_TRAKT_DETAIL_RESOLUTION = 6
    private val syncCache = context.getSharedPreferences("trakt_sync_delta", Context.MODE_PRIVATE)

    private val traktCatalogClient by lazy {
        TraktCatalogClient(
            externalSyncApi = externalSyncApi,
            traktKey = TRAKT_KEY,
            unknownName = { AppStrings.t(null, "auto.unknown") }
        )
    }

    suspend fun getExternalContinueWatching(profile: UserProfile, language: String = "en"): List<Meta> = withContext(Dispatchers.IO) {
        externalLibraryClient.getExternalContinueWatching(profile, language)
    }

    suspend fun getSyncSnapshot(profile: UserProfile, language: String = profile.safeLanguage): TraktSyncSnapshot = withContext(Dispatchers.IO) {
        val token = profile.traktAccessToken ?: return@withContext TraktSyncSnapshot(0, 0)
        if (!TraktIntegration.hasClient(TRAKT_KEY)) return@withContext TraktSyncSnapshot(0, 0)
        val key = "snapshot:${profile.id}"
        val cached = syncCache.getString(key, null)?.let { gson.fromJson(it, TraktSnapshotCache::class.java) }
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
            syncCache.edit().putString(key, gson.toJson(TraktSnapshotCache(activities, snapshot))).apply()
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

    suspend fun getRecentlyWatched(token: String, language: String = "en", profile: UserProfile? = null): List<Meta> = withContext(Dispatchers.IO) {
        if (!TraktIntegration.hasClient(TRAKT_KEY)) return@withContext emptyList()
        try {
            val metas = traktSyncClient.getRecentlyWatched(token)
            if (profile == null) {
                metas
            } else {
                val detailSemaphore = Semaphore(MAX_CONCURRENT_TRAKT_DETAIL_RESOLUTION)
                metas.map { meta ->
                    async {
                        detailSemaphore.withPermit {
                            val detail = runCatching { addonRepository.getAddonMetaDetail(meta.type, meta.id, profile.authKey, profile.safeLocalAddons) }.getOrNull()
                            if (detail == null) meta else meta.copy(
                                poster = detail.poster,
                                background = detail.background,
                                logo = detail.logo,
                                description = detail.description
                            )
                        }
                    }
                }
                    .awaitAll()
            }
        } catch(e: Exception) { emptyList() }
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

    suspend fun clearPlaybackProgress(token: String?, meta: Meta) = withContext(Dispatchers.IO) {
        if (token.isNullOrBlank()) return@withContext
        if (!TraktIntegration.hasClient(TRAKT_KEY)) return@withContext
        runCatching {
            val targetKey = TraktIntegration.contentIdentityKey(meta)
            val auth = TraktIntegration.bearer(token)
            externalSyncApi.getPlayback(auth, TRAKT_KEY)
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
                ?.let { externalSyncApi.deletePlayback(it, auth, TRAKT_KEY) }
        }
    }

    suspend fun addToHistory(token: String, request: TraktHistorySyncRequest) = withContext(Dispatchers.IO) {
        externalSyncApi.addToHistory(TraktIntegration.bearer(token), TRAKT_KEY, request)
    }

    suspend fun removeFromHistory(token: String, request: TraktHistorySyncRequest) = withContext(Dispatchers.IO) {
        externalSyncApi.removeFromHistory(TraktIntegration.bearer(token), TRAKT_KEY, request)
    }

    suspend fun getTraktWatchlist(token: String): List<Meta> = getWatchlist(token)

    suspend fun getTraktWatchlistWithListedAt(token: String): List<Pair<Meta, Long>> = getWatchlistWithListedAt(token)

    suspend fun getTraktRecentlyWatched(token: String, language: String = "en", profile: UserProfile? = null): List<Meta> =
        getRecentlyWatched(token, language, profile)

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

    suspend fun clearTraktPlaybackProgress(token: String?, meta: Meta) = clearPlaybackProgress(token, meta)

}
