package com.fluxa.app.player.subtitle

data class TextCue(
    val startUs: Long,
    val endUs: Long,
    val text: String
)

data class SubtitleFrame(
    val cues: List<TextCue>,
    val validUntilUs: Long?
)

enum class SubtitleFormat { SRT, WEBVTT, TTML }

sealed class SubtitleSource {
    data class Sidecar(
        val url: String,
        val format: SubtitleFormat,
        val headers: Map<String, String> = emptyMap()
    ) : SubtitleSource()
}
