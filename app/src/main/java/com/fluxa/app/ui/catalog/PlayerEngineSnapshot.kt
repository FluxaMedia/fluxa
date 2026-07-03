package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Immutable

/**
 * Composable-domain sub-snapshots for player engine state.
 *
 * Each sub-snapshot is a stable data class. Composables that receive only one
 * sub-snapshot skip recomposition entirely when other sub-snapshots change.
 *
 * Update rule: only copy the sub-snapshot whose fields actually changed.
 *   engine.copy(timeline = engine.timeline.copy(position = newPos))
 * → engine.playback, .buffer, .render, .playerError retain the same object reference
 * → composables reading only those sub-snapshots are skipped by Compose.
 */

/** Changes on play/pause, buffering, and episode transitions. */
@Immutable
internal data class PlaybackSnapshot(
    val isPlaying: Boolean = false,
    val playWhenReadyForScrobble: Boolean = true,
    val isBuffering: Boolean = true,
    val hasStartedPlaying: Boolean = false,
    val playbackEnded: Boolean = false,
)

/** Changes every 500 ms (position) and once on player-ready (duration). */
@Immutable
internal data class TimelineSnapshot(
    val position: Long = 0L,
    val duration: Long = 0L,
)

/** Changes on ExoPlayer buffer events and player-state transitions. */
@Immutable
internal data class BufferSnapshot(
    val bufferPercent: Int = 0,
    val loadProgress: Float = 0f,
    val seekbarBufferedProgress: Float = 0f,
)

/** Changes once per stream: false → true on first frame rendered. */
@Immutable
internal data class RenderSnapshot(
    val isVideoRendered: Boolean = false,
)

/**
 * Single atomic snapshot of all player-engine-driven state.
 * One `mutableStateOf<PlayerEngineSnapshot>` in PlayerScreenState.
 * All sub-snapshots are stable data classes; Compose uses structural equality
 * to skip recomposition of composables whose sub-snapshots did not change.
 */
@Immutable
internal data class PlayerEngineSnapshot(
    val playback: PlaybackSnapshot = PlaybackSnapshot(),
    val timeline: TimelineSnapshot = TimelineSnapshot(),
    val buffer: BufferSnapshot = BufferSnapshot(),
    val render: RenderSnapshot = RenderSnapshot(),
    val playerError: String? = null,
)
