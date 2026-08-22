package com.fluxa.app.shared.feature.player

import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

class PlayerStateStoreTest {
    @Test
    fun normalizesCommandsAndUpdatesPortableState() = runTest {
        val sink = RecordingPlayerCommandSink()
        val store = PlayerStateStore(sink, PlayerUiState(durationMs = 1_000L))

        store.dispatch(PlayerAction.SeekTo(2_000L))
        store.dispatch(PlayerAction.SetVolume(2f))
        store.dispatch(PlayerAction.PlayPause)

        assertEquals(1_000L, store.state.value.positionMs)
        assertEquals(1f, store.state.value.volume)
        assertEquals(true, store.state.value.isPlaying)
        assertEquals(listOf("seek:1000", "volume:1.0", "playing:true"), sink.commands)
    }
}

private class RecordingPlayerCommandSink : PlayerCommandSink {
    val commands = mutableListOf<String>()

    override suspend fun setPlaying(playing: Boolean) {
        commands += "playing:$playing"
    }
    override suspend fun seekTo(positionMs: Long) {
        commands += "seek:$positionMs"
    }
    override suspend fun setVolume(volume: Float) {
        commands += "volume:$volume"
    }
    override suspend fun setSubtitleEnabled(enabled: Boolean) {
        commands += "subtitles:$enabled"
    }
    override suspend fun stop() {
        commands += "stop"
    }
}
