package com.fluxa.app.plugins.cloudstream

import android.util.Log
import com.fluxa.app.BuildConfig
import com.lagradost.cloudstream3.AcraApplication
import com.lagradost.cloudstream3.AnimeLoadResponse
import com.lagradost.cloudstream3.AnimeSearchResponse
import com.lagradost.cloudstream3.ActorData
import com.lagradost.cloudstream3.Episode
import com.lagradost.cloudstream3.LiveStreamLoadResponse
import com.lagradost.cloudstream3.LoadResponse
import com.lagradost.cloudstream3.MainAPI
import com.lagradost.cloudstream3.MovieLoadResponse
import com.lagradost.cloudstream3.MovieSearchResponse
import com.lagradost.cloudstream3.SearchResponse
import com.lagradost.cloudstream3.SubtitleFile
import com.lagradost.cloudstream3.TrailerData
import com.lagradost.cloudstream3.TvSeriesLoadResponse
import com.lagradost.cloudstream3.TvSeriesSearchResponse
import com.lagradost.cloudstream3.TvType
import com.lagradost.cloudstream3.app
import com.lagradost.cloudstream3.metaproviders.TmdbProvider
import com.lagradost.cloudstream3.utils.ExtractorLink
import com.lagradost.cloudstream3.utils.ExtractorLinkType
import com.lagradost.cloudstream3.utils.Qualities
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import kotlinx.coroutines.withTimeoutOrNull

private const val TAG = "ExternalExtensionRunner"
private const val DEFAULT_TIMEOUT_MS = 30000L // 30 seconds
private const val SEARCH_TIMEOUT_MS = 15000L // 15 seconds per plugin search
private const val EXECUTION_TIMEOUT_MS = 120_000L
private const val MIN_TITLE_SIMILARITY = 0.35

private inline fun logDebug(message: () -> String) {
    if (BuildConfig.DEBUG) Log.d(TAG, message())
}

/**
 * Executes operations on loaded CloudStream3 extensions.
 * Supports both regular search-based providers and TmdbProvider-based extensions.
 */
