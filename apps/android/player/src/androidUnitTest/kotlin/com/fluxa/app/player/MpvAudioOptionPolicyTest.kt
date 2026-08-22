package com.fluxa.app.player

import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class MpvAudioOptionPolicyTest {
    @Test
    fun routeOwnedAudioOptionsCannotBeOverriddenByCustomConfig() {
        assertTrue(isRouteOwnedAudioOption("audio-spdif"))
        assertTrue(isRouteOwnedAudioOption(" AF "))
        assertTrue(isRouteOwnedAudioOption("audio-channels"))
        assertFalse(isRouteOwnedAudioOption("audio-delay"))
        assertFalse(isRouteOwnedAudioOption("video-sync"))
    }

    @Test
    fun recognizesDifferentAndroidAudioSinkFailureMessages() {
        assertTrue(
            MpvAudioErrorPolicy.isPassthroughFailure(
                "failed to initialize audio driver 'audiotrack'",
                passthroughConfigured = true,
                fallbackUsed = false,
            )
        )
        assertTrue(
            MpvAudioErrorPolicy.isPassthroughFailure(
                "AO: [audiotrack] spdif format not supported",
                passthroughConfigured = true,
                fallbackUsed = false,
            )
        )
        assertFalse(
            MpvAudioErrorPolicy.isPassthroughFailure(
                "audio output is ready",
                passthroughConfigured = true,
                fallbackUsed = false,
            )
        )
        assertFalse(
            MpvAudioErrorPolicy.isPassthroughFailure(
                "failed to initialize audio driver 'audiotrack'",
                passthroughConfigured = true,
                fallbackUsed = true,
            )
        )
    }
}
