package com.fluxa.app.common

import kotlinx.datetime.LocalDate
import kotlinx.datetime.TimeZone
import kotlinx.datetime.daysUntil
import kotlinx.datetime.toLocalDateTime
import kotlin.time.Clock
import kotlin.time.ExperimentalTime
import kotlin.time.Instant

@OptIn(ExperimentalTime::class)
object ReleaseDateUtils {
    private val isoDateRegex = Regex("""\d{4}-\d{2}-\d{2}""")
    private val offsetSuffixRegex = Regex(""".*[+-]\d{2}:?\d{2}$""")

    private fun todayDate(): LocalDate =
        Clock.System.now().toLocalDateTime(TimeZone.currentSystemDefault()).date

    fun todayIso(): String = todayDate().toString()

    fun isoDate(value: String?): String? {
        if (value.isNullOrBlank()) return null
        val trimmed = value.trim()
        if (!trimmed.contains("T")) {
            return trimmed.take(10).takeIf { it.matches(isoDateRegex) }
        }
        return runCatching {
            val instant = when {
                trimmed.endsWith("Z", ignoreCase = true) -> Instant.parse(trimmed)
                offsetSuffixRegex.matches(trimmed) -> Instant.parse(trimmed)
                else -> Instant.parse("${trimmed.substringBefore(".")}Z")
            }
            instant.toLocalDateTime(TimeZone.currentSystemDefault()).date.toString()
        }.getOrNull() ?: trimmed.take(10).takeIf { it.matches(isoDateRegex) }
    }

    fun daysSince(value: String?): Long? {
        val date = isoDate(value)?.let(LocalDate::parse) ?: return null
        return date.daysUntil(todayDate()).toLong()
    }

    fun isUpcoming(value: String?): Boolean {
        val date = isoDate(value) ?: return false
        return date > todayIso()
    }

    fun isRecentlyReleased(value: String?, windowDays: Int): Boolean {
        val days = daysSince(value) ?: return false
        return days in 0..windowDays.toLong()
    }
}
