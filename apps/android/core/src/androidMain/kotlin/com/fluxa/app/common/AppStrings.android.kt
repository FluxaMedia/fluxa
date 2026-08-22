package com.fluxa.app.common

import android.content.Context
import android.content.res.Configuration
import android.os.LocaleList
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

internal actual fun readPlatformString(language: String?, key: String): String? {
    val context = appContext ?: return null
    val resourceName = key.replace(".", "_dot_").replace("-", "_dash_")
    val localizedContext = language?.let { requestedLanguage ->
        val normalizedLanguage = requestedLanguage
            .removeSuffix(".json")
            .lowercase()
        val tag = when (normalizedLanguage) {
            "", "en", "en-us", "english_us" -> "en-US"
            "tr", "tr-tr", "tr_tr" -> "tr-TR"
            else -> return@let null
        }
        val configuration = Configuration(context.resources.configuration)
        configuration.setLocales(LocaleList(Locale.forLanguageTag(tag)))
        context.createConfigurationContext(configuration)
    } ?: context
    val resourceId = localizedContext.resources.getIdentifier(
        resourceName,
        "string",
        context.packageName,
    )
    return resourceId.takeIf { it != 0 }?.let { id ->
        runCatching { localizedContext.getString(id) }.getOrNull()
    }
}

internal actual fun createStringsCache(): MutableMap<String, AppStrings> = ConcurrentHashMap()