class ExternalExtensionRunner(
    private val extractorRegistry: ExternalExtractorRegistry = ExternalExtractorRegistry()
) {
    init {
        extractorRegistry.installGlobal()
    }

    /**
     * Execute a scraper by TMDB ID - supports both TmdbProvider and search-based APIs.
     * This is the main entry point for resolving streams from extensions.
     */
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
                    executeSearchBased(api, tmdbId, mediaType, season, episode, title, year)
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Extension ${api.name} failed: ${e.javaClass.simpleName}: ${e.message}", e)
            ScraperStreamResult(false, emptyList(), emptyList(), e.message)
        } catch (e: Error) {
            val missing = extractMissingClass(e)
            if (missing != null) {
                Log.e(TAG, "Extension ${api.name} MISSING CLASS: $missing", e)
            } else {
                Log.e(TAG, "Extension ${api.name} linkage error: ${e.javaClass.simpleName}: ${e.message}", e)
            }
            ScraperStreamResult(false, emptyList(), emptyList(), e.message)
        }
    }

    /**
     * Execute a TmdbProvider extension using JSON load format.
     */
    private suspend fun executeTmdbProvider(
        api: MainAPI,
        tmdbId: String,
        mediaType: String,
        season: Int?,
        episode: Int?
    ): ScraperStreamResult {
        val tmdbIdInt = tmdbId.toIntOrNull()
        val isMovie = mediaType.lowercase() == "movie"
        val type = if (isMovie) "movie" else "tv"

        // Construct the JSON that TmdbProvider extensions expect in load()
        val loadJson = """{"id":$tmdbIdInt,"type":"$type"}"""

        logDebug { "TmdbProvider ${api.name}: load($loadJson)" }
        val loadResponse = try {
            api.load(loadJson)
        } catch (e: Exception) {
            Log.w(TAG, "TmdbProvider ${api.name} load(json) threw: ${e.javaClass.simpleName}: ${e.message?.take(100)}")
            null
        } catch (e: Error) {
            val missing = extractMissingClass(e)
            Log.w(TAG, "TmdbProvider ${api.name} load(json) error: ${missing ?: e.message?.take(100)}")
            null
        }

        if (loadResponse != null) {
            val data = extractData(loadResponse, mediaType, season, episode)
            if (data != null) {
                return executeLoadLinks(api, data)
            }
        }

        // Fallback: try with TMDB URL format
        val tmdbUrl = if (isMovie) {
            "https://www.themoviedb.org/movie/$tmdbId"
        } else {
            "https://www.themoviedb.org/tv/$tmdbId"
        }
        logDebug { "TmdbProvider ${api.name}: fallback load($tmdbUrl)" }
        val fallbackResponse = try {
            api.load(tmdbUrl)
        } catch (e: Exception) {
            null
        } catch (e: Error) { null }

        if (fallbackResponse != null) {
            val data = extractData(fallbackResponse, mediaType, season, episode)
            if (data != null) {
                return executeLoadLinks(api, data)
            }
        }

        Log.w(TAG, "TmdbProvider ${api.name}: both load() paths failed")
        return ScraperStreamResult(false, emptyList(), emptyList())
    }

    /**
     * Execute a search-based extension using title search.
     */
    private suspend fun executeSearchBased(
        api: MainAPI,
        tmdbId: String,
        mediaType: String,
        season: Int?,
        episode: Int?,
        title: String?,
        year: Int?
    ): ScraperStreamResult {
        if (title.isNullOrBlank()) {
            return ScraperStreamResult(false, emptyList(), emptyList(), "No title provided for search")
        }

        val isMovie = mediaType.lowercase() == "movie"

        logDebug { "SearchBased ${api.name}: searching for \"$title\"" }
        var searchResults = try {
            api.search(title, 1)?.items
        } catch (e: Exception) {
            Log.e(TAG, "SearchBased ${api.name} search() threw: ${e.javaClass.simpleName}: ${e.message}", e)
            null
        } catch (e: Error) {
            val missing = extractMissingClass(e)
            Log.e(TAG, "SearchBased ${api.name} search() error: ${missing ?: e.message}", e)
            null
        }

        // Retry with simplified title if special characters present
        if (searchResults.isNullOrEmpty() && title.contains(Regex("[:\\-]"))) {
            val simplified = title.replace(Regex("[:\\-]"), " ").replace(Regex("\\s+"), " ").trim()
            logDebug { "SearchBased ${api.name}: retrying with simplified \"$simplified\"" }
            searchResults = try {
                api.search(simplified, 1)?.items
            } catch (e: Exception) {
                null
            } catch (e: Error) { null }
        }

        if (searchResults.isNullOrEmpty()) {
            Log.w(TAG, "SearchBased ${api.name}: 0 search results for \"$title\"")
            return ScraperStreamResult(false, emptyList(), emptyList())
        }

        val bestMatch = findBestMatch(searchResults, title, year, isMovie)
        if (bestMatch == null) {
            logDebug { "No suitable match in ${api.name} results for: $title ($year)" }
            return ScraperStreamResult(false, emptyList(), emptyList())
        }

        logDebug { "Best match from ${api.name}: ${bestMatch.name}" }

        val loadResponse = try {
            api.load(bestMatch.url)
        } catch (e: Exception) {
            Log.e(TAG, "SearchBased ${api.name} load() threw: ${e.javaClass.simpleName}: ${e.message}", e)
            null
        } catch (e: Error) {
            val missing = extractMissingClass(e)
            Log.e(TAG, "SearchBased ${api.name} load() error: ${missing ?: e.message}", e)
            null
        }

        if (loadResponse == null) {
            Log.w(TAG, "SearchBased ${api.name}: load() returned null")
            return ScraperStreamResult(false, emptyList(), emptyList())
        }

        val data = extractData(loadResponse, mediaType, season, episode)
        if (data == null) {
            logDebug { "No data extracted from ${api.name} for S${season}E${episode}" }
            return ScraperStreamResult(false, emptyList(), emptyList())
        }

        return executeLoadLinks(api, data)
    }

    /**
     * Execute loadLinks and collect all results.
     */
    private suspend fun executeLoadLinks(api: MainAPI, data: String): ScraperStreamResult {
        // Use thread-safe lists for concurrent collection
        val links = java.util.Collections.synchronizedList(mutableListOf<ExtractorLink>())
        val subtitles = java.util.Collections.synchronizedList(mutableListOf<SubtitleFile>())

        val success = try {
            api.loadLinks(
                data = data,
                isCasting = false,
                subtitleCallback = { subtitles.add(it) },
                callback = { links.add(it) }
            )
        } catch (e: Exception) {
            Log.e(TAG, "${api.name} loadLinks threw: ${e.javaClass.simpleName}: ${e.message}", e)
            false
        } catch (e: Error) {
            val missing = extractMissingClass(e)
            Log.e(TAG, "${api.name} loadLinks error: ${missing ?: e.message}", e)
            false
        }

        if (!success && links.isEmpty()) {
            Log.w(TAG, "${api.name}: loadLinks returned false, 0 links")
            return ScraperStreamResult(false, emptyList(), emptyList())
        }

        logDebug { "${api.name}: ${links.size} links, ${subtitles.size} subs" }

        val validLinks = links.filter { link ->
            when {
                link.url.isBlank() -> false
                link.url == "error" || link.url == "null" -> false
                else -> true
            }
        }

        return ScraperStreamResult(
            success = true,
            links = validLinks.map { it.toScraperStreamLink() },
            subtitles = subtitles.map { ScraperSubtitle(it.lang, it.url) }
        )
    }

    private fun extractMissingClass(e: Error): String? {
        val msg = e.message ?: return null
        val match = Regex("""(?:L?)([\w/.]+)(?:;)?""").find(msg)
        return match?.groupValues?.get(1)?.replace('/', '.')
    }

    private fun extractData(
        response: LoadResponse,
        mediaType: String,
        season: Int?,
        episode: Int?
    ): String? = when (response) {
        is MovieLoadResponse -> response.dataUrl
        is LiveStreamLoadResponse -> response.dataUrl
        is TvSeriesLoadResponse -> {
            findEpisode(response.episodes, season, episode)?.data
        }
        is AnimeLoadResponse -> {
            val allEpisodes = response.episodes.values.flatten()
            findEpisode(allEpisodes, season, episode)?.data
        }
        else -> null
    }

    private fun findEpisode(episodes: List<Episode>, season: Int?, episode: Int?): Episode? {
        if (episodes.isEmpty()) {
            logDebug { "findEpisode: Empty episode list" }
            return null
        }

        logDebug { "findEpisode: Looking for S${season}E${episode} among ${episodes.size} episodes" }
        // Log first few episodes for debugging
        episodes.take(5).forEach { ep ->
            val dataPreview = ep.data.take(40)
            logDebug { "  Available: S${ep.season}E${ep.episode} data='$dataPreview'" }
        }

        if (season != null && episode != null) {
            val exactMatch = episodes.firstOrNull { it.season == season && it.episode == episode }
            if (exactMatch != null) {
                val hasValidData = !exactMatch.data.isNullOrBlank() && exactMatch.data != "null"
                logDebug { "findEpisode: Found exact match S${season}E${episode} - hasValidData=$hasValidData" }
                return if (hasValidData) exactMatch else null
            }
        }

        if (episode != null) {
            val episodeMatch = episodes.firstOrNull { it.episode == episode && (it.season == null || it.season == season) }
            if (episodeMatch != null) {
                val hasValidData = !episodeMatch.data.isNullOrBlank() && episodeMatch.data != "null"
                logDebug { "findEpisode: Found episode match E${episode} (season=${episodeMatch.season}) - hasValidData=$hasValidData" }
                return if (hasValidData) episodeMatch else null
            }
        }

        // Debug: Show season distribution to understand the issue
        val seasonDist = episodes.groupBy { it.season }.mapValues { it.value.size }
        Log.w(TAG, "findEpisode: No match for S${season}E${episode}. Season distribution: $seasonDist")
        return null
    }

    private fun findBestMatch(
        results: List<SearchResponse>,
        targetTitle: String,
        targetYear: Int?,
        isMovie: Boolean
    ): SearchResponse? {
        return results
            .map { result ->
                val titleSimilarity = calculateSimilarity(result.name, targetTitle)
                val typeBonus = when {
                    result.type == null -> 0.0
                    isMovie && result.type in listOf(TvType.Movie, TvType.AnimeMovie, TvType.Documentary) -> 0.1
                    !isMovie && result.type in listOf(TvType.TvSeries, TvType.Anime, TvType.OVA, TvType.Cartoon) -> 0.1
                    else -> -0.1
                }
                val resultYear = when (result) {
                    is MovieSearchResponse -> result.year
                    is TvSeriesSearchResponse -> result.year
                    is AnimeSearchResponse -> result.year
                    else -> null
                }
                val yearBonus = if (targetYear != null && resultYear == targetYear) 0.1 else 0.0
                val score = titleSimilarity + typeBonus + yearBonus
                result to score
            }
            .filter { it.second >= MIN_TITLE_SIMILARITY }
            .maxByOrNull { it.second }
            ?.first
    }

    suspend fun searchScraper(
        api: MainAPI,
        query: String
    ): List<ScraperSearchResult> = withContext(Dispatchers.IO) {
        try {
            val searchJob = async {
                try {
                    val responseList = api.search(query, 1)
                    responseList?.items?.map { it.toScraperResult(api.name) } ?: emptyList()
                } catch (e: Throwable) {
                    emptyList<ScraperSearchResult>()
                }
            }
            withTimeoutOrNull(SEARCH_TIMEOUT_MS) { searchJob.await() } ?: emptyList()
        } catch (e: Exception) { emptyList() }
    }

    fun findBestScraperMatch(
        results: List<ScraperSearchResult>,
        targetTitle: String,
        originalTitle: String?,
        targetYear: Int?,
        isMovie: Boolean
    ): ScraperSearchResult? {
        return results
            .map { result ->
                val similarity = calculateSimilarity(result.title, targetTitle)
                val originalSimilarity = originalTitle?.let { calculateSimilarity(result.title, it) } ?: 0.0
                var score = maxOf(similarity, originalSimilarity)
                if (targetYear != null && result.year == targetYear) score += 0.25
                if (isMovie && result.type in listOf(TvType.Movie, TvType.AnimeMovie)) score += 0.15
                if (!isMovie && result.type in listOf(TvType.TvSeries, TvType.Anime, TvType.AsianDrama)) score += 0.15
                result to score
            }
            .filter { it.second >= 0.4 }
            .maxByOrNull { it.second }
            ?.first
    }

    fun calculateSimilarity(s1: String, s2: String): Double {
        val a = s1.lowercase().trim()
        val b = s2.lowercase().trim()
        if (a == b) return 1.0
        if (a.contains(b) || b.contains(a)) return 0.85
        val distance = levenshteinDistance(a, b)
        val maxLen = maxOf(a.length, b.length)
        val levenScore = 1.0 - (distance.toDouble() / maxLen)
        val aWords = a.split(Regex("[\\s\\-_:]+")).filter { it.length > 1 }.toSet()
        val bWords = b.split(Regex("[\\s\\-_:]+")).filter { it.length > 1 }.toSet()
        val commonWords = aWords.intersect(bWords).size
        val totalWords = maxOf(aWords.size, bWords.size, 1)
        val wordScore = commonWords.toDouble() / totalWords
        return maxOf(levenScore, wordScore * 0.9)
    }

    private fun levenshteinDistance(s1: String, s2: String): Int {
        val dp = Array(s1.length + 1) { IntArray(s2.length + 1) }
        for (i in 0..s1.length) dp[i][0] = i
        for (j in 0..s2.length) dp[0][j] = j
        for (i in 1..s1.length) {
            for (j in 1..s2.length) {
                val cost = if (s1[i - 1] == s2[j - 1]) 0 else 1
                dp[i][j] = minOf(dp[i - 1][j] + 1, dp[i][j - 1] + 1, dp[i - 1][j - 1] + cost)
            }
        }
        return dp[s1.length][s2.length]
    }

    suspend fun loadContent(api: MainAPI, url: String): ScraperLoadResult? = withContext(Dispatchers.IO) {
        try {
            withTimeout(DEFAULT_TIMEOUT_MS) {
                api.load(url)?.toScraperLoadResult(api.name)
            }
        } catch (e: Exception) {
            Log.e(TAG, "${api.name} loadContent() threw: ${e.javaClass.simpleName}: ${e.message}", e)
            null
        }
    }

    suspend fun loadStreams(api: MainAPI, data: String): ScraperStreamResult = withContext(Dispatchers.IO) {
        val links = mutableListOf<ScraperStreamLink>()
        val subtitles = mutableListOf<ScraperSubtitle>()
        try {
            withTimeout(DEFAULT_TIMEOUT_MS) {
                val success = api.loadLinks(data, false, { sub ->
                    subtitles.add(ScraperSubtitle(sub.lang, sub.url))
                }, { link ->
                    links.add(link.toScraperStreamLink())
                })
                ScraperStreamResult(success, links, subtitles)
            }
        } catch (e: Exception) {
            ScraperStreamResult(false, links, subtitles, e.message)
        }
    }

    fun extractEpisodeData(response: ScraperLoadResult, season: Int?, episode: Int?): String? {
        val epList = response.episodes
        if (epList.isNullOrEmpty()) {
            logDebug { "extractEpisodeData: Empty episode list for ${response.title}" }
            return null
        }

        logDebug { "extractEpisodeData: Looking for S${season}E${episode} in ${response.title} - Total episodes: ${epList.size}" }
        epList.take(15).forEach { ep ->
            val hasData = !ep.data.isNullOrBlank() && ep.data != "null"
            logDebug { "  Episode: S${ep.season}E${ep.episode} - name='${ep.name}' - hasData=$hasData - data='${ep.data.take(60)}'" }
        }

        val target = if (season != null && episode != null) {
            val exactMatch = epList.firstOrNull { it.season == season && it.episode == episode }
            if (exactMatch != null) {
                val hasValidData = !exactMatch.data.isNullOrBlank() && exactMatch.data != "null"
                logDebug { "extractEpisodeData: Found exact match S${season}E${episode} - hasValidData=$hasValidData" }
                if (!hasValidData) {
                    Log.w(TAG, "extractEpisodeData: Match found but data is invalid: '${exactMatch.data}'")
                }
                exactMatch
            } else if (epList.any { it.season == season }) {
                // The provider does list this season, just not this episode — an
                // episode-only match here would silently grab a different season's
                // episode with the same number, so treat it as no match instead.
                val seasonCounts = epList.groupBy { it.season }.mapValues { it.value.size }
                Log.w(TAG, "extractEpisodeData: No match found for S${season}E${episode}. Season distribution: $seasonCounts")
                null
            } else {
                val episodeOnlyMatch = epList.firstOrNull { it.episode == episode }
                if (episodeOnlyMatch != null) {
                    logDebug { "extractEpisodeData: Found episode-only match E${episode} (season was ${episodeOnlyMatch.season})" }
                } else {
                    val seasonCounts = epList.groupBy { it.season }.mapValues { it.value.size }
                    Log.w(TAG, "extractEpisodeData: No match found for S${season}E${episode}. Season distribution: $seasonCounts")
                }
                episodeOnlyMatch
            }
        } else {
            logDebug { "extractEpisodeData: No season/episode specified, returning first episode" }
            epList.firstOrNull()
        }
        val data = target?.data
        return if (data.isNullOrBlank() || data == "null") null else data
    }
}

