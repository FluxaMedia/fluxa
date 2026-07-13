package com.fluxa.app.common

import kotlinx.cinterop.ExperimentalForeignApi
import platform.Foundation.NSCalendar
import platform.Foundation.NSDateComponents
import platform.Foundation.NSDateFormatter
import platform.Foundation.NSLocale

@OptIn(ExperimentalForeignApi::class)
private fun resolveLocale(language: String?): NSLocale {
    val tag = language?.substringBefore('_')?.takeIf { it.isNotBlank() } ?: "en_US"
    return NSLocale(localeIdentifier = tag)
}

@OptIn(ExperimentalForeignApi::class)
actual fun localizedShortWeekdayNames(language: String?): List<String> {
    val formatter = NSDateFormatter().apply { locale = resolveLocale(language) }
    @Suppress("UNCHECKED_CAST")
    val sundayFirst = formatter.shortWeekdaySymbols as List<String>
    return (0 until 7).map { index -> sundayFirst[(index + 1) % 7] }
}

@OptIn(ExperimentalForeignApi::class)
actual fun localizedMonthTitle(year: Int, month: Int, language: String?): String {
    val components = NSDateComponents().apply {
        this.year = year.toLong()
        this.month = month.toLong()
        this.day = 1
    }
    val date = NSCalendar.currentCalendar.dateFromComponents(components) ?: return "$year-$month"
    val formatter = NSDateFormatter().apply {
        locale = resolveLocale(language)
        dateFormat = "MMMM yyyy"
    }
    return formatter.stringFromDate(date)
}

@OptIn(ExperimentalForeignApi::class)
actual fun localizedShortMonthDay(year: Int, month: Int, day: Int, language: String?): String {
    val components = NSDateComponents().apply {
        this.year = year.toLong()
        this.month = month.toLong()
        this.day = day.toLong()
    }
    val date = NSCalendar.currentCalendar.dateFromComponents(components) ?: return "$month/$day"
    val formatter = NSDateFormatter().apply {
        locale = resolveLocale(language)
        dateFormat = "MMM d"
    }
    return formatter.stringFromDate(date)
}
