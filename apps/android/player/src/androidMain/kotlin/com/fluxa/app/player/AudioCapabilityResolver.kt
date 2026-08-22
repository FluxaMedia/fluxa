@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.player

import android.content.Context
import android.media.AudioFormat
import android.media.AudioManager
import android.os.Build
import androidx.media3.common.AudioAttributes
import androidx.media3.common.C
import androidx.media3.common.Format
import androidx.media3.exoplayer.audio.AudioCapabilities
import com.fluxa.app.shared.feature.player.AudioPcmChannelPolicy

enum class AudioOutputMode {
    PASSTHROUGH,
    PCM,
    DOWNMIX
}

data class AudioOutputCapabilities(
    val maxPcmChannels: Int,
    val passthroughEncodings: Set<Int>,
    val speakerLayoutMasks: List<Int>,
    val spatialLayoutMasks: List<Int>,
    val routeDescription: String,
    private val media3Capabilities: AudioCapabilities,
    val pcmSampleRates: Set<Int> = emptySet(),
    private val directPlaybackChecker: ((Format, AudioAttributes) -> Boolean)? = null,
) {
    fun supportsPassthrough(format: Format, attributes: AudioAttributes): Boolean {
        // MPEG-H has several Android direct-playback encodings. A generic
        // MIME does not identify one of them, so never treat it as a safe
        // bitstream candidate until the profile is present in MIME/codecs.
        if (isGenericMpegH(format)) return false
        val encoding = AudioCapabilityResolver.encodingForFormat(format) ?: return false
        if (encoding !in passthroughEncodings) return false
        return media3Capabilities.isPassthroughPlaybackSupported(format, attributes) &&
            (directPlaybackChecker?.invoke(format, attributes) ?: true)
    }

    fun supportsPcm(channelCount: Int): Boolean =
        channelCount in 1..maxPcmChannels

    fun supportsPassthroughMime(
        sampleMimeType: String?,
        channelCount: Int = 2,
        sampleRate: Int = 48_000,
    ): Boolean {
        val mime = sampleMimeType?.lowercase()?.replace('.', '-') ?: ""
        val directSupported: () -> Boolean = {
            if (directPlaybackChecker == null) {
                true
            } else {
                val format = Format.Builder()
                    .setSampleMimeType(sampleMimeType)
                    .setChannelCount(channelCount.takeIf { it > 0 } ?: 2)
                    .setSampleRate(sampleRate.takeIf { it > 0 } ?: 48_000)
                    .build()
                // MPV does not expose a Media3 Format, so use the same
                // Android direct bitstream check used by ExoPlayer.
                directPlaybackChecker.invoke(format, AudioCapabilityResolver.mediaAudioAttributes())
            }
        }
        if (Build.VERSION.SDK_INT >= 31 &&
            (mime.contains("mpegh") || mime.contains("mha1") || mime.contains("mhm1"))
        ) {
            val exactEncoding = when {
                mime.contains("mhm1-03") || mime.contains("mha1-03") ->
                    AudioFormat.ENCODING_MPEGH_BL_L3
                mime.contains("mhm1-04") || mime.contains("mha1-04") ->
                    AudioFormat.ENCODING_MPEGH_BL_L4
                mime.contains("mhm1-0d") || mime.contains("mha1-0d") ->
                    AudioFormat.ENCODING_MPEGH_LC_L3
                mime.contains("mhm1-0e") || mime.contains("mha1-0e") ->
                    AudioFormat.ENCODING_MPEGH_LC_L4
                else -> null
            }
            return if (exactEncoding != null) {
                exactEncoding in passthroughEncodings && directSupported()
            } else {
                // MPEG-H's Android encoding constants are profile/level
                // specific. A generic MIME string does not tell us whether
                // the stream is BL L3/L4 or LC L3/L4, so accepting it based
                // on any route MPEG-H capability would be a guess and could
                // send an incompatible bitstream. Let Media3/FFmpeg decode
                // it unless the exact profile is present in the format.
                false
            }
        }
        val encoding = when {
            mime.contains("truehd") || mime.contains("true-hd") -> AudioFormat.ENCODING_DOLBY_TRUEHD
            mime.contains("eac3-joc") || mime.contains("ec-3-joc") -> {
                if (Build.VERSION.SDK_INT >= 28) AudioFormat.ENCODING_E_AC3_JOC else return false
            }
            mime.contains("eac3") || mime.contains("ec-3") -> AudioFormat.ENCODING_E_AC3
            mime.contains("ac3") || mime.contains("ac-3") -> AudioFormat.ENCODING_AC3
            mime.contains("dolby-mat") || mime.contains("dolbymat") -> {
                if (Build.VERSION.SDK_INT >= 29) AudioFormat.ENCODING_DOLBY_MAT else return false
            }
            mime.contains("dts-hd-ma") || mime.contains("dtshd-ma") -> {
                if (Build.VERSION.SDK_INT >= 34) AudioFormat.ENCODING_DTS_HD_MA else AudioFormat.ENCODING_DTS_HD
            }
            mime.contains("dts-uhd-p1") || mime.contains("dtsuhdp1") -> {
                if (Build.VERSION.SDK_INT >= 34) AudioFormat.ENCODING_DTS_UHD_P1 else return false
            }
            mime.contains("dts-uhd-p2") || mime.contains("dtsuhdp2") -> {
                if (Build.VERSION.SDK_INT >= 34) AudioFormat.ENCODING_DTS_UHD_P2 else return false
            }
            mime.contains("dts-uhd") -> return false
            mime.contains("dts-hd") || mime.contains("dtshd") -> AudioFormat.ENCODING_DTS_HD
            mime.contains("dts") -> AudioFormat.ENCODING_DTS
            mime.contains("ac4") && Build.VERSION.SDK_INT >= 29 -> AudioFormat.ENCODING_AC4
            mime.contains("dra") && Build.VERSION.SDK_INT >= 31 -> AudioFormat.ENCODING_DRA
            else -> return false
        }
        if (encoding !in passthroughEncodings) return false
        return directSupported()
    }

    fun sinkCapabilities(processingMode: String): AudioCapabilities =
        if (processingMode == "balanced" || processingMode == "night") {
            // Processing modes need decoded PCM so their DSP can run.
            // Keep the route's PCM channel budget, but deliberately advertise no
            // encoded profiles to DefaultAudioSink.
            // Keep the PCM profile so the constructor preserves maxPcmChannels;
            // an empty encoding array would make Media3 report zero channels.
            AudioCapabilities(intArrayOf(C.ENCODING_PCM_16BIT), maxPcmChannels)
        } else {
            media3Capabilities
        }

    fun modeFor(format: Format, attributes: AudioAttributes): AudioOutputMode {
        if (supportsPassthrough(format, attributes)) return AudioOutputMode.PASSTHROUGH
        return if (supportsPcm(format.channelCount.takeIf { it > 0 } ?: 2)) {
            AudioOutputMode.PCM
        } else {
            AudioOutputMode.DOWNMIX
        }
    }

    private fun isGenericMpegH(format: Format): Boolean {
        val mime = format.sampleMimeType?.lowercase()?.replace('.', '-') ?: return false
        if (!mime.contains("mpegh") && !mime.contains("mha1") && !mime.contains("mhm1")) return false
        val descriptor = "${mime}|${format.codecs.orEmpty().lowercase().replace('.', '-')}"
        return !descriptor.contains("-03") &&
            !descriptor.contains("-04") &&
            !descriptor.contains("-0d") &&
            !descriptor.contains("-0e")
    }
}

