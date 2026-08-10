package com.fluxa.app.shared.feature.localmedia

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.ui.catalog.formatRuntimeLabel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

internal data class LocalMediaMetadataMatch(
    val catalog: LocalMediaCatalogEntry,
    val detail: MetaDetail?,
    val confidence: Float,
)

internal class LocalMediaMetadataResolver(
    private val addonRepository: AddonRepository,
    private val authKey: () -> String,
    private val localAddons: () -> List<String>,
    private val language: () -> String,
) {
    suspend fun resolve(
        parsed: LocalMediaFilenameParser.ParsedName,
        kind: LocalMediaKind,
    ): LocalMediaMetadataMatch? = withContext(Dispatchers.IO) {
        val requestedType = if (kind == LocalMediaKind.Movies) "movie" else "series"

        if (parsed.explicitMetadataProvider == "imdb" && !parsed.explicitMetadataId.isNullOrBlank()) {
            val detail = runCatching {
                addonRepository.getAddonMetaDetail(
                    type = requestedType,
                    id = parsed.explicitMetadataId,
                    authKey = authKey(),
                    localAddons = localAddons(),
                )
            }.getOrNull()
            if (detail != null) {
                return@withContext LocalMediaMetadataMatch(
                    catalog = detail.toCatalogEntry(kind, metadataAddonUrl = null, fileCount = 1),
                    detail = detail,
                    confidence = 1f,
                )
            }
        }

        val rows = runCatching {
            addonRepository.searchRows(
                query = parsed.title,
                language = language(),
                authKey = authKey(),
                localAddons = localAddons(),
            )
        }.getOrDefault(emptyList())

        data class Candidate(val meta: Meta, val addonUrl: String?, val score: Float)
        val best = rows.asSequence()
            .flatMap { row -> row.items.asSequence().map { Candidate(it, row.sourceAddonTransportUrl, score(parsed, it, kind)) } }
            .filter { candidate ->
                val type = candidate.meta.type.lowercase()
                if (kind == LocalMediaKind.Movies) type == "movie" else type in setOf("series", "tv", "anime")
            }
            .maxByOrNull { it.score }
            ?.takeIf { it.score >= 0.62f }
            ?: return@withContext null

        val detail = best.addonUrl?.let { url ->
            runCatching {
                addonRepository.getMetaDetailFromSpecificAddon(
                    transportUrl = url,
                    type = best.meta.type.ifBlank { requestedType },
                    id = best.meta.id,
                    alternateTypes = if (requestedType == "series") listOf("series", "tv", "anime") else emptyList(),
                )
            }.getOrNull()
        }
        LocalMediaMetadataMatch(
            catalog = (detail?.toCatalogEntry(kind, best.addonUrl, 1)
                ?: best.meta.toCatalogEntry(kind, best.addonUrl, 1)),
            detail = detail,
            confidence = best.score,
        )
    }

    fun resolveVideo(
        parsed: LocalMediaFilenameParser.ParsedName,
        detail: MetaDetail?,
    ): Video? {
        val videos = detail?.videos.orEmpty()
        if (videos.isEmpty()) return null
        if (parsed.season != null && parsed.episode != null) {
            return videos.firstOrNull { it.season == parsed.season && it.number == parsed.episode }
                ?: videos.firstOrNull { it.number == parsed.episode }
        }
        val absolute = parsed.absoluteEpisode ?: return null
        videos.firstOrNull { it.number == absolute && (it.season ?: 1) <= 1 }?.let { return it }
        return videos
            .filter { (it.season ?: 0) >= 0 && (it.number ?: 0) > 0 }
            .sortedWith(compareBy<Video> { it.season ?: 0 }.thenBy { it.number ?: 0 })
            .getOrNull(absolute - 1)
    }

    private fun score(parsed: LocalMediaFilenameParser.ParsedName, meta: Meta, kind: LocalMediaKind): Float {
        var score = LocalMediaFilenameParser.titleSimilarity(parsed.title, meta.name) * 0.82f
        val metaYear = meta.releaseInfo?.take(4)?.toIntOrNull() ?: meta.released?.take(4)?.toIntOrNull()
        if (parsed.year != null && metaYear != null) {
            score += when (kotlin.math.abs(parsed.year - metaYear)) {
                0 -> 0.18f
                1 -> 0.06f
                else -> -0.12f
            }
        } else {
            score += 0.08f
        }
        val type = meta.type.lowercase()
        val typeMatches = if (kind == LocalMediaKind.Movies) type == "movie" else type in setOf("series", "tv", "anime")
        if (!typeMatches) score -= 0.35f
        return score.coerceIn(0f, 1f)
    }

    private fun Meta.toCatalogEntry(kind: LocalMediaKind, metadataAddonUrl: String?, fileCount: Int) = LocalMediaCatalogEntry(
        contentId = id,
        contentType = if (kind == LocalMediaKind.Movies) "movie" else "series",
        kind = kind,
        title = name,
        year = releaseInfo?.take(4)?.toIntOrNull() ?: released?.take(4)?.toIntOrNull(),
        posterUrl = poster,
        backdropUrl = background,
        logoUrl = logo,
        description = description,
        releaseLabel = releaseInfo,
        ratingLabel = imdbRating,
        ageRating = ageRating,
        genres = genres.orEmpty(),
        seasonsCount = seasonsCount,
        runtimeLabel = formatRuntimeLabel(runtime),
        metadataAddonUrl = metadataAddonUrl,
        fileCount = fileCount,
    )

    private fun MetaDetail.toCatalogEntry(kind: LocalMediaKind, metadataAddonUrl: String?, fileCount: Int) = LocalMediaCatalogEntry(
        contentId = id,
        contentType = if (kind == LocalMediaKind.Movies) "movie" else "series",
        kind = kind,
        title = name,
        year = releaseInfo?.take(4)?.toIntOrNull() ?: released?.take(4)?.toIntOrNull(),
        posterUrl = poster,
        backdropUrl = background,
        logoUrl = logo,
        description = description,
        releaseLabel = releaseInfo,
        ratingLabel = imdbRating,
        ageRating = ageRating,
        genres = genres.orEmpty(),
        seasonsCount = seasonsCount,
        runtimeLabel = formatRuntimeLabel(runtime),
        metadataAddonUrl = metadataAddonUrl,
        fileCount = fileCount,
    )
}
