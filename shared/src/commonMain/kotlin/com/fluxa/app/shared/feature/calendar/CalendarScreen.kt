package com.fluxa.app.shared.feature.calendar

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.ArrowDropDown
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.common.localizedLongDate
import com.fluxa.app.common.localizedMonthTitle
import com.fluxa.app.common.localizedShortWeekdayNames
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.FluxaIcons
import kotlinx.datetime.LocalDate
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import kotlin.time.Clock
import kotlin.time.ExperimentalTime

private enum class CalendarFilterTab { All, Releases, MyList }

private data class CalendarCell(val day: Int, val inCurrentMonth: Boolean)

@OptIn(ExperimentalTime::class)
@Composable
fun CalendarScreen(
    state: CalendarUiState,
    language: String?,
    onAction: (CalendarAction) -> Unit,
    onBack: () -> Unit = {},
    modifier: Modifier = Modifier
) {
    var selectedDay by remember(state.year, state.month) { mutableStateOf<Int?>(null) }
    var activeTab by remember { mutableStateOf(CalendarFilterTab.All) }
    val today = remember { Clock.System.now().toLocalDateTime(TimeZone.currentSystemDefault()).date }
    val todayDay = today.day.takeIf { today.year == state.year && today.month.ordinal + 1 == state.month }
    val itemsByDay = remember(state.items) {
        state.items.groupBy { it.dateIso.substringAfterLast("-").toIntOrNull() ?: 0 }
    }

    Column(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        CalendarTopBar(
            language = language,
            onBack = onBack,
            onJumpToToday = {
                selectedDay = null
                onAction(CalendarAction.MonthSelected(today.year, today.month.ordinal + 1))
            }
        )
        if (state.year > 0 && state.month > 0) {
            val dayReleases = selectedDay?.let { itemsByDay[it].orEmpty() } ?: state.items
            val visibleReleases = dayReleases.filterByTab(activeTab, today)
            LazyColumn(
                contentPadding = PaddingValues(bottom = 20.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                item(key = "header") {
                    Column(
                        modifier = Modifier.padding(horizontal = 20.dp),
                        verticalArrangement = Arrangement.spacedBy(18.dp)
                    ) {
                        CalendarMonthHeader(
                            year = state.year,
                            month = state.month,
                            language = language,
                            onMonthSelected = { year, month ->
                                selectedDay = null
                                onAction(CalendarAction.MonthSelected(year, month))
                            }
                        )
                        Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                            CalendarWeekdayHeader(language = language)
                            CalendarMonthGrid(
                                year = state.year,
                                month = state.month,
                                itemsByDay = itemsByDay,
                                selectedDay = selectedDay,
                                todayDay = todayDay,
                                onDaySelected = { day -> selectedDay = if (selectedDay == day) null else day }
                            )
                        }
                        CalendarFilterTabs(
                            activeTab = activeTab,
                            language = language,
                            onTabSelected = { activeTab = it }
                        )
                    }
                }
                if (visibleReleases.isEmpty()) {
                    item(key = "empty") {
                        Text(
                            text = AppStrings.t(
                                language,
                                if (selectedDay == null) "calendar.empty" else "calendar.no_releases_this_day"
                            ),
                            color = Color.White.copy(alpha = 0.6f),
                            modifier = Modifier.padding(horizontal = 20.dp, vertical = 20.dp)
                        )
                    }
                } else {
                    items(visibleReleases, key = { "${it.id}:${it.dateIso}" }) { release ->
                        CalendarReleaseRow(
                            release = release,
                            today = today,
                            language = language,
                            onItemSelected = { onAction(CalendarAction.ItemSelected(it)) }
                        )
                    }
                }
            }
        } else {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator(color = Color.White)
            }
        }
    }
}

private fun List<CalendarReleaseUiModel>.filterByTab(
    tab: CalendarFilterTab,
    today: LocalDate
): List<CalendarReleaseUiModel> = when (tab) {
    CalendarFilterTab.All -> this
    CalendarFilterTab.Releases -> filter { it.dateIso <= today.toString() }
    CalendarFilterTab.MyList -> filter { it.isInWatchlist }
}

@Composable
private fun CalendarTopBar(
    language: String?,
    onBack: () -> Unit,
    onJumpToToday: () -> Unit
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 44.dp, bottom = 16.dp, start = 12.dp, end = 20.dp)
    ) {
        Icon(
            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
            contentDescription = AppStrings.t(language, "common.back"),
            tint = Color.White,
            modifier = Modifier.size(28.dp).clickable(onClick = onBack)
        )
        Text(
            text = AppStrings.t(language, "nav.calendar"),
            color = Color.White,
            fontWeight = FontWeight.Bold,
            fontSize = 22.sp,
            modifier = Modifier.weight(1f).padding(start = 16.dp)
        )
        Icon(
            imageVector = FluxaIcons.BottomCalendarOutline,
            contentDescription = AppStrings.t(language, "calendar.today"),
            tint = Color.White,
            modifier = Modifier.size(26.dp).clickable(onClick = onJumpToToday)
        )
    }
}

