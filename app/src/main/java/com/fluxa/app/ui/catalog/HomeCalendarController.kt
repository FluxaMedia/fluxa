package com.fluxa.app.ui.catalog

import android.content.Context
import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.core.rust.FluxaCoreStateHandle
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.remote.Meta
import com.google.gson.Gson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

internal data class HomeCalendarSnapshot(
    val localItems: List<Meta>,
    val externalItems: List<Meta>
)

internal class HomeCalendarController(
    private val context: Context,
    private val episodeCalendarLoader: EpisodeCalendarLoader,
    private val watchlistManager: WatchlistManager,
    private val scope: CoroutineScope,
    private val activeProfile: () -> UserProfile?,
    private val setActiveProfile: (UserProfile?) -> Unit,
    private val onSnapshotLoaded: (HomeCalendarSnapshot) -> Unit,
    private val coreState: FluxaCoreStateHandle
) {
    private val gson = Gson()
    private val _items = MutableStateFlow<List<CalendarUpcomingItem>>(emptyList())
    val items: StateFlow<List<CalendarUpcomingItem>> = _items.asStateFlow()

    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    fun loadMonth(profile: UserProfile?, year: Int, month: Int) {
        setActiveProfile(profile ?: activeProfile())
        scope.launch(Dispatchers.IO) {
            dispatchCalendar("setCalendarLoading", true)
            try {
                val calendarProfile = profile ?: activeProfile()
                val result = episodeCalendarLoader.loadMonth(calendarProfile, year, month)
                if (result.externalItems.isNotEmpty()) {
                    watchlistManager.replaceExternalContinueWatching(result.externalItems)
                }
                onSnapshotLoaded(
                    HomeCalendarSnapshot(
                        localItems = result.localItems,
                        externalItems = result.externalItems
                    )
                )
                dispatchCalendar("setCalendarItems", result.items)
                CalendarWidgetProvider.updateCalendar(
                    context = context,
                    items = result.items,
                    language = calendarProfile?.safeLanguage ?: "en",
                    accentColorArgb = calendarProfile?.safeAccentColorArgb ?: 0xFFFFFFFF.toInt()
                )
                EpisodeNotificationHelper.notifyReleasedEpisodes(
                    context = context,
                    profile = profile,
                    items = result.items,
                    todayIso = ReleaseDateUtils.todayIso()
                )
            } finally {
                dispatchCalendar("setCalendarLoading", false)
            }
        }
    }

    private fun dispatchCalendar(type: String, value: Any?) {
        val snapshotJson = coreState.dispatch(CoreAction(type = type, value = value))
        val calendar = gson.fromJson(snapshotJson, CoreStateSnapshot::class.java)?.calendar ?: return
        _items.value = calendar.items
        _isLoading.value = calendar.isLoading
    }

    private data class CoreAction(
        val type: String,
        val value: Any?
    )

    private data class CoreStateSnapshot(
        val calendar: CoreCalendarSnapshot = CoreCalendarSnapshot()
    )

    private data class CoreCalendarSnapshot(
        val items: List<CalendarUpcomingItem> = emptyList(),
        val isLoading: Boolean = false
    )
}
