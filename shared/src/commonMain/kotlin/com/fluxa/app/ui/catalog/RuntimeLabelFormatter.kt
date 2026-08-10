package com.fluxa.app.ui.catalog

private val HOURS_MINUTES_REGEX = Regex("""^(?:(\d+)\s*h(?:ours?)?\s*)?(?:(\d+)\s*(?:m|min|mins|minute|minutes))?$""")
private val HOURS_ONLY_REGEX = Regex("""^(\d+)\s*h(?:ours?)?$""")
private val MINUTES_ONLY_REGEX = Regex("""^(\d+)\s*(?:m|min|mins|minute|minutes)$""")

fun formatRuntimeLabel(raw: String?): String? {
    val value = raw?.trim()?.takeIf { it.isNotBlank() } ?: return null
    val normalized = value.lowercase().replace("·", " ").replace("  ", " ").trim()

    HOURS_MINUTES_REGEX.matchEntire(normalized)?.let { match ->
        val hours = match.groupValues.getOrNull(1)?.takeIf { it.isNotBlank() }?.toIntOrNull() ?: 0
        val minutes = match.groupValues.getOrNull(2)?.takeIf { it.isNotBlank() }?.toIntOrNull() ?: 0
        if (hours > 0 || minutes > 0) return buildRuntimeLabel(hours, minutes)
    }

    HOURS_ONLY_REGEX.matchEntire(normalized)?.groupValues?.getOrNull(1)?.toIntOrNull()?.let { hours ->
        if (hours > 0) return buildRuntimeLabel(hours = hours, minutes = 0)
    }

    MINUTES_ONLY_REGEX.matchEntire(normalized)?.groupValues?.getOrNull(1)?.toIntOrNull()?.let { totalMinutes ->
        if (totalMinutes > 0) return buildRuntimeLabel(hours = totalMinutes / 60, minutes = totalMinutes % 60)
    }

    return value
        .replace(Regex("""(\d+)h(\d+)"""), "$1h $2")
        .replace(Regex("""(\d+)\s*min""", RegexOption.IGNORE_CASE), "$1min")
        .replace(Regex("""\s+"""), " ")
        .trim()
}

private fun buildRuntimeLabel(hours: Int, minutes: Int): String = when {
    hours > 0 && minutes > 0 -> "${hours}h ${minutes}min"
    hours > 0 -> "${hours}h"
    else -> "${minutes}min"
}
