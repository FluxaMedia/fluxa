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
    fun profileTenWithUnknownCompatIsTreatedAsSafetyNotStripped() {
        // "09" here is the codec-string *level*, not dv_bl_signal_compatibility_id
        // — that only exists in the dvcC/dvvC box. The old code read it as
        // compat=9 (outside {0,2,3}) and silently rewrote the codec string to
        // plain HEVC, assuming an HDR10 base layer it never confirmed exists.
        // Unknown compat must instead leave the stream as Dolby Vision.
        val result = DolbyVisionFallbackPolicy.rewriteManifest(
            manifest = "codecs=\"dvhe.10.09\"",
            mode = DolbyVisionFallbackMode.Auto,
            capabilities = noDolbyVision
        )

        assertTrue(result.decision.contains("safety_gate"))
        assertTrue(result.manifest.contains("dvhe.10.09"))
        assertFalse(result.manifest.contains("hvc1.2.4.L153.B0"))
    }

    @Test
    fun contentRangeStartIsPortable() {
        assertEquals(131072L, DolbyVisionFallbackPolicy.parseContentRangeStart("bytes 131072-200000/5000000"))
    }
}
