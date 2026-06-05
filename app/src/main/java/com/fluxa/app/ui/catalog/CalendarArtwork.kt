package com.fluxa.app.ui.catalog

internal fun CalendarUpcomingItem.artworkUrl(): String? {
    return listOf(
        poster,
        meta.poster,
        meta.continueWatchingPoster,
        episodePoster,
        meta.background,
        meta.continueWatchingBackground
    ).firstOrNull(::isUsableCalendarArtwork)
}

internal fun isUsableCalendarArtwork(url: String?): Boolean {
    val value = url?.trim().orEmpty()
    if (value.isBlank()) return false
    val lower = value.lowercase()
    return lower != "null" &&
        !lower.contains("placeholder") &&
        !lower.contains("no-image") &&
        !lower.contains("no_image") &&
        !lower.contains("default-poster")
}
