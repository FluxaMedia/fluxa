package com.fluxa.app.common

import android.content.Context
import java.util.Locale
import java.util.concurrent.ConcurrentHashMap

private var appContext: Context? = null

fun AppStrings.Companion.initialize(context: Context) {
    appContext = context.applicationContext
}

fun AppStrings.Companion.locale(language: String?): Locale {
    val tag = language
        ?.substringBefore('_')
        ?.takeIf { it.isNotBlank() }
        ?: return Locale.US
    return Locale.forLanguageTag(tag)
}

internal actual fun readI18nAssetText(fileName: String): String? {
    val context = appContext ?: return null
    return runCatching {
        context.assets.open("i18n/$fileName").bufferedReader().use { it.readText() }
    }.getOrNull()
}

internal actual fun createStringsCache(): MutableMap<String, AppStrings> = ConcurrentHashMap()
