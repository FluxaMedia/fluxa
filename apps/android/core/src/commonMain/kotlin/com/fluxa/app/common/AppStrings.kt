package com.fluxa.app.common

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

internal expect fun readI18nAssetText(fileName: String): String?
internal expect fun readPlatformString(language: String?, key: String): String?
internal expect fun createStringsCache(): MutableMap<String, AppStrings>

class AppStrings private constructor(
    private val language: String?,
    private val values: Map<String, String>,
    private val fallback: Map<String, String>
) {
    fun get(key: String): String = readPlatformString(language, key) ?: values[key] ?: fallback[key] ?: key

    companion object {
        private val cache = createStringsCache()

        fun t(language: String?, key: String): String {
            return load(language).get(key)
        }

        fun format(language: String?, key: String, vararg args: Any?): String {
            return args.fold(t(language, key)) { value, arg ->
                value.replaceFirst("%s", arg?.toString().orEmpty())
            }
        }

        fun list(language: String?, key: String): List<String> {
            return t(language, key).split("|").map { it.trim() }.filter { it.isNotEmpty() }
        }

        fun runtimeMinutes(language: String?, minutes: Int): String {
            val hours = minutes / 60
            val remainder = minutes % 60
            return if (hours > 0) {
                format(language, "format.runtime_hours", hours, remainder)
            } else {
                format(language, "format.runtime_minutes", minutes)
            }
        }

        fun englishArtworkFallback(language: String?, fallbackUrl: String?): String? {
            return fallbackUrl
        }

        fun allowsEnglishImageFallback(language: String?): Boolean {
            return true
        }

        private fun load(language: String?): AppStrings {
            val fileName = languageFileName(language)
            return cache.getOrPut(fileName) {
                val fallback = readAsset("english_us.json")
                val values = if (fileName == "english_us.json") fallback else readAsset(fileName)
                AppStrings(language, values, fallback)
            }
        }

        private fun languageFileName(language: String?): String {
            val normalized = language
                ?.trim()
                .orEmpty()
            val languageCode = normalized.removeSuffix(".json")
            return when (languageCode.lowercase()) {
                "" -> "english_us.json"
                "en", "en-us", "english_us" -> "english_us.json"
                "tr", "tr-tr", "tr_tr" -> "tr_tr.json"
                else -> if (normalized.endsWith(".json")) normalized else "$normalized.json"
            }
        }

        private fun readAsset(fileName: String): Map<String, String> {
            val json = readI18nAssetText(fileName) ?: return emptyMap()
            return runCatching {
                Json.parseToJsonElement(json).jsonObject
                    .entries
                    .associate { (key, value) -> key to (value.jsonPrimitive.contentOrNull ?: key) }
            }.getOrDefault(emptyMap())
        }
    }
}
