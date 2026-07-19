package com.fluxa.app.player

import androidx.media3.common.C
import androidx.media3.common.ColorInfo
import androidx.media3.common.Format
import androidx.media3.common.MimeTypes
import com.fluxa.app.data.remote.Stream
import java.net.URI
import java.util.Locale

internal object MediaProbeClassifier {
    fun sourceFor(url: String?, stream: Stream?): ProbeValue {
        val lowerUrl = url.orEmpty().lowercase(Locale.ROOT)
        val source = when {
            stream?.infoHash?.isNotBlank() == true -> "torrent"
            lowerUrl.startsWith("magnet:") || lowerUrl.endsWith(".torrent") || lowerUrl.contains("/stream/fname?link=magnet") -> "torrent"
            stream?.isDebrid == true || lowerUrl.contains("real-debrid") || lowerUrl.contains("alldebrid") || lowerUrl.contains("premiumize") -> "debrid"
            lowerUrl.startsWith("smb://") -> "smb"
            lowerUrl.startsWith("webdav://") || lowerUrl.contains("/webdav/") -> "webdav"
            lowerUrl.endsWith(".m3u8") || lowerUrl.contains(".m3u8?") -> "hls"
            lowerUrl.endsWith(".mpd") || lowerUrl.contains(".mpd?") -> "dash"
            lowerUrl.startsWith("content:") || lowerUrl.startsWith("file:") -> "local"
            lowerUrl.startsWith("http://") || lowerUrl.startsWith("https://") -> "http"
            else -> null
        }
        return ProbeValue.inferred(source)
    }

    fun containerFromUrl(url: String?, stream: Stream?): ProbeValue {
        val filename = stream?.effectiveFilename ?: url.orEmpty().substringBefore('?').substringAfterLast('/')
        val ext = filename.substringAfterLast('.', missingDelimiterValue = "").lowercase(Locale.ROOT)
        val container = when (ext) {
            "mkv", "mk3d", "mka", "mks" -> "mkv"
            "mp4", "m4v", "m4a", "mov", "qt" -> "mp4/mov"
            "ts", "m2ts", "mts" -> "mpeg-ts"
            "avi" -> "avi"
            "flv" -> "flv"
            "webm" -> "webm"
            "iso" -> "iso"
            "bdmv" -> "bdmv"
            "m3u8" -> "hls"
            "mpd" -> "dash"
            else -> null
        }
        return ProbeValue.inferred(container)
    }

    fun codecFromMimeOrCodec(sampleMimeType: String?, codecs: String?): ProbeValue {
        val raw = codecs?.takeIf { it.isNotBlank() } ?: sampleMimeType.orEmpty()
        val lower = raw.lowercase(Locale.ROOT)
        val codec = when {
            lower.contains("av01") || lower.contains("av1") -> "av1"
            lower.contains("hev1") || lower.contains("hvc1") || lower.contains("hevc") || lower.contains("h265") -> "hevc"
            lower.contains("avc1") || lower.contains("avc3") || lower.contains("h264") -> "h264"
            lower.contains("vp09") || lower.contains("vp9") -> "vp9"
            lower.contains("vp08") || lower.contains("vp8") -> "vp8"
            lower.contains("mpeg2") || lower.contains("mp2v") -> "mpeg2"
            lower.contains("vc1") || lower.contains("wvc1") -> "vc1"
            lower.contains("xvid") -> "xvid"
            lower.contains("prores") -> "prores"
            lower.contains("dolby-vision") || lower.contains("dvh") || lower.contains("dvhe") || lower.contains("dva1") || lower.contains("dvav") -> "dolby-vision"
            lower.contains("true-hd") || lower.contains("truehd") -> "truehd"
            lower.contains("eac3") || lower.contains("ec-3") -> "eac3"
            lower.contains("ac3") || lower.contains("ac-3") -> "ac3"
            lower.contains("aac") || lower.contains("mp4a") -> "aac"
            lower.contains("dts") -> "dts"
            lower.contains("flac") -> "flac"
            lower.contains("opus") -> "opus"
            lower.contains("vorbis") -> "vorbis"
            lower.contains("pcm") || lower.contains("raw") -> "pcm"
            lower.contains("subrip") || lower.contains("srt") -> "srt"
            lower.contains("ssa") || lower.contains("ass") -> "ass/ssa"
            lower.contains("pgs") || lower.contains("sup") -> "pgs"
            lower.contains("vobsub") -> "vobsub"
            lower.contains("dvb") -> "dvb"
            lower.contains("tx3g") -> "tx3g"
            raw.isNotBlank() -> raw
            else -> null
        }
        return ProbeValue.inferred(codec)
    }

