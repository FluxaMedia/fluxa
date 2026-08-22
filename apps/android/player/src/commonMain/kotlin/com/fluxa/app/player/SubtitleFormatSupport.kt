package com.fluxa.app.player

object SubtitleFormatSupport {
    const val WEB_VTT: String = "text/vtt"
    const val SSA: String = "text/x-ssa"
    const val TTML: String = "application/ttml+xml"
    const val SUBRIP: String = "application/x-subrip"

    fun mimeTypeForUrl(url: String): String {
        val path = url.substringBefore('?').substringBefore('#').lowercase()
        return when {
            path.endsWith(".vtt") || path.endsWith(".webvtt") -> WEB_VTT
            path.endsWith(".ass") || path.endsWith(".ssa") -> SSA
            path.endsWith(".ttml") || path.endsWith(".xml") -> TTML
            else -> SUBRIP
        }
    }
}
