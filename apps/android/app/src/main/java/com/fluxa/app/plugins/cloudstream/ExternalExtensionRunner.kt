package com.fluxa.app.plugins.cloudstream

import android.util.Log
import com.fluxa.app.BuildConfig
import com.lagradost.cloudstream3.LoadResponse
import com.lagradost.cloudstream3.MainAPI
import com.lagradost.cloudstream3.SubtitleFile
import com.lagradost.cloudstream3.metaproviders.TmdbProvider
import com.lagradost.cloudstream3.utils.ExtractorLink
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import kotlinx.coroutines.withTimeoutOrNull

private const val TAG = "ExternalExtensionRunner"
private const val DEFAULT_TIMEOUT_MS = 30000L
private const val SEARCH_TIMEOUT_MS = 15000L
private const val EXECUTION_TIMEOUT_MS = 120_000L

private inline fun logDebug(message: () -> String) {
    if (BuildConfig.DEBUG) Log.d(TAG, message())
}

class ExternalExtensionRunner(
    private val extractorRegistry: ExternalExtractorRegistry = ExternalExtractorRegistry()
) {
    init {
        extractorRegistry.installGlobal()
    }

    suspend fun executeByTmdbId(
        api: MainAPI,
        tmdbId: String,
        mediaType: String,
        season: Int? = null,
        episode: Int? = null,
        title: String? = null,
        year: Int? = null
    ): ScraperStreamResult = withContext(Dispatchers.IO) {
        try {
            withTimeout(EXECUTION_TIMEOUT_MS) {
                if (api is TmdbProvider) {
                    executeTmdbProvider(api, tmdbId, mediaType, season, episode)
                } else {
                    executeSearchBased(api, mediaType, season, episode, title, year)
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Extension ${api.name} failed: ${e.javaClass.simpleName}: ${e.message}", e)
            ScraperStreamResult(false, emptyList(), emptyList(), e.message)
        } catch (e: Error) {
            val missing = extractMissingClass(e)
            Log.e(TAG, "Extension ${api.name} linkage error: ${missing ?: e.message}", e)
            ScraperStreamResult(false, emptyList(), emptyList(), e.message)
        }
    }

    private suspend fun executeTmdbProvider(
        api: MainAPI,
        tmdbId: String,
        mediaType: String,
        season: Int?,
        episode: Int?
    ): ScraperStreamResult {
        val tmdbIdInt = tmdbId.toIntOrNull()
        val type = if (mediaType.equals("movie", ignoreCase = true)) "movie" else "tv"
        val loadJson = """{"id":$tmdbIdInt,"type":"$type"}"""
        logDebug { "TmdbProvider ${api.name}: load($loadJson)" }
        runCatching { api.load(loadJson) }.getOrNull()
            ?.let { extractLoadData(it, mediaType, season, episode) }
            ?.let { return executeLoadLinks(api, it) }

        val tmdbUrl = "https://www.themoviedb.org/${if (type == "movie") "movie" else "tv"}/$tmdbId"
        logDebug { "TmdbProvider ${api.name}: fallback load($tmdbUrl)" }
        runCatching { api.load(tmdbUrl) }.getOrNull()
            ?.let { extractLoadData(it, mediaType, season, episode) }
            ?.let { return executeLoadLinks(api, it) }

        return ScraperStreamResult(false, emptyList(), emptyList())
    }

    private suspend fun executeSearchBased(
        api: MainAPI,
        mediaType: String,
        season: Int?,
        episode: Int?,
        title: String?,
        year: Int?
    ): ScraperStreamResult {
        if (title.isNullOrBlank()) return ScraperStreamResult(false, emptyList(), emptyList(), "No title provided for search")
        val isMovie = mediaType.equals("movie", ignoreCase = true)
        var searchResults = runCatching { api.search(title, 1)?.items }.getOrNull()
        if (searchResults.isNullOrEmpty() && title.contains(Regex("[:\\-]"))) {
            val simplified = title.replace(Regex("[:\\-]"), " ").replace(Regex("\\s+"), " ").trim()
            searchResults = runCatching { api.search(simplified, 1)?.items }.getOrNull()
        }
        if (searchResults.isNullOrEmpty()) return ScraperStreamResult(false, emptyList(), emptyList())
        val bestMatch = findBestSearchResponse(searchResults, title, year, isMovie)
            ?: return ScraperStreamResult(false, emptyList(), emptyList())
        val loadResponse = runCatching { api.load(bestMatch.url) }.getOrNull()
            ?: return ScraperStreamResult(false, emptyList(), emptyList())
        val data = extractLoadData(loadResponse, mediaType, season, episode)
            ?: return ScraperStreamResult(false, emptyList(), emptyList())
        return executeLoadLinks(api, data)
    }

    private suspend fun executeLoadLinks(api: MainAPI, data: String): ScraperStreamResult {
        val links = java.util.Collections.synchronizedList(mutableListOf<ExtractorLink>())
        val subtitles = java.util.Collections.synchronizedList(mutableListOf<SubtitleFile>())
        val success = runCatching {
            api.loadLinks(
                data = data,
                isCasting = false,
                subtitleCallback = { subtitles.add(it) },
                callback = { links.add(it) }
            )
        }.getOrDefault(false)
        if (!success && links.isEmpty()) return ScraperStreamResult(false, emptyList(), emptyList())
        return ScraperStreamResult(
            success = true,
            links = links.filter { it.url.isNotBlank() && it.url != "error" && it.url != "null" }
                .map { it.toScraperStreamLink() },
            subtitles = subtitles.map { ScraperSubtitle(it.lang, it.url) }
        )
    }

    private fun extractMissingClass(error: Error): String? = error.message
        ?.let { Regex("""(?:L?)([\\w/.]+)(?:;)?""").find(it)?.groupValues?.get(1) }
        ?.replace('/', '.')

    suspend fun searchScraper(api: MainAPI, query: String): List<ScraperSearchResult> = withContext(Dispatchers.IO) {
        try {
            val searchJob = async {
                runCatching { api.search(query, 1)?.items?.map { it.toScraperResult(api.name) }.orEmpty() }
                    .getOrDefault(emptyList())
            }
            withTimeoutOrNull(SEARCH_TIMEOUT_MS) { searchJob.await() }.orEmpty()
        } catch (_: Exception) {
            emptyList()
        }
    }

    fun findBestScraperMatch(
        results: List<ScraperSearchResult>,
        targetTitle: String,
        originalTitle: String?,
        targetYear: Int?,
        isMovie: Boolean
    ): ScraperSearchResult? = findBestScraperResult(results, targetTitle, originalTitle, targetYear, isMovie)

    suspend fun loadContent(api: MainAPI, url: String): ScraperLoadResult? = withContext(Dispatchers.IO) {
        try {
            withTimeout(DEFAULT_TIMEOUT_MS) { api.load(url)?.toScraperLoadResult(api.name) }
        } catch (e: Exception) {
            Log.e(TAG, "${api.name} loadContent() failed: ${e.message}", e)
            null
        }
    }

    suspend fun loadStreams(api: MainAPI, data: String): ScraperStreamResult = withContext(Dispatchers.IO) {
        val links = mutableListOf<ScraperStreamLink>()
        val subtitles = mutableListOf<ScraperSubtitle>()
        try {
            withTimeout(DEFAULT_TIMEOUT_MS) {
                val success = api.loadLinks(data, false, { sub -> subtitles.add(ScraperSubtitle(sub.lang, sub.url)) }) { link ->
                    links.add(link.toScraperStreamLink())
                }
                ScraperStreamResult(success, links, subtitles)
            }
        } catch (e: Exception) {
            ScraperStreamResult(false, links, subtitles, e.message)
        }
    }

    fun extractEpisodeData(response: ScraperLoadResult, season: Int?, episode: Int?): String? =
        extractEpisodeDataFromResult(response, season, episode)
}
