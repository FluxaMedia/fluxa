package com.fluxa.app.shared.feature.player

import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class AudioPassthroughPolicyTest {
    @Test
    fun recognizesMpvEncodedCandidates() {
        assertTrue(AudioPassthroughPolicy.isMpvCandidate("audio/true-hd"))
        assertTrue(AudioPassthroughPolicy.isMpvCandidate("audio/eac3"))
        assertTrue(AudioPassthroughPolicy.isMpvCandidate("audio/vnd.dts.hd"))
        assertTrue(AudioPassthroughPolicy.isMpvCandidate("audio/ac4"))
    }

    @Test
    fun keepsUnsupportedOrAmbiguousFormatsOnDecodedPath() {
        assertFalse(AudioPassthroughPolicy.isMpvCandidate("audio/vnd.dolby.mat"))
        assertFalse(AudioPassthroughPolicy.isMpvCandidate("audio/mpegh-mha1"))
        assertFalse(AudioPassthroughPolicy.isMpvCandidate("audio/dts-uhd-p2"))
        assertFalse(AudioPassthroughPolicy.isMpvCandidate("audio/flac"))
        assertFalse(AudioPassthroughPolicy.isMpvCandidate(null))
    }
}