// Our unified result models
data class ScraperSearchResult(
    val title: String,
    val url: String,
    val posterUrl: String?,
    val type: TvType?,
    val year: Int?,
    val quality: String?,
    val scraperName: String
)

data class ScraperLoadResult(
    val title: String,
    val url: String,
    val posterUrl: String?,
    val plot: String?,
    val type: TvType,
    val year: Int?,
    val rating: Int?,
    val tags: List<String>?,
    val duration: Int?,
    val scraperName: String,
    val episodes: List<ScraperEpisode>?,
    val data: String?,
    val recommendations: List<ScraperSearchResult>?,
    val actors: List<ScraperActor>?,
    val trailers: List<ScraperTrailer>?,
    val comingSoon: Boolean,
    val syncData: Map<String, String>?,
    val posterHeaders: Map<String, String>?,
    val backgroundPosterUrl: String?,
    val logoUrl: String?,
    val contentRating: String?,
    val uniqueUrl: String?,
    val status: String?,
    val nextAiringUnixTime: Long?,
    val nextAiringEpisode: Int?,
    val nextAiringSeason: Int?,
    val seasonNames: List<ScraperSeason>?,
    val synonyms: List<String>?
)

data class ScraperEpisode(
    val data: String,
    val name: String?,
    val season: Int?,
    val episode: Int?,
    val posterUrl: String?,
    val description: String?,
    val date: Long?,
    val rating: Int?,
    val runTime: Int?
)