@Composable
private fun CalendarMonthHeader(
    year: Int,
    month: Int,
    language: String?,
    onMonthSelected: (Int, Int) -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = Icons.AutoMirrored.Filled.KeyboardArrowLeft,
            contentDescription = null,
            tint = Color.White,
            modifier = Modifier
                .clickable {
                    val (prevYear, prevMonth) = shiftMonth(year, month, -1)
                    onMonthSelected(prevYear, prevMonth)
                }
                .padding(horizontal = 16.dp, vertical = 8.dp)
        )
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = localizedMonthTitle(year, month, language),
                color = Color.White,
                fontWeight = FontWeight.Bold,
                fontSize = 17.sp
            )
            Icon(
                imageVector = Icons.Filled.ArrowDropDown,
                contentDescription = null,
                tint = Color.White.copy(alpha = 0.8f),
                modifier = Modifier.padding(start = 2.dp).size(20.dp)
            )
        }
        Icon(
            imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
            contentDescription = null,
            tint = Color.White,
            modifier = Modifier
                .clickable {
                    val (nextYear, nextMonth) = shiftMonth(year, month, 1)
                    onMonthSelected(nextYear, nextMonth)
                }
                .padding(horizontal = 16.dp, vertical = 8.dp)
        )
    }
}

@Composable
private fun CalendarWeekdayHeader(language: String?) {
    Row(modifier = Modifier.fillMaxWidth()) {
        localizedShortWeekdayNames(language).forEach { name ->
            Text(
                text = name.uppercase(),
                color = Color.White.copy(alpha = 0.45f),
                fontWeight = FontWeight.SemiBold,
                fontSize = 11.sp,
                textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                modifier = Modifier.weight(1f)
            )
        }
    }
}

