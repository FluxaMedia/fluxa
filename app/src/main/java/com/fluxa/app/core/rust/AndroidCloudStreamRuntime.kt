package com.fluxa.app.core.rust

import android.util.Log
import com.fluxa.app.data.remote.CastMember
import com.fluxa.app.data.remote.DetailTrailer
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.MetaLink
import com.fluxa.app.data.remote.MetaRating
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.SubtitleData
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.CloudStreamCatalogClient
import com.fluxa.app.data.repository.toStremioType
import com.fluxa.app.plugins.PluginManager
import com.fluxa.app.plugins.cloudstream.ExternalExtensionRunner
import com.fluxa.app.plugins.cloudstream.ScraperActor
import com.fluxa.app.plugins.cloudstream.ScraperLoadResult
import com.fluxa.app.plugins.cloudstream.ScraperSearchResult
import com.fluxa.app.plugins.cloudstream.ScraperSubtitle
import com.fluxa.app.plugins.cloudstream.ScraperTrailer
import kotlinx.coroutines.withTimeoutOrNull

internal class AndroidCloudStreamRuntime(private val pluginManager: PluginManager) {
    suspend fun loadMetaDetail(id: String): MetaDetail? {
        val (apiName, url) = CloudStreamCatalogClient.decodeCsId(id) ?: return null
        val api = pluginManager.loadedApis.value.firstOrNull { it.name == apiName } ?: run {
            Log.w("CS3Detail", "Plugin not found: $apiName")
            return null
        }
        val runner = ExternalExtensionRunner()
        val load = withTimeoutOrNull(20_000L) { runner.loadContent(api, url) } ?: run {
            Log.w("CS3Detail", "$apiName: loadContent timed out or returned null for url=$url")
            return null
        }
        val videos = load.episodes?.mapIndexed { index, episode ->
            Video(
                id = CloudStreamCatalogClient.encodeCsId(apiName, episode.data),
                name = episode.name ?: "Episode ${index + 1}",
                season = episode.season,
                number = episode.episode,
                released = episode.date?.toIsoDate(),
                thumbnail = episode.posterUrl,
                overview = episode.description,
                rating = episode.rating?.let(::ratingString),
                episodeRuntime = episode.runTime
            )
        }
        return MetaDetail(
            id = id,
            type = load.type.toStremioType(),
            name = load.title,
            genres = load.tags,
            poster = load.posterUrl,
            background = load.backgroundPosterUrl ?: load.posterUrl,
            logo = load.logoUrl,
            description = load.plot,
            releaseInfo = load.year?.toString(),
            released = load.year?.let { "$it-01-01" },
            runtime = load.duration?.let { "${it}m" },
            videos = videos,
            trailers = load.trailers?.mapIndexedNotNull { index, trailer -> trailer.toDetailTrailer(apiName, index) },
            imdbRating = load.rating?.let(::ratingString),
            ageRating = load.contentRating,
            ratings = load.rating?.let { listOf(MetaRating("Cloudstream", ratingString(it))) },
            cast = load.actors?.map { it.toCastMember() },
            links = load.toMetaLinks(),
            status = if (load.comingSoon) "Coming Soon" else load.status,
            originalName = load.synonyms?.firstOrNull { it != load.title },
            collectionParts = load.recommendations?.mapNotNull { it.toMeta(apiName) }
        )
    }

    suspend fun loadStreams(id: String, directTimeoutMs: Long): List<Stream> {
        val (apiName, data) = CloudStreamCatalogClient.decodeCsId(id) ?: return emptyList()
        val api = pluginManager.loadedApis.value.firstOrNull { it.name == apiName } ?: return emptyList()
        val runner = ExternalExtensionRunner()
        val direct = try {
            withTimeoutOrNull(directTimeoutMs) { runner.loadStreams(api, data) }
        } catch (_: Throwable) {
            null
        }
        if (direct != null && direct.links.isNotEmpty()) return direct.links.toStreams(apiName, direct.subtitles)
        val streamData = try {
            withTimeoutOrNull(15_000L) { runner.loadContent(api, data)?.data }
        } catch (_: Exception) {
            null
        } ?: data
        val result = withTimeoutOrNull(30_000L) { runner.loadStreams(api, streamData) } ?: return emptyList()
        return result.links.toStreams(apiName, result.subtitles)
    }

