package com.fluxa.app.player

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class DolbyVisionFallbackModeTest {

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

    private val HDR_BASE = "hvc1.2.4.L153.B0"

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

    @Test
    fun p7NativeDecoderKeepsNative() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.07.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, caps(nativeP7 = true))
        assertEquals(manifest, result.manifest)
        assertFalse(result.hasP81Conversion)
    }

    @Test
    fun p7P8DecoderConvertsRegardlessOfDisplay() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.07.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, caps(p8Decoder = true))
        assertEquals("""#EXT-X-STREAM-INF:CODECS="dvhe.08.06,ec-3"""", result.manifest)
        assertTrue(result.hasP81Conversion)
    }

    @Test
    fun p7NoDecoderStripsToHdr() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.07.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
        assertFalse(result.hasP81Conversion)
    }

    @Test
    fun p4SafetyGateWhenNoDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.04.06,ac-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto)
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun p4KeptWhenDecoderPresent() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.04.06,ac-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, caps(anyDv = true))
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun p5Cid1StripsToHdrWhenNoDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.05.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
        assertTrue(result.requiresIptPqc2ToneMap)
    }

    @Test
    fun p5Cid1KeptWhenDecoderPresent() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.05.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, caps(anyDv = true))
        assertEquals(manifest, result.manifest)
        assertFalse(result.requiresIptPqc2ToneMap)
    }

    @Test
    fun p5Cid0StripsWithoutIptPqc2WhenNoDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.05.01,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
        assertFalse(result.requiresIptPqc2ToneMap)
    }

    @Test
    fun p8KeptWhenDecoderPresent() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.08.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto, caps(p8Decoder = true))
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun p8StripsToHdrWhenNoDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.08.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
    }

    @Test
    fun p10Cid1StripsToHdrWhenNoDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.10.01,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto)
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
    }

    @Test
    fun p10Cid0SafetyGateWhenNoDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.10.00,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto)
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun p10Cid2SafetyGateWhenNoDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.10.02,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto)
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun p10Cid3SafetyGateWhenNoDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.10.03,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto)
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun multiTrackManifestRewritesAllDvProfiles() {
        val manifest = """
            #EXT-X-STREAM-INF:BANDWIDTH=20000000,CODECS="dvhe.07.06,ec-3"
            high.m3u8
            #EXT-X-STREAM-INF:BANDWIDTH=10000000,CODECS="dvhe.05.06,ec-3"
            mid.m3u8
            #EXT-X-STREAM-INF:BANDWIDTH=5000000,CODECS="dvhe.08.06,ac-3"
            low.m3u8
        """.trimIndent()
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto)
        assertFalse(result.manifest.contains("dvhe.07.06"))
        assertFalse(result.manifest.contains("dvhe.05.06"))
        assertFalse(result.manifest.contains("dvhe.08.06"))
        assertTrue(result.manifest.contains(HDR_BASE))
        assertTrue(result.requiresIptPqc2ToneMap)
    }

    @Test
    fun forceHdr10P7StripsEvenWithP8Decoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.07.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.ForceHdr10, caps(p8Decoder = true))
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
        assertFalse(result.hasP81Conversion)
    }

    @Test
    fun forceHdr10P7StripsEvenWithNativeDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.07.06,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.ForceHdr10, caps(nativeP7 = true))
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
        assertFalse(result.hasP81Conversion)
    }

    @Test
    fun forceHdr10P8StripsEvenWithDecoder() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.08.01,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.ForceHdr10, caps(p8Decoder = true))
        assertEquals("""#EXT-X-STREAM-INF:CODECS="$HDR_BASE,ec-3"""", result.manifest)
    }

    @Test
    fun forceHdr10P4KeepsWhenNoFallback() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.04.06,ac-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.ForceHdr10)
        assertEquals(manifest, result.manifest)
    }

    @Test
    fun forceHdr10MultiTrackStripsStrippableProfiles() {
        val manifest = """
            #EXT-X-STREAM-INF:BANDWIDTH=20000000,CODECS="dvhe.07.06,ec-3"
            high.m3u8
            #EXT-X-STREAM-INF:BANDWIDTH=5000000,CODECS="dvhe.07.06,ac-3"
            low.m3u8
        """.trimIndent()
        val result = rewrite(manifest, DolbyVisionFallbackMode.ForceHdr10, caps(p8Decoder = true))
        assertFalse(result.manifest.contains("dvhe.07.06"))
        assertTrue(result.manifest.contains(HDR_BASE))
        assertFalse(result.hasP81Conversion)
    }

    @Test
    fun p5Cid0SetsIptPqc2Flag() {
        val manifest = """#EXT-X-STREAM-INF:CODECS="dvhe.05.00,ec-3""""
        val result = rewrite(manifest, DolbyVisionFallbackMode.Auto)
        assertTrue(result.requiresIptPqc2ToneMap)
    }
}