@Composable
private fun CalendarMonthGrid(
    year: Int,
    month: Int,
    itemsByDay: Map<Int, List<CalendarReleaseUiModel>>,
    selectedDay: Int?,
    todayDay: Int?,
    onDaySelected: (Int) -> Unit
) {
    val totalDays = daysInMonth(year, month)
    val leadingBlanks = firstWeekdayOfMonth(year, month)
    val (prevYear, prevMonth) = shiftMonth(year, month, -1)
    val prevMonthDays = daysInMonth(prevYear, prevMonth)

    val cells = buildList {
        for (i in 0 until leadingBlanks) {
            add(CalendarCell(prevMonthDays - leadingBlanks + 1 + i, inCurrentMonth = false))
        }
        for (day in 1..totalDays) add(CalendarCell(day, inCurrentMonth = true))
        var trailingDay = 1
        while (size % 7 != 0) {
            add(CalendarCell(trailingDay, inCurrentMonth = false))
            trailingDay++
        }
    }

    Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
        cells.chunked(7).forEach { week ->
            Row(modifier = Modifier.fillMaxWidth()) {
                week.forEach { cell ->
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .aspectRatio(0.85f)
                            .then(
                                if (cell.inCurrentMonth) {
                                    Modifier
                                        .clip(RoundedCornerShape(10.dp))
                                        .then(
                                            if (cell.day == selectedDay) {
                                                Modifier.padding(4.dp).background(Color.White, CircleShape)
                                            } else {
                                                Modifier
                                            }
                                        )
                                        .then(
                                            if (cell.day == todayDay && cell.day != selectedDay) {
                                                Modifier.padding(6.dp).border(1.dp, Color.White.copy(alpha = 0.8f), RoundedCornerShape(6.dp))
                                            } else {
                                                Modifier
                                            }
                                        )
                                        .clickable { onDaySelected(cell.day) }
                                } else {
                                    Modifier
                                }
                            ),
                        contentAlignment = Alignment.Center
                    ) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Text(
                                text = cell.day.toString(),
                                color = when {
                                    !cell.inCurrentMonth -> Color.White.copy(alpha = 0.28f)
                                    cell.day == selectedDay -> Color.Black
                                    else -> Color.White
                                },
                                fontWeight = if (cell.day == selectedDay) FontWeight.Bold else FontWeight.Medium,
                                fontSize = 15.sp
                            )
                            if (cell.inCurrentMonth && itemsByDay.containsKey(cell.day)) {
                                Box(
                                    modifier = Modifier
                                        .padding(top = 3.dp)
                                        .size(4.dp)
                                        .background(
                                            if (cell.day == selectedDay) Color.Black else Color.White,
                                            CircleShape
                                        )
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun CalendarFilterTabs(
    activeTab: CalendarFilterTab,
    language: String?,
    onTabSelected: (CalendarFilterTab) -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .border(1.dp, Color.White.copy(alpha = 0.18f), RoundedCornerShape(10.dp))
    ) {
        CalendarFilterTab.entries.forEach { tab ->
            val selected = tab == activeTab
            Box(
                modifier = Modifier
                    .weight(1f)
                    .clickable { onTabSelected(tab) }
                    .then(
                        if (selected) {
                            Modifier
                                .padding(3.dp)
                                .clip(RoundedCornerShape(8.dp))
                                .border(1.dp, Color.White, RoundedCornerShape(8.dp))
                                .padding(vertical = 9.dp)
                        } else {
                            Modifier.padding(vertical = 12.dp)
                        }
                    ),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = AppStrings.t(
                        language,
                        when (tab) {
                            CalendarFilterTab.All -> "auto.all"
                            CalendarFilterTab.Releases -> "calendar.releases"
                            CalendarFilterTab.MyList -> "auto.my_list"
                        }
                    ),
                    color = if (selected) Color.White else Color.White.copy(alpha = 0.5f),
                    fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
                    fontSize = 14.sp
                )
            }
        }
    }
}

private fun shiftMonth(year: Int, month: Int, delta: Int): Pair<Int, Int> {
    val totalMonths = (year * 12 + (month - 1)) + delta
    val newYear = totalMonths / 12
    val newMonth = totalMonths % 12 + 1
    return newYear to newMonth
}

private fun isLeapYear(year: Int): Boolean = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)

private fun daysInMonth(year: Int, month: Int): Int = when (month) {
    1, 3, 5, 7, 8, 10, 12 -> 31
    4, 6, 9, 11 -> 30
    2 -> if (isLeapYear(year)) 29 else 28
    else -> 30
}

private fun firstWeekdayOfMonth(year: Int, month: Int): Int {
    val t = intArrayOf(0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4)
    val y = if (month < 3) year - 1 else year
    val sundayFirst = (y + y / 4 - y / 100 + y / 400 + t[month - 1] + 1) % 7
    return (sundayFirst + 6) % 7
}

private fun formatReleaseDate(dateIso: String, today: LocalDate, language: String?): String {
    if (dateIso == today.toString()) return AppStrings.t(language, "calendar.today")
    val parts = dateIso.split("-")
    val year = parts.getOrNull(0)?.toIntOrNull()
    val month = parts.getOrNull(1)?.toIntOrNull()
    val day = parts.getOrNull(2)?.toIntOrNull()
    return if (year != null && month != null && day != null) {
        localizedLongDate(year, month, day, language)
    } else {
        dateIso
    }
}

@Composable
private fun CalendarReleaseRow(
    release: CalendarReleaseUiModel,
    today: LocalDate,
    language: String?,
    onItemSelected: (CatalogItemUiModel) -> Unit
) {
    Column(modifier = Modifier.fillMaxWidth().clickable { onItemSelected(release.item) }) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp, vertical = 12.dp),
            verticalAlignment = Alignment.Top
        ) {
            Box(
                modifier = Modifier
                    .width(56.dp)
                    .aspectRatio(2f / 3f)
                    .clip(RoundedCornerShape(6.dp))
                    .background(FluxaColors.surfaceCard)
            ) {
                FluxaRemoteImage(
                    imageUrl = release.artworkUrl,
                    cacheKey = "calendar-release:${release.id}:${release.artworkUrl}",
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop
                )
            }
            Column(
                modifier = Modifier.weight(1f).padding(start = 14.dp),
                verticalArrangement = Arrangement.spacedBy(3.dp)
            ) {
                Text(
                    text = release.title,
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 15.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
                if (release.subtitle.isNotBlank()) {
                    Text(
                        text = release.subtitle,
                        color = Color.White.copy(alpha = 0.6f),
                        fontSize = 13.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
                val isNewToday = release.dateIso == today.toString()
                if (isNewToday) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                        modifier = Modifier.padding(top = 2.dp)
                    ) {
                        Text(
                            text = AppStrings.t(language, "calendar.new_badge"),
                            color = Color.White,
                            fontWeight = FontWeight.Bold,
                            fontSize = 10.sp,
                            modifier = Modifier
                                .border(1.dp, Color.White.copy(alpha = 0.6f), RoundedCornerShape(3.dp))
                                .padding(horizontal = 5.dp, vertical = 1.dp)
                        )
                        Text(
                            text = AppStrings.t(language, "calendar.new_release"),
                            color = Color.White.copy(alpha = 0.6f),
                            fontSize = 12.sp
                        )
                    }
                } else {
                    Text(
                        text = formatReleaseDate(release.dateIso, today, language),
                        color = Color.White.copy(alpha = 0.5f),
                        fontSize = 12.sp,
                        modifier = Modifier.padding(top = 2.dp)
                    )
                }
            }
            Icon(
                imageVector = Icons.Filled.ChevronRight,
                contentDescription = null,
                tint = Color.White.copy(alpha = 0.35f),
                modifier = Modifier.padding(start = 4.dp).size(20.dp)
            )
        }
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .padding(start = 20.dp)
                .height(1.dp)
                .background(Color.White.copy(alpha = 0.08f))
        )
    }
}
