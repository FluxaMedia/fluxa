package com.fluxa.app.player

import android.content.Context
import androidx.media3.common.C
import androidx.media3.common.Format
import androidx.media3.common.Tracks
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.decoder.ffmpeg.FfmpegLibrary
import com.fluxa.app.data.remote.Stream
import java.util.Locale
import kotlin.math.log10

internal object MediaTechnicalInfoBuilder {
    fun build(
        context: Context,
        exoPlayer: ExoPlayer,
        extraDroppedFrames: Int? = null,
        url: String? = null,
        stream: Stream? = null,
        externalSubtitles: List<ExternalSubtitleTrack> = emptyList(),
        audioDecoderMode: String = "hw_prefer"
    ): String {
        val report = buildReport(exoPlayer, url, stream, externalSubtitles)
        val selectedVideoFormat = exoPlayer.currentTracks.groups
            .firstOrNull { it.type == C.TRACK_TYPE_VIDEO && (0 until it.length).any { index -> it.isTrackSelected(index) } }
            ?.let { group -> (0 until group.length).firstOrNull { group.isTrackSelected(it) }?.let(group::getTrackFormat) }
        val videoFormat = exoPlayer.videoFormat ?: selectedVideoFormat
        val audioFormat = exoPlayer.audioFormat
        val bufferHealth = "${exoPlayer.bufferedPercentage}% / ${exoPlayer.bufferedPosition - exoPlayer.currentPosition}ms"
        val color = colorLine(videoFormat)
        val audioManager = context.getSystemService(Context.AUDIO_SERVICE) as android.media.AudioManager
        val maxDeviceVolume = audioManager.getStreamMaxVolume(android.media.AudioManager.STREAM_MUSIC).coerceAtLeast(1)
        val deviceVolume = audioManager.getStreamVolume(android.media.AudioManager.STREAM_MUSIC)
        val playerGainDb = if (exoPlayer.volume > 0f) String.format(Locale.US, "%.1f dB", 20f * log10(exoPlayer.volume)) else "-inf dB"
        val volume = "player ${(exoPlayer.volume * 100).toInt()}% ($playerGainDb) / device $deviceVolume/$maxDeviceVolume (${(deviceVolume * 100) / maxDeviceVolume}%) / session=${exoPlayer.audioSessionId}"
        val dropped = extraDroppedFrames?.toString() ?: "unknown"
        val ffmpeg = if (FfmpegLibrary.isAvailable()) {
            "available ${FfmpegLibrary.getVersion()}"
        } else {
            "unavailable"
        }
        return listOf(
            MediaSourceInfoBuilder.build(url, stream, externalSubtitles),
            "player_data",
            report.format(),
            "audio_decoder=$audioDecoderMode",
            "ffmpeg=$ffmpeg",
            "buffer=$bufferHealth",
            color,
            "volume=$volume",
            "dropped=$dropped"
        ).joinToString("\n")
    }

    private fun buildReport(
        exoPlayer: ExoPlayer,
        url: String?,
        stream: Stream?,
        externalSubtitles: List<ExternalSubtitleTrack>
    ): MediaProbeReport {
        val videoTracks = mutableListOf<VideoProbeTrack>()
        val audioTracks = mutableListOf<AudioProbeTrack>()
        val subtitleTracks = mutableListOf<SubtitleProbeTrack>()
        val liveVideoFormat = exoPlayer.videoFormat
        val liveAudioFormat = exoPlayer.audioFormat
        var selectedVideoFormat: Format? = liveVideoFormat
        var selectedContainer = ProbeValue.unknown()
        exoPlayer.currentTracks.groups.forEach { group ->
            for (index in 0 until group.length) {
                val format = group.getTrackFormat(index)
                val selected = group.isTrackSelected(index)
                if (selected && selectedVideoFormat == null && group.type == C.TRACK_TYPE_VIDEO) {
                    selectedVideoFormat = format
                }
                if (selected && selectedContainer.value == null) {
                    selectedContainer = MediaProbeClassifier.containerFromMime(format.containerMimeType)
                }
                when (group.type) {
                    C.TRACK_TYPE_VIDEO -> videoTracks += videoTrack(format, selected, if (selected) liveVideoFormat else null)
                    C.TRACK_TYPE_AUDIO -> audioTracks += audioTrack(format, selected, if (selected) liveAudioFormat else null)
                    C.TRACK_TYPE_TEXT -> subtitleTracks += subtitleTrack(format, external = false, selected = selected)
                }
            }
        }

        val activeVideo = selectedVideoFormat ?: liveVideoFormat
        if (videoTracks.isEmpty() && activeVideo != null) {
            videoTracks += videoTrack(activeVideo, selected = true)
        }
        exoPlayer.audioFormat?.let { format ->
            if (audioTracks.none { it.selected }) audioTracks += audioTrack(format, selected = true)
        }
        externalSubtitles.forEach { subtitle ->
            subtitleTracks += SubtitleProbeTrack(
                codec = MediaProbeClassifier.subtitleCodecFromUrl(subtitle.url),
                language = subtitle.language,
                title = subtitle.label,
                external = true,
                selected = false
            )
        }

        val window = androidx.media3.common.Timeline.Window()
        val seekable = if (exoPlayer.currentTimeline.isEmpty) {
            ProbeValue.unknown()
        } else {
            exoPlayer.currentTimeline.getWindow(exoPlayer.currentMediaItemIndex, window)
            ProbeValue.verified(window.isSeekable.toString())
        }
        val container = selectedContainer.takeIf { it.value != null }
            ?: MediaProbeClassifier.containerFromUrl(url, stream)
        val hdr = MediaProbeClassifier.hdrFrom(activeVideo)
        val dv = MediaProbeClassifier.dolbyVisionFromCodecs(activeVideo?.codecs)
        val quirks = buildList {
            if (seekable.value == "false") add(ProbeValue.verified("non_seekable"))
            val selectedFps = activeVideo?.frameRate?.takeIf { it > 0f }
            val groupFps = videoTracks.mapNotNull { it.fps.value?.toFloatOrNull() }.distinct()
            if (selectedFps != null && groupFps.size > 1) add(ProbeValue.inferred("multiple_video_frame_rates"))
        }

        return MediaProbeReport(
            engine = "ExoPlayer",
            source = MediaProbeClassifier.sourceFor(url, stream),
            container = container,
            durationMs = exoPlayer.duration.takeIf { it > 0 },
            bitrate = activeVideo?.bitrate?.takeIf { it > 0 }?.let { ProbeValue.verified(it.bitrateLabel()) } ?: ProbeValue.unknown(),
            seekable = seekable,
            video = videoTracks,
            audio = audioTracks,
            subtitles = subtitleTracks,
            hdr = hdr,
            dolbyVision = dv,
            quirks = quirks
        )
    }

