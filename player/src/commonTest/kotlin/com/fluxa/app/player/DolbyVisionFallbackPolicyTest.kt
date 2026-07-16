package com.fluxa.app.player

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class DolbyVisionFallbackPolicyTest {
    private val noDolbyVision = DolbyVisionCapabilities(
        displaySupportsDolbyVision = false,
        displaySupportsHdr10 = true,
        displaySupportsHdr10Plus = false,
        displaySupportsHlg = false,
        decoderNativeP7 = false,
        decoderP5 = false,
        decoderP8 = false,
        decoderAnyDv = false
    )

    @Test
    fun profileSevenFallsBackToHdr10WithoutDolbyVisionDecoder() {
        val result = DolbyVisionFallbackPolicy.rewriteManifest(
            manifest = "codecs=\"dvh1.07.06\"",
            mode = DolbyVisionFallbackMode.Auto,
            capabilities = noDolbyVision
        )

        assertTrue(result.manifest.contains("hvc1.2.4.L153.B0"))
        assertFalse(result.hasP81Conversion)
    }

    @Test
    fun profileSevenConvertsToProfileEightWhenSupported() {
        val result = DolbyVisionFallbackPolicy.rewriteManifest(
            manifest = "codecs=\"dvhe.07.06\"",
            mode = DolbyVisionFallbackMode.Auto,
            capabilities = noDolbyVision.copy(decoderP8 = true)
        )

        assertTrue(result.manifest.contains("dvhe.08.06"))
        assertTrue(result.hasP81Conversion)
    }

    @Test
    fun contentRangeStartIsPortable() {
        assertEquals(131072L, DolbyVisionFallbackPolicy.parseContentRangeStart("bytes 131072-200000/5000000"))
    }
}
