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
        parsed: LocalMediaParsedName,
        kind: LocalMediaKind,
    ): LocalMediaMetadataMatch? = withContext(Dispatchers.IO) {
        val requestedType = if (kind == LocalMediaKind.Movies) "movie" else "series"

        if (parsed.explicitMetadataProvider in setOf("imdb", "tmdb") && !parsed.explicitMetadataId.isNullOrBlank()) {
            val detail = runCatching {
                addonRepository.getAddonMetaDetail(
                    type = requestedType,
                    id = if (parsed.explicitMetadataProvider == "tmdb") "tmdb:${parsed.explicitMetadataId}" else parsed.explicitMetadataId,
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
        parsed: LocalMediaParsedName,
        detail: MetaDetail?,
    ): Video? {
        val videos = detail?.videos.orEmpty()
        val matched = jvmLocalMediaCorePolicy.resolveVideo(
            parsed,
            videos.map { LocalMediaCoreVideo(it.id, it.season, it.number) },
        ) ?: return null
        return videos.firstOrNull { it.id == matched.id }
    }

    private fun score(parsed: LocalMediaParsedName, meta: Meta, kind: LocalMediaKind): Float {
        return jvmLocalMediaCorePolicy.score(
            parsed,
            LocalMediaCoreMeta(meta.id, meta.name, meta.type, meta.releaseInfo, meta.released),
            kind,
        )
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