data class AudioOutputPlan(
    val mode: AudioOutputMode,
    val reason: String,
    val sourceMimeType: String?,
    val sourceChannels: Int,
    val sourceSampleRate: Int,
)

object AudioCapabilityResolver {
    fun mediaAudioAttributes(): AudioAttributes = AudioAttributes.Builder()
        .setUsage(C.USAGE_MEDIA)
        .setContentType(C.AUDIO_CONTENT_TYPE_MOVIE)
        .setSpatializationBehavior(C.SPATIALIZATION_BEHAVIOR_AUTO)
        .setIsContentSpatialized(false)
        .build()

    private fun passthroughEncodingNames(): Map<Int, String> = buildMap {
        put(AudioFormat.ENCODING_AC3, "AC3")
        put(AudioFormat.ENCODING_E_AC3, "E-AC3")
        put(AudioFormat.ENCODING_DTS, "DTS")
        put(AudioFormat.ENCODING_DTS_HD, "DTS-HD")
        put(AudioFormat.ENCODING_DOLBY_TRUEHD, "TrueHD")
        if (Build.VERSION.SDK_INT >= 29) put(AudioFormat.ENCODING_DOLBY_MAT, "Dolby MAT")
        if (Build.VERSION.SDK_INT >= 28) put(AudioFormat.ENCODING_E_AC3_JOC, "E-AC3 JOC")
        if (Build.VERSION.SDK_INT >= 29) put(AudioFormat.ENCODING_AC4, "AC4")
        if (Build.VERSION.SDK_INT >= 31) {
            put(AudioFormat.ENCODING_DRA, "DRA")
            put(AudioFormat.ENCODING_MPEGH_BL_L3, "MPEG-H BL L3")
            put(AudioFormat.ENCODING_MPEGH_BL_L4, "MPEG-H BL L4")
            put(AudioFormat.ENCODING_MPEGH_LC_L3, "MPEG-H LC L3")
            put(AudioFormat.ENCODING_MPEGH_LC_L4, "MPEG-H LC L4")
        }
        if (Build.VERSION.SDK_INT >= 34) {
            put(AudioFormat.ENCODING_DTS_HD_MA, "DTS-HD MA")
            put(AudioFormat.ENCODING_DTS_UHD_P1, "DTS-UHD P1")
            put(AudioFormat.ENCODING_DTS_UHD_P2, "DTS-UHD P2")
        }
    }