    fun profileFromCodecs(codecs: String?): ProbeValue {
        val lower = codecs.orEmpty().lowercase(Locale.ROOT)
        dolbyVisionProfileFromCodecs(lower)?.let { return ProbeValue.inferred("dolby vision profile $it") }
        val profile = when {
            lower.contains("hvc1.2") || lower.contains("hev1.2") -> "hevc main10"
            lower.contains("hvc1.1") || lower.contains("hev1.1") -> "hevc main"
            lower.contains("avc1.64") || lower.contains("avc3.64") -> "h264 high"
            lower.contains("avc1.4d") || lower.contains("avc3.4d") -> "h264 main"
            lower.contains("avc1.42") || lower.contains("avc3.42") -> "h264 baseline"
            lower.contains("av01.0") -> "av1 main"
            lower.contains("av01.1") -> "av1 high"
            lower.contains("av01.2") -> "av1 professional"
            else -> null
        }
        return ProbeValue.inferred(profile)
    }

    fun levelFromCodecs(codecs: String?): ProbeValue {
        val lower = codecs.orEmpty().lowercase(Locale.ROOT)
        val hevc = Regex("\\.l(\\d+)").find(lower)?.groupValues?.getOrNull(1)
        if (hevc != null) return ProbeValue.inferred("L$hevc")
        val avcHex = Regex("avc[13]\\.([0-9a-f]{6})").find(lower)?.groupValues?.getOrNull(1)
        val avcLevel = avcHex?.takeLast(2)?.toIntOrNull(16)
        if (avcLevel != null) return ProbeValue.inferred("L${avcLevel / 10}.${avcLevel % 10}")
        val av1Level = Regex("av01\\.[012]\\.(\\d{2})").find(lower)?.groupValues?.getOrNull(1)
        return av1Level?.let { ProbeValue.inferred("L$it") } ?: ProbeValue.unknown()
    }

    fun hdrFrom(format: Format?): ProbeValue {
        val colorInfo = format?.colorInfo ?: return ProbeValue.unknown()
        return hdrFrom(colorInfo, format.codecs, format.sampleMimeType)
    }

    fun hdrFrom(colorInfo: ColorInfo?, codecs: String?, mime: String?): ProbeValue {
        val lower = "${codecs.orEmpty()} ${mime.orEmpty()}".lowercase(Locale.ROOT)
        val hdr = when {
            lower.contains("dvh") || lower.contains("dvhe") || lower.contains("dva1") || lower.contains("dvav") -> "Dolby Vision"
            colorInfo?.colorTransfer == C.COLOR_TRANSFER_ST2084 -> "HDR10/PQ"
            colorInfo?.colorTransfer == C.COLOR_TRANSFER_HLG -> "HLG"
            colorInfo?.colorTransfer == C.COLOR_TRANSFER_SDR -> "SDR"
            lower.contains("hdr10+") -> "HDR10+"
            lower.contains("hdr") -> "HDR"
            else -> null
        }
        return if (colorInfo != null) ProbeValue.verified(hdr) else ProbeValue.inferred(hdr)
    }

    fun dolbyVisionFromCodecs(codecs: String?): ProbeValue {
        return ProbeValue.inferred(dolbyVisionProfileFromCodecs(codecs.orEmpty().lowercase(Locale.ROOT)))
    }

