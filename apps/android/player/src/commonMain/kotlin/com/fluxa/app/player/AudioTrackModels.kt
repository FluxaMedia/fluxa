package com.fluxa.app.player

data class ExternalAudioTrack(
    val id: String,
    val url: String,
    val label: String,
    val language: String?,
    val sourceName: String,
    val audioOnly: Boolean,
    val headers: Map<String, String> = emptyMap()
)
