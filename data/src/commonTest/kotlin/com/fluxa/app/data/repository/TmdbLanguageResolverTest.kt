package com.fluxa.app.data.repository

import kotlin.test.Test
import kotlin.test.assertEquals

class TmdbLanguageResolverTest {
    @Test
    fun createsPortableLanguageTags() {
        assertEquals("en", TmdbLanguageResolver.languageTag(null))
        assertEquals("en-US", TmdbLanguageResolver.languageTag("english_us.json"))
        assertEquals("tr-TR", TmdbLanguageResolver.languageTag("tr-tr"))
    }
}
