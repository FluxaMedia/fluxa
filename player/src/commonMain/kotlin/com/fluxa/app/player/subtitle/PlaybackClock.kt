package com.fluxa.app.player.subtitle

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlin.time.TimeMark
import kotlin.time.TimeSource

interface PlaybackClock {
    fun positionUs(): Long
    val playbackRate: StateFlow<Float>
    val discontinuities: Flow<Long>
}

class MonotonicPlaybackClock : PlaybackClock {
    private data class SyncPoint(val mediaPositionUs: Long, val mark: TimeMark, val rate: Float)

    private var sync = SyncPoint(0L, TimeSource.Monotonic.markNow(), 0f)

    private val _playbackRate = MutableStateFlow(0f)
    override val playbackRate: StateFlow<Float> = _playbackRate

    private val _discontinuities = MutableSharedFlow<Long>(extraBufferCapacity = 1)
    override val discontinuities: Flow<Long> = _discontinuities

    override fun positionUs(): Long {
        val point = sync
        val elapsedUs = point.mark.elapsedNow().inWholeMicroseconds
        return point.mediaPositionUs + (elapsedUs * point.rate).toLong()
    }

    fun resync(mediaPositionUs: Long, rate: Float = sync.rate, discontinuity: Boolean = false) {
        sync = SyncPoint(mediaPositionUs, TimeSource.Monotonic.markNow(), rate)
        _playbackRate.value = rate
        if (discontinuity) _discontinuities.tryEmit(mediaPositionUs)
    }
}
