package com.fluxa.app.shared.feature.player

import kotlin.test.Test
import kotlin.test.assertEquals

class AudioTrackQualityPolicyTest {
    @Test
    fun preferredLanguageWinsBeforeCodecQuality() {
        val tracks = listOf(
            track("en_truehd", "en", "audio/true-hd", 8),
            track("tr_eac3", "tr", "audio/eac3", 6),
        )

        assertEquals("tr_eac3", AudioTrackQualityPolicy.choose(tracks, "tr")?.id)
    }

    @Test
    fun losslessCodecAndChannelLayoutWinWithinLanguage() {
        val tracks = listOf(
            track("stereo_aac", "en", "audio/mp4a-latm", 2),
            track("surround_flac", "en", "audio/flac", 6),
            track("surround_dts", "en", "audio/vnd.dts", 8),
        )

        assertEquals("surround_flac", AudioTrackQualityPolicy.choose(tracks, "en")?.id)
    }

    @Test
    fun unsupportedTracksAreNeverSelected() {
        val tracks = listOf(
            track("unsupported_truehd", "en", "audio/true-hd", 8, supported = false),
            track("fallback_aac", "en", "audio/mp4a-latm", 2),
        )

        assertEquals("fallback_aac", AudioTrackQualityPolicy.choose(tracks, "en")?.id)
    }

    @Test
    fun sameCodecAndLayoutUsesHigherBitrateWithoutSampleRateBias() {
        val tracks = listOf(
            track("low", "en", "audio/eac3", 6, bitrate = 384_000L, sampleRate = 48_000),
            track("high", "en", "audio/eac3", 6, bitrate = 768_000L, sampleRate = 48_000),
        )

        assertEquals("high", AudioTrackQualityPolicy.choose(tracks, "en")?.id)
    }

    @Test
    fun ac4IsPreferredOverBasicAacWithinTheSameLanguage() {
        val tracks = listOf(
            track("aac", "en", "audio/mp4a-latm", 2),
            track("ac4", "en", "audio/ac4", 6),
        )

        assertEquals("ac4", AudioTrackQualityPolicy.choose(tracks, "en")?.id)
    }

    @Test
    fun immersiveCodecIsPreferredOverBasicAacWithinTheSameLanguage() {
        val tracks = listOf(
            track("aac", "en", "audio/mp4a-latm", 2),
            track("mpegh", "en", "audio/mpegh-mha1", 8),
        )

        assertEquals("mpegh", AudioTrackQualityPolicy.choose(tracks, "en")?.id)
    }

    @Test
    fun newerLosslessBitstreamFamiliesRemainHighPriority() {
        val tracks = listOf(
            track("aac", "en", "audio/mp4a-latm", 2),
            track("mat", "en", "audio/vnd.dolby.mat", 8),
        )

        assertEquals("mat", AudioTrackQualityPolicy.choose(tracks, "en")?.id)
    }

    @Test
    fun uncompressedPcmIsPreferredOverLossyAac() {
        val tracks = listOf(
            track("aac", "en", "audio/mp4a-latm", 2),
            track("pcm", "en", "audio/raw", 2),
        )

        assertEquals("pcm", AudioTrackQualityPolicy.choose(tracks, "en")?.id)
    }

    @Test
    fun uncommonLosslessCodecIsPreferredOverLossyAac() {
        val tracks = listOf(
            track("aac", "en", "audio/mp4a-latm", 2),
            track("wavpack", "en", "audio/wavpack", 2),
        )

        assertEquals("wavpack", AudioTrackQualityPolicy.choose(tracks, "en")?.id)
    }

    @Test
    fun unsupportedSampleRateLosesBeforeBitrateComparison() {
        val tracks = listOf(
            track("resampled", "en", "audio/eac3", 6, bitrate = 1_000_000L, sampleRate = 192_000),
            track("native_rate", "en", "audio/eac3", 6, bitrate = 384_000L, sampleRate = 48_000),
        )

        assertEquals(
            "native_rate",
            AudioTrackQualityPolicy.choose(tracks, "en", supportedSampleRates = setOf(48_000))?.id
        )
    }

    @Test
    fun passthroughTrackWinsOverHigherRankedDecodedTrack() {
        val tracks = listOf(
            track("truehd", "en", "audio/true-hd", 8),
            track("eac3", "en", "audio/eac3", 6),
        )

        assertEquals(
            "eac3",
            AudioTrackQualityPolicy.choose(tracks, "en", passthroughTrackIds = setOf("eac3"))?.id
        )
    }

    @Test
    fun emptyLanguageStillChoosesRouteCompatibleTrack() {
        val tracks = listOf(
            track("truehd", "en", "audio/true-hd", 8, sampleRate = 192_000),
            track("eac3", "en", "audio/eac3", 6, sampleRate = 48_000),
        )

        assertEquals(
            "eac3",
            AudioTrackQualityPolicy.choose(
                tracks,
                preferredLanguage = "",
                supportedSampleRates = setOf(48_000),
                maxPcmChannels = 8,
            )?.id
        )
    }

    @Test
    fun stereoRouteKeepsLosslessSurroundAndDownmixesInsteadOfChoosingLossyStereo() {
        val tracks = listOf(
            track("surround_flac", "en", "audio/flac", 6),
            track("stereo_aac", "en", "audio/mp4a-latm", 2),
        )

        assertEquals("surround_flac", AudioTrackQualityPolicy.choose(tracks, "en", maxPcmChannels = 2)?.id)
    }

    @Test
    fun multichannelRouteKeepsHigherQualitySurroundTrack() {
        val tracks = listOf(
            track("stereo_aac", "en", "audio/mp4a-latm", 2),
            track("surround_flac", "en", "audio/flac", 6),
        )

        assertEquals(
            "surround_flac",
            AudioTrackQualityPolicy.choose(tracks, "en", maxPcmChannels = 8)?.id
        )
    }

    @Test
    fun equalCandidatesUseStableIdInsteadOfInputOrder() {
        val tracks = listOf(
            track("audio_b", "en", "audio/aac", 2),
            track("audio_a", "en", "audio/aac", 2),
        )

        assertEquals("audio_b", AudioTrackQualityPolicy.choose(tracks, "en")?.id)
    }

    @Test
    fun equalCandidatesKeepTheCurrentlySelectedTrack() {
        val tracks = listOf(
            track("audio_a", "en", "audio/aac", 2, isSelected = false),
            track("audio_b", "en", "audio/aac", 2, isSelected = true),
        )

        assertEquals("audio_b", AudioTrackQualityPolicy.choose(tracks, "en")?.id)
    }

    private fun track(
        id: String,
        language: String,
        mime: String,
        channels: Int,
        supported: Boolean = true,
        bitrate: Long? = null,
        sampleRate: Int? = null,
        isSelected: Boolean = false,
    ) = MediaTrack(
        id = id,
        label = id,
        language = language,
        type = 1,
        groupIndex = 0,
        trackIndex = 0,
        isSelected = isSelected,
        isSupported = supported,
        channelCount = channels,
        sampleMimeType = mime,
        bitrate = bitrate,
        sampleRate = sampleRate,
    )
}
