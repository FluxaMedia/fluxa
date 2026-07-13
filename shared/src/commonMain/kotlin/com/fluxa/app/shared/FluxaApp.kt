package com.fluxa.app.shared

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.calendar.CalendarAction
import com.fluxa.app.shared.feature.calendar.CalendarScreen
import com.fluxa.app.shared.feature.calendar.CalendarUiState
import com.fluxa.app.shared.feature.detail.DetailAction
import com.fluxa.app.shared.feature.detail.DetailScreen
import com.fluxa.app.shared.feature.detail.DetailUiState
import com.fluxa.app.shared.feature.discover.DiscoverAction
import com.fluxa.app.shared.feature.discover.DiscoverScreen
import com.fluxa.app.shared.feature.discover.DiscoverUiState
import com.fluxa.app.shared.feature.library.LibraryScreen
import com.fluxa.app.shared.feature.library.LibraryUiState
import com.fluxa.app.shared.feature.profile.ProfileSettingsScreen
import com.fluxa.app.shared.feature.profile.ProfileUiState
import com.fluxa.app.shared.feature.profile.SettingsUiState
import com.fluxa.app.shared.feature.search.SearchAction
import com.fluxa.app.shared.feature.search.SearchScreen
import com.fluxa.app.shared.feature.search.SearchUiState
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.FluxaColors

enum class FluxaDestination(val titleKey: String) {
    Home("nav.home"),
    Search("auto.search"),
    Discover("nav.discover"),
    Calendar("nav.calendar"),
    Library("nav.library"),
    Settings("nav.settings")
}

data class FluxaAppUiState(
    val language: String? = null,
    val destination: FluxaDestination = FluxaDestination.Home,
    val catalogHome: CatalogHomeUiState = CatalogHomeUiState(),
    val selectedDetail: CatalogItemUiModel? = null
)

@Composable
fun FluxaApp(
    state: FluxaAppUiState,
    onDestinationSelected: (FluxaDestination) -> Unit,
    onCatalogAction: (CatalogAction) -> Unit,
    detailState: DetailUiState? = null,
    onDetailAction: (DetailAction) -> Unit = {},
    searchState: SearchUiState? = null,
    onSearchAction: (SearchAction) -> Unit = {},
    discoverState: DiscoverUiState? = null,
    onDiscoverAction: (DiscoverAction) -> Unit = {},
    calendarState: CalendarUiState? = null,
    onCalendarAction: (CalendarAction) -> Unit = {},
    libraryState: LibraryUiState? = null,
    onLibraryItemSelected: (CatalogItemUiModel) -> Unit = {},
    profileState: ProfileUiState? = null,
    settingsState: SettingsUiState? = null,
    onProfileSelected: (String) -> Unit = {},
    onSettingsChanged: (SettingsUiState) -> Unit = {},
    showNavigationBar: Boolean = true,
    modifier: Modifier = Modifier
) {
    MaterialTheme {
        Column(
            modifier = modifier
                .fillMaxSize()
                .background(FluxaColors.background)
        ) {
            if (showNavigationBar) {
                FluxaNavigationBar(
                    language = state.language,
                    destination = state.destination,
                    onDestinationSelected = onDestinationSelected
                )
            }
            when {
                state.selectedDetail != null && detailState != null -> DetailScreen(
                    state = detailState,
                    language = state.language,
                    onAction = onDetailAction,
                    modifier = Modifier.weight(1f)
                )
                state.destination == FluxaDestination.Search && searchState != null -> SearchScreen(
                    state = searchState,
                    language = state.language,
                    onQueryChanged = { value -> onSearchAction(SearchAction.QueryChanged(value)) },
                    onItemSelected = { item -> onSearchAction(SearchAction.ItemSelected(item)) },
                    modifier = Modifier.weight(1f)
                )
                state.destination == FluxaDestination.Discover && discoverState != null -> DiscoverScreen(
                    state = discoverState,
                    language = state.language,
                    onFiltersChanged = { filters -> onDiscoverAction(DiscoverAction.FiltersChanged(filters)) },
                    onItemSelected = { item -> onDiscoverAction(DiscoverAction.ItemSelected(item)) },
                    modifier = Modifier.weight(1f)
                )
                state.destination == FluxaDestination.Calendar && calendarState != null -> CalendarScreen(
                    state = calendarState,
                    language = state.language,
                    onAction = onCalendarAction,
                    modifier = Modifier.weight(1f)
                )
                state.destination == FluxaDestination.Library && libraryState != null -> LibraryScreen(
                    state = libraryState,
                    language = state.language,
                    onItemSelected = onLibraryItemSelected,
                    modifier = Modifier.weight(1f)
                )
                state.destination == FluxaDestination.Settings && profileState != null && settingsState != null -> ProfileSettingsScreen(
                    profileState = profileState,
                    settingsState = settingsState,
                    language = state.language,
                    onProfileSelected = { profile -> onProfileSelected(profile.id) },
                    onSettingsChanged = onSettingsChanged,
                    modifier = Modifier.weight(1f)
                )
                state.destination == FluxaDestination.Home -> FluxaHomeContent(
                    state = state,
                    onCatalogAction = onCatalogAction,
                    modifier = Modifier.weight(1f)
                )
                else -> FluxaDestinationPlaceholder(
                    language = state.language,
                    destination = state.destination,
                    modifier = Modifier.weight(1f)
                )
            }
        }
    }
}

