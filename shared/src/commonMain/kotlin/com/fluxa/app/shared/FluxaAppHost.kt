package com.fluxa.app.shared

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import com.fluxa.app.shared.feature.addonstore.AddonStoreAction
import com.fluxa.app.shared.feature.addonstore.AddonStoreDataSource
import com.fluxa.app.shared.feature.addonstore.AddonStoreStore
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.catalog.CatalogHomeStore
import com.fluxa.app.shared.feature.calendar.CalendarAction
import com.fluxa.app.shared.feature.calendar.CalendarDataSource
import com.fluxa.app.shared.feature.calendar.CalendarStore
import com.fluxa.app.shared.feature.detail.DetailAction
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.detail.DetailRequestUiModel
import com.fluxa.app.shared.feature.detail.DetailStore
import com.fluxa.app.shared.feature.discover.DiscoverAction
import com.fluxa.app.shared.feature.discover.DiscoverDataSource
import com.fluxa.app.shared.feature.discover.DiscoverFiltersUiModel
import com.fluxa.app.shared.feature.discover.DiscoverStore
import com.fluxa.app.shared.feature.library.LibraryAction
import com.fluxa.app.shared.feature.library.LibraryDataSource
import com.fluxa.app.shared.feature.library.LibraryStore
import com.fluxa.app.shared.feature.search.SearchAction
import com.fluxa.app.shared.feature.search.SearchDataSource
import com.fluxa.app.shared.feature.search.SearchStore
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.profile.ProfileSettingsStore
import com.fluxa.app.shared.feature.profile.ProfileStore
import com.fluxa.app.shared.platform.FluxaAddonStoreServices
import com.fluxa.app.shared.platform.FluxaDetailServices
import com.fluxa.app.shared.platform.FluxaCalendarServices
import com.fluxa.app.shared.platform.FluxaDiscoverServices
import com.fluxa.app.shared.platform.FluxaLibraryServices
import com.fluxa.app.shared.platform.FluxaPlatformServices
import com.fluxa.app.shared.platform.FluxaProfileServices
import com.fluxa.app.shared.platform.FluxaSearchServices
import kotlinx.coroutines.launch

@Composable
fun FluxaAppHost(
    platformServices: FluxaPlatformServices,
    language: String? = null,
    onCatalogAction: (CatalogAction) -> Unit = {},
    destination: FluxaDestination? = null,
    showNavigationBar: Boolean = true,
    onPlayRequested: () -> Unit = {},
    onOpenUrlRequested: (String) -> Unit = {},
    onAddonStoreBackRequested: () -> Unit = {},
    modifier: Modifier = Modifier
) {
    FluxaAppHost(
        catalogHomeDataSource = platformServices.catalogHomeDataSource,
        detailDataSource = (platformServices as? FluxaDetailServices)?.detailDataSource,
        calendarDataSource = (platformServices as? FluxaCalendarServices)?.calendarDataSource,
        discoverDataSource = (platformServices as? FluxaDiscoverServices)?.discoverDataSource,
        libraryDataSource = (platformServices as? FluxaLibraryServices)?.libraryDataSource,
        searchDataSource = (platformServices as? FluxaSearchServices)?.searchDataSource,
        profileDataSource = (platformServices as? FluxaProfileServices)?.profileDataSource,
        addonStoreDataSource = (platformServices as? FluxaAddonStoreServices)?.addonStoreDataSource,
        language = language,
        onCatalogAction = onCatalogAction,
        destination = destination,
        showNavigationBar = showNavigationBar,
        onPlayRequested = onPlayRequested,
        onOpenUrlRequested = onOpenUrlRequested,
        onAddonStoreBackRequested = onAddonStoreBackRequested,
        modifier = modifier
    )
}

