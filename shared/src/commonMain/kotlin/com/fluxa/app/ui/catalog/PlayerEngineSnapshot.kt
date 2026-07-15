package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Immutable


@Immutable
data class PlaybackSnapshot(
    val isPlaying: Boolean = false,
    val playWhenReadyForScrobble: Boolean = true,
    val isBuffering: Boolean = true,
    val hasStartedPlaying: Boolean = false,
    val playbackEnded: Boolean = false,
)

@Immutable
data class TimelineSnapshot(
    val position: Long = 0L,
    val duration: Long = 0L,
)

@Immutable
data class BufferSnapshot(
    val bufferPercent: Int = 0,
    val loadProgress: Float = 0f,
    val seekbarBufferedProgress: Float = 0f,
)

@Immutable
data class RenderSnapshot(
    val isVideoRendered: Boolean = false,
)

@Immutable
data class PlayerEngineSnapshot(
    val playback: PlaybackSnapshot = PlaybackSnapshot(),
    val timeline: TimelineSnapshot = TimelineSnapshot(),
    val buffer: BufferSnapshot = BufferSnapshot(),
    val render: RenderSnapshot = RenderSnapshot(),
    val playerError: String? = null,
)
