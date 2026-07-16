package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.TraktApi
import com.fluxa.app.data.remote.TraktSummary
import com.fluxa.app.common.AppStrings
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.withContext

internal class TraktCatalogClient(
    private val traktApi: TraktApi,
    private val traktKey: String,
    private val unknownName: () -> String
) {
    suspend fun getHype(language: String = "en"): List<Meta> = withContext(Dispatchers.IO) {
        if (!TraktIntegration.hasClient(traktKey)) return@withContext emptyList()
        try {
            val trendingMovies = async { try { traktApi.getTrendingMovies(traktKey) } catch(e:Exception){ emptyList() } }
            val trendingShows = async { try { traktApi.getTrendingShows(traktKey) } catch(e:Exception){ emptyList() } }
            val anticipatedMovies = async { try { traktApi.getAnticipatedMovies(traktKey) } catch(e:Exception){ emptyList() } }
            val anticipatedShows = async { try { traktApi.getAnticipatedShows(traktKey) } catch(e:Exception){ emptyList() } }
            val allItems = mutableListOf<Pair<TraktSummary?, String>>()
            allItems.addAll(trendingMovies.await().map { it.movie to "movie" })
            allItems.addAll(trendingShows.await().map { it.show to "series" })
            allItems.addAll(anticipatedMovies.await().map { it.movie to "movie" })
            allItems.addAll(anticipatedShows.await().map { it.show to "series" })
            allItems.mapNotNull { (trakt, type) -> trakt?.let { it to type } }
                .distinctBy { (trakt, _) -> trakt.ids.slug ?: trakt.ids.imdb ?: trakt.ids.tmdb?.toString() }
                .mapNotNull { (trakt, type) ->
                    val id = TraktIntegration.contentIdFrom(trakt.ids) ?: return@mapNotNull null
                    Meta(id = id, name = trakt.title ?: unknownName(), type = type, poster = null, releaseInfo = trakt.year?.toString())
                }
        } catch(e: Exception) { emptyList() }
    }

    suspend fun getTrending(language: String = "en"): List<Meta> = withContext(Dispatchers.IO) {
        if (!TraktIntegration.hasClient(traktKey)) return@withContext emptyList()
        try {
            val trendingMovies = async { try { traktApi.getTrendingMovies(traktKey) } catch (e: Exception) { emptyList() } }
            val trendingShows = async { try { traktApi.getTrendingShows(traktKey) } catch (e: Exception) { emptyList() } }

            val movies = trendingMovies.await().mapNotNull { item ->
                item.movie?.toTraktMeta(
                    type = "movie",
                    reason = AppStrings.t(language, "auto.rising_fast_right_now")
                )
            }
            val shows = trendingShows.await().mapNotNull { item ->
                item.show?.toTraktMeta(
                    type = "series",
                    reason = AppStrings.t(language, "auto.rising_fast_right_now")
                )
            }

            interleaveMetaLists(shows, movies)
        } catch (e: Exception) {
            emptyList()
        }
    }

    suspend fun getAnticipated(language: String = "en"): List<Meta> = withContext(Dispatchers.IO) {
        if (!TraktIntegration.hasClient(traktKey)) return@withContext emptyList()
        try {
            val anticipatedMovies = async { try { traktApi.getAnticipatedMovies(traktKey) } catch (e: Exception) { emptyList() } }
            val anticipatedShows = async { try { traktApi.getAnticipatedShows(traktKey) } catch (e: Exception) { emptyList() } }

            val movies = anticipatedMovies.await().mapNotNull { item ->
                item.movie?.toTraktMeta(
                    type = "movie",
                    reason = AppStrings.t(language, "auto.everyone_will_be_talking_about_these_soon")
                )
            }
            val shows = anticipatedShows.await().mapNotNull { item ->
                item.show?.toTraktMeta(
                    type = "series",
                    reason = AppStrings.t(language, "auto.everyone_will_be_talking_about_these_soon")
                )
            }

            interleaveMetaLists(shows, movies)
        } catch (e: Exception) {
            emptyList()
        }
    }

    private fun TraktSummary.toTraktMeta(type: String, reason: String? = null): Meta? {
        val id = TraktIntegration.contentIdFrom(ids) ?: return null
        return Meta(
            id = id,
            name = title ?: unknownName(),
            type = type,
            poster = null,
            releaseInfo = year?.toString(),
            released = year?.let { "$it-01-01" },
            reason = reason
        )
    }

    private fun interleaveMetaLists(primary: List<Meta>, secondary: List<Meta>): List<Meta> {
        val result = mutableListOf<Meta>()
        val maxSize = maxOf(primary.size, secondary.size)
        for (index in 0 until maxSize) {
            primary.getOrNull(index)?.let(result::add)
            secondary.getOrNull(index)?.let(result::add)
        }
        return result.distinctBy { it.id }
    }
}
