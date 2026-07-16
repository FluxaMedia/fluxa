package com.fluxa.app.data.repository

object TmdbLanguageResolver {
    fun languageTag(language: String?): String {
        val normalized = normalizedLanguage(language)
        val parts = normalized.split('_').filter { it.isNotBlank() }
        val languageCode = languageCode(language)
        val explicitCountry = parts.getOrNull(1)?.takeIf { it.length == 2 }?.uppercase()
        return if (explicitCountry != null) "$languageCode-$explicitCountry" else languageCode
    }

    fun languageCode(language: String?): String {
        val parts = normalizedLanguage(language).split('_').filter { it.isNotBlank() }
        return when (parts.firstOrNull()) {
            "english" -> "en"
            null, "" -> "en"
            else -> parts.first().take(2)
        }
    }

    private fun normalizedLanguage(language: String?): String = language
        ?.trim()
        ?.lowercase()
        ?.removeSuffix(".json")
        ?.replace('-', '_')
        .orEmpty()
}
