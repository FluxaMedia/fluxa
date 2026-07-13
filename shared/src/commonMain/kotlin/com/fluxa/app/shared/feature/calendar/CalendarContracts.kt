package com.fluxa.app.shared.feature.calendar

import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import kotlinx.coroutines.flow.Flow

data class CalendarReleaseUiModel(
    val id: String,
    val dateIso: String,
    val title: String,
    val subtitle: String,
    val artworkUrl: String?,
    val item: CatalogItemUiModel
)

data class CalendarUiState(
    val year: Int = 0,
    val month: Int = 0,
    val items: List<CalendarReleaseUiModel> = emptyList(),
    val isLoading: Boolean = false
)

sealed interface CalendarAction {
    data object Refresh : CalendarAction
    data class MonthSelected(val year: Int, val month: Int) : CalendarAction
    data class ItemSelected(val item: CatalogItemUiModel) : CalendarAction
}

interface CalendarDataSource {
    fun observeCalendar(): Flow<CalendarUiState>
    suspend fun refresh()
    suspend fun loadMonth(year: Int, month: Int)
}
