package com.fluxa.app.plugins.cloudstream

import com.lagradost.cloudstream3.ActorData
import com.lagradost.cloudstream3.AnimeLoadResponse
import com.lagradost.cloudstream3.AnimeSearchResponse
import com.lagradost.cloudstream3.Episode
import com.lagradost.cloudstream3.LiveStreamLoadResponse
import com.lagradost.cloudstream3.LoadResponse
import com.lagradost.cloudstream3.MovieLoadResponse
import com.lagradost.cloudstream3.MovieSearchResponse
import com.lagradost.cloudstream3.SearchResponse
import com.lagradost.cloudstream3.SubtitleFile
import com.lagradost.cloudstream3.TrailerData
import com.lagradost.cloudstream3.TvSeriesLoadResponse
import com.lagradost.cloudstream3.TvSeriesSearchResponse
import com.lagradost.cloudstream3.TvType
import com.lagradost.cloudstream3.utils.Qualities
import com.lagradost.cloudstream3.utils.ExtractorLink
import com.lagradost.cloudstream3.utils.ExtractorLinkType

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

internal fun SearchResponse.toScraperResult(scraperName: String): ScraperSearchResult {
    val yearVal = when (this) {
        is MovieSearchResponse -> year
        is TvSeriesSearchResponse -> year
        is AnimeSearchResponse -> year
        else -> null
    }
    return ScraperSearchResult(name, url, posterUrl, type, yearVal, quality?.name, scraperName)
}

internal fun LoadResponse.toScraperLoadResult(scraperName: String): ScraperLoadResult {
    val epList = when (this) {
        is TvSeriesLoadResponse -> episodes
        is AnimeLoadResponse -> episodes.values.flatten()
        else -> emptyList()
    }
    val contentData = when (this) {
        is MovieLoadResponse -> dataUrl
        is LiveStreamLoadResponse -> dataUrl
        else -> null
    }
    val showStatus = when (this) {
        is TvSeriesLoadResponse -> showStatus?.name
        is AnimeLoadResponse -> showStatus?.name
        else -> null
    }
    val nextAiring = when (this) {
        is TvSeriesLoadResponse -> nextAiring
        is AnimeLoadResponse -> nextAiring
        else -> null
    }
    val seasonNames = when (this) {
        is TvSeriesLoadResponse -> seasonNames
        is AnimeLoadResponse -> seasonNames
        else -> null
    }
    val synonyms = when (this) {
        is AnimeLoadResponse -> listOfNotNull(engName, japName) + synonyms.orEmpty()
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

private fun TrailerData.toScraperTrailer() = ScraperTrailer(
    url = extractorUrl,
    referer = referer,
    raw = raw,
    headers = headers
)

internal fun ExtractorLink.toScraperStreamLink(): ScraperStreamLink {
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
    val cloudStreamQuality = Qualities.getStringByIntFull(quality)
        .takeUnless { it.equals("unknown", ignoreCase = true) }
    if (!cloudStreamQuality.isNullOrBlank()) return cloudStreamQuality
    val reflected = listOfNotNull(
        reflectString("getDisplayName"),
        reflectString("getQualityName"),
        reflectString("getFullName")
    ).firstNotNullOfOrNull(::extractResolutionLabel)
    if (!reflected.isNullOrBlank()) return reflected
    return extractResolutionLabel(listOf(source, name, url).joinToString(" ")) ?: "unknown"
}

private fun Any.reflectString(methodName: String): String? = runCatching {
    javaClass.methods.firstOrNull { it.name == methodName && it.parameterTypes.isEmpty() }
        ?.invoke(this) as? String
}.getOrNull()?.takeIf { it.isNotBlank() }

private fun extractResolutionLabel(value: String): String? {
    val text = value.takeIf { it.isNotBlank() } ?: return null
    Regex("""\b\d{3,5}\s*x\s*\d{3,5}\b""")
        .find(text)?.value?.replace(Regex("""\s+"""), "")?.let { return it }
    Regex("""\b\d{3,4}p\b""", RegexOption.IGNORE_CASE)
        .find(text)?.value?.lowercase()?.let { return it }
    return null
}
