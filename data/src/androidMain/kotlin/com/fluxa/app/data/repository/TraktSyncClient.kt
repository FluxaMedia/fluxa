package com.fluxa.app.data.repository

import com.fluxa.app.data.BuildConfig
import com.fluxa.app.data.local.WatchedContentDurationRecord
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.TraktApi
import com.fluxa.app.common.AppStrings
import com.fluxa.app.domain.ContentIdentity
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class TraktSyncClient @Inject constructor(
    private val traktApi: TraktApi,
    private val failureReporter: DataFailureReporter
) {
    private val traktKey = BuildConfig.TRAKT_CLIENT_ID

    suspend fun getWatchlist(token: String): List<Meta> = withContext(Dispatchers.IO) {
        getWatchlistResult(token).getOrReport(emptyList())
    }

    suspend fun getWatchlistResult(token: String): DataResult<List<Meta>> = withContext(Dispatchers.IO) {
        if (!TraktIntegration.hasClient(traktKey)) return@withContext DataResult.AuthUnavailable("trakt.watchlist")
        try {
            val auth = TraktIntegration.bearer(token)
            DataResult.Success(fetchTraktSyncPages { page, limit -> traktApi.getWatchlist(auth, traktKey, page, limit) }
                .mapNotNull { item ->
                    val type = if (item.movie != null) "movie" else "series"
                    item.toMeta(type) { AppStrings.t(null, "auto.unknown") }
                })
        } catch (e: Exception) {
            DataResult.NetworkError("trakt.watchlist", e)
        }
    }

    suspend fun getRecentlyWatched(token: String): List<Meta> = withContext(Dispatchers.IO) {
        getRecentlyWatchedResult(token).getOrReport(emptyList())
    }

    suspend fun getRecentlyWatchedResult(token: String): DataResult<List<Meta>> = withContext(Dispatchers.IO) {
        if (!TraktIntegration.hasClient(traktKey)) return@withContext DataResult.AuthUnavailable("trakt.recentlyWatched")
        try {
            val auth = TraktIntegration.bearer(token)
            val movies = async { runCatching { fetchTraktSyncPages { page, limit -> traktApi.getWatchedMovies(auth, traktKey, page, limit) } }.getOrDefault(emptyList()) }
            val shows = async { runCatching { fetchTraktSyncPages { page, limit -> traktApi.getWatchedShows(auth, traktKey, page, limit) } }.getOrDefault(emptyList()) }
            DataResult.Success((movies.await().mapNotNull { it.toMeta("movie") { AppStrings.t(null, "auto.unknown") } } +
                shows.await().mapNotNull { it.toMeta("series") { AppStrings.t(null, "auto.unknown") } })
                .distinctBy { it.id }
                .take(40))
        } catch (e: Exception) {
            DataResult.NetworkError("trakt.recentlyWatched", e)
        }
    }

    suspend fun getWatchedEpisodeIds(token: String): Map<String, Set<String>> = withContext(Dispatchers.IO) {
        getWatchedEpisodeIdsResult(token).getOrReport(emptyMap())
    }

    suspend fun getWatchedEpisodeIdsResult(token: String): DataResult<Map<String, Set<String>>> = withContext(Dispatchers.IO) {
        if (!TraktIntegration.hasClient(traktKey)) return@withContext DataResult.AuthUnavailable("trakt.watchedEpisodeIds")
        try {
            val auth = TraktIntegration.bearer(token)
            DataResult.Success(fetchTraktSyncPages { page, limit -> traktApi.getWatchedShows(auth, traktKey, page, limit) }
                .mapNotNull { item ->
                    val show = item.show ?: return@mapNotNull null
                    val seriesId = TraktIntegration.contentIdFrom(show.ids) ?: return@mapNotNull null
                    val episodeIds = item.seasons.orEmpty().flatMap { season ->
                        val seasonNumber = season.number ?: return@flatMap emptyList()
                        season.episodes.orEmpty().mapNotNull { episode ->
                            val episodeNumber = episode.number ?: return@mapNotNull null
                            "$seriesId:$seasonNumber:$episodeNumber"
                        }
                    }.toSet()
                    if (episodeIds.isEmpty()) null else seriesId to episodeIds
                }
                .toMap())
        } catch (e: Exception) {
            DataResult.NetworkError("trakt.watchedEpisodeIds", e)
        }
    }

    suspend fun getWatchedState(token: String): TraktWatchedState = withContext(Dispatchers.IO) {
        getWatchedStateResult(token).getOrReport(TraktWatchedState())
    }

    suspend fun getWatchedStateResult(token: String): DataResult<TraktWatchedState> = withContext(Dispatchers.IO) {
        if (!TraktIntegration.hasClient(traktKey)) return@withContext DataResult.AuthUnavailable("trakt.watchedState")
        try {
            val auth = TraktIntegration.bearer(token)
            val movies = async { fetchTraktSyncPages { page, limit -> traktApi.getWatchedMovies(auth, traktKey, page, limit, "full") } }
            val shows = async { fetchTraktSyncPages { page, limit -> traktApi.getWatchedShows(auth, traktKey, page, limit, "full") } }
            val movieItems = movies.await()
            val showItems = shows.await()

            val movieMetas = movieItems.mapNotNull { it.toMeta("movie") { AppStrings.t(null, "auto.unknown") } }
            val movieKeys = ContentIdentity.watchedKeysBatch(movieMetas).flatten().toSet()

            val durationRecords = mutableListOf<WatchedContentDurationRecord>()
            movieItems.forEach { item ->
                val movie = item.movie ?: return@forEach
                val contentId = TraktIntegration.contentIdFrom(movie.ids) ?: return@forEach
                val duration = movie.runtime.minutesToMsOrZero()
                if (duration > 0L) {
                    durationRecords += WatchedContentDurationRecord(
                        contentId = contentId,
                        videoId = contentId,
                        duration = duration
                    )
                }
            }

            val showEntries = showItems.mapNotNull { item ->
                val show = item.show ?: return@mapNotNull null
                val seriesId = TraktIntegration.contentIdFrom(show.ids) ?: return@mapNotNull null
                val showMeta = Meta(
                    id = seriesId,
                    name = show.title ?: AppStrings.t(null, "auto.unknown"),
                    type = "series",
                    poster = null,
                    releaseInfo = show.year?.toString()
                )
                Triple(item, seriesId, showMeta)
            }
            val showKeysBatch = ContentIdentity.watchedKeysBatch(showEntries.map { it.third })

            val episodeIdsBySeries = linkedMapOf<String, Set<String>>()
            val episodeKeys = showEntries.flatMapIndexed { index, (item, seriesId, _) ->
                val showKeys = showKeysBatch[index]
                val episodeIds = item.seasons.orEmpty().flatMap { season ->
                    val seasonNumber = season.number ?: return@flatMap emptyList()
                    season.episodes.orEmpty().mapNotNull { episode ->
                        val episodeNumber = episode.number ?: return@mapNotNull null
                        val duration = episode.runtime.minutesToMsOrZero()
                        if (duration > 0L) {
                            durationRecords += WatchedContentDurationRecord(
                                contentId = seriesId,
                                videoId = "$seriesId:$seasonNumber:$episodeNumber",
                                duration = duration
                            )
                        }
                        "$seriesId:$seasonNumber:$episodeNumber"
                    }
                }.toSet()
                if (episodeIds.isNotEmpty()) episodeIdsBySeries[seriesId] = episodeIds
                item.seasons.orEmpty().flatMap { season ->
                    val seasonNumber = season.number ?: return@flatMap emptyList()
                    season.episodes.orEmpty().flatMap { episode ->
                        val episodeNumber = episode.number ?: return@flatMap emptyList()
                        showKeys.map { key -> traktWatchedEpisodeKey(key, seasonNumber, episodeNumber) }
                    }
                }
            }.toSet()

            DataResult.Success(TraktWatchedState(
                movieKeys = movieKeys,
                episodeKeys = episodeKeys,
                episodeIdsBySeries = episodeIdsBySeries,
                durationRecords = durationRecords.distinctBy { "${it.contentId}|${it.videoId}" }
            ))
        } catch (e: Exception) {
            DataResult.NetworkError("trakt.watchedState", e)
        }
    }

    private fun Int?.minutesToMsOrZero(): Long {
        return this?.takeIf { it > 0 }?.toLong()?.times(60_000L) ?: 0L
    }

    suspend fun getCollection(token: String): List<Meta> = withContext(Dispatchers.IO) {
        getCollectionResult(token).getOrReport(emptyList())
    }

    suspend fun getCollectionResult(token: String): DataResult<List<Meta>> = withContext(Dispatchers.IO) {
        if (!TraktIntegration.hasClient(traktKey)) return@withContext DataResult.AuthUnavailable("trakt.collection")
        try {
            val auth = TraktIntegration.bearer(token)
            val movies = async { runCatching { fetchTraktSyncPages { page, limit -> traktApi.getMovieCollection(auth, traktKey, page, limit) } }.getOrDefault(emptyList()) }
            val shows = async { runCatching { fetchTraktSyncPages { page, limit -> traktApi.getShowCollection(auth, traktKey, page, limit) } }.getOrDefault(emptyList()) }
            DataResult.Success((movies.await().mapNotNull { it.toMeta("movie") { AppStrings.t(null, "auto.unknown") } } +
                shows.await().mapNotNull { it.toMeta("series") { AppStrings.t(null, "auto.unknown") } })
                .distinctBy { it.id })
        } catch (e: Exception) {
            DataResult.NetworkError("trakt.collection", e)
        }
    }

    private fun <T> DataResult<T>.getOrReport(defaultValue: T): T {
        asFailure()?.let(failureReporter::report)
        return getOrDefault(defaultValue)
    }
}