    fun resolve(context: Context, attributes: AudioAttributes = AudioAttributes.DEFAULT): AudioOutputCapabilities {
        val route = preferredOutput(context)
        // Passing the routed device is important: a phone's default output
        // must not donate its codec list to an active HDMI/eARC AVR route.
        val capabilities = AudioCapabilities.getCapabilities(context, attributes, route)
        val encodingNames = passthroughEncodingNames()
        val maxPcmChannels = effectivePcmChannelCount(capabilities, route)
        val passthroughEncodings = encodingNames.keys
            .filter { encoding -> capabilities.supportsEncoding(encoding) }
            .toSet()
        return AudioOutputCapabilities(
            maxPcmChannels = maxPcmChannels,
            passthroughEncodings = passthroughEncodings,
            speakerLayoutMasks = capabilities.speakerLayoutChannelMasks.toList(),
            spatialLayoutMasks = capabilities.spatializerChannelMasks.toList(),
            routeDescription = describeRoute(route),
            media3Capabilities = capabilities,
            pcmSampleRates = route?.sampleRates
                ?.filter { it > 0 }
                ?.toSet()
                .orEmpty(),
            directPlaybackChecker = if (Build.VERSION.SDK_INT >= 31) {
                { format, mediaAttributes ->
                    directPlaybackSupport(format, mediaAttributes, passthroughEncodings)
                }
            } else {
                null
            },
        )
    }

    fun plan(
        context: Context,
        format: Format,
        attributes: AudioAttributes = AudioAttributes.DEFAULT,
    ): AudioOutputPlan {
        val capabilities = resolve(context, attributes)
        val passthrough = capabilities.supportsPassthrough(format, attributes)
        val channels = format.channelCount.takeIf { it > 0 } ?: 2
        val mode = when {
            passthrough -> AudioOutputMode.PASSTHROUGH
            channels <= capabilities.maxPcmChannels -> AudioOutputMode.PCM
            else -> AudioOutputMode.DOWNMIX
        }
        return AudioOutputPlan(
            mode = mode,
            reason = when (mode) {
                AudioOutputMode.PASSTHROUGH -> "active route accepts encoded ${format.sampleMimeType}"
                AudioOutputMode.PCM -> "active route accepts ${channels}ch PCM"
                AudioOutputMode.DOWNMIX -> "active route reports max ${capabilities.maxPcmChannels} PCM channels"
            },
            sourceMimeType = format.sampleMimeType,
            sourceChannels = channels,
            sourceSampleRate = format.sampleRate.takeIf { it > 0 } ?: 0,
        )
    }

