package com.fluxa.app.player

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.player.subtitle.SubtitleCueDecoder
import com.fluxa.app.player.subtitle.TextCue

internal object CoreSubtitleCueDecoder : SubtitleCueDecoder {
    override fun decode(content: String): List<TextCue> {
        val cues = runCatching {
            FluxaCoreNative.subtitleSyncCapture(content, 0.0).getAsJsonArray("cues")
        }.getOrNull() ?: return emptyList()
        return cues.mapNotNull { element ->
            val cue = element.asJsonObject
            val start = cue.get("start")?.asDouble ?: return@mapNotNull null
            val end = cue.get("end")?.asDouble ?: return@mapNotNull null
            val text = cue.get("text")?.asString.orEmpty()
            if (end <= start || text.isBlank()) null
            else TextCue((start * 1_000_000.0).toLong(), (end * 1_000_000.0).toLong(), text)
        }
    }
}
