package com.fluxa.app.data.repository

import android.util.Log
import com.fluxa.app.BuildConfig
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.SimklEpisode
import com.fluxa.app.data.remote.SimklItem
import com.fluxa.app.data.remote.TraktApi
import com.fluxa.app.data.remote.TraktEpisode
import com.fluxa.app.ui.catalog.AppStrings
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

private const val EPISODE_PROGRESS_UNIT_MS = 45 * 60_000L

@Singleton
class ExternalLibraryClient @Inject constructor(
    private val traktApi: TraktApi,
    private val addonRepository: AddonRepository,
    private val traktSyncClient: TraktSyncClient
) {
    private val traktKey = BuildConfig.TRAKT_CLIENT_ID

    private fun unknownName(language: String?): String = AppStrings.t(language, "auto.unknown")

    private suspend fun resolveMetaDetail(type: String, id: String, language: String, profile: UserProfile): MetaDetail? {
        return addonRepository.getAddonMetaDetail(type, id, profile.authKey, profile.safeLocalAddons)
    }
    suspend fun getExternalContinueWatching(profile: UserProfile, language: String = "en"): List<Meta> = withContext(Dispatchers.IO) {
        supervisorScope {
            val trakt = async { getTraktPlaybackItems(profile, language) }
            val mal = async { getMalContinueWatchingItems(profile.malAccessToken) }
            val simkl = async { getSimklContinueWatchingItems(profile.simklAccessToken) }
            (trakt.await() + mal.await() + simkl.await())
                .distinctBy { TraktIntegration.contentIdentityKey(it) }
        }
    }

    suspend fun getTraktSyncSnapshot(profile: UserProfile, language: String = profile.safeLanguage): TraktSyncSnapshot = withContext(Dispatchers.IO) {
        val token = profile.traktAccessToken
        if (token.isNullOrBlank() || !TraktIntegration.hasClient(traktKey)) {
            return@withContext TraktSyncSnapshot(0, 0)
        }
        supervisorScope {
            val continueWatching = async { getTraktPlaybackItems(profile, language).size }
            val watchlist = async { traktSyncClient.getWatchlist(token).size }
            TraktSyncSnapshot(
                continueWatchingCount = continueWatching.await(),
                watchlistCount = watchlist.await()
            )
        }
    }

    suspend fun getMalLibraryItems(token: String?, status: String): List<Meta> = withContext(Dispatchers.IO) {
        if (token.isNullOrBlank()) return@withContext emptyList()
        runCatching {
            traktApi.getMalAnimeList("Bearer $token", status = status)
                .data
                .map { entry ->
                    Meta(
                        id = "mal:${entry.node.id}",
                        name = entry.node.title,
                        type = "series",
                        poster = null,
                        releaseInfo = entry.node.numEpisodes?.takeIf { it > 0 }?.let { "$it episodes" },
                        reason = "MyAnimeList"
                    )
                }
                .distinctBy { it.id }
        }.getOrDefault(emptyList())
    }

    suspend fun getSimklLibraryItems(token: String?, status: String): List<Meta> = withContext(Dispatchers.IO) {
        if (token.isNullOrBlank() || BuildConfig.SIMKL_CLIENT_ID.isBlank()) return@withContext emptyList()
        val types = listOf("movies" to "movie", "shows" to "series", "anime" to "series")
        supervisorScope {
            types.map { (apiType, metaType) ->
                async {
                    runCatching {
                        val response = traktApi.getSimklAllItems(
                            type = apiType,
                            status = status,
                            token = "Bearer $token",
                            apiKey = BuildConfig.SIMKL_CLIENT_ID
                        )
                        val items = when (apiType) {
                            "movies" -> response.movies
                            "anime" -> response.anime
                            else -> response.shows
                        }
                        items.mapNotNull { it.toLibraryMeta(metaType, "Simkl") }
                    }.getOrDefault(emptyList())
                }
            }.awaitAll().flatten().distinctBy { TraktIntegration.contentIdentityKey(it) }
        }
    }

    private suspend fun getTraktPlaybackItems(profile: UserProfile, language: String): List<Meta> {
        val token = profile.traktAccessToken
        if (token.isNullOrBlank()) return emptyList()
        if (!TraktIntegration.hasClient(traktKey)) return emptyList()
        return try {
            val rawItems = traktApi.getPlayback(TraktIntegration.bearer(token), traktKey)
                .filter { item ->
                    val summary = item.movie ?: item.show ?: return@filter false
                    val progress = item.progress?.coerceIn(0f, 100f) ?: return@filter false
                    TraktIntegration.contentIdFrom(summary.ids) != null && progress > 0f && progress < 95f
                }
            if (rawItems.isEmpty()) return emptyList()
            val semaphore = Semaphore(4)
            supervisorScope {
                rawItems.map { item ->
                    async {
                        runCatching {
                            semaphore.withPermit {
                                val summary = item.movie ?: item.show ?: return@withPermit null
                                val type = if (item.movie != null) "movie" else "series"
                                val id = TraktIntegration.contentIdFrom(summary.ids) ?: return@withPermit null
                                val progress = item.progress?.coerceIn(0f, 100f) ?: return@withPermit null
                                val lastVideoId = if (type == "series" && item.episode != null) "$id:${item.episode.season}:${item.episode.number}" else null
                                val detail = resolveMetaDetail(type, id, language, profile)
                                val duration = estimatedDurationMs(type, detail)
                                val episodeArtwork = findEpisodeArtwork(detail, lastVideoId, item.episode)
                                Meta(
                                    id = id,
                                    name = detail?.name ?: summary.title ?: unknownName(language),
                                    type = type,
                                    poster = detail?.poster,
                                    background = detail?.background,
                                    logo = detail?.logo,
                                    description = detail?.description,
                                    releaseInfo = detail?.releaseInfo ?: summary.year?.toString(),
                                    released = detail?.released ?: summary.year?.let { "$it-01-01" },
                                    timeOffset = ((duration * progress) / 100f).toLong(),
                                    duration = duration,
                                    lastVideoId = lastVideoId,
                                    lastEpisodeName = item.episode?.title,
                                    reason = "Trakt.tv",
                                    continueWatchingPoster = if (type == "series") episodeArtwork else detail?.poster,
                                    continueWatchingBackground = if (type == "series") episodeArtwork else detail?.background
                                )
                            }
                        }.getOrNull()
                    }
                }.awaitAll().filterNotNull()
            }
        } catch (e: Exception) {
            Log.w("ExternalLibraryClient", "Failed to load Trakt playback items", e)
            emptyList()
        }
    }

    private suspend fun getMalContinueWatchingItems(token: String?): List<Meta> {
        if (token.isNullOrBlank()) return emptyList()
        return try {
            traktApi.getMalAnimeList("Bearer $token")
                .data
                .mapNotNull { entry ->
                    val watched = entry.listStatus?.numEpisodesWatched ?: return@mapNotNull null
                    val total = entry.node.numEpisodes?.takeIf { it > 0 } ?: (watched + 1)
                    if (watched <= 0 || watched >= total) return@mapNotNull null
                    Meta(
                        id = "mal:${entry.node.id}",
                        name = entry.node.title,
                        type = "series",
                        poster = null,
                        timeOffset = watched * EPISODE_PROGRESS_UNIT_MS,
                        duration = total * EPISODE_PROGRESS_UNIT_MS,
                        lastVideoId = "mal:${entry.node.id}:1:${watched + 1}",
                        lastEpisodeName = "Episode ${watched + 1}",
                        reason = "MyAnimeList"
                    )
                }
        } catch (e: Exception) {
            Log.w("ExternalLibraryClient", "Failed to load MyAnimeList continue watching items", e)
            emptyList()
        }
    }

    private suspend fun getSimklContinueWatchingItems(token: String?): List<Meta> {
        if (token.isNullOrBlank() || BuildConfig.SIMKL_CLIENT_ID.isBlank()) return emptyList()
        return try {
            val types = listOf("movies" to "movie", "shows" to "series", "anime" to "series")
            types.flatMap { (apiType, metaType) ->
                runCatching {
                    val response = traktApi.getSimklAllItems(
                        type = apiType,
                        status = "watching",
                        token = "Bearer $token",
                        apiKey = BuildConfig.SIMKL_CLIENT_ID
                    )
                    val items = when (apiType) {
                        "movies" -> response.movies
                        "anime" -> response.anime
                        else -> response.shows
                    }
                    items.mapNotNull { it.toContinueMeta(metaType) }
                }.getOrDefault(emptyList())
            }
        } catch (e: Exception) {
            Log.w("ExternalLibraryClient", "Failed to load Simkl continue watching items", e)
            emptyList()
        }
    }

    private fun SimklItem.toLibraryMeta(type: String, source: String): Meta? {
        val id = ids?.imdb ?: ids?.tmdb ?: ids?.slug?.let { "simkl:$it" } ?: ids?.simkl?.let { "simkl:$it" } ?: return null
        return Meta(
            id = id,
            name = title ?: unknownName(null),
            type = type,
            poster = null,
            releaseInfo = year?.toString(),
            released = year?.let { "$it-01-01" },
            reason = source
        )
    }

    private fun SimklItem.toContinueMeta(type: String): Meta? {
        val id = ids?.imdb ?: ids?.tmdb ?: ids?.slug?.let { "simkl:$it" } ?: ids?.simkl?.let { "simkl:$it" } ?: return null
        val latestEpisode = seasons.orEmpty()
            .flatMap { season -> season.episodes.orEmpty().map { season.number to it } }
            .filter { (_, episode) -> !episode.watchedAt.isNullOrBlank() || (episode.number ?: 0) > 0 }
            .maxWithOrNull(compareBy<Pair<Int?, SimklEpisode>>({ it.first ?: 0 }, { it.second.number ?: 0 }))
        val watchedCount = seasons.orEmpty().sumOf { it.episodes.orEmpty().size }.takeIf { it > 0 } ?: 1
        val totalUnits = (watchedCount + 1).coerceAtLeast(2)
        return Meta(
            id = id,
            name = title ?: unknownName(null),
            type = type,
            poster = null,
            releaseInfo = year?.toString(),
            released = year?.let { "$it-01-01" },
            timeOffset = watchedCount * EPISODE_PROGRESS_UNIT_MS,
            duration = totalUnits * EPISODE_PROGRESS_UNIT_MS,
            lastVideoId = latestEpisode?.let { "$id:${it.first ?: 1}:${(it.second.number ?: watchedCount) + 1}" },
            lastEpisodeName = latestEpisode?.second?.number?.let { "Episode ${it + 1}" },
            reason = "Simkl"
        )
    }

    private fun findEpisodeArtwork(detail: MetaDetail?, lastVideoId: String?, episode: TraktEpisode?): String? {
        val videos = detail?.videos.orEmpty()
        if (videos.isEmpty()) return null
        lastVideoId?.let { videoId ->
            videos.firstOrNull { it.id == videoId }?.thumbnail?.let { return it }
        }
        val season = episode?.season ?: return null
        val number = episode.number
        return videos.firstOrNull {
            it.season == season && it.number == number
        }?.thumbnail
    }

    private fun estimatedDurationMs(type: String, detail: MetaDetail? = null): Long {
        return parseRuntimeMinutes(detail?.runtime)?.takeIf { it > 0 }?.let { it * 60_000L }
            ?: if (type == "movie") 120 * 60_000L else EPISODE_PROGRESS_UNIT_MS
    }

    private fun parseRuntimeMinutes(value: String?): Long? {
        if (value.isNullOrBlank()) return null
        val hours = Regex("""(\d+)\s*h""", RegexOption.IGNORE_CASE).find(value)?.groupValues?.getOrNull(1)?.toLongOrNull() ?: 0L
        val minutes = Regex("""(\d+)\s*m""", RegexOption.IGNORE_CASE).find(value)?.groupValues?.getOrNull(1)?.toLongOrNull()
            ?: Regex("""(\d+)""").find(value)?.groupValues?.getOrNull(1)?.toLongOrNull()
        return (hours * 60L + (minutes ?: 0L)).takeIf { it > 0 }
    }
}
