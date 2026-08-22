package com.fluxa.app.player.subtitle

interface CueParser {
    fun parse(content: String): List<TextCue>
}

fun cueParserFor(format: SubtitleFormat): CueParser = when (format) {
    SubtitleFormat.SRT -> SrtCueParser
    SubtitleFormat.WEBVTT -> WebVttCueParser
    SubtitleFormat.TTML -> TtmlCueParser
}

private val ARROW_LINE = Regex("""^\s*(\S+)\s*-->\s*(\S+)""")

private fun parseTimedBlocks(content: String): List<TextCue> {
    val cues = mutableListOf<TextCue>()
    val blocks = content.replace("\r\n", "\n").split(Regex("\n{2,}"))
    for (block in blocks) {
        val lines = block.split("\n").filter { it.isNotBlank() }
        val arrowIndex = lines.indexOfFirst { ARROW_LINE.containsMatchIn(it) }
        if (arrowIndex < 0) continue
        val match = ARROW_LINE.find(lines[arrowIndex]) ?: continue
        val startUs = parseTimecodeUs(match.groupValues[1]) ?: continue
        val endUs = parseTimecodeUs(match.groupValues[2]) ?: continue
        val text = lines.drop(arrowIndex + 1).joinToString("\n") { stripMarkup(it) }.trim()
        if (text.isEmpty() || endUs <= startUs) continue
        cues += TextCue(startUs, endUs, text)
    }
    return cues.sortedBy { it.startUs }
}

object SrtCueParser : CueParser {
    override fun parse(content: String): List<TextCue> = parseTimedBlocks(content)
}

object WebVttCueParser : CueParser {
    override fun parse(content: String): List<TextCue> = parseTimedBlocks(content)
}

object TtmlCueParser : CueParser {
    private val PARAGRAPH = Regex(
        """<p\b[^>]*\bbegin="([^"]+)"[^>]*\bend="([^"]+)"[^>]*>(.*?)</p>""",
        setOf(RegexOption.DOT_MATCHES_ALL, RegexOption.IGNORE_CASE)
    )
    private val BREAK = Regex("<br\\s*/?>", RegexOption.IGNORE_CASE)

    override fun parse(content: String): List<TextCue> {
        val cues = mutableListOf<TextCue>()
        for (match in PARAGRAPH.findAll(content)) {
            val startUs = parseTtmlTimeUs(match.groupValues[1]) ?: continue
            val endUs = parseTtmlTimeUs(match.groupValues[2]) ?: continue
            val text = stripMarkup(match.groupValues[3].replace(BREAK, "\n")).trim()
            if (text.isEmpty() || endUs <= startUs) continue
            cues += TextCue(startUs, endUs, text)
        }
        return cues.sortedBy { it.startUs }
    }

    private fun parseTtmlTimeUs(raw: String): Long? {
        if (raw.endsWith("s")) {
            return raw.dropLast(1).toDoubleOrNull()?.let { (it * 1_000_000).toLong() }
        }
        return parseTimecodeUs(raw)
    }
}