    fun describePassthroughSupport(context: Context, attributes: AudioAttributes = AudioAttributes.DEFAULT): String {
        val capabilities = resolve(context, attributes)
        val names = supportedPassthroughNames(context, attributes).joinToString(",")
            .ifBlank { "none" }
        return "route=${capabilities.routeDescription} pcm_channels=${capabilities.maxPcmChannels} passthrough=$names"
    }

    /** Human-readable, route-confirmed names for the device-info screen and diagnostics. */
    fun supportedPassthroughNames(
        context: Context,
        attributes: AudioAttributes = AudioAttributes.DEFAULT,
    ): List<String> {
        val capabilities = resolve(context, attributes)
        return passthroughEncodingNames()
            .filterKeys { encoding -> encoding in capabilities.passthroughEncodings }
            .values
            .toList()
    }

    private fun effectivePcmChannelCount(
        capabilities: AudioCapabilities,
        route: android.media.AudioDeviceInfo?,
    ): Int {
        val deviceMax = route?.channelCounts?.maxOrNull()?.takeIf { it > 0 }
        // A spatializer route may expose only a stereo physical device while
        // accepting multichannel PCM as its input. Do not collapse surround
        // tracks merely because AudioDeviceInfo describes the final speakers.
        val spatializerMax = capabilities.spatializerChannelMasks
            .asSequence()
            .map { Integer.bitCount(it) }
            .filter { it > 0 }
            .maxOrNull()
        val speakerLayoutMax = capabilities.speakerLayoutChannelMasks
            .asSequence()
            .map { Integer.bitCount(it) }
            .filter { it > 0 }
            .maxOrNull()
        val routeSupportsMultichannel = route?.type == android.media.AudioDeviceInfo.TYPE_HDMI ||
            (Build.VERSION.SDK_INT >= 29 && route?.type == android.media.AudioDeviceInfo.TYPE_HDMI_ARC) ||
            (Build.VERSION.SDK_INT >= 31 && route?.type == android.media.AudioDeviceInfo.TYPE_HDMI_EARC) ||
            route?.type == android.media.AudioDeviceInfo.TYPE_USB_DEVICE ||
            route?.type == android.media.AudioDeviceInfo.TYPE_USB_HEADSET ||
            route?.type == android.media.AudioDeviceInfo.TYPE_USB_ACCESSORY
        // Some HDMI/USB drivers report the physical endpoint as stereo while
        // their capability masks advertise accepted 5.1/7.1 PCM layouts. The
        // shared policy keeps those layouts instead of downmixing too early;
        // ordinary headphones and Bluetooth remain conservative.
        return AudioPcmChannelPolicy.resolve(
            deviceMaxChannels = deviceMax,
            capabilitiesMaxChannels = capabilities.maxChannelCount,
            speakerLayoutMaxChannels = speakerLayoutMax,
            spatializerMaxChannels = spatializerMax,
            routeSupportsMultichannel = routeSupportsMultichannel,
        )
    }

