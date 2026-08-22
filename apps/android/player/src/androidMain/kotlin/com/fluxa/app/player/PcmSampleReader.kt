@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.player

import androidx.media3.common.C
import java.nio.ByteBuffer

/** One PCM-to-float boundary shared by every Android audio processor. */
internal object PcmSampleReader {
    fun isSupported(encoding: Int): Boolean = encoding == C.ENCODING_PCM_8BIT ||
        encoding == C.ENCODING_PCM_16BIT ||
        encoding == C.ENCODING_PCM_16BIT_BIG_ENDIAN ||
        encoding == C.ENCODING_PCM_24BIT ||
        encoding == C.ENCODING_PCM_24BIT_BIG_ENDIAN ||
        encoding == C.ENCODING_PCM_32BIT ||
        encoding == C.ENCODING_PCM_32BIT_BIG_ENDIAN ||
        encoding == C.ENCODING_PCM_FLOAT

    fun bytesPerSample(encoding: Int): Int = when (encoding) {
        C.ENCODING_PCM_8BIT -> 1
        C.ENCODING_PCM_24BIT,
        C.ENCODING_PCM_24BIT_BIG_ENDIAN -> 3
        C.ENCODING_PCM_16BIT,
        C.ENCODING_PCM_16BIT_BIG_ENDIAN -> 2
        else -> 4
    }

    fun read(source: ByteBuffer, encoding: Int): Float = when (encoding) {
        // Android PCM_8BIT is unsigned PCM centered at 128, unlike the
        // signed 16/24/32-bit encodings below.
        C.ENCODING_PCM_8BIT -> ((source.get().toInt() and 0xff) - 128) / 128f
        C.ENCODING_PCM_16BIT -> source.short / 32768f
        C.ENCODING_PCM_16BIT_BIG_ENDIAN -> readSigned16(source, bigEndian = true) / 32768f
        C.ENCODING_PCM_24BIT -> readSigned24(source, bigEndian = false) / 8_388_608f
        C.ENCODING_PCM_24BIT_BIG_ENDIAN -> readSigned24(source, bigEndian = true) / 8_388_608f
        C.ENCODING_PCM_32BIT -> source.int / 2_147_483_648f
        C.ENCODING_PCM_32BIT_BIG_ENDIAN -> readSigned32(source, bigEndian = true) / 2_147_483_648f
        else -> source.float
    }

    private fun readSigned16(source: ByteBuffer, bigEndian: Boolean): Int {
        val first = source.get().toInt() and 0xff
        val second = source.get().toInt() and 0xff
        val raw = if (bigEndian) (first shl 8) or second else first or (second shl 8)
        return if (raw and 0x8000 != 0) raw or -0x10000 else raw
    }

    private fun readSigned24(source: ByteBuffer, bigEndian: Boolean): Int {
        val first = source.get().toInt() and 0xff
        val second = source.get().toInt() and 0xff
        val third = source.get().toInt() and 0xff
        val raw = if (bigEndian) {
            (first shl 16) or (second shl 8) or third
        } else {
            first or (second shl 8) or (third shl 16)
        }
        return if (raw and 0x800000 != 0) raw or -0x1000000 else raw
    }

    private fun readSigned32(source: ByteBuffer, bigEndian: Boolean): Int {
        val first = source.get().toInt() and 0xff
        val second = source.get().toInt() and 0xff
        val third = source.get().toInt() and 0xff
        val fourth = source.get().toInt() and 0xff
        return if (bigEndian) {
            (first shl 24) or (second shl 16) or (third shl 8) or fourth
        } else {
            first or (second shl 8) or (third shl 16) or (fourth shl 24)
        }
    }
}
