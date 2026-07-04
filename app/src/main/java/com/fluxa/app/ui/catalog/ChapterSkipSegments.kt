package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.player.Chapter

private fun classifyChapterSkipType(title: String): String? {
    val t = title.trim().lowercase()
    when (t) {
        "op", "opening", "intro", "introduction", "op sequence", "mixed-intro",
        "opening sequence", "opening theme" -> return "intro"
        "ed", "ending", "outro", "credits", "end credits", "closing",
        "ending theme", "ending sequence" -> return "outro"
        "recap", "previously", "previously on", "cold open" -> return "recap"
    }
    if (t.startsWith("op ") || t.startsWith("opening ") || t.contains("intro") || t.contains("opening")) {
        return "intro"
    }
    if (t.startsWith("ed ") || t.startsWith("ending ") || t.contains("ending") ||
        t.contains("outro") || t.contains("credits")
    ) {
        return "outro"
    }
    if (t.contains("recap") || t.contains("previously")) {
        return "recap"
    }
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
