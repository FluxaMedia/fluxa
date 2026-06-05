package com.fluxa.app.ui.catalog

import android.content.Context
import org.json.JSONObject
import java.util.Locale
import java.util.concurrent.ConcurrentHashMap

class AppStrings private constructor(
    private val values: Map<String, String>,
    private val fallback: Map<String, String>
) {
    fun get(key: String): String = values[key] ?: fallback[key] ?: key

    companion object {
        private val cache = ConcurrentHashMap<String, AppStrings>()
        private var appContext: Context? = null
        private val RUNTIME_UNIT_REGEX = Regex("""\b(min|m|dk)\.?\b""", RegexOption.IGNORE_CASE)
        private val WHITESPACE_REGEX = Regex("""\s+""")
        private val ISO_DURATION_REGEX = Regex("""^PT(?:(\d+)H)?(?:(\d+)M)?$""", RegexOption.IGNORE_CASE)
        private val HOURS_REGEX = Regex("""(\d+)\s*(?:h|hr|hour|hours|sa|saat)""", RegexOption.IGNORE_CASE)
        private val MINUTES_REGEX = Regex("""(\d+)\s*(?:m|min|minute|minutes|dk)""", RegexOption.IGNORE_CASE)

        fun initialize(context: Context) {
            appContext = context.applicationContext
        }

        fun t(language: String?, key: String): String {
            val context = appContext ?: return key
            return load(context, language).get(key)
        }

        fun format(language: String?, key: String, vararg args: Any?): String {
            return args.fold(t(language, key)) { value, arg ->
                value.replaceFirst("%s", arg?.toString().orEmpty())
            }
        }

        fun list(language: String?, key: String): List<String> {
            return t(language, key).split("|").map { it.trim() }.filter { it.isNotEmpty() }
        }

        fun locale(language: String?): Locale {
            val tag = language
                ?.substringBefore('_')
                ?.takeIf { it.isNotBlank() }
                ?: return Locale.US
            return Locale.forLanguageTag(tag)
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

        fun load(context: Context, language: String?): AppStrings {
            val fileName = languageFileName(language)
            return cache.getOrPut(fileName) {
                val fallback = readAsset(context, "i18n/english_us.json")
                val values = if (fileName == "english_us.json") fallback else readAsset(context, "i18n/$fileName")
                AppStrings(values, fallback)
            }
        }

        private fun languageFileName(language: String?): String {
            val normalized = language
                ?.trim()
                ?.lowercase(Locale.ROOT)
                ?.replace('-', '_')
                .orEmpty()
            return when (normalized) {
                "" -> "english_us.json"
                "en", "en_us", "english_us" -> "english_us.json"
                "tr", "tr_tr" -> "tr_tr.json"
                else -> if (normalized.endsWith(".json")) normalized else "$normalized.json"
            }
        }

        private fun readAsset(context: Context, path: String): Map<String, String> {
            return runCatching {
                val json = context.assets.open(path).bufferedReader().use { it.readText() }
                val obj = JSONObject(json)
                obj.keys().asSequence().associateWith { key -> obj.optString(key, key) }
            }.getOrDefault(emptyMap())
        }
    }
}
