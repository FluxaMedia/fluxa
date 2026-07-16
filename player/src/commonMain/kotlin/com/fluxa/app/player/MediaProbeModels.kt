package com.fluxa.app.player

data class MediaProbeReport(
    val engine: String,
    val source: ProbeValue = ProbeValue.unknown(),
    val container: ProbeValue = ProbeValue.unknown(),
    val durationMs: Long? = null,
    val bitrate: ProbeValue = ProbeValue.unknown(),
    val seekable: ProbeValue = ProbeValue.unknown(),
    val video: List<VideoProbeTrack> = emptyList(),
    val audio: List<AudioProbeTrack> = emptyList(),
    val subtitles: List<SubtitleProbeTrack> = emptyList(),
    val hdr: ProbeValue = ProbeValue.unknown(),
    val dolbyVision: ProbeValue = ProbeValue.unknown(),
    val quirks: List<ProbeValue> = emptyList()
) {
    fun format(): String {
        val lines = mutableListOf(
            "Engine: $engine",
            "Source: ${source.format()}",
            "Container: ${container.format()}"
        )
        durationMs?.let { lines += "Duration: ${formatDuration(it)}" }
        lines += "Bitrate: ${bitrate.format()}"
        lines += "Seekable: ${seekable.format()}"
        lines += "HDR: ${hdr.format()}"
        lines += "Dolby Vision: ${dolbyVision.format()}"
        lines += if (video.isEmpty()) listOf("Video: unknown") else video.mapIndexed { index, track ->
            val label = if (video.size == 1) "Video" else "Video[${index + 1}]"
            val active = if (track.selected && video.size > 1) " (active)" else ""
            "$label$active: ${track.format()}"
        }
        lines += if (audio.isEmpty()) listOf("Audio: unknown") else audio.mapIndexed { index, track ->
            val label = if (audio.size == 1) "Audio" else "Audio[${index + 1}]"
            val active = if (track.selected && audio.size > 1) " (active)" else ""
            "$label$active: ${track.format()}"
        }
        lines += if (subtitles.isEmpty()) listOf("Subtitles: none") else subtitles.mapIndexed { index, track ->
            val label = if (subtitles.size == 1) "Subtitle" else "Subtitle[${index + 1}]"
            val active = if (track.selected && subtitles.size > 1) " (active)" else ""
            "$label$active: ${track.format()}"
        }
        if (quirks.isNotEmpty()) lines += "Quirks: ${quirks.joinToString(", ") { it.format() }}"
        return lines.joinToString("\n")
    }

    private fun formatDuration(value: Long): String {
        val totalSeconds = value / 1000L
        val hours = totalSeconds / 3600L
        val minutes = (totalSeconds % 3600L) / 60L
        val seconds = totalSeconds % 60L
        return if (hours > 0) "$hours:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}"
        else "$minutes:${seconds.toString().padStart(2, '0')}"
    }
}

data class ProbeValue(val value: String?, val confidence: ProbeConfidence = ProbeConfidence.Unknown) {
    fun format(): String {
        val text = value?.takeIf { it.isNotBlank() } ?: "unknown"
        return if (confidence == ProbeConfidence.Inferred) "$text (inferred)" else text
    }

    companion object {
        fun verified(value: String?) = ProbeValue(value?.takeIf { it.isNotBlank() }, ProbeConfidence.Verified)
        fun inferred(value: String?) = ProbeValue(value?.takeIf { it.isNotBlank() }, ProbeConfidence.Inferred)
        fun unknown() = ProbeValue(null, ProbeConfidence.Unknown)
    }
}

enum class ProbeConfidence { Verified, Inferred, Unknown }

data class VideoProbeTrack(
    val codec: ProbeValue = ProbeValue.unknown(),
    val profile: ProbeValue = ProbeValue.unknown(),
    val level: ProbeValue = ProbeValue.unknown(),
    val pixelFormat: ProbeValue = ProbeValue.unknown(),
    val resolution: ProbeValue = ProbeValue.unknown(),
    val fps: ProbeValue = ProbeValue.unknown(),
    val bitrate: ProbeValue = ProbeValue.unknown(),
    val language: String? = null,
    val selected: Boolean = false
) {
    fun format() = listOfNotNull(
        codec.value,
        profile.value,
        level.value,
        pixelFormat.value,
        resolution.value,
        fps.value?.let { "$it fps" },
        bitrate.value,
        language?.takeIf { it.isNotBlank() }?.uppercase()
    ).joinToString(" / ")
}

data class AudioProbeTrack(
    val codec: ProbeValue = ProbeValue.unknown(),
    val channels: ProbeValue = ProbeValue.unknown(),
    val sampleRate: ProbeValue = ProbeValue.unknown(),
    val bitrate: ProbeValue = ProbeValue.unknown(),
    val language: String? = null,
    val title: String? = null,
    val selected: Boolean = false
) {
    fun format() = listOfNotNull(
        codec.value,
        channels.value,
        sampleRate.value,
        bitrate.value,
        language?.takeIf { it.isNotBlank() }?.uppercase(),
        title?.takeIf { it.isNotBlank() }
    ).joinToString(" / ")
}

data class SubtitleProbeTrack(
    val codec: ProbeValue = ProbeValue.unknown(),
    val language: String? = null,
    val title: String? = null,
    val external: Boolean = false,
    val selected: Boolean = false
) {
    fun format() = listOfNotNull(
        codec.value,
        language?.takeIf { it.isNotBlank() }?.uppercase(),
        title?.takeIf { it.isNotBlank() },
        if (external) "external" else null
    ).joinToString(" / ")
}
