package com.fluxa.app.core.rust.models

data class NativeDiscordPresenceSnapshot(
    val title: String,
    val episodeLine: String,
    val status: String,
    val positionMs: Long,
    val durationMs: Long,
    val artworkUrl: String?
)
