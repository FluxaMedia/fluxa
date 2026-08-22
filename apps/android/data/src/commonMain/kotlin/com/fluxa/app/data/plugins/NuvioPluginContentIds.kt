package com.fluxa.app.data.plugins

/** Mirrors NuvioMobile's plugin content-id normalization before getStreams(). */
fun nuvioPluginContentId(
    videoId: String,
    season: Int?,
    episode: Int?,
): String {
    val trimmed = videoId.trim()
    if (trimmed.isBlank()) return videoId

    val withoutPrefix = when {
        trimmed.startsWith("tmdb:") -> trimmed.removePrefix("tmdb:")
        trimmed.startsWith("tmdb/") -> trimmed.removePrefix("tmdb/")
        else -> trimmed
    }

    val withoutEpisodeSuffix = if (season != null && episode != null) {
        withoutPrefix.removeSuffix(":$season:$episode")
    } else {
        withoutPrefix
    }

    return withoutEpisodeSuffix.substringBefore('/').ifBlank { trimmed }
}

fun normalizeNuvioPluginType(value: String): String = when (value.trim().lowercase()) {
    "tv", "series", "show", "tvshow" -> "tv"
    "movie", "film" -> "movie"
    else -> value.trim().lowercase()
}