data class ScraperActor(
    val name: String,
    val image: String?,
    val role: String?,
    val voiceActorName: String?,
    val voiceActorImage: String?
)

data class ScraperTrailer(
    val url: String,
    val referer: String?,
    val raw: Boolean,
    val headers: Map<String, String>
)

data class ScraperSeason(
    val season: Int,
    val name: String?,
    val displaySeason: Int?
)

data class ScraperStreamResult(
    val success: Boolean,
    val links: List<ScraperStreamLink>,
    val subtitles: List<ScraperSubtitle>,
    val error: String? = null
)

data class ScraperStreamLink(
    val source: String,
    val name: String,
    val url: String,
    val referer: String?,
    val quality: String,
    val qualityValue: Int,
    val isM3u8: Boolean,
    val isDash: Boolean,
    val headers: Map<String, String>,
    val type: String
)

data class ScraperSubtitle(
    val lang: String,
    val url: String
)

// Mappers
private fun SearchResponse.toScraperResult(scraperName: String): ScraperSearchResult {
    val yearVal = when(this) {
        is MovieSearchResponse -> this.year
        is TvSeriesSearchResponse -> this.year
        is AnimeSearchResponse -> this.year
        else -> null
    }
    return ScraperSearchResult(name, url, posterUrl, type, yearVal, quality?.name, scraperName)
}

