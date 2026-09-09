package com.fluxa.app.common

import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import kotlin.time.Clock
import kotlin.time.ExperimentalTime

@OptIn(ExperimentalTime::class)
object ReleaseDateUtils {
    fun todayIso(): String =
        Clock.System.now().toLocalDateTime(TimeZone.currentSystemDefault()).date.toString()
}
