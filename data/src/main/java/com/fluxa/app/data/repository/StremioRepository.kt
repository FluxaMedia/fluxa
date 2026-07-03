package com.fluxa.app.data.repository

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.domain.discovery.*

import android.util.Log
import com.fluxa.app.data.BuildConfig
import kotlinx.coroutines.*
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class StremioRepository @Inject constructor(
    private val authService: StremioService, 
    private val introService: IntroDbService, 
    private val aniSkipService: AniSkipService,
    private val traktApi: TraktApi,
    private val addonManifestClient: StremioAddonManifestClient,
    private val addonResourceClient: StremioAddonResourceClient,
    private val externalLibraryClient: ExternalLibraryClient,
    private val oauthClient: ExternalOAuthClient,
    private val addonRepository: AddonRepository,
    private val tmdbRepository: TmdbRepository,
    private val traktRepository: TraktRepository,
    private val failureReporter: DataFailureReporter
) {
    private val introRepository = IntroRepository(introService, aniSkipService)

    private val httpClient get() = StremioService.sharedClient

    suspend fun getIntro(imdbId: String, season: Int, episode: Int, title: String? = null, useIntroDb: Boolean = true, useAniSkip: Boolean = true): List<IntroTimestamps> {
        return introRepository.getIntro(imdbId, season, episode, title, useIntroDb, useAniSkip)
    }

    suspend fun submitIntroSegment(
        apiKey: String,
        segmentType: String,
        imdbId: String,
        season: Int,
        episode: Int,
        startSec: Double,
        endSec: Double
    ): IntroDbSubmitResult {
        return introRepository.submitSegment(apiKey, segmentType, imdbId, season, episode, startSec, endSec)
    }

    suspend fun login(request: LoginRequest) = withContext(Dispatchers.IO) { authService.login(request) }

    suspend fun getUserAddons(
        authKey: String,
        localAddons: List<String>? = emptyList(),
        forceRefresh: Boolean = false
    ): List<AddonDescriptor> = addonResourceClient.getUserAddons(authKey, localAddons, forceRefresh)

    suspend fun getMetaDetail(
        type: String,
        id: String,
        language: String = "en",
        authKey: String = "",
        localAddons: List<String>? = emptyList(),
        useConfiguredAddons: Boolean = false,
        preferredAddonTransportUrl: String? = null,
        preferredAddonCatalogType: String? = null
    ): MetaDetail? {
        if (preferredAddonTransportUrl?.isNotBlank() == true) {
            val alternateTypes = buildList {
                preferredAddonCatalogType?.takeIf { it.isNotBlank() }?.let { add(it) }
                when (type) {
                    "movie" -> add("series")
                    "series" -> add("movie")
                }
            }
            val specific = addonRepository.getMetaDetailFromSpecificAddon(
                transportUrl = preferredAddonTransportUrl,
                type = type,
                id = id,
                alternateTypes = alternateTypes
            )
            Log.d("MetaFetch", "getMetaDetail specificAddon($preferredAddonTransportUrl): ${if (specific != null) "OK name=${specific.name}" else "NULL"}")
            return specific
        }
        val plan = FluxaCoreNative.repositoryMetaDetailPlan(useConfiguredAddons, authKey, localAddons)
        Log.d("MetaFetch", "getMetaDetail plan: preferAddon=${plan.preferAddonMetaDetail} fallbackStremio=${plan.fallbackToStremioMetaDetail} localAddons=${localAddons?.size}")
        if (plan.preferAddonMetaDetail) {
            val fromAddon = addonRepository.getAddonMetaDetail(type, id, authKey, localAddons)
            Log.d("MetaFetch", "getMetaDetail fromAddon: ${if (fromAddon != null) "OK name=${fromAddon.name}" else "NULL"}")
            fromAddon?.let { return it }
        }
        return if (plan.fallbackToStremioMetaDetail) authService.getMetaDetail(type, id).meta else null
    }

    suspend fun getAddonMetaDetail(
        type: String,
        id: String,
        authKey: String,
        localAddons: List<String>? = emptyList()
    ): MetaDetail? = addonRepository.getAddonMetaDetail(type, id, authKey, localAddons)

    suspend fun getSimilar(type: String, id: String, language: String = "en"): List<Meta> =
        tmdbRepository.getSimilar(type, id, language)

    suspend fun getTmdbTrailers(type: String, id: String, language: String = "en", apiKey: String): List<DetailTrailer> =
        tmdbRepository.getTrailers(type, id, language, apiKey)

    suspend fun enrichDetailWithTmdb(
        detail: MetaDetail,
        apiKey: String,
        profile: com.fluxa.app.data.local.UserProfile,
        language: String
    ): MetaDetail = tmdbRepository.enrichDetail(detail, apiKey, profile, language)

    suspend fun enrichSeasonEpisodesWithTmdb(
        tmdbId: String,
        seasonNumber: Int,
        episodes: List<Video>,
        apiKey: String,
        language: String
    ): List<Video> = tmdbRepository.enrichSeasonEpisodes(tmdbId, seasonNumber, episodes, apiKey, language)

    suspend fun getTvSeason(
        id: String,
        seasonNumber: Int,
        language: String = "en",
        authKey: String = "",
        localAddons: List<String>? = emptyList(),
        useConfiguredAddons: Boolean = false
    ): List<Video> = FluxaCoreNative.repositorySeasonVideos(
        getMetaDetail("series", id, language, authKey, localAddons, useConfiguredAddons),
        seasonNumber
    )

    suspend fun getSubtitlesFromAddon(baseUrl: String, type: String, id: String, extra: String = ""): List<SubtitleData> =
        addonResourceClient.getSubtitlesFromAddon(baseUrl, type, id, extra)

    suspend fun getStreamsFromAddon(addonTransportUrl: String, addonName: String, type: String, id: String): List<Stream> =
        addonResourceClient.getStreamsFromAddon(addonTransportUrl, addonName, type, id)

    suspend fun getAddonCatalog(
        transportUrl: String,
        type: String,
        id: String,
        skip: Int = 0,
        genre: String? = null,
        search: String? = null
    ): List<Meta> = addonResourceClient.getAddonCatalog(transportUrl, type, id, skip, genre, search)

    suspend fun getAddonManifest(
        transportUrl: String,
        forceRefresh: Boolean = false
    ): AddonDescriptor? = addonManifestClient.getAddonManifest(transportUrl, forceRefresh)

    suspend fun exchangeTraktCode(code: String): TraktTokenResponse = oauthClient.exchangeTraktCode(code)

    suspend fun refreshTraktToken(refreshToken: String): TraktTokenResponse = oauthClient.refreshTraktToken(refreshToken)

    suspend fun createTraktDeviceCode(): TraktDeviceCodeResponse = oauthClient.createTraktDeviceCode()

    suspend fun exchangeTraktDeviceCode(deviceCode: String): retrofit2.Response<TraktTokenResponse> =
        oauthClient.exchangeTraktDeviceCode(deviceCode)

    suspend fun exchangeMalCode(code: String, codeVerifier: String): ExternalOAuthTokenResponse =
        oauthClient.exchangeMalCode(code, codeVerifier)

    suspend fun refreshMalToken(refreshToken: String): ExternalOAuthTokenResponse =
        oauthClient.refreshMalToken(refreshToken)

    suspend fun exchangeSimklCode(code: String): ExternalOAuthTokenResponse = oauthClient.exchangeSimklCode(code)

    suspend fun getExternalContinueWatching(profile: UserProfile, language: String = profile.safeLanguage): List<Meta> =
        externalLibraryClient.getExternalContinueWatching(profile, language)

    suspend fun getMalLibraryItems(token: String?, status: String): List<Meta> =
        externalLibraryClient.getMalLibraryItems(token, status)

    suspend fun getSimklLibraryItems(token: String?, status: String): List<Meta> =
        externalLibraryClient.getSimklLibraryItems(token, status)

    suspend fun clearTraktPlaybackProgress(token: String?, meta: Meta) =
        traktRepository.clearPlaybackProgress(token, meta)

    suspend fun getLibraryItems(authKey: String): List<Meta> = withContext(Dispatchers.IO) {
        try {
            val response = authService.getDatastore(DatastoreRequest(authKey, "library"))
            FluxaCoreNative.libraryContinueWatchingItems(response.body()?.result.orEmpty())
        } catch (e: Exception) {
            failureReporter.report("stremio.library.getItems", e)
            emptyList()
        }
    }

    suspend fun getWatchedVideoIds(authKey: String, imdbId: String): List<String> = withContext(Dispatchers.IO) {
        try {
            val response = authService.getDatastore(DatastoreRequest(authKey, "library"))
            FluxaCoreNative.watchedVideoIds(response.body()?.result.orEmpty(), imdbId)
        } catch (e: Exception) {
            failureReporter.report("stremio.library.getWatchedVideoIds", e)
            emptyList()
        }
    }

    suspend fun savePlaybackProgress(authKey: String, meta: Meta, timeOffset: Long, duration: Long) = withContext(Dispatchers.IO) {
        try {
            FluxaCoreNative.playbackProgressItem(meta, timeOffset, duration, utcNow())?.let { item ->
                authService.datastorePut(DatastorePutRequest(authKey, "library", listOf(item)))
            }
        } catch (e: Exception) {
            failureReporter.report("stremio.library.savePlaybackProgress", e)
            Log.w("StremioRepository", "Failed to save playback progress for ${meta.id}", e)
        }
    }

    suspend fun clearPlaybackProgress(authKey: String, meta: Meta) = withContext(Dispatchers.IO) {
        try {
            FluxaCoreNative.clearPlaybackProgressItem(meta)?.let { item ->
                authService.datastorePut(DatastorePutRequest(authKey, "library", listOf(item)))
            }
        } catch (e: Exception) {
            failureReporter.report("stremio.library.clearPlaybackProgress", e)
            Log.w("StremioRepository", "Failed to clear playback progress for ${meta.id}", e)
        }
    }

    suspend fun syncWatchedState(
        authKey: String?,
        traktToken: String?,
        meta: Meta,
        episodes: List<Video> = emptyList(),
        watched: Boolean = true
    ) = withContext(Dispatchers.IO) {
        val watchedAt = if (watched) utcNow() else null
        if (!authKey.isNullOrBlank()) {
            runCatching {
                val items = FluxaCoreNative.watchedStateItems(meta, episodes, watched, watchedAt)
                if (items.isNotEmpty()) authService.datastorePut(DatastorePutRequest(authKey, "library", items))
            }
        }

        if (!traktToken.isNullOrBlank() && TraktIntegration.hasClient(BuildConfig.TRAKT_CLIENT_ID)) {
            val request = FluxaCoreNative.traktHistoryRequest(meta, episodes)
            if (request != null) {
                runCatching {
                    if (watched) {
                        traktApi.addToHistory(TraktIntegration.bearer(traktToken), BuildConfig.TRAKT_CLIENT_ID, request)
                    } else {
                        traktApi.removeFromHistory(TraktIntegration.bearer(traktToken), BuildConfig.TRAKT_CLIENT_ID, request)
                    }
                }
            }
        }
    }

    private fun utcNow(): String {
        return SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss.SSS'Z'", Locale.US)
            .apply { timeZone = java.util.TimeZone.getTimeZone("UTC") }
            .format(Date())
    }
}