    private fun videoTrack(format: Format, selected: Boolean, liveFormat: Format? = null): VideoProbeTrack {
        val src = liveFormat ?: format
        val width = (src.width.takeIf { it > 0 } ?: format.width.takeIf { it > 0 })
        val height = (src.height.takeIf { it > 0 } ?: format.height.takeIf { it > 0 })
        val codecsStr = format.codecs?.takeIf { it.isNotBlank() } ?: src.codecs
        val mimeType = format.sampleMimeType ?: src.sampleMimeType
        val fps = src.frameRate.takeIf { it > 0f } ?: format.frameRate.takeIf { it > 0f }
        val bitrate = src.bitrate.takeIf { it > 0 } ?: format.bitrate.takeIf { it > 0 }
        val colorInfo = src.colorInfo ?: format.colorInfo
        return VideoProbeTrack(
            codec = MediaProbeClassifier.codecFromMimeOrCodec(mimeType, codecsStr),
            profile = MediaProbeClassifier.profileFromCodecs(codecsStr),
            level = MediaProbeClassifier.levelFromCodecs(codecsStr),
            pixelFormat = MediaProbeClassifier.pixelFormatFrom(codecsStr, mimeType, colorInfo),
            resolution = if (width != null && height != null) ProbeValue.verified("${width}x$height") else ProbeValue.unknown(),
            fps = fps?.let { ProbeValue.verified(String.format(Locale.US, "%.2f", it)) } ?: ProbeValue.unknown(),
            bitrate = bitrate?.let { ProbeValue.verified(it.bitrateLabel()) } ?: ProbeValue.unknown(),
            language = format.language,
            selected = selected
        )
    }

    private fun audioTrack(format: Format, selected: Boolean, liveFormat: Format? = null): AudioProbeTrack {
        val src = liveFormat ?: format
        val channels = src.channelCount.takeIf { it > 0 } ?: format.channelCount.takeIf { it > 0 }
        val sampleRate = src.sampleRate.takeIf { it > 0 } ?: format.sampleRate.takeIf { it > 0 }
        val bitrate = src.bitrate.takeIf { it > 0 } ?: format.bitrate.takeIf { it > 0 }
        return AudioProbeTrack(
            codec = MediaProbeClassifier.codecFromMimeOrCodec(
                format.sampleMimeType ?: src.sampleMimeType,
                format.codecs?.takeIf { it.isNotBlank() } ?: src.codecs
            ),
            channels = channels?.let { ProbeValue.verified("${it}ch") } ?: ProbeValue.unknown(),
            sampleRate = sampleRate?.let { ProbeValue.verified("${it}Hz") } ?: ProbeValue.unknown(),
            bitrate = bitrate?.let { ProbeValue.verified(it.bitrateLabel()) } ?: ProbeValue.unknown(),
            language = format.language,
            title = format.label,
            selected = selected
        )
    }

    private fun subtitleTrack(format: Format, external: Boolean, selected: Boolean): SubtitleProbeTrack {
        return SubtitleProbeTrack(
            codec = MediaProbeClassifier.codecFromMimeOrCodec(format.sampleMimeType, format.codecs),
            language = format.language,
            title = format.label,
            external = external,
            selected = selected
        )
    }

    private fun colorLine(videoFormat: Format?): String {
        val colorInfo = videoFormat?.colorInfo ?: return "color=unknown"
        val hdr = MediaProbeClassifier.hdrFrom(colorInfo, videoFormat.codecs, videoFormat.sampleMimeType).format()
        return "color=${colorSpaceLabel(colorInfo.colorSpace)} / ${colorRangeLabel(colorInfo.colorRange)} / ${colorTransferLabel(colorInfo.colorTransfer)} / $hdr"
    }

    private fun colorSpaceLabel(value: Int): String = when (value) {
        C.COLOR_SPACE_BT709 -> "BT.709"
        C.COLOR_SPACE_BT601 -> "BT.601"
        C.COLOR_SPACE_BT2020 -> "BT.2020"
        else -> "unknown($value)"
    }

    private fun colorRangeLabel(value: Int): String = when (value) {
        C.COLOR_RANGE_FULL -> "full"
        C.COLOR_RANGE_LIMITED -> "limited"
        else -> "unknown($value)"
    }

    private fun colorTransferLabel(value: Int): String = when (value) {
        C.COLOR_TRANSFER_SDR -> "SDR"
        C.COLOR_TRANSFER_ST2084 -> "PQ/ST2084"
        C.COLOR_TRANSFER_HLG -> "HLG"
        else -> "unknown($value)"
    }
}
