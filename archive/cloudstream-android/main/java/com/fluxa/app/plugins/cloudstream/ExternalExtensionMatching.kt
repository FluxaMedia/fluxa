package com.fluxa.app.plugins.cloudstream

import android.util.Log
import com.fluxa.app.core.rust.FluxaCoreNative
import com.lagradost.cloudstream3.AnimeLoadResponse
import com.lagradost.cloudstream3.AnimeSearchResponse
import com.lagradost.cloudstream3.Episode
import com.lagradost.cloudstream3.LiveStreamLoadResponse
import com.lagradost.cloudstream3.LoadResponse
import com.lagradost.cloudstream3.MainAPI
import com.lagradost.cloudstream3.MovieLoadResponse
import com.lagradost.cloudstream3.MovieSearchResponse
import com.lagradost.cloudstream3.SearchResponse
import com.lagradost.cloudstream3.TvSeriesLoadResponse
import com.lagradost.cloudstream3.TvSeriesSearchResponse

private const val TAG = "ExternalExtensionMatching"

internal fun extractLoadData(
    response: LoadResponse,
    mediaType: String,
    season: Int?,
    episode: Int?
): String? = when (response) {
    is MovieLoadResponse -> response.dataUrl
    is LiveStreamLoadResponse -> response.dataUrl
    is TvSeriesLoadResponse -> findEpisode(response.episodes, season, episode)?.data
    is AnimeLoadResponse -> findEpisode(response.episodes.values.flatten(), season, episode)?.data
    else -> null
}

internal fun findEpisode(episodes: List<Episode>, season: Int?, episode: Int?): Episode? {
    if (episodes.isEmpty()) return null
    if (season != null && episode != null) {
        val exactMatch = episodes.firstOrNull { it.season == season && it.episode == episode }
        if (exactMatch != null) return exactMatch.takeIf(::hasValidEpisodeData)
    }
    if (episode != null) {
        val episodeMatch = episodes.firstOrNull { it.episode == episode && (it.season == null || it.season == season) }
        if (episodeMatch != null) return episodeMatch.takeIf(::hasValidEpisodeData)
    }
    Log.w(TAG, "No episode match for S$season E$episode among ${episodes.size} episodes")
    return null
}

internal fun findBestSearchResponse(
    results: List<SearchResponse>,
    targetTitle: String,
    targetYear: Int?,
    isMovie: Boolean
): SearchResponse? = results
    .mapNotNull { result ->
        val resultYear = when (result) {
            is MovieSearchResponse -> result.year
            is TvSeriesSearchResponse -> result.year
            is AnimeSearchResponse -> result.year
            else -> null
        }
        FluxaCoreNative.cloudstreamMatchScore(
            mode = "search",
            targetTitle = targetTitle,
            originalTitle = null,
            candidateTitle = result.name,
            targetYear = targetYear,
            candidateYear = resultYear,
            candidateType = result.type?.name,
            isMovie = isMovie
        )?.let { score -> result to score }
    }
    .maxByOrNull { it.second }
    ?.first

internal fun findBestScraperResult(
    results: List<ScraperSearchResult>,
    targetTitle: String,
    originalTitle: String?,
    targetYear: Int?,
    isMovie: Boolean
): ScraperSearchResult? = results
    .mapNotNull { result ->
        FluxaCoreNative.cloudstreamMatchScore(
            mode = "scraper",
            targetTitle = targetTitle,
            originalTitle = originalTitle,
            candidateTitle = result.title,
            targetYear = targetYear,
            candidateYear = result.year,
            candidateType = result.type?.name,
            isMovie = isMovie
        )?.let { score -> result to score }
    }
    .maxByOrNull { it.second }
    ?.first

internal fun extractEpisodeDataFromResult(
    response: ScraperLoadResult,
    season: Int?,
    episode: Int?
): String? {
    val episodes = response.episodes.orEmpty()
    if (episodes.isEmpty()) return null
    val target = when {
        season != null && episode != null -> {
            episodes.firstOrNull { it.season == season && it.episode == episode }
                ?: if (episodes.any { it.season == season }) null else episodes.firstOrNull { it.episode == episode }
        }
        else -> episodes.firstOrNull()
    }
    return target?.data?.takeIf { it.isNotBlank() && it != "null" }
}

private fun hasValidEpisodeData(episode: Episode): Boolean =
    !episode.data.isNullOrBlank() && episode.data != "null"
