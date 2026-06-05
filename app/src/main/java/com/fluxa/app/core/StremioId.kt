package com.fluxa.app.core

import com.fluxa.app.core.rust.FluxaCoreNative

internal object StremioId {
    private val imdbRegex = Regex("""tt\d+""")

    fun imdbId(id: String?): String? {
        val raw = id ?: return null
        return imdbRegex.find(raw)?.value
    }

    fun baseContentId(id: String): String {
        return FluxaCoreNative.parseEpisodeLocator(id)?.baseId?.takeIf { it.isNotBlank() } ?: id
    }

    fun parseEpisodeLocator(id: String?): Pair<Int, Int>? {
        return FluxaCoreNative.parseEpisodeLocator(id)?.let { it.season to it.episode }
    }

    fun normalizeSeriesLookupId(rawId: String): String {
        return imdbId(rawId) ?: baseContentId(rawId)
    }

    fun isTmdbLikeContentId(id: String): Boolean {
        val base = baseContentId(id)
        return base.startsWith("tmdb:", ignoreCase = true) || base.toIntOrNull() != null
    }

    fun tmdbNumericId(id: String): String? {
        val base = baseContentId(id)
        return base.removePrefix("tmdb:").takeIf { it.toIntOrNull() != null }
    }

    fun streamRequestIds(
        type: String,
        id: String,
        detailId: String?,
        currentSeriesLookupId: String?,
        canonicalBaseId: String?
    ): List<String> {
        return FluxaCoreNative.streamRequestIds(
            type = type,
            id = id,
            detailId = detailId,
            currentSeriesLookupId = currentSeriesLookupId,
            canonicalBaseId = canonicalBaseId
        )
    }
}