    private fun preferredOutput(context: Context): android.media.AudioDeviceInfo? {
        val audioManager = context.getSystemService(Context.AUDIO_SERVICE) as? AudioManager ?: return null
        if (Build.VERSION.SDK_INT >= 31) {
            val routed = runCatching {
                audioManager.getAudioDevicesForAttributes(
                    android.media.AudioAttributes.Builder()
                        .setUsage(android.media.AudioAttributes.USAGE_MEDIA)
                        .setContentType(android.media.AudioAttributes.CONTENT_TYPE_MOVIE)
                        .build()
                )
            }.getOrNull()?.maxByOrNull(::routePriority)
            if (routed != null) return routed
        }
        val outputs = audioManager.getDevices(AudioManager.GET_DEVICES_OUTPUTS)
        if (Build.VERSION.SDK_INT < 31) {
            // Before getAudioDevicesForAttributes(), getDevices() reports
            // connected sinks rather than the media-selected sink. Prefer the
            // legacy active-route signals so a connected HDMI cable does not
            // steal capability resolution from active Bluetooth/wired audio.
            val activeTypes = buildSet {
                if (audioManager.isBluetoothA2dpOn) add(android.media.AudioDeviceInfo.TYPE_BLUETOOTH_A2DP)
                if (audioManager.isBluetoothScoOn) add(android.media.AudioDeviceInfo.TYPE_BLUETOOTH_SCO)
                if (audioManager.isWiredHeadsetOn) {
                    add(android.media.AudioDeviceInfo.TYPE_WIRED_HEADPHONES)
                    add(android.media.AudioDeviceInfo.TYPE_WIRED_HEADSET)
                    add(android.media.AudioDeviceInfo.TYPE_LINE_ANALOG)
                }
                if (audioManager.isSpeakerphoneOn) add(android.media.AudioDeviceInfo.TYPE_BUILTIN_SPEAKER)
            }
            outputs.firstOrNull { it.type in activeTypes }?.let { return it }
        }
        return outputs
            .filter { routePriority(it) > 0 }
            .maxByOrNull(::routePriority)
            ?: outputs.firstOrNull()
    }

    private fun routePriority(route: android.media.AudioDeviceInfo): Int = when (route.type) {
        android.media.AudioDeviceInfo.TYPE_HDMI -> 100
        android.media.AudioDeviceInfo.TYPE_HDMI_ARC -> if (Build.VERSION.SDK_INT >= 29) 110 else 0
        android.media.AudioDeviceInfo.TYPE_HDMI_EARC -> if (Build.VERSION.SDK_INT >= 31) 120 else 0
        android.media.AudioDeviceInfo.TYPE_USB_DEVICE,
        android.media.AudioDeviceInfo.TYPE_USB_HEADSET,
        android.media.AudioDeviceInfo.TYPE_USB_ACCESSORY -> 80
        android.media.AudioDeviceInfo.TYPE_BLE_HEADSET,
        android.media.AudioDeviceInfo.TYPE_BLE_SPEAKER,
        android.media.AudioDeviceInfo.TYPE_BLE_BROADCAST -> if (Build.VERSION.SDK_INT >= 31) 60 else 0
        android.media.AudioDeviceInfo.TYPE_BLUETOOTH_A2DP -> 50
        android.media.AudioDeviceInfo.TYPE_BLUETOOTH_SCO -> if (Build.VERSION.SDK_INT >= 23) 45 else 0
        android.media.AudioDeviceInfo.TYPE_WIRED_HEADPHONES,
        android.media.AudioDeviceInfo.TYPE_WIRED_HEADSET,
        android.media.AudioDeviceInfo.TYPE_LINE_ANALOG -> 40
        else -> 0
    }

    private fun describeRoute(route: android.media.AudioDeviceInfo?): String {
        if (route == null) return "speaker"
        val name = route.productName?.toString()?.takeIf { it.isNotBlank() }
        val type = route.type.toString()
        val address = route.address.takeIf { it.isNotBlank() }
        return listOfNotNull(name, "type=$type", address?.let { "address=$it" })
            .joinToString(" ")
    }

    private fun directPlaybackSupport(
        format: Format,
        @Suppress("UNUSED_PARAMETER") mediaAttributes: AudioAttributes,
        routePassthroughEncodings: Set<Int>,
    ): Boolean {
        if (Build.VERSION.SDK_INT < 31) return true
        // The resolver deliberately exposes only encodings that the active
        // route reported. Do not let Media3's broader MIME support turn an
        // unlisted/unknown bitstream into a passthrough guess.
        val encoding = encodingForFormat(format) ?: return false
        if (encoding !in routePassthroughEncodings) return false
        val channelMask = channelMaskForCount(format.channelCount)
            ?: return false // Never guess an uncommon bitstream layout (for example 5.0).
        val androidAttributes = android.media.AudioAttributes.Builder()
            .setUsage(android.media.AudioAttributes.USAGE_MEDIA)
            .setContentType(android.media.AudioAttributes.CONTENT_TYPE_MOVIE)
            .build()
        val audioFormat = android.media.AudioFormat.Builder()
            .setEncoding(encoding)
            .setSampleRate(format.sampleRate.takeIf { it > 0 } ?: 48_000)
            .setChannelMask(channelMask)
            .build()
        return runCatching {
            android.media.AudioManager.getDirectPlaybackSupport(audioFormat, androidAttributes) ==
                android.media.AudioManager.DIRECT_PLAYBACK_BITSTREAM_SUPPORTED
        }.getOrDefault(false)
    }

