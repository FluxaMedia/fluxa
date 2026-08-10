package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.remote.Meta

internal fun parseShortVideoId(id: String?): String? {
    if (id.isNullOrBlank() || id.startsWith("cs3:")) return null
    val parts = id.split(":")
    if (parts.size >= 3) {
        val season = parts[parts.size - 2].toIntOrNull()
        val episode = parts[parts.size - 1].toIntOrNull()
        if (season != null && episode != null) {
            return "S$season E$episode"
        }
    }
    return null
}

fun continueWatchingEpisodeLabel(meta: Meta): String? {
    val code = parseShortVideoId(meta.lastVideoId) ?: parseEpisodeCodeFromName(meta.lastEpisodeName)
    val title = continueWatchingEpisodeTitle(meta)
    return when {
        code != null && title != null -> "$code · $title"
        code != null -> code
        else -> title
    }
}

private val EPISODE_CODE_IN_NAME_REGEX = Regex("""(?i)^S\s*(\d+)\s*[, ]*\s*E\s*(\d+)""")
private val EPISODE_TITLE_PREFIX_REGEX = Regex("""(?i)^S\s*\d+\s*[, ]*\s*E\s*\d+\s*(?:[·:\-]\s*)?""")

private fun parseEpisodeCodeFromName(name: String?): String? {
    val match = name?.trim()?.let(EPISODE_CODE_IN_NAME_REGEX::find) ?: return null
    val season = match.groupValues.getOrNull(1)?.toIntOrNull() ?: return null
    val episode = match.groupValues.getOrNull(2)?.toIntOrNull() ?: return null
    return "S$season E$episode"
}

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
        (duration ?: 0L) <= 0L &&
        resumeProgressPercent == null
}

fun Meta.remainingLabel(language: String?): String? {
    val offset = timeOffset
    val total = duration
    if (offset != null && total != null && offset > 0L && total > 0L) {
        val remainingMinutes = ((total - offset).coerceAtLeast(0L) / 60_000L).toInt()
        return when {
            remainingMinutes < 1 -> AppStrings.t(language, "format.remaining_almost_done")
            remainingMinutes < 60 -> AppStrings.format(language, "format.remaining_minutes", remainingMinutes)
            remainingMinutes % 60 == 0 -> AppStrings.format(language, "format.remaining_hours", remainingMinutes / 60)
            else -> AppStrings.format(language, "format.remaining_hours_minutes", remainingMinutes / 60, remainingMinutes % 60)
        }
    }
    return resumeProgressPercent?.let { AppStrings.format(language, "format.watched_percent", it.toInt()) }
}
