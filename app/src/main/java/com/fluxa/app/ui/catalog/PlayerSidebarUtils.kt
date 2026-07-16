package com.fluxa.app.ui.catalog

import java.util.Locale

internal fun nativeLanguageName(code: String): String {
    val normalized = code.lowercase(Locale.ROOT)
    val locale = Locale.forLanguageTag(normalized)
    val native = locale.getDisplayLanguage(locale).trim()
    return native.takeIf { it.isNotBlank() }?.replaceFirstChar { it.titlecase(locale) } ?: code
}
