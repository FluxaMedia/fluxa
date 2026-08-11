package com.fluxa.app.shared

private const val SYNOPSIS_MAX_LENGTH = 200
private const val SYNOPSIS_TOLERANCE = 60
private val DASH_REGEX = Regex("\\s*[—–]\\s*")

private fun collapseWhitespace(value: String): String = value.split(Regex("\\s+")).filter { it.isNotEmpty() }.joinToString(" ")

private fun normalizeDashesToCommas(text: String): String =
    collapseWhitespace(DASH_REGEX.replace(text, ", ").replace(",,", ","))

private fun firstSentence(text: String): String? {
    for (i in text.indices) {
        val ch = text[i]
        if ((ch == '.' || ch == '!' || ch == '?') && i >= 20) {
            val atBoundary = i + 1 >= text.length || text[i + 1].isWhitespace()
            if (atBoundary) return text.substring(0, i + 1)
        }
    }
    return null
}

actual fun shortenHeroSynopsis(text: String): String {
    val normalized = normalizeDashesToCommas(text.trim())
    firstSentence(normalized)?.let { sentence ->
        if (sentence.length <= SYNOPSIS_MAX_LENGTH + SYNOPSIS_TOLERANCE) return sentence
    }
    if (normalized.length <= SYNOPSIS_MAX_LENGTH + SYNOPSIS_TOLERANCE) return normalized
    val window = normalized.take(SYNOPSIS_MAX_LENGTH)
    val lastComma = window.lastIndexOf(',')
    if (lastComma > SYNOPSIS_MAX_LENGTH * 0.4) return "${window.substring(0, lastComma)}."
    val lastSpace = window.lastIndexOf(' ')
    if (lastSpace > 0) return "${window.substring(0, lastSpace)}."
    return "$window."
}
