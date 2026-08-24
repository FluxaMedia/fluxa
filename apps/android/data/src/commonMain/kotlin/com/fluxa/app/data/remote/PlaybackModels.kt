package com.fluxa.app.data.remote

data class SubtitleData(
    val attributes: SubtitleAttributes = SubtitleAttributes(),
    val id: String? = null,
    val url: String? = null,
    val lang: String? = null
)

data class AudioTrackData(
    val id: String = "",
    val url: String = "",
    val lang: String = "",
    val label: String? = null,
    val headers: Map<String, String>? = null
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
