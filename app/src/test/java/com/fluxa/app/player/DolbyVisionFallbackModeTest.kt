package com.fluxa.app.player

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class DolbyVisionFallbackModeTest {

    private fun rewrite(manifest: String, mode: DolbyVisionFallbackMode, dvDecoder: Boolean = false, dvDisplay: Boolean = false) =
        DolbyVisionFallbackPolicy.rewriteManifest(
            manifest = manifest,
            mode = mode,
            capabilities = DolbyVisionCapabilities(
                displaySupportsDolbyVision = dvDisplay,
                mediaCodecSupportsDolbyVision = dvDecoder
            )
        )

    private val HDR_BASE = "hvc1.2.4.L153.B0"

    // ── Off mode ──────────────────────────────────────────────────────────────

    @Test
    fun offModeLeavesProfile7CodecUntouched() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.07.06,ec-3""""
        assertEquals(manifest, rewrite(manifest, DolbyVisionFallbackMode.Off).manifest)
    }

    @Test
    fun offModeNeverRewritesAnyProfile() {
        val manifest = """
            #EXT-X-STREAM-INF:CODECS="dvhe.07.06,ec-3"
            #EXT-X-STREAM-INF:CODECS="dvhe.08.06,ec-3"
            #EXT-X-STREAM-INF:CODECS="dvhe.04.06,ac-3"
        """.trimIndent()
        assertEquals(manifest, rewrite(manifest, DolbyVisionFallbackMode.Off).manifest)
    }

    // ── Auto mode — Profile 7 (HDR10-compatible base) ─────────────────────────

    @Test
    fun autoModeKeepsProfile7WhenDisplayAndCodecSupportDolbyVision() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.07.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = true, dvDisplay = true)
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun autoModeRewritesProfile7ToHdrWhenDisplayDoesNotSupportDolbyVision() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.07.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = true, dvDisplay = false)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
    }

    // ── Auto mode — Profile 4 (DV-only, no HDR base) ─────────────────────────

    @Test
    fun autoModeDoesNotRewriteProfile4WhenNoDecoderOrDisplay() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.04.06,ac-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = false, dvDisplay = false)
        // P4 has no HDR base; the only safe action is to leave it as-is.
        assertEquals(manifest, result.manifest)
    }

    // ── Auto mode — Profile 5 (no HDR fallback layer, IPTPQc2 base) ─────────────

    @Test
    fun autoModeRewritesProfile5ToHdrBaseWhenNoDecoder() {
        // P5 CID!=1 rewrites to HDR_BASE (same as P7/P8 fallback).
        // The IPTPQc2 shader is triggered via requiresIptPqc2ToneMap flag in the rewrite result,
        // not via a custom codec string marker.
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.05.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = false, dvDisplay = false)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
        assertTrue("requiresIptPqc2ToneMap must be set for P5 CID≠1", result.requiresIptPqc2ToneMap)
    }

    @Test
    fun autoModeKeepsProfile5Cid6WhenDecoderPresentNoDisplay() {
        // P5 CID=6: notHasHdrFallback=true, so decoder alone satisfies device_supports_dv.
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.05.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = true, dvDisplay = false)
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun autoModeRewritesProfile5Cid1ToHdrBaseWhenDecoderButNoDisplay() {
        // P5 CID=1: notHasHdrFallback=false (has HDR10 base), so DV display is also required.
        // With decoder but no display: same rewrite path as P7/P8 → HDR_BASE.
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.05.01,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = true, dvDisplay = false)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
    }

    @Test
    fun autoModeKeepsProfile5Cid1WhenBothDecoderAndDisplay() {
        // P5 CID=1 with full DV device: device_supports_dv=true → keep DV.
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.05.01,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = true, dvDisplay = true)
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun autoModeRewritesProfile5Cid0ToHdrBase() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.05.00,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = false, dvDisplay = false)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
        assertTrue(result.requiresIptPqc2ToneMap)
    }

    @Test
    fun autoModeRewritesProfile5Cid1ToHdrBase() {
        // P5 CID=1 has an HDR10-compatible base layer — same rewrite path as P8.
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.05.01,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = false, dvDisplay = false)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
    }

    @Test
    fun p5Cid0RewriteSetsIptPqc2Flag() {
        // P5 CID=0 rewrite sets requiresIptPqc2ToneMap — the flag, not the codec string,
        // is the signal for shader activation.
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.05.00,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = false, dvDisplay = false)
        assertTrue("requiresIptPqc2ToneMap must be true for P5 CID=0", result.requiresIptPqc2ToneMap)
    }

    // ── Auto mode — Profile 8 (HDR10-compatible base) ─────────────────────────

    @Test
    fun autoModeKeepsProfile8WhenDecoderPresent() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.08.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = true, dvDisplay = true)
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun autoModeRewritesProfile8ToHdrWhenNoDecoderAndNoDisplay() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.08.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = false, dvDisplay = false)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
    }

    // ── Auto mode — Profile 10 (compatibility ID matters) ────────────────────

    @Test
    fun autoModeRewritesProfile10CompatId1ToHdrWhenNoDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.10.01,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = false, dvDisplay = false)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
    }

    @Test
    fun autoModeDoesNotRewriteProfile10CompatId0WhenNoDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.10.00,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = false, dvDisplay = false)
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun autoModeDoesNotRewriteProfile10CompatId2WhenNoDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.10.02,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = false, dvDisplay = false)
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun autoModeDoesNotRewriteProfile10CompatId3WhenNoDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.10.03,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = false, dvDisplay = false)
        assertEquals(manifest, result.manifest)
    }

    // ── Auto mode — multi-track manifest ──────────────────────────────────────

    @Test
    fun autoModeRewritesAllDvProfilesInMultiTrackManifest() {
        val manifest = """
            #EXT-X-STREAM-INF:BANDWIDTH=20000000,CODECS="dvhe.07.06,ec-3"
            high.m3u8
            #EXT-X-STREAM-INF:BANDWIDTH=10000000,CODECS="dvhe.05.06,ec-3"
            mid.m3u8
            #EXT-X-STREAM-INF:BANDWIDTH=5000000,CODECS="dvhe.08.06,ac-3"
            low.m3u8
        """.trimIndent()

        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = false, dvDisplay = false)

        // P7 and P8 → HDR base
        assertTrue("P7 must be rewritten", !result.manifest.contains("dvhe.07.06"))
        assertTrue("P7 and P8 must use HDR base codec", result.manifest.contains(HDR_BASE))

        // P5 → HDR base (same as P7/P8); shader is triggered via requiresIptPqc2ToneMap flag
        assertTrue("P5 must be rewritten", !result.manifest.contains("dvhe.05.06"))
        assertTrue("requiresIptPqc2ToneMap must be set when P5 is present", result.requiresIptPqc2ToneMap)

        // P8 → HDR base
        assertTrue("P8 must be rewritten", !result.manifest.contains("dvhe.08.06"))
    }

    // ── ConvertToDv81 mode ────────────────────────────────────────────────────

    @Test
    fun convertDv81RewritesP7CodecToP8WhenDecoderPresentNoDisplay() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.07.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.ConvertToDv81, dvDecoder = true, dvDisplay = false)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="dvhe.08.06,ec-3"""", result.manifest)
    }

    @Test
    fun convertDv81KeepsNativeDvWhenBothDecoderAndDisplay() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.07.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.ConvertToDv81, dvDecoder = true, dvDisplay = true)
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun convertDv81FallsBackToHdrBaseWhenNoDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.07.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.ConvertToDv81, dvDecoder = false, dvDisplay = false)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
    }

    @Test
    fun convertDv81KeepsP8NativelyWhenDecoderPresent() {
        // P8.1 decoder can handle it natively even without DV display.
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.08.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.ConvertToDv81, dvDecoder = true, dvDisplay = false)
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun convertDv81PreservesCompatIdOnP7Rewrite() {
        // Compat-id must survive the 07 → 08 rewrite.
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.07.01,ac-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.ConvertToDv81, dvDecoder = true, dvDisplay = false)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="dvhe.08.01,ac-3"""", result.manifest)
    }

    @Test
    fun convertDv81P7RewriteInMultiTrackManifest() {
        val manifest = """
            #EXT-X-STREAM-INF:BANDWIDTH=20000000,CODECS="dvhe.07.06,ec-3"
            high.m3u8
            #EXT-X-STREAM-INF:BANDWIDTH=5000000,CODECS="dvhe.07.06,ac-3"
            low.m3u8
        """.trimIndent()
        val result = rewrite(manifest, DolbyVisionFallbackMode.ConvertToDv81, dvDecoder = true, dvDisplay = false)
        assertTrue("P7 must be rewritten to P8.1", !result.manifest.contains("dvhe.07.06"))
        assertTrue("P8.1 codec string must appear", result.manifest.contains("dvhe.08.06"))
    }
}
