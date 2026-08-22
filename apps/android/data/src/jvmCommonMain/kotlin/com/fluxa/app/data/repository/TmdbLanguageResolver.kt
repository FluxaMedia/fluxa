package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreNative

object TmdbLanguageResolver {
    fun languageTag(language: String?): String = FluxaCoreNative.tmdbLanguage(language)
}
