package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.shared.feature.calendar.CalendarDataSource
import com.fluxa.app.shared.feature.calendar.CalendarReleaseUiModel
import com.fluxa.app.shared.feature.calendar.CalendarUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import java.util.Calendar

class AndroidCalendarDataSource(
    private val homeViewModel: HomeViewModel,
    private val activeProfile: () -> UserProfile?
) : CalendarDataSource {
    private val selectedMonth = MutableStateFlow(0 to 0)

    override fun observeCalendar(): Flow<CalendarUiState> = combine(
        homeViewModel.calendarUiState,
        selectedMonth
    ) { state, selected ->
        CalendarUiState(
            year = selected.first,
            month = selected.second,
            items = state.items.map { item ->
                val catalogItem = listOf(item.meta).toCatalogItems(activeProfile()).first()
                CalendarReleaseUiModel(
                    id = item.meta.id,
                    dateIso = item.dateIso,
                    title = item.title,
                    subtitle = item.subtitle.orEmpty(),
                    artworkUrl = item.episodePoster ?: item.poster,
                    item = catalogItem
                )
            },
            isLoading = state.isLoading
        )
    }

    override suspend fun refresh() {
        if (selectedMonth.value.first == 0 || selectedMonth.value.second == 0) {
            val now = Calendar.getInstance()
            selectedMonth.value = now.get(Calendar.YEAR) to (now.get(Calendar.MONTH) + 1)
        }
        val selected = selectedMonth.value
        homeViewModel.loadCalendarMonth(activeProfile(), selected.first, selected.second)
    }

    override suspend fun loadMonth(year: Int, month: Int) {
        selectedMonth.value = year to month.coerceIn(1, 12)
        homeViewModel.loadCalendarMonth(activeProfile(), selectedMonth.value.first, selectedMonth.value.second)
    }
}