private fun LoadResponse.toScraperLoadResult(scraperName: String): ScraperLoadResult {
    val epList = when(this) {
        is TvSeriesLoadResponse -> this.episodes
        is AnimeLoadResponse -> this.episodes.values.flatten()
        else -> emptyList()
    }
    val contentData = when (this) {
        is MovieLoadResponse -> this.dataUrl
        is LiveStreamLoadResponse -> this.dataUrl
        else -> null
    }
    val showStatus = when (this) {
        is TvSeriesLoadResponse -> this.showStatus?.name
        is AnimeLoadResponse -> this.showStatus?.name
        else -> null
    }
    val nextAiring = when (this) {
        is TvSeriesLoadResponse -> this.nextAiring
        is AnimeLoadResponse -> this.nextAiring
        else -> null
    }
    val seasonNames = when (this) {
        is TvSeriesLoadResponse -> this.seasonNames
        is AnimeLoadResponse -> this.seasonNames
        else -> null
    }
    val synonyms = when (this) {
        is AnimeLoadResponse -> listOfNotNull(this.engName, this.japName) + this.synonyms.orEmpty()
        else -> emptyList()
    }.map { it.trim() }.filter { it.isNotBlank() }.distinct()
    return ScraperLoadResult(
        title = name,
        url = url,
        posterUrl = posterUrl,
        plot = plot,
        type = type,
        year = year,
        rating = score?.toInt(100),
        tags = tags,
        duration = duration,
        scraperName = scraperName,
        episodes = epList.map { it.toScraperEpisode() },
        data = contentData,
        recommendations = recommendations?.map { it.toScraperResult(scraperName) },
        actors = actors?.mapNotNull { it.toScraperActor() },
        trailers = trailers.map { it.toScraperTrailer() },
        comingSoon = comingSoon,
        syncData = syncData,
        posterHeaders = posterHeaders,
        backgroundPosterUrl = backgroundPosterUrl,
        logoUrl = reflectString("getLogoUrl"),
        contentRating = contentRating,
        uniqueUrl = uniqueUrl,
        status = showStatus,
        nextAiringUnixTime = nextAiring?.unixTime,
        nextAiringEpisode = nextAiring?.episode,
        nextAiringSeason = nextAiring?.season,
        seasonNames = seasonNames?.map { ScraperSeason(it.season, it.name, it.displaySeason) },
        synonyms = synonyms.takeIf { it.isNotEmpty() }
    )
}

