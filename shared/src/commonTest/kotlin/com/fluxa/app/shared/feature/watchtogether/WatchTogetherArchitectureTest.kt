package com.fluxa.app.shared.feature.watchtogether

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

class WatchTogetherArchitectureTest {
    @Test
    fun addressNormalizationPreservesQueryAndAddsSecret() {
        assertEquals(
            "wss://watch.example.com/ws?region=eu&token=a%20b",
            WatchTogetherAddress.websocketUrl("https://watch.example.com?region=eu", "a b"),
        )
        assertEquals(
            "ws://127.0.0.1:8787/ws",
            WatchTogetherAddress.websocketUrl("http://127.0.0.1:8787/", ""),
        )
        assertNull(WatchTogetherAddress.websocketUrl("watch.example.com", ""))
    }

    @Test
    fun roomCodeNormalizationIsSharedAndStrict() {
        assertEquals("AB12CD", WatchTogetherAddress.normalizeRoomCode(" ab-12 cd "))
        assertTrue(WatchTogetherAddress.isValidRoomCode("AB12CD"))
    }

    @Test
    fun driftPolicyUsesSeekOnlyForLargeDrift() {
        assertIs<WatchTogetherCorrection.None>(
            WatchTogetherDriftCorrector.correction(1_000, 1_100, true, false)
        )
        assertEquals(
            WatchTogetherCorrection.Speed(1.03f),
            WatchTogetherDriftCorrector.correction(1_000, 1_600, true, false),
        )
        assertEquals(
            WatchTogetherCorrection.Seek(2_500),
            WatchTogetherDriftCorrector.correction(1_000, 2_500, true, true),
        )
        assertEquals(
            WatchTogetherCorrection.ResetSpeed,
            WatchTogetherDriftCorrector.correction(1_000, 1_100, true, true),
        )
    }

    @Test
    fun protocolRoundTripsCommonMessages() {
        val content = WatchTogetherContent("tt123", "series", "tt123:1:2", "Episode 2")
        val state = WatchTogetherPlaybackSnapshot(12_000, 30_000, true, false)
        val parsed = WatchTogetherProtocol.parse(WatchTogetherProtocol.playbackState(state, content).toString())!!

        assertEquals(WatchTogetherProtocol.STATE, WatchTogetherProtocol.messageType(parsed))
        assertEquals(12_000, WatchTogetherProtocol.positionMs(parsed))
        assertEquals(content, WatchTogetherProtocol.contentFrom(parsed))
    }
}
