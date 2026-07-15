package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta

internal fun parseShortVideoId(id: String?): String? {
    if (id.isNullOrBlank() || id.startsWith("cs3:")) return null
    val parts = id.split(":")
    if (parts.size >= 3) {
        val season = parts[parts.size - 2].toIntOrNull()
        val episode = parts[parts.size - 1].toIntOrNull()
        if (season != null && episode != null) {
            return "S$season, E$episode"
        }
    }
    return null
}

fun continueWatchingEpisodeLabel(meta: Meta): String? {
    val code = parseShortVideoId(meta.lastVideoId)
    val title = continueWatchingEpisodeTitle(meta)
    return when {
        code != null && title != null && !title.startsWith(code, ignoreCase = true) -> "$code: $title"
        code != null && title != null -> title
        code != null -> code
        else -> title
    }
}

private val EPISODE_TITLE_PREFIX_REGEX = Regex("""(?i)^S\s*\d+\s*[,: \-]*E\s*\d+\s*[:\-]?\s*""")

fun continueWatchingEpisodeTitle(meta: Meta): String? {
    return meta.lastEpisodeName
        ?.trim()
        ?.replace(EPISODE_TITLE_PREFIX_REGEX, "")
        ?.trim()
        ?.takeIf { it.isNotBlank() }
}

fun Meta.isUpNextContinueItem(): Boolean {
    val isSeries = type == "series" || type == "tv" || type == "anime"
    return isSeries &&
        !lastVideoId.isNullOrBlank() &&
        (timeOffset ?: 0L) <= 0L &&
        (duration ?: 0L) <= 0L
}