private fun Episode.toScraperEpisode() = ScraperEpisode(
    data = data,
    name = name,
    season = season,
    episode = episode,
    posterUrl = posterUrl,
    description = description,
    date = date,
    rating = score?.toInt(100),
    runTime = runTime
)

private fun ActorData.toScraperActor(): ScraperActor? {
    val actorName = actor.name.takeIf { it.isNotBlank() } ?: return null
    return ScraperActor(
        name = actorName,
        image = actor.image,
        role = roleString?.takeIf { it.isNotBlank() } ?: role?.name,
        voiceActorName = null,
        voiceActorImage = null
    )
}

private fun Any.reflectString(methodName: String): String? {
    return runCatching {
        javaClass.methods.firstOrNull { it.name == methodName && it.parameterTypes.isEmpty() }
            ?.invoke(this) as? String
    }.getOrNull()?.takeIf { it.isNotBlank() }
}

private fun TrailerData.toScraperTrailer() = ScraperTrailer(
    url = extractorUrl,
    referer = referer,
    raw = raw,
    headers = headers
)

private fun ExtractorLink.toScraperStreamLink(): ScraperStreamLink {
    val qualityLabel = resolvedQualityLabel()
    return ScraperStreamLink(
        source = source,
        name = name,
        url = url,
        referer = referer,
        quality = qualityLabel,
        qualityValue = quality,
        isM3u8 = type == ExtractorLinkType.M3U8,
        isDash = type == ExtractorLinkType.DASH,
        headers = headers,
        type = type.name
    )
}

private fun ExtractorLink.resolvedQualityLabel(): String {
    val cloudstreamQuality = Qualities.getStringByIntFull(quality).takeUnless { it.equals("unknown", ignoreCase = true) }
    if (!cloudstreamQuality.isNullOrBlank()) return cloudstreamQuality
    val reflected = listOfNotNull(
        reflectString("getDisplayName"),
        reflectString("getQualityName"),
        reflectString("getFullName")
    ).firstNotNullOfOrNull(::extractResolutionLabel)
    if (!reflected.isNullOrBlank()) return reflected
    return extractResolutionLabel(listOf(source, name, url).joinToString(" ")) ?: "unknown"
}

private fun extractResolutionLabel(value: String): String? {
    val text = value.takeIf { it.isNotBlank() } ?: return null
    Regex("""\b\d{3,5}\s*x\s*\d{3,5}\b""")
        .find(text)
        ?.value
        ?.replace(Regex("""\s+"""), "")
        ?.let { return it }
    Regex("""\b\d{3,4}p\b""", RegexOption.IGNORE_CASE)
        .find(text)
        ?.value
        ?.lowercase()
        ?.let { return it }
    return null
}
