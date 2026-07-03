package com.fluxa.app.player

import org.junit.Assert.*
import org.junit.Test

class DolbyVisionIntegrationTest {

    private fun caps(
        dvDisplay: Boolean = false,
        hdr10Display: Boolean = false,
        nativeP7: Boolean = false,
        p8Decoder: Boolean = false,
        anyDv: Boolean = p8Decoder || nativeP7,
    ) = DolbyVisionCapabilities(
        displaySupportsDolbyVision = dvDisplay,
        displaySupportsHdr10 = hdr10Display,
        displaySupportsHdr10Plus = false,
        displaySupportsHlg = false,
        decoderNativeP7 = nativeP7,
        decoderP5 = anyDv,
        decoderP8 = p8Decoder,
        decoderAnyDv = anyDv,
    )

    private fun rewrite(
        manifest: String,
        mode: DolbyVisionFallbackMode,
        caps: DolbyVisionCapabilities = caps(),
    ) = DolbyVisionFallbackPolicy.rewriteManifest(manifest, mode, caps)

    private val HDR10 = "hvc1.2.4.L153.B0"

    @Test
    fun p5Cid0_noDecoder_setsIptPqc2AndRewritesToHdr10() {
        val r = rewrite("""CODECS="dvhe.05.06"""", DolbyVisionFallbackMode.Auto)
        assertTrue(r.requiresIptPqc2ToneMap)
        assertTrue(r.manifest.contains(HDR10))
    }

    @Test
    fun p5Cid1_noDecoder_doesNotSetIptPqc2() {
        val r = rewrite("""CODECS="dvhe.05.01"""", DolbyVisionFallbackMode.Auto)
        assertFalse(r.requiresIptPqc2ToneMap)
        assertTrue(r.manifest.contains(HDR10))
    }

    @Test
    fun p5Cid0_withDecoder_keeps() {
        val r = rewrite("""CODECS="dvhe.05.06"""", DolbyVisionFallbackMode.Auto, caps(anyDv = true))
        assertFalse(r.requiresIptPqc2ToneMap)
        assertFalse(r.manifest.contains(HDR10))
    }

    @Test
    fun p7_nativeP7Decoder_passthrough() {
        val r = rewrite("""CODECS="dvhe.07.06"""", DolbyVisionFallbackMode.Auto, caps(nativeP7 = true))
        assertTrue(r.manifest.contains("dvhe.07.06"))
        assertFalse(r.hasP81Conversion)
    }

    @Test
    fun p7_p8Decoder_convertsRegardlessOfDisplay() {
        val r = rewrite("""CODECS="dvhe.07.06"""", DolbyVisionFallbackMode.Auto, caps(p8Decoder = true))
        assertTrue(r.manifest.contains("dvhe.08.06"))
        assertTrue(r.hasP81Conversion)
        assertFalse(r.requiresIptPqc2ToneMap)
    }

    @Test
    fun p7_p8Decoder_hdr10OnlyDisplay_stillConverts() {
        val r = rewrite("""CODECS="dvhe.07.06"""", DolbyVisionFallbackMode.Auto, caps(hdr10Display = true, p8Decoder = true))
        assertTrue(r.manifest.contains("dvhe.08.06"))
        assertTrue(r.hasP81Conversion)
    }

    @Test
    fun p7_noDecoder_stripsToHdr10() {
        val r = rewrite("""CODECS="dvhe.07.06"""", DolbyVisionFallbackMode.Auto)
        assertTrue(r.manifest.contains(HDR10))
        assertFalse(r.hasP81Conversion)
    }

    @Test
    fun forceHdr10_p7_p8Decoder_stripsInsteadOfConverts() {
        val r = rewrite("""CODECS="dvhe.07.06"""", DolbyVisionFallbackMode.ForceHdr10, caps(p8Decoder = true))
        assertTrue(r.manifest.contains(HDR10))
        assertFalse(r.hasP81Conversion)
    }

    @Test
    fun forceHdr10_p7_nativeDecoder_stripsToHdr10() {
        val r = rewrite("""CODECS="dvhe.07.06"""", DolbyVisionFallbackMode.ForceHdr10, caps(nativeP7 = true))
        assertTrue(r.manifest.contains(HDR10))
        assertFalse(r.hasP81Conversion)
    }

    @Test
    fun forceHdr10_p8_decoderPresent_stripsToHdr10() {
        val r = rewrite("""CODECS="dvhe.08.01"""", DolbyVisionFallbackMode.ForceHdr10, caps(p8Decoder = true))
        assertTrue(r.manifest.contains(HDR10))
        assertFalse(r.hasP81Conversion)
    }

    @Test
    fun p4_neverRewritten_inAnyMode() {
        for (mode in listOf(DolbyVisionFallbackMode.Auto, DolbyVisionFallbackMode.ForceHdr10)) {
            val r = rewrite("""CODECS="dvhe.04.06"""", mode)
            assertFalse("P4 must not be stripped (mode=$mode)", r.manifest.contains(HDR10))
            assertTrue(r.manifest.contains("dvhe.04.06"))
        }
    }

    @Test
    fun offMode_neverTouchesAnything() {
        val manifests = listOf(
            """CODECS="dvhe.07.06"""",
            """CODECS="dvhe.05.06"""",
            """CODECS="dvhe.08.01""""
        )
        for (m in manifests) {
            val r = rewrite(m, DolbyVisionFallbackMode.Off)
            assertEquals(m, r.manifest)
            assertFalse(r.requiresIptPqc2ToneMap)
            assertFalse(r.hasP81Conversion)
        }
    }

    @Test
    fun p10Cid1_noDecoder_rewritesToHdr10() {
        val r = rewrite("""CODECS="dvhe.10.01"""", DolbyVisionFallbackMode.Auto)
        assertTrue(r.manifest.contains(HDR10))
    }

    @Test
    fun p10Cid0_noDecoder_safetyGate() {
        val r = rewrite("""CODECS="dvhe.10.00"""", DolbyVisionFallbackMode.Auto)
        assertFalse(r.manifest.contains(HDR10))
    }

    @Test
    fun multiVariant_p5CidMix_tagsIptPqc2() {
        val manifest = """
            #EXT-X-STREAM-INF:CODECS="dvhe.07.06"
            high.m3u8
            #EXT-X-STREAM-INF:CODECS="dvhe.05.06"
            p5.m3u8
        """.trimIndent()
        assertTrue(rewrite(manifest, DolbyVisionFallbackMode.Auto).requiresIptPqc2ToneMap)
    }

    @Test
    fun multiVariant_noP5_noIptPqc2Flag() {
        val manifest = """
            #EXT-X-STREAM-INF:CODECS="dvhe.07.06"
            high.m3u8
            #EXT-X-STREAM-INF:CODECS="dvhe.08.01"
            low.m3u8
        """.trimIndent()
        assertFalse(rewrite(manifest, DolbyVisionFallbackMode.Auto).requiresIptPqc2ToneMap)
    }

    @Test
    fun containerInfoNotHasHdrFallback_correctForAllProfiles() {
        assertTrue(DvContainerInfo(4, 0).notHasHdrFallback)
        assertTrue(DvContainerInfo(5, 0).notHasHdrFallback)
        assertTrue(DvContainerInfo(5, 6).notHasHdrFallback)
        assertFalse(DvContainerInfo(5, 1).notHasHdrFallback)
        assertFalse(DvContainerInfo(7, 6).notHasHdrFallback)
        assertFalse(DvContainerInfo(8, 1).notHasHdrFallback)
        assertTrue(DvContainerInfo(10, 0).notHasHdrFallback)
        assertTrue(DvContainerInfo(10, 2).notHasHdrFallback)
        assertFalse(DvContainerInfo(10, 1).notHasHdrFallback)
    }

    @Test
    fun hasP81Conversion_falseWhenNoDvCodecInManifest() {
        val r = rewrite("no dolby vision here", DolbyVisionFallbackMode.ForceHdr10, caps(p8Decoder = true))
        assertFalse(r.hasP81Conversion)
    }
}
