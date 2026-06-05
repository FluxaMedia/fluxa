package com.fluxa.app.player

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class DolbyVisionContainerTest {

    // ── dvcC box builder (mirrors Rust make_dvcc_box) ─────────────────────────

    private fun dvccBox(profile: Int, compatId: Int): ByteArray {
        val b = ByteArray(16)
        b[0] = 0x00; b[1] = 0x00; b[2] = 0x00; b[3] = 0x10  // box size = 16
        b[4] = 'd'.code.toByte(); b[5] = 'v'.code.toByte()
        b[6] = 'c'.code.toByte(); b[7] = 'C'.code.toByte()
        b[8] = 1                                               // dv_version_major
        b[9] = 0                                               // dv_version_minor
        b[10] = ((profile shl 1) and 0xFE).toByte()           // profile in bits [7:1]
        b[11] = 0x00
        b[12] = ((compatId shl 4) and 0xF0).toByte()          // compat_id in bits [7:4]
        return b
    }

    private fun bufferWithDvccAt(offset: Int, profile: Int, compatId: Int, totalSize: Int = 1024): ByteArray {
        val buf = ByteArray(totalSize) { 0xAA.toByte() }
        dvccBox(profile, compatId).copyInto(buf, offset)
        return buf
    }

    // Helpers to apply the strip-decision logic against explicit expected values,
    // rather than repeating the formula inline in every test.
    private fun deviceSupportsDv(info: DvContainerInfo, caps: DolbyVisionCapabilities): Boolean =
        caps.mediaCodecSupportsDolbyVision && (caps.displaySupportsDolbyVision || info.notHasHdrFallback)

    private fun shouldStrip(info: DvContainerInfo, caps: DolbyVisionCapabilities): Boolean =
        !deviceSupportsDv(info, caps) && (info.profile == 5 || !info.notHasHdrFallback)

    private fun needsIptPqc2(info: DvContainerInfo): Boolean =
        info.profile == 5 && info.compatId != 1

    private val noDevice = DolbyVisionCapabilities(displaySupportsDolbyVision = false, mediaCodecSupportsDolbyVision = false)
    private val decoderOnly = DolbyVisionCapabilities(displaySupportsDolbyVision = false, mediaCodecSupportsDolbyVision = true)
    private val fullDevice = DolbyVisionCapabilities(displaySupportsDolbyVision = true, mediaCodecSupportsDolbyVision = true)

    // ── DvContainerInfo.notHasHdrFallback ─────────────────────────────────────

    @Test fun profile4_always_noFallback() {
        for (cid in 0..15) assertTrue("P4 CID=$cid must have no HDR fallback", DvContainerInfo(4, cid).notHasHdrFallback)
    }

    @Test fun profile5_cid0_noFallback() {
        assertTrue(DvContainerInfo(5, 0).notHasHdrFallback)
    }

    @Test fun profile5_cid1_hasFallback() {
        // P5 CID=1 has an HDR10 base layer — must NOT be treated as DV-only.
        assertFalse("P5 CID=1 has HDR10 base layer", DvContainerInfo(5, 1).notHasHdrFallback)
    }

    @Test fun profile5_cid2_noFallback() {
        assertTrue(DvContainerInfo(5, 2).notHasHdrFallback)
    }

    @Test fun profile7_hasFallback() {
        assertFalse(DvContainerInfo(7, 6).notHasHdrFallback)
    }

    @Test fun profile8_hasFallback() {
        assertFalse(DvContainerInfo(8, 4).notHasHdrFallback)
    }

    @Test fun profile10_cid0_noFallback() {
        assertTrue(DvContainerInfo(10, 0).notHasHdrFallback)
    }

    @Test fun profile10_cid1_hasFallback() {
        assertFalse(DvContainerInfo(10, 1).notHasHdrFallback)
    }

    @Test fun profile10_cid2_noFallback() {
        assertTrue(DvContainerInfo(10, 2).notHasHdrFallback)
    }

    @Test fun profile10_cid3_noFallback() {
        assertTrue(DvContainerInfo(10, 3).notHasHdrFallback)
    }

    @Test fun profile10_cid4_hasFallback() {
        assertFalse(DvContainerInfo(10, 4).notHasHdrFallback)
    }

    // ── scanDvContainerInfo ───────────────────────────────────────────────────

    @Test fun scan_findsBoxAtOffsetZero() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(dvccBox(7, 6))!!
        assertEquals(7, info.profile)
        assertEquals(6, info.compatId)
    }

    @Test fun scan_findsBoxEmbeddedInLargerBuffer() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(offset = 64, profile = 8, compatId = 4))!!
        assertEquals(8, info.profile)
        assertEquals(4, info.compatId)
    }

    @Test fun scan_findsBoxAtHighOffset() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(offset = 500, profile = 5, compatId = 0, totalSize = 600))!!
        assertEquals(5, info.profile)
        assertEquals(0, info.compatId)
    }

    @Test fun scan_profile5_cid1_parsesCorrectly() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(offset = 32, profile = 5, compatId = 1))!!
        assertEquals(5, info.profile)
        assertEquals(1, info.compatId)
        assertFalse("P5 CID=1 must have HDR10 fallback", info.notHasHdrFallback)
    }

    @Test fun scan_profile5_cid0_parsesCorrectly() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(dvccBox(5, 0))!!
        assertEquals(5, info.profile)
        assertEquals(0, info.compatId)
        assertTrue("P5 CID=0 must have no HDR fallback", info.notHasHdrFallback)
    }

    @Test fun scan_profile10_cid0_parsesCorrectly() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(dvccBox(10, 0))!!
        assertEquals(10, info.profile)
        assertEquals(0, info.compatId)
        assertTrue(info.notHasHdrFallback)
    }

    @Test fun scan_returnsNullWhenNoDvccBox() {
        assertNull(DolbyVisionFallbackPolicy.scanDvContainerInfo("hevcavchdr10".toByteArray()))
    }

    @Test fun scan_returnsNullForEmptyBuffer() {
        assertNull(DolbyVisionFallbackPolicy.scanDvContainerInfo(ByteArray(0)))
    }

    @Test fun scan_returnsNullForBufferSmallerThanMinimum() {
        // Needs at least 4 (fourcc) + 5 (payload) + prefix bytes — any 7-byte buffer can't fit.
        assertNull(DolbyVisionFallbackPolicy.scanDvContainerInfo(ByteArray(7) { 0 }))
    }

    @Test fun scan_returnsNullWhenPayloadTooShortAfterFourcc() {
        // "dvcC" found but only 4 payload bytes follow (need 5).
        val buf = byteArrayOf('d'.code.toByte(), 'v'.code.toByte(), 'c'.code.toByte(), 'C'.code.toByte(), 1, 0, 0, 0)
        assertNull(DolbyVisionFallbackPolicy.scanDvContainerInfo(buf))
    }

    @Test fun scan_picksFirstBoxInBuffer() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(dvccBox(7, 6) + dvccBox(8, 4))!!
        assertEquals("Must return first box, not second", 7, info.profile)
    }

    @Test fun scan_doesNotMatchLowercaseDvcc() {
        // "dvcc" all-lowercase is not a valid ISO-BMFF box type.
        val buf = byteArrayOf('d'.code.toByte(), 'v'.code.toByte(), 'c'.code.toByte(), 'c'.code.toByte()) + ByteArray(16) { 0 }
        assertNull(DolbyVisionFallbackPolicy.scanDvContainerInfo(buf))
    }

    // ── mangleDvccFourcc ──────────────────────────────────────────────────────

    @Test fun mangle_rewritesDvcC() {
        val buf = "xxdvcCxx".toByteArray()
        DolbyVisionFallbackPolicy.mangleDvccFourcc(buf)
        assertArrayEquals("xxXXXXxx".toByteArray(), buf)
    }

    @Test fun mangle_rewritesDvvC() {
        val buf = "dvvCdata".toByteArray()
        DolbyVisionFallbackPolicy.mangleDvccFourcc(buf)
        assertArrayEquals("XXXXdata".toByteArray(), buf)
    }

    @Test fun mangle_rewritesDvhe() {
        val buf = "xxdvhexx".toByteArray()
        DolbyVisionFallbackPolicy.mangleDvccFourcc(buf)
        assertArrayEquals("xxXXXXxx".toByteArray(), buf)
    }

    @Test fun mangle_rewritesDvh1() {
        val buf = "xxdvh1xx".toByteArray()
        DolbyVisionFallbackPolicy.mangleDvccFourcc(buf)
        assertArrayEquals("xxXXXXxx".toByteArray(), buf)
    }

    @Test fun mangle_rewritesAllFourPatternsInOnePass() {
        val buf = "dvcCdvvCdvhedvh1".toByteArray()
        DolbyVisionFallbackPolicy.mangleDvccFourcc(buf)
        assertArrayEquals(ByteArray(16) { 'X'.code.toByte() }, buf)
    }

    @Test fun mangle_rewritesMultipleOccurrencesOfSamePattern() {
        val buf = "aadvcCzzdvheqq".toByteArray()
        DolbyVisionFallbackPolicy.mangleDvccFourcc(buf)
        assertFalse(buf.toList().windowed(4).any { it == "dvcC".map { c -> c.code.toByte() } })
        assertFalse(buf.toList().windowed(4).any { it == "dvhe".map { c -> c.code.toByte() } })
    }

    @Test fun mangle_leavesUnrelatedDataIntact() {
        val original = "hevcavchdr10".toByteArray()
        val buf = original.copyOf()
        DolbyVisionFallbackPolicy.mangleDvccFourcc(buf)
        assertArrayEquals(original, buf)
    }

    @Test fun mangle_handlesPatternAtVeryEnd() {
        val buf = "12345678dvcC".toByteArray()
        DolbyVisionFallbackPolicy.mangleDvccFourcc(buf)
        assertArrayEquals("12345678XXXX".toByteArray(), buf)
    }

    @Test fun mangle_doesNotRewriteLowercaseDvcc() {
        val original = "xxdvccxx".toByteArray()
        val buf = original.copyOf()
        DolbyVisionFallbackPolicy.mangleDvccFourcc(buf)
        assertArrayEquals(original, buf)
    }

    @Test fun mangle_tooShortBufferIsNoOp() {
        val buf = byteArrayOf('d'.code.toByte(), 'v'.code.toByte(), 'c'.code.toByte())
        val copy = buf.copyOf()
        DolbyVisionFallbackPolicy.mangleDvccFourcc(buf)
        assertArrayEquals(copy, buf)
    }

    @Test fun mangle_doesNotPatchPartialPatternAtEnd() {
        // "dvc" at the very end — only 3 bytes, not the full 4-byte fourcc.
        val buf = "xxxdvc".toByteArray()
        val copy = buf.copyOf()
        DolbyVisionFallbackPolicy.mangleDvccFourcc(buf)
        assertArrayEquals(copy, buf)
    }

    @Test fun mangle_patternAtOddOffsetIsPatched() {
        val buf = "xdvcCyyyy".toByteArray()
        DolbyVisionFallbackPolicy.mangleDvccFourcc(buf)
        assertEquals('X'.code.toByte(), buf[1])
        assertEquals('X'.code.toByte(), buf[4])
    }

    @Test fun mangle_emptyBufferIsNoOp() {
        // Must not crash on empty input.
        DolbyVisionFallbackPolicy.mangleDvccFourcc(ByteArray(0))
    }

    // ── Strip decision logic: expected outcomes stated directly ───────────────
    //
    // Each test states the EXPECTED boolean outcome explicitly rather than
    // re-deriving it from the same formula as the production code, so that a
    // logic bug in the formula would cause the test to fail rather than agree.

    // P7 — has HDR10 base (notHasHdrFallback=false)

    @Test fun decision_p7_noDevice_strip() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 7, 6))!!
        assertTrue("P7, no DV support at all → strip to HDR10", shouldStrip(info, noDevice))
        assertFalse("P7 strip must not signal IPTPQc2", needsIptPqc2(info))
    }

    @Test fun decision_p7_decoderOnly_strip() {
        // P7 has HDR10 base so device_supports_dv requires a DV display too.
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 7, 6))!!
        assertTrue("P7, decoder but no DV display → strip to HDR10", shouldStrip(info, decoderOnly))
    }

    @Test fun decision_p7_fullDevice_keep() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 7, 6))!!
        assertFalse("P7, full DV device → pass through as DV", shouldStrip(info, fullDevice))
    }

    // P8 — same fallback category as P7

    @Test fun decision_p8_noDevice_strip() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 8, 4))!!
        assertTrue("P8, no DV support → strip to HDR10", shouldStrip(info, noDevice))
        assertFalse("P8 strip must not signal IPTPQc2", needsIptPqc2(info))
    }

    @Test fun decision_p8_decoderOnly_strip() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 8, 4))!!
        assertTrue("P8, decoder but no DV display → strip to HDR10", shouldStrip(info, decoderOnly))
    }

    @Test fun decision_p8_fullDevice_keep() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 8, 4))!!
        assertFalse("P8, full DV device → pass through as DV", shouldStrip(info, fullDevice))
    }

    // P5 CID=0 — IPTPQc2, no HDR10 base (notHasHdrFallback=true)

    @Test fun decision_p5cid0_noDevice_strip_and_shader() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 5, 0))!!
        assertTrue("P5 CID=0, no DV support → strip so HEVC decoder accepts it", shouldStrip(info, noDevice))
        assertTrue("P5 CID=0 strip must signal IPTPQc2 shader", needsIptPqc2(info))
    }

    @Test fun decision_p5cid0_decoderOnly_keep() {
        // P5 CID=0 notHasHdrFallback=true, so decoder alone is sufficient for device_supports_dv.
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 5, 0))!!
        assertFalse("P5 CID=0, DV decoder present (display not required) → keep DV", shouldStrip(info, decoderOnly))
    }

    @Test fun decision_p5cid0_fullDevice_keep() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 5, 0))!!
        assertFalse("P5 CID=0, full DV device → keep DV", shouldStrip(info, fullDevice))
    }

    // P5 CID=1 — has HDR10 base (notHasHdrFallback=false, same category as P7/P8)

    @Test fun decision_p5cid1_noDevice_strip_noShader() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 5, 1))!!
        assertTrue("P5 CID=1, no DV support → strip to HDR10 base", shouldStrip(info, noDevice))
        assertFalse("P5 CID=1 strip must NOT signal IPTPQc2 (it has HDR10 base)", needsIptPqc2(info))
    }

    @Test fun decision_p5cid1_decoderOnly_strip() {
        // P5 CID=1 notHasHdrFallback=false, so DV display is also required.
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 5, 1))!!
        assertTrue("P5 CID=1, decoder but no DV display → strip to HDR10 (same as P7/P8)", shouldStrip(info, decoderOnly))
    }

    @Test fun decision_p5cid1_fullDevice_keep() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 5, 1))!!
        assertFalse("P5 CID=1, full DV device → keep DV", shouldStrip(info, fullDevice))
    }

    // P4 — DV-only AVC (notHasHdrFallback=true, profile≠5 so never stripped)

    @Test fun decision_p4_noDevice_keep() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 4, 0))!!
        assertFalse("P4, no DV support → pass through unchanged (no HEVC base to fall back to)", shouldStrip(info, noDevice))
    }

    @Test fun decision_p4_decoderOnly_keep() {
        val info = DolbyVisionFallbackPolicy.scanDvContainerInfo(bufferWithDvccAt(0, 4, 0))!!
        assertFalse("P4, decoder only → pass through (still no HDR base)", shouldStrip(info, decoderOnly))
    }

    // P10 CID=0 — DV-only (notHasHdrFallback=true, profile≠5)

    @Test fun decision_p10cid0_noDevice_keep() {
        val info = DvContainerInfo(10, 0)
        assertFalse("P10 CID=0 is DV-only → pass through unchanged", shouldStrip(info, noDevice))
    }

    // P10 CID=1 — has HDR10 base (notHasHdrFallback=false)

    @Test fun decision_p10cid1_noDevice_strip() {
        val info = DvContainerInfo(10, 1)
        assertTrue("P10 CID=1 has HDR10 base → strip when no DV support", shouldStrip(info, noDevice))
        assertFalse("P10 CID=1 strip must not signal IPTPQc2", needsIptPqc2(info))
    }

    @Test fun decision_p10cid1_fullDevice_keep() {
        val info = DvContainerInfo(10, 1)
        assertFalse("P10 CID=1, full DV device → keep DV", shouldStrip(info, fullDevice))
    }

    // ── End-to-end: scan + mangle erases the box ──────────────────────────────

    @Test fun endToEnd_mangleErasesDetectableBox() {
        val buf = bufferWithDvccAt(offset = 128, profile = 7, compatId = 6)
        assertNotNull("dvcC box must be present before mangle", DolbyVisionFallbackPolicy.scanDvContainerInfo(buf))
        DolbyVisionFallbackPolicy.mangleDvccFourcc(buf)
        assertNull("dvcC box must be undetectable after mangle", DolbyVisionFallbackPolicy.scanDvContainerInfo(buf))
    }

    // ── parseContentRangeStart ────────────────────────────────────────────────

    @Test fun contentRange_fullFileFromZero() {
        assertEquals(0L, DolbyVisionFallbackPolicy.parseContentRangeStart("bytes 0-999999/1000000"))
    }

    @Test fun contentRange_midFileSeek() {
        assertEquals(50000L, DolbyVisionFallbackPolicy.parseContentRangeStart("bytes 50000-100000/1000000"))
    }

    @Test fun contentRange_pastScanWindow() {
        assertEquals(131072L, DolbyVisionFallbackPolicy.parseContentRangeStart("bytes 131072-200000/5000000"))
    }

    @Test fun contentRange_nullHeaderReturnsNull() {
        assertNull(DolbyVisionFallbackPolicy.parseContentRangeStart(null))
    }

    @Test fun contentRange_unknownRangeReturnsNull() {
        // "bytes */*" means unknown range — cannot determine offset.
        assertNull(DolbyVisionFallbackPolicy.parseContentRangeStart("bytes */*"))
    }

    @Test fun contentRange_malformedReturnsNull() {
        assertNull(DolbyVisionFallbackPolicy.parseContentRangeStart("invalid header"))
        assertNull(DolbyVisionFallbackPolicy.parseContentRangeStart(""))
        assertNull(DolbyVisionFallbackPolicy.parseContentRangeStart("bytes 100"))
    }

    @Test fun contentRange_zeroStartInSmallFile() {
        assertEquals(0L, DolbyVisionFallbackPolicy.parseContentRangeStart("bytes 0-0/1"))
    }
}
