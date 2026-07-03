package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta

import androidx.compose.runtime.Composable

@Composable
fun CalendarScreen(
    activeProfile: UserProfile?,
    viewModel: HomeViewModel,
    onMovieClick: (Meta) -> Unit,
    plannedItems: List<Meta> = emptyList()
) {
    if (LocalDeviceType.current == DeviceType.Mobile) {
        MobileCalendarScreen(activeProfile, viewModel, onMovieClick, plannedItems)
    } else {
        TvCalendarScreen(activeProfile, viewModel, onMovieClick, plannedItems)
    }
}
