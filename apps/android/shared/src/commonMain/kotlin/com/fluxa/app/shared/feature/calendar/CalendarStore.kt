package com.fluxa.app.shared.feature.calendar

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn

class CalendarStore(
    private val dataSource: CalendarDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<CalendarUiState> = dataSource.observeCalendar()
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), CalendarUiState(isLoading = true))

    suspend fun dispatch(action: CalendarAction) {
        when (action) {
            CalendarAction.Refresh -> dataSource.refresh()
            is CalendarAction.MonthSelected -> dataSource.loadMonth(action.year, action.month)
            is CalendarAction.ItemSelected -> Unit
        }
    }
}
