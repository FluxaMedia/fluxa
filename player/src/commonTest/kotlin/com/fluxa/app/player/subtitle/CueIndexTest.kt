package com.fluxa.app.player.subtitle

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class CueIndexTest {
    private val index = CueIndex(
        listOf(
            TextCue(1_000_000L, 2_000_000L, "first"),
            TextCue(3_000_000L, 4_000_000L, "second"),
            TextCue(4_500_000L, 6_000_000L, "third")
        )
    )

    @Test
    fun noCueActiveInGapBetweenEvents() {
        assertTrue(index.activeAt(2_500_000L).isEmpty())
    }

    @Test
    fun cueActiveExactlyAtStart() {
        assertEquals(listOf("first"), index.activeAt(1_000_000L).map { it.text })
    }

    @Test
    fun cueInactiveExactlyAtEnd() {
        assertTrue(index.activeAt(2_000_000L).isEmpty())
    }

    @Test
    fun seekIntoMiddleOfCueFindsItImmediately() {
        assertEquals(listOf("third"), index.activeAt(5_000_000L).map { it.text })
    }

    @Test
    fun nextBoundaryDuringGapIsUpcomingCueStart() {
        assertEquals(3_000_000L, index.nextBoundaryUs(2_500_000L))
    }

    @Test
    fun nextBoundaryWhileCueActiveIsItsEnd() {
        assertEquals(2_000_000L, index.nextBoundaryUs(1_500_000L))
    }

    @Test
    fun nextBoundaryAfterLastCueIsNull() {
        assertEquals(null, index.nextBoundaryUs(7_000_000L))
    }

    @Test
    fun overlappingCuesAreBothActive() {
        val overlapping = CueIndex(
            listOf(
                TextCue(0L, 5_000_000L, "bottom"),
                TextCue(1_000_000L, 3_000_000L, "top")
            )
        )
        assertEquals(setOf("bottom", "top"), overlapping.activeAt(2_000_000L).map { it.text }.toSet())
    }
}