    private fun dolbyVisionProfileFromCodecs(codecs: String): String? {
        val match = Regex("(?:dvhe|dvh1|dva1|dvav)\\.(\\d{2})(?:\\.(\\d{2}))?")
            .find(codecs)
            ?: return null
        val profile = match.groupValues.getOrNull(1)?.toIntOrNull() ?: return null
        val compatibility = match.groupValues.getOrNull(2)?.takeIf { it.isNotBlank() }?.toIntOrNull()
        return when (profile) {
            5 -> "5"
            7 -> "7"
            8 -> when (compatibility) {
                4 -> "8.4"
                1 -> "8.1"
                null -> "8"
                else -> "8.$compatibility"
            }
            else -> profile.toString()
        }
    }

    fun pixelFormatFrom(codecs: String?, mimeType: String?, colorInfo: ColorInfo?): ProbeValue {
        val is10bit = colorInfo?.colorTransfer?.let { it == C.COLOR_TRANSFER_ST2084 || it == C.COLOR_TRANSFER_HLG } == true
            || codecs.orEmpty().lowercase(Locale.ROOT).let { it.contains("hvc1.2") || it.contains("hev1.2") || it.contains(".10.") }
        val lower = "${codecs.orEmpty()} ${mimeType.orEmpty()}".lowercase(Locale.ROOT)
        val fmt = when {
            lower.contains("av01") || lower.contains("av1") -> if (is10bit) "yuv420p10le" else "yuv420p"
            lower.contains("hev1") || lower.contains("hvc1") || lower.contains("hevc") -> if (is10bit) "yuv420p10le" else "yuv420p"
            lower.contains("avc1") || lower.contains("avc3") || lower.contains("h264") -> "yuv420p"
            lower.contains("vp09") || lower.contains("vp9") -> if (is10bit) "yuv420p10le" else "yuv420p"
            lower.contains("vp08") || lower.contains("vp8") -> "yuv420p"
            lower.contains("dvh") || lower.contains("dvhe") -> if (is10bit) "yuv420p10le" else "yuv420p"
            else -> null
        }
        return if (fmt != null) ProbeValue.inferred(fmt) else ProbeValue.unknown()
    }

    fun subtitleCodecFromUrl(url: String): ProbeValue {
        val path = url.substringBefore('?').lowercase(Locale.ROOT)
        val codec = when {
            path.endsWith(".srt") -> "srt"
            path.endsWith(".ass") -> "ass"
            path.endsWith(".ssa") -> "ssa"
            path.endsWith(".vtt") || path.endsWith(".webvtt") -> "webvtt"
            path.endsWith(".sub") || path.endsWith(".idx") -> "vobsub"
            path.endsWith(".sup") -> "pgs"
            else -> null
        }
        return ProbeValue.inferred(codec)
    }

    fun containerFromMime(mimeType: String?): ProbeValue {
        val container = when (mimeType) {
            MimeTypes.APPLICATION_MATROSKA -> "mkv"
            MimeTypes.APPLICATION_MP4, MimeTypes.VIDEO_MP4 -> "mp4"
            MimeTypes.APPLICATION_M3U8 -> "hls"
            MimeTypes.APPLICATION_MPD -> "dash"
            MimeTypes.VIDEO_WEBM, MimeTypes.AUDIO_WEBM -> "webm"
            MimeTypes.VIDEO_MP2T -> "mpeg-ts"
            else -> null
        }
        return ProbeValue.verified(container)
    }

    fun hostFor(url: String?): String? = runCatching {
        URI(url.orEmpty()).host?.takeIf { it.isNotBlank() }
    }.getOrNull()
}

