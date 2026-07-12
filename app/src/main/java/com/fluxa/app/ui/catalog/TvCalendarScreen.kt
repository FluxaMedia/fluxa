package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.common.locale
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
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
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import coil3.compose.AsyncImage
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.Calendar as JavaCalendar

@Composable
internal fun TvCalendarScreen(
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
    var selectedDateIso by remember(year, month) {
        mutableStateOf(todayIso.takeIf { it.startsWith("%04d-%02d".format(Locale.US, year, month)) })
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(background)
            .padding(horizontal = 58.dp, vertical = 40.dp)
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.fillMaxWidth()) {
            Text(
                text = AppStrings.t(lang, "nav.calendar"),
                color = Color.White,
                fontSize = 32.sp,
                fontWeight = FontWeight.Black,
                letterSpacing = 2.sp,
                modifier = Modifier.weight(1f)
            )
            if (loading) {
                CircularProgressIndicator(color = accent, strokeWidth = 2.dp, modifier = Modifier.size(22.dp))
                Spacer(Modifier.width(16.dp))
            }
            IconButton(onClick = { visibleMonth = visibleMonth.shiftMonth(-1) }) {
                Icon(FluxaIcons.ChevronLeft, null, tint = Color.White)
            }
            Text(
                text = monthTitle,
                color = Color.White,
                fontSize = 20.sp,
                fontWeight = FontWeight.Black,
                textAlign = TextAlign.Center,
                modifier = Modifier.width(220.dp)
            )
            IconButton(onClick = { visibleMonth = visibleMonth.shiftMonth(1) }) {
                Icon(FluxaIcons.ChevronRight, null, tint = Color.White)
            }
        }

        Spacer(Modifier.height(24.dp))

        Row(modifier = Modifier.fillMaxWidth()) {
            weekdays.forEach { day ->
                Text(
                    text = day,
                    color = Color.White.copy(alpha = 0.5f),
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Bold,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.weight(1f)
                )
            }
        }

        Spacer(Modifier.height(8.dp))

        days.chunked(7).forEach { week ->
            Row(modifier = Modifier.fillMaxWidth()) {
                week.forEach { cell ->
                    Box(modifier = Modifier.weight(1f).padding(4.dp)) {
                        if (cell != null) {
                            TvCalendarDayCell(
                                day = cell.day,
                                isCurrentMonth = cell.isCurrentMonth,
                                isToday = cell.dateIso == todayIso,
                                isSelected = cell.dateIso == selectedDateIso,
                                hasEvents = itemsByDate.containsKey(cell.dateIso),
                                accent = accent,
                                onClick = { selectedDateIso = cell.dateIso }
                            )
                        }
                    }
                }
            }
        }

        Spacer(Modifier.height(28.dp))

        val selectedEvents = selectedDateIso?.let { itemsByDate[it] }.orEmpty()
        if (selectedEvents.isNotEmpty()) {
            Text(
                text = formatCalendarListDate(selectedDateIso.orEmpty(), locale),
                color = Color.White,
                fontSize = 16.sp,
                fontWeight = FontWeight.Bold
            )
            Spacer(Modifier.height(12.dp))
            LazyRow(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                items(selectedEvents, key = { it.meta.id + it.dateIso }) { event ->
                    TvCalendarEventCard(event, accent, onClick = { onMovieClick(event.meta) })
                }
            }
        } else if (!loading) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(100.dp)
                    .clip(RoundedCornerShape(14.dp))
                    .background(Color.White.copy(alpha = 0.05f))
                    .border(1.dp, Color.White.copy(alpha = 0.06f), RoundedCornerShape(14.dp)),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = AppStrings.t(lang, "calendar.empty"),
                    color = Color.White.copy(alpha = 0.6f),
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Bold
                )
            }
        }
    }
}

@Composable
private fun TvCalendarDayCell(
    day: Int,
    isCurrentMonth: Boolean,
    isToday: Boolean,
    isSelected: Boolean,
    hasEvents: Boolean,
    accent: Color,
    onClick: () -> Unit
) {
    var focused by remember { mutableStateOf(false) }
    Box(
        modifier = Modifier
            .aspectRatio(1.3f)
            .clip(RoundedCornerShape(10.dp))
            .background(
                when {
                    isSelected -> accent.copy(alpha = 0.9f)
                    focused -> Color.White.copy(alpha = 0.16f)
                    else -> Color.White.copy(alpha = 0.04f)
                }
            )
            .border(
                width = if (focused) 2.dp else if (isToday) 1.dp else 0.dp,
                color = if (focused) Color.White else if (isToday) accent else Color.Transparent,
                shape = RoundedCornerShape(10.dp)
            )
            .onFocusChanged { focused = it.isFocused }
            .clickable { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(
                text = day.toString(),
                color = if (isSelected) contrastOn(accent) else Color.White.copy(alpha = if (isCurrentMonth) 1f else 0.3f),
                fontWeight = FontWeight.Bold,
                fontSize = 15.sp
            )
            if (hasEvents) {
                Box(
                    modifier = Modifier
                        .padding(top = 3.dp)
                        .size(5.dp)
                        .clip(CircleShape)
                        .background(if (isSelected) contrastOn(accent) else accent)
                )
            }
        }
    }
}

@Composable
private fun TvCalendarEventCard(event: CalendarUpcomingItem, accent: Color, onClick: () -> Unit) {
    var focused by remember { mutableStateOf(false) }
    Column(
        modifier = Modifier
            .width(160.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(if (focused) Color.White.copy(alpha = 0.12f) else Color.White.copy(alpha = 0.04f))
            .border(if (focused) 2.dp else 0.dp, Color.White, RoundedCornerShape(12.dp))
            .onFocusChanged { focused = it.isFocused }
            .clickable { onClick() }
            .padding(10.dp)
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(0.66f)
                .clip(RoundedCornerShape(8.dp))
        ) {
            AsyncImage(
                model = event.episodePoster ?: event.poster,
                contentDescription = event.title,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop
            )
            Box(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .fillMaxWidth()
                    .background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(alpha = 0.8f))))
            )
        }
        Spacer(Modifier.height(6.dp))
        Text(event.title, color = Color.White, fontWeight = FontWeight.Bold, fontSize = 13.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
        event.subtitle?.let {
            Text(it, color = Color.White.copy(alpha = 0.6f), fontSize = 11.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
    }
}

private fun contrastOn(color: Color): Color {
    return if (color.luminance() > 0.5f) Color.Black else Color.White
}
