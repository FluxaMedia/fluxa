package com.fluxa.app.core

import com.fluxa.app.core.rust.FluxaCoreNative

object StremioId {
    fun imdbId(id: String?): String? = id?.let(FluxaCoreNative::contentImdbId)

    fun baseContentId(id: String): String = FluxaCoreNative.contentBaseId(id)

    fun parseEpisodeLocator(id: String?): Pair<Int, Int>? {
        return FluxaCoreNative.parseEpisodeLocator(id)?.let { it.season to it.episode }
    }

    fun normalizeSeriesLookupId(rawId: String): String = FluxaCoreNative.normalizeSeriesLookupId(rawId)

    fun isTmdbLikeContentId(id: String): Boolean = FluxaCoreNative.isTmdbLikeContentId(id)

    fun tmdbNumericId(id: String): String? = FluxaCoreNative.tmdbNumericId(id)

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
