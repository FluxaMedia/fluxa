package com.fluxa.app.shared.feature.calendar

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.common.localizedMonthTitle
import com.fluxa.app.common.localizedShortMonthDay
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun CalendarScreen(
    state: CalendarUiState,
    language: String?,
    onAction: (CalendarAction) -> Unit,
    modifier: Modifier = Modifier
) {
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
                isLoading = state.isLoading,
                onMonthSelected = { year, month -> onAction(CalendarAction.MonthSelected(year, month)) }
            )
        }
        when {
            state.isLoading && state.items.isEmpty() -> Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center
            ) { CircularProgressIndicator(color = Color.White) }
            state.items.isEmpty() -> Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center
            ) { Text(AppStrings.t(language, "calendar.empty"), color = Color.White) }
            else -> {
                val grouped = state.items.groupBy { it.dateIso }.toSortedMap()
                LazyColumn(
                    contentPadding = PaddingValues(bottom = 20.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    grouped.forEach { (dateIso, releases) ->
                        item(key = "header:$dateIso") {
                            Text(
                                text = formatDateHeader(dateIso, language),
                                color = Color.White.copy(alpha = 0.7f),
                                fontWeight = FontWeight.SemiBold,
                                modifier = Modifier.padding(top = 8.dp, bottom = 4.dp)
                            )
                        }
                        items(releases, key = { "${it.dateIso}:${it.id}" }) { release ->
                            CalendarReleaseRow(release = release, onItemSelected = {
                                onAction(CalendarAction.ItemSelected(it))
                            })
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun CalendarMonthHeader(
    year: Int,
    month: Int,
    language: String?,
    isLoading: Boolean,
    onMonthSelected: (Int, Int) -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            text = "‹",
            color = Color.White,
            fontWeight = FontWeight.Bold,
            modifier = Modifier
                .clickable {
                    val (prevYear, prevMonth) = shiftMonth(year, month, -1)
                    onMonthSelected(prevYear, prevMonth)
                }
                .padding(horizontal = 16.dp, vertical = 8.dp)
        )
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Text(
                text = localizedMonthTitle(year, month, language),
                color = Color.White,
                fontWeight = FontWeight.Bold
            )
            if (isLoading) {
                CircularProgressIndicator(color = Color.White, modifier = Modifier.padding(2.dp))
            }
        }
        Text(
            text = "›",
            color = Color.White,
            fontWeight = FontWeight.Bold,
            modifier = Modifier
                .clickable {
                    val (nextYear, nextMonth) = shiftMonth(year, month, 1)
                    onMonthSelected(nextYear, nextMonth)
                }
                .padding(horizontal = 16.dp, vertical = 8.dp)
        )
    }
}

private fun shiftMonth(year: Int, month: Int, delta: Int): Pair<Int, Int> {
    val totalMonths = (year * 12 + (month - 1)) + delta
    val newYear = totalMonths / 12
    val newMonth = totalMonths % 12 + 1
    return newYear to newMonth
}

private fun formatDateHeader(dateIso: String, language: String?): String {
    val parts = dateIso.split("-")
    val year = parts.getOrNull(0)?.toIntOrNull()
    val month = parts.getOrNull(1)?.toIntOrNull()
    val day = parts.getOrNull(2)?.toIntOrNull()
    return if (year != null && month != null && day != null) {
        localizedShortMonthDay(year, month, day, language)
    } else {
        dateIso
    }
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
