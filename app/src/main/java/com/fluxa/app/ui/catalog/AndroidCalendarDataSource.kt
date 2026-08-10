package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.shared.feature.calendar.CalendarDataSource
import com.fluxa.app.shared.feature.calendar.CalendarReleaseUiModel
import com.fluxa.app.shared.feature.calendar.CalendarUiState
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import java.util.Calendar

class AndroidCalendarDataSource(
    private val homeViewModel: HomeViewModel,
    private val activeProfile: () -> UserProfile?,
    private val deviceType: DeviceType = DeviceType.Mobile,
) : CalendarDataSource {
    private val selectedMonth = MutableStateFlow(0 to 0)

    override fun observeCalendar(): Flow<CalendarUiState> = combine(
        homeViewModel.calendarUiState,
        selectedMonth,
        homeViewModel.watchlist
    ) { state, selected, watchlist ->
        val profile = activeProfile()
        withContext(Dispatchers.Default) {
            val watchlistIds = watchlist.mapTo(HashSet()) { it.id }
            val catalogItems = state.items
                .map { it.meta }
                .toCatalogItems(profile, deviceType = deviceType)
            CalendarUiState(
                year = selected.first,
                month = selected.second,
                items = state.items.zip(catalogItems) { item, catalogItem ->
                    CalendarReleaseUiModel(
                        id = item.meta.id,
                        dateIso = item.dateIso,
                        title = item.title,
                        subtitle = item.subtitle.orEmpty(),
                        artworkUrl = item.episodePoster ?: item.poster,
                        item = catalogItem,
                        isInWatchlist = item.meta.id in watchlistIds
                    )
                },
                isLoading = state.isLoading
            )
        }
    }

    override suspend fun refresh() {
        if (selectedMonth.value.first == 0 || selectedMonth.value.second == 0) {
            val now = Calendar.getInstance()
            selectedMonth.value = now.get(Calendar.YEAR) to (now.get(Calendar.MONTH) + 1)
        }
        val selected = selectedMonth.value
        homeViewModel.loadCalendarMonth(activeProfile(), selected.first, selected.second, librarySourcePlannedItems())
    }

    override suspend fun loadMonth(year: Int, month: Int) {
        selectedMonth.value = year to month.coerceIn(1, 12)
        homeViewModel.loadCalendarMonth(activeProfile(), selectedMonth.value.first, selectedMonth.value.second, librarySourcePlannedItems())
    }

    private suspend fun librarySourcePlannedItems(): List<com.fluxa.app.data.remote.Meta> {
        val profile = activeProfile()
        if (profile?.integrationLibrarySource != "local") {
            homeViewModel.loadLibraryItems(profile)
            withTimeoutOrNull(8_000L) {
                homeViewModel.libraryUiState.filter { !it.isLoading }.first()
            }
        }
        val library = homeViewModel.libraryUiState.value
        return when (profile?.integrationLibrarySource) {
            "trakt" -> library.traktPlanned
            "simkl" -> library.simklPlanned + library.simklWatching
            "anilist" -> library.anilistPlanned + library.anilistWatching
            "stremio" -> library.stremioPlanned
            "nuvio" -> library.nuvioPlanned
            else -> emptyList()
        }
    }
}
