package com.fluxa.app.common

import java.util.Locale
import java.util.concurrent.ConcurrentHashMap

fun AppStrings.Companion.locale(language: String?): Locale {
    val tag = language
        ?.substringBefore('_')
        ?.takeIf { it.isNotBlank() }
        ?: return Locale.US
    return Locale.forLanguageTag(tag)
}

internal actual fun readI18nAssetText(fileName: String): String? {
    val resource = object {}.javaClass.getResourceAsStream("/i18n/$fileName") ?: return null
    return resource.bufferedReader().use { it.readText() }
}

internal actual fun createStringsCache(): MutableMap<String, AppStrings> = ConcurrentHashMap()
