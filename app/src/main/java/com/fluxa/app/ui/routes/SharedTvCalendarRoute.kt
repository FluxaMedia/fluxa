package com.fluxa.app.ui.routes

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import com.fluxa.app.shared.feature.calendar.CalendarAction
import com.fluxa.app.shared.feature.calendar.CalendarScreen
import com.fluxa.app.shared.feature.calendar.CalendarStore
import com.fluxa.app.shared.platform.FluxaCalendarServices
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import kotlinx.coroutines.launch

@Composable
internal fun SharedTvCalendarRoute(
    platformServices: FluxaCalendarServices,
    navigator: AppNavigator,
    language: String?
) {
    val scope = rememberCoroutineScope()
    val store = remember(platformServices.calendarDataSource) {
        CalendarStore(platformServices.calendarDataSource, scope)
    }
    val state by store.state.collectAsState()

    LaunchedEffect(store) {
        store.dispatch(CalendarAction.Refresh)
    }

    CalendarScreen(
        state = state,
        language = language,
        onAction = { action ->
            if (action is CalendarAction.ItemSelected) {
                val item = action.item
                navigator.navigateTo(
                    Screen.Detail(
                        type = item.type,
                        id = item.id,
                        sourceAddonTransportUrl = item.source.addonTransportUrl,
                        sourceAddonCatalogType = item.source.catalogType
                    )
                )
            } else {
                scope.launch { store.dispatch(action) }
            }
        }
    )
}
