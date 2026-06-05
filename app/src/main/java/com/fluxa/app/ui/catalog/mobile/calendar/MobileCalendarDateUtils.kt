@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import java.text.DateFormatSymbols
import java.text.ParseException
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.Calendar as JavaCalendar

internal fun buildMonthCells(monthStart: JavaCalendar): List<MonthCell?> {
    val calendar = monthStart.clone() as JavaCalendar
    val currentMonth = calendar.get(JavaCalendar.MONTH)
    val formatter = SimpleDateFormat("yyyy-MM-dd", Locale.US)

    val leading = (calendar.get(JavaCalendar.DAY_OF_WEEK) - JavaCalendar.MONDAY + 7) % 7
    val daysInMonth = calendar.getActualMaximum(JavaCalendar.DAY_OF_MONTH)

    val totalNeeded = leading + daysInMonth
    val totalCells = ((totalNeeded + 6) / 7) * 7

    calendar.add(JavaCalendar.DAY_OF_MONTH, -leading)

    return List(totalCells) {
        val cell = MonthCell(
            day = calendar.get(JavaCalendar.DAY_OF_MONTH),
            dateIso = formatter.format(calendar.time),
            isCurrentMonth = calendar.get(JavaCalendar.MONTH) == currentMonth
        )
        calendar.add(JavaCalendar.DAY_OF_MONTH, 1)
        cell
    }
}
internal fun shortWeekdays(locale: Locale): List<String> {
    val symbols = DateFormatSymbols(locale).shortWeekdays
    return (0 until 7).map { index ->
        symbols[((JavaCalendar.MONDAY - 1 + index) % 7) + 1].take(3)
    }
}

internal fun JavaCalendar.shiftMonth(delta: Int): JavaCalendar {
    return (clone() as JavaCalendar).apply {
        add(JavaCalendar.MONTH, delta)
        set(JavaCalendar.DAY_OF_MONTH, 1)
    }
}

internal fun JavaCalendar.monthTitle(locale: Locale): String {
    return SimpleDateFormat("MMMM yyyy", locale).format(time).replaceFirstChar {
        if (it.isLowerCase()) it.titlecase(locale) else it.toString()
    }
}

internal fun formatCalendarListDate(dateIso: String, locale: Locale): String {
    return try {
        val parsed = SimpleDateFormat("yyyy-MM-dd", Locale.US).parse(dateIso)
        parsed?.let { SimpleDateFormat("MMM d", locale).format(it) } ?: dateIso
    } catch (_: ParseException) {
        dateIso
    }
}

internal fun CalendarUpcomingItem.calendarEpisodeLabel(): String {
    val match = Regex("""\bS(\d+):E(\d+)\b""").find(subtitle.orEmpty()) ?: return ""
    return "S${match.groupValues[1]} E${match.groupValues[2]}"
}

internal fun CalendarUpcomingItem.calendarSeasonEpisodeText(lang: String): String {
    val match = Regex("""\bS(\d+):E(\d+)\b""").find(subtitle.orEmpty()) ?: return ""
    return AppStrings.format(lang, "calendar.season_episode", match.groupValues[1], match.groupValues[2])
}

internal fun CalendarUpcomingItem.calendarSecondaryTitle(): String {
    val value = subtitle.orEmpty().replace(Regex("""\bS\d+:E\d+\b"""), "").trim()
    return value.takeIf { it.isNotBlank() && it != title }.orEmpty()
}
