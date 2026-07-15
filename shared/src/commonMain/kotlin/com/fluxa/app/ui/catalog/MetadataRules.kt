package com.fluxa.app.ui.catalog

import com.fluxa.app.common.ReleaseDateUtils

private val youtubeIdRegex = Regex("^[A-Za-z0-9_-]{11}$")
private val youtubeUrlRegex = Regex("(?:youtube\\.com/watch\\?[^#]*v=|youtube\\.com/embed/|youtube\\.com/shorts/)([A-Za-z0-9_-]{6,})", RegexOption.IGNORE_CASE)
private val youtubeShortRegex = Regex("youtu\\.be/([A-Za-z0-9_-]{6,})", RegexOption.IGNORE_CASE)
private val videoExtensions = setOf(".mp4", ".m3u8", ".mpd", ".webm", ".mov")

fun detailIsUpcoming(date: String?): Boolean = ReleaseDateUtils.isUpcoming(date)

fun String.extractYoutubeVideoId(): String? {
    val value = trim()
    if (youtubeIdRegex.matches(value)) return value
    return youtubeUrlRegex.find(value)?.groupValues?.getOrNull(1)
        ?: youtubeShortRegex.find(value)?.groupValues?.getOrNull(1)
}

fun String.isDirectVideoPreviewUrl(): Boolean {
    val value = trim().lowercase()
    return videoExtensions.any(value::contains) || value.contains("googlevideo.com") || value.contains("video.twimg.com")
}
