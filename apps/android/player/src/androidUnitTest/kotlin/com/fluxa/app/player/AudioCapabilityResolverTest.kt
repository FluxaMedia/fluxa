package com.fluxa.app.player

import android.media.AudioFormat
import androidx.media3.common.Format
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class AudioCapabilityResolverTest {
    @Test
    fun mapsCommonEncodedFormatsToTheirAndroidAudioEncodings() {
        assertEquals(AudioFormat.ENCODING_AC3, encoding("audio/ac3"))
        assertEquals(AudioFormat.ENCODING_E_AC3, encoding("audio/eac3"))
        assertEquals(AudioFormat.ENCODING_DOLBY_TRUEHD, encoding("audio/true-hd"))
        assertEquals(AudioFormat.ENCODING_DTS, encoding("audio/vnd.dts"))
        assertEquals(AudioFormat.ENCODING_DTS_HD, encoding("audio/vnd.dts.hd"))
    }

    @Test
    fun genericMpegHIsNeverMappedWithoutAnExactProfile() {
        assertNull(encoding("audio/mpegh-mha1"))
        assertNull(encoding("audio/mpeg-h"))
    }

    @Test
    fun exactMpegHProfilesRemainProfileSpecific() {
        assertEquals(
            AudioFormat.ENCODING_MPEGH_BL_L3,
            encoding("audio/mha1-03"),
        )
        assertEquals(
            AudioFormat.ENCODING_MPEGH_LC_L4,
            encoding("audio/mhm1-0e"),
        )
    }

    private fun encoding(mime: String): Int? = AudioCapabilityResolver.encodingForFormat(
        Format.Builder()
            .setSampleMimeType(mime)
            .setChannelCount(2)
            .setSampleRate(48_000)
            .build(),
    )
}
