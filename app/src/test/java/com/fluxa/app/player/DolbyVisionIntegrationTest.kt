package com.fluxa.app.player

import org.junit.Assert.*
import org.junit.Test

/**
 * End-to-end manifest rewrite scenarios matching the DV fallback decision matrix.
 * Tests cover profile × device capability × mode combinations that are most likely
 * to regress silently.
 */
class DolbyVisionIntegrationTest {

    private fun rewrite(
        manifest: String,
        mode: DolbyVisionFallbackMode,
        dvDecoder: Boolean = false,
        dvDisplay: Boolean = false
    ) = DolbyVisionFallbackPolicy.rewriteManifest(
        manifest,
        mode,
        DolbyVisionCapabilities(
            displaySupportsDolbyVision = dvDisplay,
            mediaCodecSupportsDolbyVision = dvDecoder
        )
    )

    private val HDR10 = "hvc1.2.4.L153.B0"

    // ── P5 IPTPQc2 shader flag ────────────────────────────────────────────────

    @Test
    fun p5Cid0_noDecoder_setsIptPqc2FlagAndRewritesToHdr10() {
        val r = rewrite("""CODECS="dvhe.05.06"""", DolbyVisionFallbackMode.Auto, dvDecoder = false)
        assertTrue("requiresIptPqc2ToneMap must be true", r.requiresIptPqc2ToneMap)
        assertTrue("codec must be rewritten to HDR10 base", r.manifest.contains(HDR10))
        assertFalse("original DV codec must be gone", r.manifest.contains("dvhe.05"))
    }

    @Test
    fun p5Cid1_noDecoder_doesNotSetIptPqc2Flag() {
        // P5 CID=1 has HDR10 base layer — same path as P7/P8 HDR10 fallback, no shader
        val r = rewrite("""CODECS="dvhe.05.01"""", DolbyVisionFallbackMode.Auto, dvDecoder = false)
        assertFalse("P5 CID=1 must not require IPTPQc2 shader", r.requiresIptPqc2ToneMap)
        assertTrue("must be rewritten to HDR10 base", r.manifest.contains(HDR10))
    }

    @Test
    fun p5Cid0_withDecoder_noDisplay_doesNotSetIptPqc2AndKeepsNative() {
        // DV decoder + noHdrFallback (P5 CID≠1) → device_supports_dv=true → keep native DV
        val r = rewrite("""CODECS="dvhe.05.06"""", DolbyVisionFallbackMode.Auto, dvDecoder = true, dvDisplay = false)
        assertFalse("native DV path must not set IPTPQc2 flag", r.requiresIptPqc2ToneMap)
        assertFalse("codec must be kept as DV", r.manifest.contains(HDR10))
    }

    // ── ConvertToDv81 codec string rewrite ───────────────────────────────────

    @Test
    fun convertDv81_p7_decoderNoDisplay_rewritesCodecAndNoIptPqc2() {
        val r = rewrite("""CODECS="dvhe.07.06"""", DolbyVisionFallbackMode.ConvertToDv81, dvDecoder = true, dvDisplay = false)
        assertTrue("P7 must be rewritten to P8.1", r.manifest.contains("dvhe.08.06"))
        assertFalse("P7→P8.1 path must not set IPTPQc2 flag", r.requiresIptPqc2ToneMap)
    }

    @Test
    fun convertDv81_p7_decoderAndDisplay_nativePassthrough() {
        val r = rewrite("""CODECS="dvhe.07.06"""", DolbyVisionFallbackMode.ConvertToDv81, dvDecoder = true, dvDisplay = true)
        assertTrue("native DV must not be rewritten", r.manifest.contains("dvhe.07.06"))
        assertFalse(r.requiresIptPqc2ToneMap)
    }

    @Test
    fun convertDv81_p8_decoderNoDisplay_keepsNativeDv() {
        // P8.1 with DV decoder but no display: decoder handles it natively in ConvertToDv81 mode
        val r = rewrite("""CODECS="dvhe.08.01"""", DolbyVisionFallbackMode.ConvertToDv81, dvDecoder = true, dvDisplay = false)
        assertFalse("P8.1 must stay as DV in ConvertToDv81 with decoder", r.manifest.contains(HDR10))
    }

    // ── P4 safety gate ────────────────────────────────────────────────────────

    @Test
    fun p4_allModes_neverRewritten() {
        // P4 has no HDR base layer — stripping would expose DV-only bitstream to wrong decoder
        for (mode in listOf(DolbyVisionFallbackMode.Auto, DolbyVisionFallbackMode.ConvertToDv81)) {
            val r = rewrite("""CODECS="dvhe.04.06"""", mode, dvDecoder = false, dvDisplay = false)
            assertFalse("P4 must never be rewritten (mode=$mode)", r.manifest.contains(HDR10))
            assertTrue("P4 codec must remain unchanged", r.manifest.contains("dvhe.04.06"))
        }
    }

    // ── Off mode ──────────────────────────────────────────────────────────────

    @Test
    fun offMode_neverTouchesAnything() {
        val manifests = listOf(
            """CODECS="dvhe.07.06"""",
            """CODECS="dvhe.05.06"""",
            """CODECS="dvhe.08.01""""
        )
        for (m in manifests) {
            val r = rewrite(m, DolbyVisionFallbackMode.Off, dvDecoder = false, dvDisplay = false)
            assertEquals("Off mode must not change manifest", m, r.manifest)
            assertFalse(r.requiresIptPqc2ToneMap)
        }
    }

    // ── P10 HDR10 compat ──────────────────────────────────────────────────────

    @Test
    fun p10Cid1_noDecoder_rewritesToHdr10() {
        val r = rewrite("""CODECS="dvhe.10.01"""", DolbyVisionFallbackMode.Auto, dvDecoder = false)
        assertTrue("P10 CID=1 must fall back to HDR10", r.manifest.contains(HDR10))
        assertFalse(r.requiresIptPqc2ToneMap)
    }

    @Test
    fun p10Cid0_noDecoder_neverRewritten() {
        // P10 CID=0: no HDR base layer → safety gate, cannot strip safely
        val r = rewrite("""CODECS="dvhe.10.00"""", DolbyVisionFallbackMode.Auto, dvDecoder = false)
        assertFalse("P10 CID=0 must not be rewritten", r.manifest.contains(HDR10))
    }

    // ── Multi-variant manifest ────────────────────────────────────────────────

    @Test
    fun multiVariant_p5CidMix_correctlyTagsIptPqc2() {
        // P5 CID=0 present in manifest → requiresIptPqc2ToneMap must be set
        val manifest = """
            #EXT-X-STREAM-INF:CODECS="dvhe.07.06"
            high.m3u8
            #EXT-X-STREAM-INF:CODECS="dvhe.05.06"
            p5.m3u8
        """.trimIndent()
        val r = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = false)
        assertTrue("P5 CID≠1 in manifest must set requiresIptPqc2ToneMap", r.requiresIptPqc2ToneMap)
    }

    @Test
    fun multiVariant_noP5_doesNotSetIptPqc2Flag() {
        val manifest = """
            #EXT-X-STREAM-INF:CODECS="dvhe.07.06"
            high.m3u8
            #EXT-X-STREAM-INF:CODECS="dvhe.08.01"
            low.m3u8
        """.trimIndent()
        val r = rewrite(manifest, DolbyVisionFallbackMode.Auto, dvDecoder = false)
        assertFalse("No P5 CID≠1 → must not set requiresIptPqc2ToneMap", r.requiresIptPqc2ToneMap)
    }

    // ── DvContainerInfo.notHasHdrFallback ────────────────────────────────────

    @Test
    fun containerInfoNotHasHdrFallback_correctForAllProfiles() {
        assertTrue(DvContainerInfo(4, 0).notHasHdrFallback)   // P4 always no fallback
        assertTrue(DvContainerInfo(5, 0).notHasHdrFallback)   // P5 CID=0 no fallback
        assertTrue(DvContainerInfo(5, 6).notHasHdrFallback)   // P5 CID≠1 no fallback
        assertFalse(DvContainerInfo(5, 1).notHasHdrFallback)  // P5 CID=1 HAS HDR10 base
        assertFalse(DvContainerInfo(7, 6).notHasHdrFallback)  // P7 has HDR10 base
        assertFalse(DvContainerInfo(8, 1).notHasHdrFallback)  // P8.1 has HDR10 base
        assertTrue(DvContainerInfo(10, 0).notHasHdrFallback)  // P10 CID=0 no fallback
        assertTrue(DvContainerInfo(10, 2).notHasHdrFallback)  // P10 CID=2 no fallback
        assertFalse(DvContainerInfo(10, 1).notHasHdrFallback) // P10 CID=1 has HDR10 base
    }
}
