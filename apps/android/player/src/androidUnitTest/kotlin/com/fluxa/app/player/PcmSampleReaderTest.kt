package com.fluxa.app.player

import androidx.media3.common.C
import java.nio.ByteBuffer
import java.nio.ByteOrder
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class PcmSampleReaderTest {
    @Test
    fun readsUnsigned8BitPcmAroundZero() {
        val source = ByteBuffer.wrap(byteArrayOf(0x00, 0x80.toByte(), 0xff.toByte()))

        assertEquals(-1f, PcmSampleReader.read(source, C.ENCODING_PCM_8BIT))
        assertEquals(0f, PcmSampleReader.read(source, C.ENCODING_PCM_8BIT))
        assertEquals(127 / 128f, PcmSampleReader.read(source, C.ENCODING_PCM_8BIT))
    }

    @Test
    fun readsSigned16LittleAndBigEndian() {
        val little = ByteBuffer.allocate(4).order(ByteOrder.LITTLE_ENDIAN)
            .putShort(Short.MIN_VALUE)
            .putShort(Short.MAX_VALUE)
            .flip() as ByteBuffer
        val big = ByteBuffer.wrap(byteArrayOf(0x80.toByte(), 0x00, 0x7f, 0xff.toByte()))

        assertEquals(-1f, PcmSampleReader.read(little, C.ENCODING_PCM_16BIT))
        assertEquals(Short.MAX_VALUE / 32768f, PcmSampleReader.read(little, C.ENCODING_PCM_16BIT))
        assertEquals(-1f, PcmSampleReader.read(big, C.ENCODING_PCM_16BIT_BIG_ENDIAN))
        assertEquals(Short.MAX_VALUE / 32768f, PcmSampleReader.read(big, C.ENCODING_PCM_16BIT_BIG_ENDIAN))
    }

    @Test
    fun readsSigned24LittleAndBigEndianWithSignExtension() {
        val little = ByteBuffer.wrap(byteArrayOf(0x00, 0x00, 0x80.toByte(), 0xff.toByte(), 0xff.toByte(), 0x7f))
        val big = ByteBuffer.wrap(byteArrayOf(0x80.toByte(), 0x00, 0x00, 0x7f, 0xff.toByte(), 0xff.toByte()))

        assertEquals(-1f, PcmSampleReader.read(little, C.ENCODING_PCM_24BIT))
        assertEquals(0x7fffff / 8_388_608f, PcmSampleReader.read(little, C.ENCODING_PCM_24BIT))
        assertEquals(-1f, PcmSampleReader.read(big, C.ENCODING_PCM_24BIT_BIG_ENDIAN))
        assertEquals(0x7fffff / 8_388_608f, PcmSampleReader.read(big, C.ENCODING_PCM_24BIT_BIG_ENDIAN))
    }

    @Test
    fun readsSigned32LittleAndBigEndian() {
        val little = ByteBuffer.allocate(8).order(ByteOrder.LITTLE_ENDIAN)
            .putInt(Int.MIN_VALUE)
            .putInt(Int.MAX_VALUE)
            .flip() as ByteBuffer
        val big = ByteBuffer.wrap(
            byteArrayOf(
                0x80.toByte(), 0x00, 0x00, 0x00,
                0x7f, 0xff.toByte(), 0xff.toByte(), 0xff.toByte(),
            )
        )

        assertEquals(-1f, PcmSampleReader.read(little, C.ENCODING_PCM_32BIT))
        assertEquals(Int.MAX_VALUE / 2_147_483_648f, PcmSampleReader.read(little, C.ENCODING_PCM_32BIT))
        assertEquals(-1f, PcmSampleReader.read(big, C.ENCODING_PCM_32BIT_BIG_ENDIAN))
        assertEquals(Int.MAX_VALUE / 2_147_483_648f, PcmSampleReader.read(big, C.ENCODING_PCM_32BIT_BIG_ENDIAN))
    }

    @Test
    fun preservesFloatSamplesAndRejectsUnsupportedEncodings() {
        val source = ByteBuffer.allocate(8).order(ByteOrder.LITTLE_ENDIAN)
            .putFloat(-0.25f)
            .putFloat(0.75f)
            .flip() as ByteBuffer

        assertTrue(PcmSampleReader.isSupported(C.ENCODING_PCM_FLOAT))
        assertEquals(-0.25f, PcmSampleReader.read(source, C.ENCODING_PCM_FLOAT))
        assertEquals(0.75f, PcmSampleReader.read(source, C.ENCODING_PCM_FLOAT))
        assertFalse(PcmSampleReader.isSupported(C.ENCODING_INVALID))
    }
}
