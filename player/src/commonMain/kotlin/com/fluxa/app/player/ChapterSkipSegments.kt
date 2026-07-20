package com.fluxa.app.player

import com.fluxa.app.shared.feature.player.Chapter

import com.fluxa.app.data.remote.IntroTimestamps

private fun classifyChapterSkipType(title: String): String? {
    val normalized = title.trim().lowercase()
    when (normalized) {
        "op", "opening", "intro", "introduction", "op sequence", "mixed-intro",
        "opening sequence", "opening theme" -> return "intro"
        "ed", "ending", "outro", "credits", "end credits", "closing",
        "ending theme", "ending sequence" -> return "outro"
        "recap", "previously", "previously on", "cold open" -> return "recap"
    }
    if (
        normalized.startsWith("op ") || normalized.startsWith("opening ") ||
        normalized.contains("intro") || normalized.contains("opening")
    ) return "intro"
    if (
        normalized.startsWith("ed ") || normalized.startsWith("ending ") ||
        normalized.contains("ending") || normalized.contains("outro") || normalized.contains("credits")
    ) return "outro"
    if (normalized.contains("recap") || normalized.contains("previously")) return "recap"
    return null
}

fun deriveSkipSegmentsFromChapters(chapters: List<Chapter>): List<IntroTimestamps> {
    return chapters.mapIndexedNotNull { index, chapter ->
        val type = classifyChapterSkipType(chapter.title) ?: return@mapIndexedNotNull null
        val endMs = chapters.getOrNull(index + 1)?.startMs ?: return@mapIndexedNotNull null
        if (endMs <= chapter.startMs) return@mapIndexedNotNull null
        IntroTimestamps(startTime = chapter.startMs, endTime = endMs, type = type)
    }
}
