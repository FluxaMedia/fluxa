package com.fluxa.app.player

import androidx.media3.common.MimeTypes
import java.util.Locale

object SubtitleFormatSupport {
    fun mimeTypeForUrl(url: String): String {
        val path = url.substringBefore('?').substringBefore('#').lowercase(Locale.ROOT)
        return when {
            path.endsWith(".vtt") || path.endsWith(".webvtt") -> MimeTypes.TEXT_VTT
            path.endsWith(".ass") || path.endsWith(".ssa") -> MimeTypes.TEXT_SSA
            path.endsWith(".ttml") || path.endsWith(".xml") -> MimeTypes.APPLICATION_TTML
            else -> MimeTypes.APPLICATION_SUBRIP
        }
    }
}
