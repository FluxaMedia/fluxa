package com.fluxa.app.shared.feature.calendar

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.KeyboardArrowLeft
import androidx.compose.material.icons.filled.KeyboardArrowRight
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.common.localizedMonthTitle
import com.fluxa.app.common.localizedShortWeekdayNames
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun CalendarScreen(
    state: CalendarUiState,
    language: String?,
    onAction: (CalendarAction) -> Unit,
    modifier: Modifier = Modifier
) {
    var selectedDay by remember(state.year, state.month) { mutableStateOf<Int?>(null) }
    val itemsByDay = remember(state.items) {
        state.items.groupBy { it.dateIso.substringAfterLast("-").toIntOrNull() ?: 0 }
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .background(FluxaColors.background)
            .padding(horizontal = 20.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        if (state.year > 0 && state.month > 0) {
            CalendarMonthHeader(
                year = state.year,
                month = state.month,
                language = language,
                onMonthSelected = { year, month -> onAction(CalendarAction.MonthSelected(year, month)) }
            )
            LazyColumn(
                contentPadding = PaddingValues(bottom = 20.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                item(key = "weekdays") {
                    CalendarWeekdayHeader(language = language)
                }
                item(key = "grid") {
                    CalendarMonthGrid(
                        year = state.year,
                        month = state.month,
                        itemsByDay = itemsByDay,
                        selectedDay = selectedDay,
                        onDaySelected = { day -> selectedDay = day }
                    )
                }
                item(key = "detail") {
                    CalendarSelectedDayDetail(
                        selectedDay = selectedDay,
                        releases = selectedDay?.let { itemsByDay[it] }.orEmpty(),
                        language = language,
                        onItemSelected = { onAction(CalendarAction.ItemSelected(it)) }
                    )
                }
            }
        } else {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator(color = Color.White)
            }
        }
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
            imageVector = Icons.Filled.KeyboardArrowLeft,
            contentDescription = null,
            tint = Color.White,
            modifier = Modifier
                .clickable {
                    val (prevYear, prevMonth) = shiftMonth(year, month, -1)
                    onMonthSelected(prevYear, prevMonth)
                }
                .padding(horizontal = 16.dp, vertical = 8.dp)
        )
        Text(
            text = localizedMonthTitle(year, month, language),
            color = Color.White,
            fontWeight = FontWeight.Bold
        )
        Icon(
            imageVector = Icons.Filled.KeyboardArrowRight,
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
                text = name,
                color = Color.White.copy(alpha = 0.6f),
                fontWeight = FontWeight.SemiBold,
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
    onDaySelected: (Int) -> Unit
) {
    val totalDays = daysInMonth(year, month)
    val leadingBlanks = firstWeekdayOfMonth(year, month)
    val cells = buildList {
        repeat(leadingBlanks) { add(null) }
        for (day in 1..totalDays) add(day)
        while (size % 7 != 0) add(null)
    }
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        cells.chunked(7).forEach { week ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                week.forEach { day ->
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .aspectRatio(1f)
                            .then(
                                if (day != null) {
                                    Modifier
                                        .clip(RoundedCornerShape(10.dp))
                                        .background(
                                            if (day == selectedDay) FluxaColors.accent else Color.White.copy(alpha = 0.06f)
                                        )
                                        .clickable { onDaySelected(day) }
                                } else {
                                    Modifier
                                }
                            ),
                        contentAlignment = Alignment.Center
                    ) {
                        if (day != null) {
                            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                Text(
                                    text = day.toString(),
                                    color = Color.White,
                                    fontWeight = if (day == selectedDay) FontWeight.Bold else FontWeight.Medium
                                )
                                if (itemsByDay.containsKey(day)) {
                                    Box(
                                        modifier = Modifier
                                            .padding(top = 2.dp)
                                            .size(4.dp)
                                            .background(
                                                if (day == selectedDay) Color.White else FluxaColors.accent,
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
}

@Composable
private fun CalendarSelectedDayDetail(
    selectedDay: Int?,
    releases: List<CalendarReleaseUiModel>,
    language: String?,
    onItemSelected: (CatalogItemUiModel) -> Unit
) {
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        when {
            selectedDay == null -> Text(
                text = AppStrings.t(language, "calendar.empty"),
                color = Color.White.copy(alpha = 0.6f)
            )
            releases.isEmpty() -> Text(
                text = AppStrings.t(language, "calendar.no_releases_this_day"),
                color = Color.White.copy(alpha = 0.6f)
            )
            else -> releases.forEach { release ->
                CalendarReleaseRow(release = release, onItemSelected = onItemSelected)
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

@Composable
private fun CalendarReleaseRow(
    release: CalendarReleaseUiModel,
    onItemSelected: (CatalogItemUiModel) -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onItemSelected(release.item) }
            .background(FluxaColors.surfaceCard)
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp)
    ) {
        Text(release.title, color = Color.White, fontWeight = FontWeight.Bold)
        if (release.subtitle.isNotBlank()) {
            Text(release.subtitle, color = Color.White.copy(alpha = 0.7f))
        }
    }
}
