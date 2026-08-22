package com.fluxa.app.player.subtitle

import kotlin.test.Test
import kotlin.test.assertEquals

class CueParserTest {
    @Test
    fun srtParsesMultilineCueAndStripsTags() {
        val srt = """
            1
            00:00:01,000 --> 00:00:04,500
            <b>Hello</b>
            there

            2
            00:00:05,250 --> 00:00:07,000
            Second cue
        """.trimIndent()

        val cues = SrtCueParser.parse(srt)

        assertEquals(2, cues.size)
        assertEquals(1_000_000L, cues[0].startUs)
        assertEquals(4_500_000L, cues[0].endUs)
        assertEquals("Hello\nthere", cues[0].text)
        assertEquals(5_250_000L, cues[1].startUs)
    }

    @Test
    fun webVttIgnoresHeaderAndCueSettings() {
        val vtt = """
            WEBVTT

            00:00:01.000 --> 00:00:02.000 align:start position:10%
            Hi
        """.trimIndent()

        val cues = WebVttCueParser.parse(vtt)

        assertEquals(1, cues.size)
        assertEquals(1_000_000L, cues[0].startUs)
        assertEquals(2_000_000L, cues[0].endUs)
        assertEquals("Hi", cues[0].text)
    }

    @Test
    fun ttmlParsesParagraphsAndLineBreaks() {
        val ttml = """
            <tt><body><div>
            <p begin="00:00:01.000" end="00:00:03.000">Line one<br/>Line two</p>
            </div></body></tt>
        """.trimIndent()

        val cues = TtmlCueParser.parse(ttml)

        assertEquals(1, cues.size)
        assertEquals(1_000_000L, cues[0].startUs)
        assertEquals(3_000_000L, cues[0].endUs)
        assertEquals("Line one\nLine two", cues[0].text)
    }

    @Test
    fun ttmlParsesSecondsSuffixTiming() {
        val ttml = """<p begin="1.5s" end="3s">Hi</p>"""

        val cues = TtmlCueParser.parse(ttml)

        assertEquals(1, cues.size)
        assertEquals(1_500_000L, cues[0].startUs)
        assertEquals(3_000_000L, cues[0].endUs)
    }
}
