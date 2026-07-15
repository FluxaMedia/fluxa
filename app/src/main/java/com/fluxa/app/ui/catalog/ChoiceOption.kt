package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import java.util.Locale

internal data class ChoiceOption(val value: String, val label: String)

internal fun languageDisplayName(language: String, lang: String): String = when (language.lowercase()) {
    "none", "", "__off__" -> AppStrings.t(lang, "settings.none")
    "forced" -> AppStrings.t(lang, "settings.forced")
    "original" -> AppStrings.t(lang, "settings.original")
    "device_language" -> AppStrings.t(lang, "settings.device_language")
    "tr", "tr-tr" -> Locale.forLanguageTag("tr").getDisplayLanguage(Locale.forLanguageTag("tr"))
    "en", "en-us" -> "English"
    else -> Locale.forLanguageTag(language).getDisplayLanguage(Locale.forLanguageTag(language)).takeIf { it.isNotBlank() } ?: language
}
