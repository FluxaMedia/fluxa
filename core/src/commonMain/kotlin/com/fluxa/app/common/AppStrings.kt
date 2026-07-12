package com.fluxa.app.common

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

internal expect fun readI18nAssetText(fileName: String): String?
internal expect fun createStringsCache(): MutableMap<String, AppStrings>

class AppStrings private constructor(
    private val values: Map<String, String>,
    private val fallback: Map<String, String>
) {
    fun get(key: String): String = values[key] ?: fallback[key] ?: key

    companion object {
        private val cache = createStringsCache()
        private val RUNTIME_UNIT_REGEX = Regex("""\b(min|m|dk)\.?\b""", RegexOption.IGNORE_CASE)
        private val WHITESPACE_REGEX = Regex("""\s+""")
        private val ISO_DURATION_REGEX = Regex("""^PT(?:(\d+)H)?(?:(\d+)M)?$""", RegexOption.IGNORE_CASE)
        private val HOURS_REGEX = Regex("""(\d+)\s*(?:h|hr|hour|hours|sa|saat)""", RegexOption.IGNORE_CASE)
        private val MINUTES_REGEX = Regex("""(\d+)\s*(?:m|min|minute|minutes|dk)""", RegexOption.IGNORE_CASE)

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

        fun runtimeLabel(language: String?, value: String): String {
            parseRuntimeMinutes(value)?.let { return runtimeMinutes(language, it) }
            return value
                .replace(RUNTIME_UNIT_REGEX, t(language, "unit.minute_short"))
                .replace(WHITESPACE_REGEX, " ")
                .trim()
        }

        fun englishArtworkFallback(language: String?, fallbackUrl: String?): String? {
            return fallbackUrl
        }

        fun allowsEnglishImageFallback(language: String?): Boolean {
            return true
        }

        private fun parseRuntimeMinutes(value: String): Int? {
            val normalized = value.trim()
            if (normalized.isEmpty()) return null

            ISO_DURATION_REGEX.matchEntire(normalized)?.let { match ->
                val hours = match.groupValues.getOrNull(1)?.toIntOrNull() ?: 0
                val minutes = match.groupValues.getOrNull(2)?.toIntOrNull() ?: 0
                return (hours * 60 + minutes).takeIf { it > 0 }
            }

            val hours = HOURS_REGEX
                .find(normalized)
                ?.groupValues
                ?.getOrNull(1)
                ?.toIntOrNull()
                ?: 0
            val minutes = MINUTES_REGEX
                .find(normalized)
                ?.groupValues
                ?.getOrNull(1)
                ?.toIntOrNull()

            if (hours > 0 || minutes != null) {
                return (hours * 60 + (minutes ?: 0)).takeIf { it > 0 }
            }

            return normalized.toIntOrNull()?.takeIf { it > 0 }
        }

        private fun load(language: String?): AppStrings {
            val fileName = languageFileName(language)
            return cache.getOrPut(fileName) {
                val fallback = readAsset("en-US.json")
                val values = if (fileName == "en-US.json") fallback else readAsset(fileName)
                AppStrings(values, fallback)
            }
        }

        private fun languageFileName(language: String?): String {
            val normalized = language
                ?.trim()
                .orEmpty()
            val languageCode = normalized.removeSuffix(".json")
            return when (languageCode.lowercase()) {
                "" -> "en-US.json"
                "en", "en-us" -> "en-US.json"
                "tr", "tr-tr" -> "tr-TR.json"
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