@Composable
private fun FluxaNavigationBar(
    language: String?,
    destination: FluxaDestination,
    onDestinationSelected: (FluxaDestination) -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 20.dp, vertical = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(20.dp)
    ) {
        FluxaDestination.entries.forEach { item ->
            val selected = item == destination
            Text(
                text = AppStrings.t(language, item.titleKey),
                color = if (selected) Color.White else Color.White.copy(alpha = 0.65f),
                fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
                modifier = Modifier.clickable { onDestinationSelected(item) }
            )
        }
    }
}

@Composable
private fun FluxaHomeContent(
    state: FluxaAppUiState,
    onCatalogAction: (CatalogAction) -> Unit,
    modifier: Modifier
) {
    if (state.catalogHome.isLoading && state.catalogHome.rows.isEmpty()) {
        Box(modifier = modifier, contentAlignment = Alignment.Center) {
            CircularProgressIndicator(color = Color.White)
        }
        return
    }

    if (state.catalogHome.rows.isEmpty()) {
        FluxaDestinationPlaceholder(
            language = state.language,
            destination = FluxaDestination.Home,
            modifier = modifier
        )
        return
    }

    Column(
        modifier = modifier.padding(vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(24.dp)
    ) {
        state.catalogHome.rows.forEach { row ->
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 20.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = row.title,
                        color = Color.White,
                        fontWeight = FontWeight.Bold
                    )
                    if (row.canLoadMore) {
                        Text(
                            text = AppStrings.t(state.language, "common.view_all"),
                            color = Color.White.copy(alpha = 0.7f),
                            modifier = Modifier.clickable {
                                onCatalogAction(CatalogAction.LoadMore(row.id))
                            }
                        )
                    }
                }
                LazyRow(
                    contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 20.dp),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    items(row.items, key = { it.id }) { item ->
                        CatalogCard(
                            model = item.card,
                            onClick = { onCatalogAction(CatalogAction.ItemSelected(item)) }
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun FluxaDestinationPlaceholder(
    language: String?,
    destination: FluxaDestination,
    modifier: Modifier
) {
    Box(modifier = modifier, contentAlignment = Alignment.Center) {
        Text(
            text = AppStrings.t(language, destination.titleKey),
            color = Color.White,
            style = MaterialTheme.typography.headlineMedium
        )
    }
}
