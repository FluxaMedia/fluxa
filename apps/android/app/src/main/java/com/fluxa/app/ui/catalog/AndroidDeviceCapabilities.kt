package com.fluxa.app.ui.catalog

import android.media.MediaCodecList
import android.os.Build
import com.fluxa.app.player.AndroidDolbyVisionCapabilities
import com.fluxa.app.player.AudioCapabilityResolver
import com.fluxa.app.shared.feature.settings.SettingsDeviceCapabilitiesUiModel

private val KNOWN_VIDEO_MIMES = listOf(
    "video/avc" to "H.264 / AVC",
    "video/hevc" to "H.265 / HEVC",
    "video/av01" to "AV1",
    "video/x-vnd.on2.vp9" to "VP9",
    "video/x-vnd.on2.vp8" to "VP8",
    "video/mpeg2" to "MPEG-2",
    "video/dolby-vision" to "Dolby Vision",
)

internal fun buildAndroidDeviceCapabilities(context: android.content.Context): SettingsDeviceCapabilitiesUiModel {
    val audio = AudioCapabilityResolver.resolve(context, AudioCapabilityResolver.mediaAudioAttributes())
    val passthrough = AudioCapabilityResolver.supportedPassthroughNames(
        context,
        AudioCapabilityResolver.mediaAudioAttributes(),
    )
    val knownAudio = listOf(
        "AC3", "E-AC3", "E-AC3 JOC", "DTS", "DTS-HD", "DTS-HD MA", "DTS-UHD P1", "DTS-UHD P2",
        "TrueHD", "Dolby MAT", "AC4", "DRA", "MPEG-H BL L3", "MPEG-H BL L4", "MPEG-H LC L3", "MPEG-H LC L4",
    )

    val video = queryVideoDecoders()
    val detectedVideo = (video.hardware + video.software).map { it.substringBefore(" (") }.toSet()
    val hdr = AndroidDolbyVisionCapabilities.detect(context)
    val hdrOutput = buildList {
        if (hdr.displaySupportsDolbyVision) add("Dolby Vision")
        if (hdr.displaySupportsHdr10) add("HDR10")
        if (hdr.displaySupportsHdr10Plus) add("HDR10+")
        if (hdr.displaySupportsHlg) add("HLG")
    }
    val knownHdr = listOf("Dolby Vision", "HDR10", "HDR10+", "HLG")

    return SettingsDeviceCapabilitiesUiModel(
        deviceName = "${Build.MANUFACTURER} ${Build.MODEL}".trim(),
        platformVersion = "Android ${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})",
        audioRoute = audio.routeDescription,
        audioPcmChannels = "${audio.maxPcmChannels} kanal",
        audioPcmSampleRates = audio.pcmSampleRates.sorted().joinToString(", ").ifBlank { "Sistem bildirimi yok" },
        audioPassthroughSupported = passthrough,
        audioPassthroughNotDetected = knownAudio.filterNot { it in passthrough },
        audioFallback = "MediaCodec → FFmpeg ses çözücüsü → PCM; işleme gerekirse downmix",
        videoHardwareDecoders = video.hardware,
        videoSoftwareDecoders = video.software,
        videoNotDetected = KNOWN_VIDEO_MIMES.map { it.second }.filterNot { it in detectedVideo } +
            knownHdr.filterNot { it in hdrOutput }.map { "$it HDR çıkışı" },
        hdrOutput = hdrOutput,
    )
}

private data class VideoDecoderLists(
    val hardware: List<String>,
    val software: List<String>,
)

private fun queryVideoDecoders(): VideoDecoderLists {
    val hardware = linkedSetOf<String>()
    val software = linkedSetOf<String>()
    runCatching {
        for (codec in MediaCodecList(MediaCodecList.REGULAR_CODECS).codecInfos) {
            if (codec.isEncoder) continue
            for ((mime, label) in KNOWN_VIDEO_MIMES) {
                if (codec.supportedTypes.none { it.equals(mime, ignoreCase = true) }) continue
                val profileCount = runCatching {
                    codec.getCapabilitiesForType(mime).profileLevels?.size ?: 0
                }.getOrDefault(0)
                val suffix = " (${codec.name}, ${profileCount} profil)"
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q && codec.isHardwareAccelerated) {
                    hardware += label + suffix
                } else {
                    software += label + suffix
                }
            }
        }
    }
    return VideoDecoderLists(hardware.toList(), software.toList())
}
