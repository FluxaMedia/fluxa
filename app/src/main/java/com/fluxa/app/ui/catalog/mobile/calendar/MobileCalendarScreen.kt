@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
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

@Composable
fun MobileCalendarScreen(
    activeProfile: UserProfile?,
    viewModel: HomeViewModel,
    onMovieClick: (Meta) -> Unit,
    plannedItems: List<Meta> = emptyList()
) {
    val lang = activeProfile?.safeLanguage ?: "en"
    val locale = AppStrings.locale(lang)
    val accent = Color(activeProfile?.safeAccentColorArgb ?: Color.White.toArgb())
    val amoled = activeProfile?.safeAmoledMode == true
    val background = if (amoled) Color.Black else Color(0xFF05070B)
    val calendarUiState by viewModel.calendarUiState.collectAsStateWithLifecycle()
    val calendarItems = calendarUiState.items
    val loading = calendarUiState.isLoading
    var visibleMonth by remember {
        mutableStateOf(JavaCalendar.getInstance().apply {
            set(JavaCalendar.DAY_OF_MONTH, 1)
            set(JavaCalendar.HOUR_OF_DAY, 0)
            set(JavaCalendar.MINUTE, 0)
            set(JavaCalendar.SECOND, 0)
            set(JavaCalendar.MILLISECOND, 0)
        })
    }
    val year = visibleMonth.get(JavaCalendar.YEAR)
    val month = visibleMonth.get(JavaCalendar.MONTH) + 1

    LaunchedEffect(activeProfile?.id, year, month, plannedItems) {
        viewModel.loadCalendarMonth(activeProfile, year, month, plannedItems)
    }

    val itemsByDate = remember(calendarItems) { calendarItems.groupBy { it.dateIso } }
    val days = remember(year, month) { buildMonthCells(visibleMonth) }
    val monthTitle = remember(year, month, lang) { visibleMonth.monthTitle(locale) }
    val weekdays = remember(lang) { shortWeekdays(locale) }
    val todayIso = remember { SimpleDateFormat("yyyy-MM-dd", Locale.US).format(Date()) }
    var viewMode by remember { mutableStateOf(CalendarViewMode.Grid) }
    var selectedGridDateIso by remember(year, month) { mutableStateOf(todayIso.takeIf { it.startsWith("%04d-%02d".format(Locale.US, year, month)) }) }

    LazyColumn(
        modifier = Modifier
            .fillMaxSize()
            .background(background),
        contentPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 24.dp, bottom = 132.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp)
    ) {
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = AppStrings.t(lang, "nav.calendar"),
                    color = Color.White,
                    fontSize = 22.sp,
                    fontWeight = FontWeight.Black,
                    modifier = Modifier.weight(1f)
                )
                if (loading) {
                    CircularProgressIndicator(color = accent, strokeWidth = 2.dp, modifier = Modifier.size(20.dp))
                    Spacer(Modifier.width(12.dp))
                }
                CalendarViewModeToggle(
                    viewMode = viewMode,
                    accent = accent,
                    onViewModeChange = { viewMode = it }
                )
            }
        }
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                IconButton(onClick = { visibleMonth = visibleMonth.shiftMonth(-1) }) {
                    Icon(FluxaIcons.ChevronLeft, null, tint = Color.White)
                }
                Text(
                    text = monthTitle,
                    color = Color.White,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Black,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.weight(1f)
                )
                IconButton(onClick = { visibleMonth = visibleMonth.shiftMonth(1) }) {
                    Icon(FluxaIcons.ChevronRight, null, tint = Color.White)
                }
            }
        }
        item {
            when (viewMode) {
                CalendarViewMode.Grid -> {
                    CalendarMonthGrid(
                        weekdays = weekdays,
                        days = days,
                        itemsByDate = itemsByDate,
                        selectedDateIso = selectedGridDateIso,
                        todayIso = todayIso,
                        accent = accent,
                        onDayClick = { day -> selectedGridDateIso = day.dateIso }
                    )
                    val selectedEvents = selectedGridDateIso?.let { itemsByDate[it] }.orEmpty()
                    if (selectedEvents.isNotEmpty()) {
                        Spacer(Modifier.height(20.dp))
                        CalendarUpcomingSection(
                            dateIso = selectedGridDateIso.orEmpty(),
                            locale = locale,
                            lang = lang,
                            events = selectedEvents,
                            onMovieClick = onMovieClick
                        )
                    }
                }
                CalendarViewMode.List -> {
                    CalendarMonthList(
                        items = calendarItems,
                        locale = locale,
                        accent = accent,
                        onMovieClick = onMovieClick
                    )
                }
            }
        }
        if (!loading && calendarItems.isEmpty()) {
            item {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(120.dp)
                        .clip(RoundedCornerShape(10.dp))
                        .background(Color.White.copy(alpha = 0.05f))
                        .border(1.dp, Color.White.copy(alpha = 0.06f), RoundedCornerShape(10.dp)),
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = AppStrings.t(lang, "calendar.empty"),
                        color = Color.White.copy(alpha = 0.64f),
                        fontSize = 13.sp,
                        fontWeight = FontWeight.Bold,
                        textAlign = TextAlign.Center,
                        modifier = Modifier.padding(horizontal = 24.dp)
                    )
                }
            }
        }
    }
}
