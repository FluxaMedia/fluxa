package com.fluxa.app.common

import java.time.Instant
import java.time.LocalDate
import java.time.OffsetDateTime
import java.time.ZoneId
import java.time.temporal.ChronoUnit

object ReleaseDateUtils {
    private val isoDateRegex = Regex("""\d{4}-\d{2}-\d{2}""")
    private val offsetSuffixRegex = Regex(""".*[+-]\d{2}:?\d{2}$""")

    fun todayIso(): String = LocalDate.now(ZoneId.systemDefault()).toString()

    fun isoDate(value: String?): String? {
        if (value.isNullOrBlank()) return null
        val trimmed = value.trim()
        if (!trimmed.contains("T")) {
            return trimmed.take(10).takeIf { it.matches(isoDateRegex) }
        }
        return runCatching {
            val instant = when {
                trimmed.endsWith("Z", ignoreCase = true) -> Instant.parse(trimmed)
                offsetSuffixRegex.matches(trimmed) -> OffsetDateTime.parse(trimmed).toInstant()
                else -> Instant.parse("${trimmed.substringBefore(".")}Z")
            }
            instant.atZone(ZoneId.systemDefault()).toLocalDate().toString()
        }.getOrNull() ?: trimmed.take(10).takeIf { it.matches(isoDateRegex) }
    }

    fun daysSince(value: String?): Long? {
        val date = isoDate(value)?.let(LocalDate::parse) ?: return null
        return ChronoUnit.DAYS.between(date, LocalDate.now(ZoneId.systemDefault()))
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
