package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.shared.feature.calendar.CalendarDataSource
import com.fluxa.app.shared.feature.calendar.CalendarReleaseUiModel
import com.fluxa.app.shared.feature.calendar.CalendarUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import java.util.Calendar

class AndroidCalendarDataSource(
    private val homeViewModel: HomeViewModel,
    private val activeProfile: () -> UserProfile?
) : CalendarDataSource {
    private var selectedYear = 0
    private var selectedMonth = 0

    override fun observeCalendar(): Flow<CalendarUiState> = combine(
        homeViewModel.calendarUiState
    ) { states ->
        val state = states.single()
        CalendarUiState(
            year = selectedYear,
            month = selectedMonth,
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
        if (selectedYear == 0 || selectedMonth == 0) {
            val now = Calendar.getInstance()
            selectedYear = now.get(Calendar.YEAR)
            selectedMonth = now.get(Calendar.MONTH) + 1
        }
        homeViewModel.loadCalendarMonth(activeProfile(), selectedYear, selectedMonth)
    }

    override suspend fun loadMonth(year: Int, month: Int) {
        selectedYear = year
        selectedMonth = month.coerceIn(1, 12)
        homeViewModel.loadCalendarMonth(activeProfile(), selectedYear, selectedMonth)
    }
}
