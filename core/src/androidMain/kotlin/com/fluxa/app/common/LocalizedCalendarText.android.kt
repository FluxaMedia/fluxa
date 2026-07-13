package com.fluxa.app.common

import java.text.DateFormatSymbols
import java.text.SimpleDateFormat
import java.util.Calendar as JavaCalendar

actual fun localizedShortWeekdayNames(language: String?): List<String> {
    val locale = AppStrings.locale(language)
    val symbols = DateFormatSymbols(locale).shortWeekdays
    return (0 until 7).map { index ->
        symbols[((JavaCalendar.MONDAY - 1 + index) % 7) + 1].take(3)
    }
}

actual fun localizedMonthTitle(year: Int, month: Int, language: String?): String {
    val locale = AppStrings.locale(language)
    val calendar = JavaCalendar.getInstance(locale).apply {
        set(JavaCalendar.YEAR, year)
        set(JavaCalendar.MONTH, month - 1)
        set(JavaCalendar.DAY_OF_MONTH, 1)
    }
    return SimpleDateFormat("MMMM yyyy", locale).format(calendar.time).replaceFirstChar {
        if (it.isLowerCase()) it.titlecase(locale) else it.toString()
    }
}

actual fun localizedShortMonthDay(year: Int, month: Int, day: Int, language: String?): String {
    val locale = AppStrings.locale(language)
    val calendar = JavaCalendar.getInstance(locale).apply {
        set(JavaCalendar.YEAR, year)
        set(JavaCalendar.MONTH, month - 1)
        set(JavaCalendar.DAY_OF_MONTH, day)
    }
    return SimpleDateFormat("MMM d", locale).format(calendar.time)
}
