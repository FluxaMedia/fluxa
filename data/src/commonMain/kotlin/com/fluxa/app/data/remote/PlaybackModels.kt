package com.fluxa.app.data.remote

data class SubtitleData(
    val attributes: SubtitleAttributes = SubtitleAttributes(),
    val id: String? = null,
    val url: String? = null,
    val lang: String? = null
)

data class SubtitleAttributes(
    val url: String = "",
    val languages: List<String> = emptyList(),
    val fps: Double? = null
)

data class IntroTimestamps(
    val startTime: Long,
    val endTime: Long,
    val type: String = "intro"
)
