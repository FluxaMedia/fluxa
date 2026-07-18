package com.fluxa.app.player

data class TrailerSubtitle(
    val languageTag: String,
    val label: String,
    val url: String,
    val mimeType: String,
    val isAuto: Boolean
)

data class TrailerResult(
    val streamUrl: String,
    val audioUrl: String?,
    val subtitles: List<TrailerSubtitle>,
    val streamMimeType: String?
)

data class TrailerCue(
    val start: Double,
    val end: Double,
    val text: String
)

sealed interface TrailerResolveResult {
    data class Ok(val data: TrailerResult) : TrailerResolveResult
    data object GeoBlocked : TrailerResolveResult
    data object Failed : TrailerResolveResult
}

object TrailerPolicy {
    fun youtubeVideoId(url: String): String? =
        Regex("(?:v=|youtu\\.be/|embed/)([A-Za-z0-9_-]{11})").find(url)?.groupValues?.get(1)
}
