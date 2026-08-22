package com.fluxa.app.player.subtitle

private val TIMECODE = Regex("""(?:(\d+):)?(\d{1,2}):(\d{2})[.,](\d{1,3})""")
private val TAG = Regex("<[^>]*>")

internal fun parseTimecodeUs(raw: String): Long? {
    val match = TIMECODE.find(raw) ?: return null
    val (hours, minutes, seconds, fraction) = match.destructured
    val millis = fraction.padEnd(3, '0').take(3).toLong()
    val h = hours.toLongOrNull() ?: 0L
    val m = minutes.toLong()
    val s = seconds.toLong()
    return ((h * 3600 + m * 60 + s) * 1000 + millis) * 1000
}

internal fun stripMarkup(text: String): String =
    text.replace(TAG, "").trim()