@Composable
fun FluxaAppHost(
    catalogHomeDataSource: CatalogHomeDataSource,
    detailDataSource: DetailDataSource? = null,
    calendarDataSource: CalendarDataSource? = null,
    discoverDataSource: DiscoverDataSource? = null,
    libraryDataSource: LibraryDataSource? = null,
    searchDataSource: SearchDataSource? = null,
    profileDataSource: ProfileDataSource? = null,
    addonStoreDataSource: AddonStoreDataSource? = null,
    language: String? = null,
    onCatalogAction: (CatalogAction) -> Unit = {},
    destination: FluxaDestination? = null,
    showNavigationBar: Boolean = true,
    onPlayRequested: () -> Unit = {},
    onOpenUrlRequested: (String) -> Unit = {},
    onAddonStoreBackRequested: () -> Unit = {},
    modifier: Modifier = Modifier
) {
    val scope = rememberCoroutineScope()
    val catalogHomeStore = remember(catalogHomeDataSource) {
        CatalogHomeStore(catalogHomeDataSource, scope)
    }
    val catalogHome by catalogHomeStore.state.collectAsState()
    val searchStore = searchDataSource?.let { source ->
        remember(source) { SearchStore(source, scope) }
    }
    val searchState = searchStore?.state?.collectAsState()?.value
    val discoverStore = discoverDataSource?.let { source ->
        remember(source) { DiscoverStore(source, scope) }
    }
    val discoverState = discoverStore?.state?.collectAsState()?.value
    val calendarStore = calendarDataSource?.let { source ->
        remember(source) { CalendarStore(source, scope) }
    }
    val calendarState = calendarStore?.state?.collectAsState()?.value
    val libraryStore = libraryDataSource?.let { source ->
        remember(source) { LibraryStore(source, scope) }
    }
    val libraryState = libraryStore?.state?.collectAsState()?.value
    val profileStore = profileDataSource?.let { source ->
        remember(source) { ProfileStore(source, scope) }
    }
    val profileState = profileStore?.state?.collectAsState()?.value
    val settingsStore = profileDataSource?.let { source ->
        profileState?.activeProfile?.let { profile ->
            remember(profile.id, source) { ProfileSettingsStore(profile.id, source, scope) }
        }
    }
    val settingsState = settingsStore?.state?.collectAsState()?.value
    val addonStoreStore = addonStoreDataSource?.let { source ->
        remember(source) { AddonStoreStore(source, scope) }
    }
    val addonStoreState = addonStoreStore?.state?.collectAsState()?.value
    val appState = rememberFluxaAppState()
    val selectedDetail = appState.uiState.selectedDetail
    val detailStore = selectedDetail?.let { item ->
        detailDataSource?.let { source ->
            remember(item.id, item.type, source) {
                DetailStore(DetailRequestUiModel(item.id, item.type, item.source), source, scope)
            }
        }
    }
    val detailState = detailStore?.state?.collectAsState()?.value

    LaunchedEffect(catalogHome) {
        appState.updateCatalogHome(catalogHome)
    }
    LaunchedEffect(language) {
        appState.updateLanguage(language)
    }
    LaunchedEffect(destination) {
        destination?.let(appState::selectDestination)
    }
    LaunchedEffect(catalogHomeStore) {
        catalogHomeStore.dispatch(CatalogAction.Refresh)
    }
    LaunchedEffect(detailStore) {
        detailStore?.load()
    }
    LaunchedEffect(appState.uiState.destination, discoverStore) {
        if (appState.uiState.destination == FluxaDestination.Discover) {
            discoverStore?.dispatch(DiscoverAction.FiltersChanged(discoverState?.filters ?: DiscoverFiltersUiModel()))
        }
    }
    LaunchedEffect(appState.uiState.destination, libraryStore) {
        if (appState.uiState.destination == FluxaDestination.Library) {
            libraryStore?.dispatch(LibraryAction.Refresh)
        }
    }
    LaunchedEffect(appState.uiState.destination, calendarStore) {
        if (appState.uiState.destination == FluxaDestination.Calendar) {
            calendarStore?.dispatch(CalendarAction.Refresh)
        }
    }
    LaunchedEffect(appState.uiState.destination, addonStoreStore) {
        if (appState.uiState.destination == FluxaDestination.AddonStore) {
            addonStoreStore?.dispatch(AddonStoreAction.Refresh)
        }
    }

    FluxaApp(
        state = appState.uiState,
        onDestinationSelected = appState::selectDestination,
        onCatalogAction = { action ->
            if (action is CatalogAction.ItemSelected) {
                appState.selectDetail(action.item)
            }
            scope.launch {
                catalogHomeStore.dispatch(action)
            }
            onCatalogAction(action)
        },
        detailState = detailState,
        onDetailAction = { action ->
            if (action is DetailAction.RelatedItemSelected) {
                appState.selectDetail(action.item)
                onCatalogAction(CatalogAction.ItemSelected(action.item))
            }
            if (action is DetailAction.Play) {
                onPlayRequested()
            }
            scope.launch {
                detailStore?.dispatch(action)
            }
        },
        searchState = searchState,
        onSearchAction = { action ->
            if (action is SearchAction.ItemSelected) {
                appState.selectDetail(action.item)
                onCatalogAction(CatalogAction.ItemSelected(action.item))
            }
            scope.launch {
                searchStore?.dispatch(action)
            }
        },
        discoverState = discoverState,
        onDiscoverAction = { action ->
            if (action is DiscoverAction.ItemSelected) {
                appState.selectDetail(action.item)
                onCatalogAction(CatalogAction.ItemSelected(action.item))
            }
            scope.launch {
                discoverStore?.dispatch(action)
            }
        },
        calendarState = calendarState,
        onCalendarAction = { action ->
            if (action is CalendarAction.ItemSelected) {
                appState.selectDetail(action.item)
                onCatalogAction(CatalogAction.ItemSelected(action.item))
            }
            scope.launch {
                calendarStore?.dispatch(action)
            }
        },
        libraryState = libraryState,
        onLibraryItemSelected = { item ->
            appState.selectDetail(item)
            onCatalogAction(CatalogAction.ItemSelected(item))
        },
        profileState = profileState,
        settingsState = settingsState,
        onProfileSelected = { profileId ->
            scope.launch {
                profileStore?.selectProfile(profileState?.profiles?.firstOrNull { it.id == profileId } ?: return@launch)
            }
        },
        onSettingsChanged = { settings ->
            scope.launch {
                settingsStore?.update(settings)
            }
        },
        addonStoreState = addonStoreState,
        onAddonStoreAction = { action ->
            scope.launch {
                addonStoreStore?.dispatch(action)
            }
        },
        onOpenUrlRequested = onOpenUrlRequested,
        onAddonStoreBackRequested = onAddonStoreBackRequested,
        showNavigationBar = showNavigationBar,
        modifier = modifier
    )
}
