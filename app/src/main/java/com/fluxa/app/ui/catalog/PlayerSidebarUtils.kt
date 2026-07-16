package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta
import java.util.Locale

internal fun nativeLanguageName(code: String): String {
    val normalized = code.lowercase(Locale.ROOT)
    val locale = Locale.forLanguageTag(normalized)
    val native = locale.getDisplayLanguage(locale).trim()
    return native.takeIf { it.isNotBlank() }?.replaceFirstChar { it.titlecase(locale) } ?: code
}

internal fun Meta.withCurrentEpisodeArtwork(artwork: String?): Meta {
    val episodeArtwork = artwork?.takeIf { it.isNotBlank() } ?: return this
    if (type != "series") return this
    return copy(continueWatchingPoster = episodeArtwork, continueWatchingBackground = episodeArtwork)
}