    internal fun encodingForFormat(format: Format): Int? {
        val mime = format.sampleMimeType?.lowercase()?.replace('.', '-') ?: return null
        val descriptor = "${mime}|${format.codecs.orEmpty().lowercase().replace('.', '-')}"
        return when {
            mime.contains("truehd") || mime.contains("true-hd") -> AudioFormat.ENCODING_DOLBY_TRUEHD
            mime.contains("eac3-joc") || mime.contains("ec-3-joc") -> {
                if (Build.VERSION.SDK_INT >= 28) AudioFormat.ENCODING_E_AC3_JOC else null
            }
            mime.contains("eac3") || mime.contains("ec-3") -> AudioFormat.ENCODING_E_AC3
            mime.contains("ac3") || mime.contains("ac-3") -> AudioFormat.ENCODING_AC3
            mime.contains("dolby-mat") || mime.contains("dolbymat") -> {
                if (Build.VERSION.SDK_INT >= 29) AudioFormat.ENCODING_DOLBY_MAT else null
            }
            mime.contains("ac4") -> if (Build.VERSION.SDK_INT >= 29) AudioFormat.ENCODING_AC4 else null
            descriptor.contains("dts-hd-ma") || descriptor.contains("dtshd-ma") -> {
                if (Build.VERSION.SDK_INT >= 34) AudioFormat.ENCODING_DTS_HD_MA else AudioFormat.ENCODING_DTS_HD
            }
            descriptor.contains("dts-uhd-p1") || descriptor.contains("dtsuhdp1") -> {
                if (Build.VERSION.SDK_INT >= 34) AudioFormat.ENCODING_DTS_UHD_P1 else null
            }
            descriptor.contains("dts-uhd-p2") || descriptor.contains("dtsuhdp2") -> {
                if (Build.VERSION.SDK_INT >= 34) AudioFormat.ENCODING_DTS_UHD_P2 else null
            }
            mime.contains("dts-hd") || mime.contains("dtshd") -> AudioFormat.ENCODING_DTS_HD
            mime.contains("dra") && Build.VERSION.SDK_INT >= 31 -> AudioFormat.ENCODING_DRA
            mime.contains("dts-uhd") -> null
            mime.contains("dts") -> AudioFormat.ENCODING_DTS
            descriptor.contains("mhm1-03") || descriptor.contains("mha1-03") ->
                AudioFormat.ENCODING_MPEGH_BL_L3
            descriptor.contains("mhm1-04") || descriptor.contains("mha1-04") ->
                AudioFormat.ENCODING_MPEGH_BL_L4
            descriptor.contains("mhm1-0d") || descriptor.contains("mha1-0d") ->
                AudioFormat.ENCODING_MPEGH_LC_L3
            descriptor.contains("mhm1-0e") || descriptor.contains("mha1-0e") ->
                AudioFormat.ENCODING_MPEGH_LC_L4
            // MPEG-H profile/level is carried in the format details. Let
            // Media3 make the exact direct-playback decision instead of
            // guessing one Android profile here.
            else -> null
        }
    }

    private fun channelMaskForCount(channelCount: Int): Int? = when (channelCount) {
        1 -> AudioFormat.CHANNEL_OUT_MONO
        2 -> AudioFormat.CHANNEL_OUT_STEREO
        3 -> AudioFormat.CHANNEL_OUT_SURROUND
        4 -> AudioFormat.CHANNEL_OUT_QUAD
        6 -> AudioFormat.CHANNEL_OUT_5POINT1
        7 -> AudioFormat.CHANNEL_OUT_6POINT1
        8 -> AudioFormat.CHANNEL_OUT_7POINT1
        else -> null
    }

}
