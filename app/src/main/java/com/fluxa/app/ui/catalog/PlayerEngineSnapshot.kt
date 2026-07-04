package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Immutable


@Immutable
internal data class PlaybackSnapshot(
    val isPlaying: Boolean = false,
    val playWhenReadyForScrobble: Boolean = true,
    val isBuffering: Boolean = true,
    val hasStartedPlaying: Boolean = false,
    val playbackEnded: Boolean = false,
)

@Immutable
internal data class TimelineSnapshot(
    val position: Long = 0L,
    val duration: Long = 0L,
)

@Immutable
internal data class BufferSnapshot(
    val bufferPercent: Int = 0,
    val loadProgress: Float = 0f,
    val seekbarBufferedProgress: Float = 0f,
)

@Immutable
internal data class RenderSnapshot(
    val isVideoRendered: Boolean = false,
)

@Immutable
internal data class PlayerEngineSnapshot(
    val playback: PlaybackSnapshot = PlaybackSnapshot(),
    val timeline: TimelineSnapshot = TimelineSnapshot(),
    val buffer: BufferSnapshot = BufferSnapshot(),
    val render: RenderSnapshot = RenderSnapshot(),
    val playerError: String? = null,
)