    fun qualityScore(quality: String): Int {
        val value = quality.lowercase()
        return when {
            value.contains("4k") || value.contains("2160") -> 2160
            value.contains("1440") -> 1440
            value.contains("1080") -> 1080
            value.contains("720") -> 720
            value.contains("480") -> 480
            value.contains("360") -> 360
            value.contains("240") -> 240
            else -> 0
        }
    }

    private fun List<com.fluxa.app.plugins.cloudstream.ScraperStreamLink>.toStreams(
        apiName: String,
        subtitles: List<ScraperSubtitle>
    ): List<Stream> = sortedByDescending { qualityScore(it.quality) }.map { link ->
        Stream(
            name = " $apiName\n${link.quality}",
            title = link.name,
            url = link.url,
            subtitles = subtitles.map { it.toSubtitleData() },
            behaviorHints = buildMap {
                put("proxyHeaders", buildMap { put("request", link.headers) })
                link.referer?.let { put("referer", it) }
                put("cs3Type", link.type)
                put("isM3u8", link.isM3u8)
                put("isDash", link.isDash)
            },
            addonName = " $apiName"
        )
    }

    private fun Long.toIsoDate(): String {
        val millis = if (this > 10_000_000_000L) this else this * 1000L
        return java.time.Instant.ofEpochMilli(millis).atZone(java.time.ZoneOffset.UTC).toLocalDate().toString()
    }

    private fun ratingString(rating: Int): String = "%.1f".format(java.util.Locale.US, rating.toFloat() / 10f)

    private fun ScraperActor.toCastMember() = CastMember(name = name, character = role, profilePath = image)

    private fun ScraperTrailer.toDetailTrailer(apiName: String, index: Int): DetailTrailer? {
        val targetUrl = url.takeIf { it.isNotBlank() } ?: return null
        return DetailTrailer("cs3:$apiName:trailer:$index", "Trailer ${index + 1}", if (raw) "Trailer" else "Extractor", targetUrl, null, apiName)
    }

    private fun ScraperSearchResult.toMeta(apiName: String): Meta? {
        val name = title.takeIf { it.isNotBlank() } ?: return null
        return Meta(
            id = CloudStreamCatalogClient.encodeCsId(apiName, url),
            name = name,
            type = type?.toStremioType() ?: "movie",
            poster = posterUrl,
            releaseInfo = year?.toString(),
            imdbRating = quality,
            background = posterUrl
        )
    }

    private fun ScraperLoadResult.toMetaLinks(): List<MetaLink>? {
        val links = mutableListOf<MetaLink>()
        uniqueUrl?.takeIf { it.isNotBlank() }?.let { links += MetaLink("Source", "Cloudstream", it) }
        url.takeIf { it.isNotBlank() && it != uniqueUrl }?.let { links += MetaLink("Page", "Cloudstream", it) }
        syncData.orEmpty().forEach { (key, value) -> if (key.isNotBlank() && value.isNotBlank()) links += MetaLink(key, "Cloudstream Sync", value) }
        synonyms.orEmpty().forEach { links += MetaLink(it, "Cloudstream Synonym", it) }
        nextAiringUnixTime?.let { unix ->
            val label = listOfNotNull(nextAiringSeason?.let { "S$it" }, nextAiringEpisode?.let { "E$it" }).joinToString("").ifBlank { "Next airing" }
            links += MetaLink(label, "Cloudstream Next Airing", unix.toString())
        }
        seasonNames.orEmpty().forEach { season ->
            val name = season.name?.takeIf { it.isNotBlank() } ?: "Season ${season.displaySeason ?: season.season}"
            links += MetaLink(name, "Cloudstream Season", season.season.toString())
        }
        return links.takeIf { it.isNotEmpty() }
    }

    private fun ScraperSubtitle.toSubtitleData() = SubtitleData(url = url, lang = lang)
}
