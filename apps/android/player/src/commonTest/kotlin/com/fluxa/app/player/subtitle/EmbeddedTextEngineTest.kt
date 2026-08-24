package com.fluxa.app.player.subtitle

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlin.coroutines.EmptyCoroutineContext
import kotlin.test.Test
import kotlin.test.assertEquals

private class StoppedClock : PlaybackClock {
    override fun positionUs(): Long = 0L
    override val playbackRate: StateFlow<Float> = MutableStateFlow(0f)
    override val discontinuities: Flow<Long> = MutableSharedFlow()
}

class EmbeddedTextEngineTest {
    private fun engine() = EmbeddedTextEngine(StoppedClock(), CoroutineScope(EmptyCoroutineContext))

    @Test
    fun matroskaDeliversOneCuePerSampleAndTheyMustAccumulate() {
        val engine = engine()
        engine.addEmbeddedCues(listOf(TextCue(1_000_000L, 2_000_000L, "first")))
        engine.addEmbeddedCues(listOf(TextCue(3_000_000L, 4_000_000L, "second")))

        assertEquals(listOf("first", "second"), engine.cues.value.map { it.text })
    }

    @Test
    fun replayedSamplesAfterSeekDoNotDuplicate() {
        val engine = engine()
        val cue = TextCue(1_000_000L, 2_000_000L, "first")
        engine.addEmbeddedCues(listOf(cue))
        engine.addEmbeddedCues(listOf(cue))

        assertEquals(1, engine.cues.value.size)
    }

    @Test
    fun outOfOrderCuesAreExposedSorted() {
        val engine = engine()
        engine.addEmbeddedCues(listOf(TextCue(5_000_000L, 6_000_000L, "late")))
        engine.addEmbeddedCues(listOf(TextCue(1_000_000L, 2_000_000L, "early")))

        assertEquals(listOf("early", "late"), engine.cues.value.map { it.text })
    }

    @Test
    fun resetClearsAccumulatedCues() {
        val engine = engine()
        engine.addEmbeddedCues(listOf(TextCue(1_000_000L, 2_000_000L, "first")))
        engine.reset()

        assertEquals(emptyList(), engine.cues.value)
    }
}
