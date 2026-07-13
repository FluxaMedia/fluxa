package com.fluxa.app.common

expect fun localizedShortWeekdayNames(language: String?): List<String>
expect fun localizedMonthTitle(year: Int, month: Int, language: String?): String
expect fun localizedShortMonthDay(year: Int, month: Int, day: Int, language: String?): String