internal object MediaSourceInfoBuilder {
    fun build(
        url: String?,
        stream: Stream?,
        externalSubtitles: List<ExternalSubtitleTrack> = emptyList()
    ): String {
        val lines = mutableListOf<String>()
        lines += "file_data"
        lines += "Source: ${MediaProbeClassifier.sourceFor(url, stream).format()}"
        lines += "Container: ${MediaProbeClassifier.containerFromUrl(url, stream).format()}"
        lines += "URL Scheme: ${url.schemeOrUnknown()}"
        lines += "URL Host: ${MediaProbeClassifier.hostFor(url) ?: "unknown"}"
        lines += "URL Ext: ${url.extensionOrUnknown()}"
        lines += "Filename: ${stream?.filename ?: "unknown"}"
        lines += "Effective Filename: ${stream?.effectiveFilename ?: url.filenameOrUnknown()}"
        lines += "Video Size: ${stream?.effectiveVideoSize?.formatBytes() ?: stream?.videoSize?.formatBytes() ?: "unknown"}"
        lines += "Video Hash: ${stream?.effectiveVideoHash ?: stream?.videoHash ?: "unknown"}"
        stream?.let { value ->
            lines += "Addon: ${value.addonName ?: "unknown"}"
            lines += "Name: ${value.name ?: "unknown"}"
            lines += "Title: ${value.title ?: "unknown"}"
            lines += "Description: ${value.description ?: "unknown"}"
            lines += "URL Present: ${!value.url.isNullOrBlank()}"
            lines += "External URL Present: ${!value.externalUrl.isNullOrBlank()}"
            lines += "Info Hash: ${value.infoHash ?: "unknown"}"
            lines += "File Index: ${value.fileIdx?.toString() ?: "unknown"}"
            lines += "Debrid: ${value.isDebrid}"
            lines += "Sources: ${value.sources?.joinToString(", ") ?: "none"}"
            lines += "Behavior Hints: ${value.behaviorHints?.keys?.joinToString(", ") ?: "none"}"
            val headerKeys = value.resolveHeaders().keys.joinToString(", ").ifBlank { "none" }
            lines += "Request Headers: $headerKeys"
            value.subtitles.orEmpty().forEachIndexed { index, subtitle ->
                lines += "Stream Subtitle[${index + 1}]: ${subtitle.lang ?: "unknown"} / ${subtitle.url.extensionOrUnknown()}"
            }
        }
        externalSubtitles.forEachIndexed { index, subtitle ->
            lines += "External Subtitle[${index + 1}]: ${subtitle.language ?: "unknown"} / ${subtitle.label ?: "unknown"} / ${MediaProbeClassifier.subtitleCodecFromUrl(subtitle.url).format()}"
        }
        return lines.joinToString("\n")
    }

    private fun String?.schemeOrUnknown(): String {
        return runCatching { URI(orEmpty()).scheme?.takeIf { it.isNotBlank() } }.getOrNull() ?: "unknown"
    }

    private fun String?.filenameOrUnknown(): String {
        return orEmpty().substringBefore('?').substringAfterLast('/').takeIf { it.isNotBlank() } ?: "unknown"
    }

    private fun String?.extensionOrUnknown(): String {
        val path = orEmpty().substringBefore('?').substringBefore('#')
        return path.substringAfterLast('.', missingDelimiterValue = "").takeIf { it.isNotBlank() }?.lowercase(Locale.ROOT) ?: "unknown"
    }

    private fun Long.formatBytes(): String {
        return when {
            this >= 1024L * 1024L * 1024L -> String.format(Locale.US, "%.2f GiB (%d bytes)", this / 1024.0 / 1024.0 / 1024.0, this)
            this >= 1024L * 1024L -> String.format(Locale.US, "%.2f MiB (%d bytes)", this / 1024.0 / 1024.0, this)
            this >= 1024L -> String.format(Locale.US, "%.2f KiB (%d bytes)", this / 1024.0, this)
            else -> "$this bytes"
        }
    }
}

internal fun Long.bytesPerSecondToBitrateLabel(): String = "${(this * 8L) / 1000L} kbps"

internal fun Int.bitrateLabel(): String = takeIf { it > 0 }?.let { "${it / 1000} kbps" } ?: "unknown"
