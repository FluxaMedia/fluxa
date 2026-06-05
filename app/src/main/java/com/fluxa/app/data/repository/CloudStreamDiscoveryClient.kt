package com.fluxa.app.data.repository

import android.util.Log
import com.fluxa.app.BuildConfig
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.plugins.cloudstream.ExternalExtensionRunner
import com.fluxa.app.plugins.cloudstream.ScraperSearchResult
import com.fluxa.app.plugins.cloudstream.ScraperStreamLink
import com.lagradost.cloudstream3.metaproviders.TmdbProvider
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import javax.inject.Inject
import javax.inject.Singleton

private const val MAX_CONCURRENT_CLOUDSTREAM_DISCOVERY = 4

@Singleton
class CloudStreamDiscoveryClient @Inject constructor() {
    suspend fun getStreams(
        pluginApis: List<com.lagradost.cloudstream3.MainAPI>,
        id: String,
        title: String,
        year: Int?,
        type: String,
        season: Int? = null,
        episode: Int? = null,
        originalName: String? = null
    ): List<Stream> {
        val runner = ExternalExtensionRunner()
        val isMovieRequest = type == "movie"
        val semaphore = Semaphore(MAX_CONCURRENT_CLOUDSTREAM_DISCOVERY)

        val parsedLocator = TraktIntegration.episodeLocator(id)
        logDebug("StremioRepo") {
            "CS3 discovery: $title (id=$id, parsedS=${parsedLocator?.season}, parsedE=${parsedLocator?.episode}, paramS=$season, paramE=$episode, type=$type)"
        }

        return coroutineScope {
            pluginApis.map { api ->
                async {
                    semaphore.withPermit {
                        try {
                            if (api is TmdbProvider) {
                                loadTmdbProviderStreams(api, runner, id, type, isMovieRequest, season, episode)
                            } else {
                                loadSearchProviderStreams(api, runner, title, originalName, year, type, isMovieRequest, season, episode)
                            }
                        } catch (e: Exception) {
                            Log.e("StremioRepo", "CS3 plugin ${api.name} failed", e)
                            emptyList()
                        }
                    }
                }
            }.awaitAll().flatten()
        }.also { streams ->
            logDebug("StremioRepo") { "CS3 discovery completed with ${streams.size} streams from ${pluginApis.size} plugins" }
        }
    }

    private suspend fun loadTmdbProviderStreams(
        api: TmdbProvider,
        runner: ExternalExtensionRunner,
        id: String,
        type: String,
        isMovieRequest: Boolean,
        season: Int?,
        episode: Int?
    ): List<Stream> {
        val baseId = Regex("""tt\d+""").find(id)?.value
            ?: com.fluxa.app.core.StremioId.baseContentId(id).removePrefix("tmdb:").substringBefore(":")
        val tmdbIdInt = baseId.toIntOrNull() ?: return emptyList()
        val tmdbType = if (type == "movie") "movie" else "tv"
        val tmdbUrl = "https://www.themoviedb.org/$tmdbType/$tmdbIdInt"
        logDebug("StremioRepo") { "CS3 ${api.name}: direct TMDB lookup $tmdbUrl" }

        val response = runner.loadContent(api, tmdbUrl) ?: return emptyList()
        val streamData = if (isMovieRequest) response.data else runner.extractEpisodeData(response, season, episode)
        if (streamData == null) {
            Log.w("StremioRepo", "CS3 ${api.name}: no content data for type=$type s=$season e=$episode")
            return emptyList()
        }
        return runner.loadStreams(api, streamData).links.map { it.toStream(api.name) }
    }

    private suspend fun loadSearchProviderStreams(
        api: com.lagradost.cloudstream3.MainAPI,
        runner: ExternalExtensionRunner,
        title: String,
        originalName: String?,
        year: Int?,
        type: String,
        isMovieRequest: Boolean,
        season: Int?,
        episode: Int?
    ): List<Stream> {
        val searchQueries = buildList {
            title.takeIf { it.isNotBlank() }?.let(::add)
            originalName?.takeIf { it.isNotBlank() && it != title }?.let(::add)
        }

        var searchResults = emptyList<ScraperSearchResult>()
        var usedQuery = ""
        for (query in searchQueries) {
            searchResults = runner.searchScraper(api, query)
            if (searchResults.isNotEmpty()) {
                usedQuery = query
                break
            }
        }
        if (searchResults.isEmpty()) {
            Log.w("StremioRepo", "CS3 ${api.name}: no results found for $searchQueries")
            return emptyList()
        }

        val best = runner.findBestScraperMatch(
            results = searchResults,
            targetTitle = title,
            originalTitle = originalName,
            targetYear = year,
            isMovie = isMovieRequest
        )
        if (best == null) {
            Log.w("StremioRepo", "CS3 ${api.name}: best match below threshold for $title")
            return emptyList()
        }

        logDebug("StremioRepo") { "CS3 ${api.name}: matched ${best.title} (query=$usedQuery)" }
        val response = runner.loadContent(api, best.url) ?: return emptyList()
        val streamData = if (isMovieRequest) response.data else runner.extractEpisodeData(response, season, episode)
        if (streamData == null) {
            Log.w("StremioRepo", "CS3 ${api.name}: no content data for type=$type s=$season e=$episode")
            return emptyList()
        }
        return runner.loadStreams(api, streamData).links.map { it.toStream(api.name) }
    }

    private fun ScraperStreamLink.toStream(addonName: String) = Stream(
        name = " $addonName\n$quality",
        title = name,
        url = url,
        behaviorHints = buildMap {
            put("proxyHeaders", buildMap { put("request", headers) })
            if (referer != null) put("referer", referer)
        },
        addonName = " $addonName"
    )

    private inline fun logDebug(tag: String, message: () -> String) {
        if (BuildConfig.DEBUG) Log.d(tag, message())
    }
}
