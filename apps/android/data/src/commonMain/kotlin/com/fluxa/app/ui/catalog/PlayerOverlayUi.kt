package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.Video

data class SkipSegmentUiModel(
    val type: String,
    val startMs: Long,
    val endMs: Long
) {
    fun dismissKey(): String = "$type:$startMs:$endMs"
}

fun IntroTimestamps.toSkipSegmentUiModel(): SkipSegmentUiModel = SkipSegmentUiModel(
    type = type,
    startMs = startTime,
    endMs = endTime
)

fun IntroTimestamps.dismissKey(): String {
    return "$type:$startTime:$endTime"
}

data class NextEpisodePreviewUiModel(
    val id: String,
    val thumbnail: String?,
    val season: Int?,
    val number: Int?,
    val name: String?
)

fun Video.toNextEpisodePreviewUiModel(): NextEpisodePreviewUiModel = NextEpisodePreviewUiModel(
    id = id,
    thumbnail = thumbnail,
    season = season,
    number = number,
    name = name
)
