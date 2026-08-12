package com.fluxa.app.shared

import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import com.fluxa.app.shared.feature.addonstore.AddonStoreAction
import com.fluxa.app.shared.feature.addonstore.AddonStoreStore
import com.fluxa.app.shared.feature.plugins.PluginsAction
import com.fluxa.app.shared.feature.plugins.PluginsStore
import com.fluxa.app.shared.feature.auth.AuthAction
import com.fluxa.app.shared.feature.auth.AuthStore
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogHomeStore
import com.fluxa.app.shared.feature.calendar.CalendarAction
import com.fluxa.app.shared.feature.calendar.CalendarStore
import com.fluxa.app.shared.feature.detail.DetailAction
import com.fluxa.app.shared.feature.detail.DetailStore
import com.fluxa.app.shared.feature.discover.DiscoverAction
import com.fluxa.app.shared.feature.discover.DiscoverFiltersUiModel
import com.fluxa.app.shared.feature.discover.DiscoverStore
import com.fluxa.app.shared.feature.library.LibraryAction
import com.fluxa.app.shared.feature.library.LibraryStore
import com.fluxa.app.shared.feature.search.SearchAction
import com.fluxa.app.shared.feature.search.SearchStore
import com.fluxa.app.shared.feature.profile.ProfileAction
import com.fluxa.app.shared.feature.profile.ProfileEditTarget
import com.fluxa.app.shared.feature.profile.ProfileStore
import com.fluxa.app.shared.feature.profile.ProfileUiState
import com.fluxa.app.shared.feature.settings.SettingsAction
import com.fluxa.app.shared.feature.settings.SettingsStore
import com.fluxa.app.shared.platform.FluxaAddonStoreServices
import com.fluxa.app.shared.platform.FluxaPluginsServices
import com.fluxa.app.shared.platform.FluxaAuthServices
import com.fluxa.app.shared.platform.FluxaDetailServices
import com.fluxa.app.shared.platform.FluxaCalendarServices
import com.fluxa.app.shared.platform.FluxaDiscoverServices
import com.fluxa.app.shared.platform.FluxaLibraryServices
import com.fluxa.app.shared.platform.FluxaPlatformServices
import com.fluxa.app.shared.platform.FluxaProfileServices
import com.fluxa.app.shared.platform.FluxaSearchServices
import com.fluxa.app.shared.platform.FluxaSettingsServices
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch

@Composable
fun FluxaAppHost(
    platformServices: FluxaPlatformServices,
    config: FluxaAppHostConfig = FluxaAppHostConfig(),
    callbacks: FluxaAppHostCallbacks = FluxaAppHostCallbacks(),
    visuals: FluxaAppHostVisuals = FluxaAppHostVisuals(),
    modifier: Modifier = Modifier,
) {
    FluxaAppHost(
        dataSources = FluxaAppDataSources(
            catalogHome = platformServices.catalogHomeDataSource,
            detail = (platformServices as? FluxaDetailServices)?.detailDataSource,
            calendar = (platformServices as? FluxaCalendarServices)?.calendarDataSource,
            discover = (platformServices as? FluxaDiscoverServices)?.discoverDataSource,
            library = (platformServices as? FluxaLibraryServices)?.libraryDataSource,
            search = (platformServices as? FluxaSearchServices)?.searchDataSource,
            profile = (platformServices as? FluxaProfileServices)?.profileDataSource,
            addonStore = (platformServices as? FluxaAddonStoreServices)?.addonStoreDataSource,
            plugins = (platformServices as? FluxaPluginsServices)?.pluginsDataSource,
            auth = (platformServices as? FluxaAuthServices)?.authDataSource,
            settings = (platformServices as? FluxaSettingsServices)?.settingsDataSource,
        ),
        config = config,
        callbacks = callbacks,
        visuals = visuals,
        modifier = modifier,
    )
}

@Composable
fun FluxaAppHost(
    dataSources: FluxaAppDataSources,
    config: FluxaAppHostConfig = FluxaAppHostConfig(),
    callbacks: FluxaAppHostCallbacks = FluxaAppHostCallbacks(),
    visuals: FluxaAppHostVisuals = FluxaAppHostVisuals(),
    modifier: Modifier = Modifier,
) {
    FluxaAppHostContent(
        dataSources = dataSources,
        config = config,
        callbacks = callbacks,
        visuals = visuals,
        modifier = modifier,
    )
}

@Composable
private fun FluxaAppHostContent(
    dataSources: FluxaAppDataSources,
    config: FluxaAppHostConfig,
    callbacks: FluxaAppHostCallbacks,
    visuals: FluxaAppHostVisuals,
    modifier: Modifier,
) {
    val catalogHomeDataSource = dataSources.catalogHome
    val detailDataSource = dataSources.detail
    val calendarDataSource = dataSources.calendar
    val discoverDataSource = dataSources.discover
    val libraryDataSource = dataSources.library
    val searchDataSource = dataSources.search
    val profileDataSource = dataSources.profile
    val addonStoreDataSource = dataSources.addonStore
    val pluginsDataSource = dataSources.plugins
    val authDataSource = dataSources.auth
    val settingsDataSource = dataSources.settings
    val deviceType = config.deviceType
    val isLowRamDevice = config.isLowRamDevice
    val language = config.language
    val destination = config.destination
    val detailRequest = config.detailRequest
    val showNavigationBar = config.showNavigationBar
    val authStartOnNuvio = config.authStartOnNuvio
    val biometricAvailable = config.biometricAvailable
    val settingsPopRequestId = config.settingsPopRequestId
    val overlayPopRequestId = config.overlayPopRequestId

    val navigationCallbacks = callbacks.navigation
    val authCallbacks = callbacks.auth
    val profileCallbacks = callbacks.profile
    val settingsCallbacks = callbacks.settings
    val libraryCallbacks = callbacks.library
    val overlayCallbacks = callbacks.overlay

    val onCatalogAction = navigationCallbacks.onCatalogAction
    val onDetailNavigationEvent = navigationCallbacks.onDetailNavigationEvent
    val onDetailBackRequested = navigationCallbacks.onDetailBackRequested
    val onOpenUrlRequested = navigationCallbacks.onOpenUrlRequested
    val onAddonStoreBackRequested = navigationCallbacks.onAddonStoreBackRequested
    val onPluginsBackRequested = navigationCallbacks.onPluginsBackRequested
    val onDownloadOpened = navigationCallbacks.onDownloadOpened
    val onDestinationChanged = navigationCallbacks.onDestinationChanged

    val onAuthBackRequested = authCallbacks.onAuthBackRequested
    val onAuthCompleted = authCallbacks.onAuthCompleted
    val onConnectStremioRequested = authCallbacks.onConnectStremioRequested
    val onConnectNuvioRequested = authCallbacks.onConnectNuvioRequested
    val onConnectStremioWithCredentials = authCallbacks.onConnectStremioWithCredentials
    val onConnectNuvioWithCredentials = authCallbacks.onConnectNuvioWithCredentials
    val onConnectTraktRequested = authCallbacks.onConnectTraktRequested
    val onConnectSimklRequested = authCallbacks.onConnectSimklRequested
    val onSyncProviderRequested = authCallbacks.onSyncProviderRequested
    val onConnectAnilistRequested = authCallbacks.onConnectAnilistRequested

    val onPickAvatarRequested = profileCallbacks.onPickAvatarRequested
    val onBiometricAuthRequested = profileCallbacks.onBiometricAuthRequested
    val onProfileSelectionCompleted = profileCallbacks.onProfileSelectionCompleted

    val onManageAddonsRequested = settingsCallbacks.onManageAddonsRequested
    val onManagePluginsRequested = settingsCallbacks.onManagePluginsRequested
    val onCheckForUpdateRequested = settingsCallbacks.onCheckForUpdateRequested
    val onSettingsBackRequested = settingsCallbacks.onSettingsBackRequested
    val onSettingsCanPopChanged = settingsCallbacks.onSettingsCanPopChanged
    val onOverlayOpenChanged = overlayCallbacks.onOverlayOpenChanged

    val nuvioIcon = visuals.nuvioIcon
    val stremioIcon = visuals.stremioIcon
    val authBackdrop = visuals.authBackdrop
    val traktIcon = visuals.traktIcon
    val simklIcon = visuals.simklIcon
    val anilistIcon = visuals.anilistIcon

    val scope = rememberCoroutineScope()
    val appState = rememberFluxaAppState(
        FluxaAppUiState(
            language = language,
            destination = destination ?: FluxaDestination.Home,
        )
    )
    val currentAppState = appState.uiState

    val catalogHomeStore = remember(catalogHomeDataSource) {
        CatalogHomeStore(catalogHomeDataSource, scope)
    }
    val obscuresCatalog = currentAppState.selectedDetail != null ||
        currentAppState.showSourceSelection ||
        currentAppState.showNotifications
    val needsCatalogHome = !obscuresCatalog && (
        currentAppState.destination == FluxaDestination.Home ||
            currentAppState.selectedCategoryId != null
        )
    val catalogHome = if (needsCatalogHome) {
        catalogHomeStore.state.collectAsState().value
    } else {
        // Keep the last snapshot available for actions without subscribing the app root to
        // billboard rotations while another route is visible.
        catalogHomeStore.state.value
    }

    val searchStore = searchDataSource?.let { source ->
        remember(source) { SearchStore(source, scope) }
    }
    val routeContentObscured = currentAppState.selectedDetail != null ||
        currentAppState.showSourceSelection ||
        currentAppState.selectedCategoryId != null ||
        currentAppState.showNotifications
    val needsSearch = !routeContentObscured && (
        currentAppState.destination == FluxaDestination.Search ||
            currentAppState.destination == FluxaDestination.Discover
        )
    val searchState = if (needsSearch) searchStore?.state?.collectAsState()?.value else null

    val discoverStore = discoverDataSource?.let { source ->
        remember(source) { DiscoverStore(source, scope) }
    }
    val discoverState = if (!routeContentObscured && currentAppState.destination == FluxaDestination.Discover) {
        discoverStore?.state?.collectAsState()?.value
    } else null

    val calendarStore = calendarDataSource?.let { source ->
        remember(source) { CalendarStore(source, scope) }
    }
    val needsCalendar = currentAppState.selectedDetail == null &&
        !currentAppState.showSourceSelection &&
        (currentAppState.destination == FluxaDestination.Calendar || currentAppState.showNotifications)
    val calendarState = if (needsCalendar) {
        calendarStore?.state?.collectAsState()?.value
    } else null

    val libraryStore = libraryDataSource?.let { source ->
        remember(source) { LibraryStore(source, scope) }
    }
    val homeNeedsLibraryFolders = remember(deviceType, catalogHome.rows) {
        deviceType != com.fluxa.app.ui.catalog.DeviceType.TV &&
            catalogHome.rows.any { row -> row.items.any { it.type == "catalog_folder" } }
    }
    val needsLibrary = currentAppState.selectedDetail == null &&
        !currentAppState.showSourceSelection &&
        (currentAppState.destination == FluxaDestination.Library || homeNeedsLibraryFolders)
    val libraryState = if (needsLibrary) libraryStore?.state?.collectAsState()?.value else null

    val profileStore = profileDataSource?.let { source ->
        remember(source) { ProfileStore(source, scope) }
    }
    val needsProfile = currentAppState.destination == FluxaDestination.ProfileList ||
        currentAppState.editingProfile != null || currentAppState.showProfilePickerSettings

    // Navigation and Settings need the active avatar/profile list even while the full profile
    // picker is closed. Subscribe to a deliberately small projection so avatar-pack discovery,
    // PIN errors and picker-only state do not recompose Home/Search/Detail.
    val navigationProfilesFlow = remember(profileStore) {
        profileStore?.state?.map { state ->
            ProfileUiState(
                activeProfile = state.activeProfile,
                profiles = state.profiles,
                isLoading = state.isLoading
            )
        }?.distinctUntilChanged()
    }
    val navigationProfileState = navigationProfilesFlow?.collectAsState(initial = ProfileUiState(isLoading = true))?.value
    val fullProfileState = if (needsProfile) profileStore?.state?.collectAsState()?.value else null
    val profileState = fullProfileState ?: navigationProfileState
    LaunchedEffect(profileDataSource) {
        profileDataSource?.refreshAllAvatarPacks()
    }

    val settingsStore = settingsDataSource?.let { source ->
        remember(source) { SettingsStore(source, scope) }
    }
    // Keep global chrome and Home appearance on small dedicated flows. Provider sync counters and
    // content-feed changes can be very chatty, so the full Settings model is collected only while
    // Settings itself is visible.
    val settingsAppearance = settingsStore?.appearance?.collectAsState()?.value
    val settingsAppearanceHome = if (currentAppState.destination == FluxaDestination.Home) {
        settingsStore?.appearanceHome?.collectAsState()?.value
    } else {
        null
    }
    val fullSettingsState = if (currentAppState.destination == FluxaDestination.Settings) {
        settingsStore?.state?.collectAsState()?.value
    } else {
        null
    }
    val settingsState = fullSettingsState ?: remember(settingsStore, settingsAppearance, settingsAppearanceHome) {
        settingsStore?.state?.value?.let { snapshot ->
            snapshot.copy(
                appearance = settingsAppearance ?: snapshot.appearance,
                appearanceHome = settingsAppearanceHome ?: snapshot.appearanceHome,
            )
        }
    }

    val addonStoreStore = addonStoreDataSource?.let { source ->
        remember(source) { AddonStoreStore(source, scope) }
    }
    val addonStoreState = if (currentAppState.destination == FluxaDestination.AddonStore) {
        addonStoreStore?.state?.collectAsState()?.value
    } else null

    val pluginsStore = pluginsDataSource?.let { source ->
        remember(source) { PluginsStore(source, scope) }
    }
    val pluginsState = if (currentAppState.destination == FluxaDestination.Plugins || currentAppState.destination == FluxaDestination.AddonStore) {
        pluginsStore?.state?.collectAsState()?.value
    } else null

    val authStore = authDataSource?.let { source ->
        remember(source) { AuthStore(source, scope) }
    }
    val authState = if (currentAppState.destination == FluxaDestination.Auth) {
        authStore?.state?.collectAsState()?.value
    } else null
    LaunchedEffect(appState.uiState.destination) {
        onDestinationChanged(appState.uiState.destination)
    }
    var profileAvatarUrl by remember(appState.uiState.editingProfile) {
        val target = appState.uiState.editingProfile
        val initial = (target as? ProfileEditTarget.Existing)?.let { existing ->
            profileState?.profiles?.firstOrNull { it.id == existing.id }?.avatarUrl
        }
        mutableStateOf(initial)
    }
    val selectedDetail = appState.uiState.selectedDetail
    val detailStore = selectedDetail?.let { request ->
        detailDataSource?.let { source ->
            remember(request, source) {
                DetailStore(request, source, scope)
            }
        }
    }
    val detailState = detailStore?.state?.collectAsState()?.value
    var pendingAutoPlayId by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(detailState?.content?.id, pendingAutoPlayId) {
        val contentId = detailState?.content?.id
        if (pendingAutoPlayId != null && contentId == pendingAutoPlayId) {
            detailStore?.dispatch(DetailAction.Play())
            pendingAutoPlayId = null
        }
    }

    LaunchedEffect(language) {
        appState.updateLanguage(language)
    }
    LaunchedEffect(destination) {
        if (destination != null && destination != appState.uiState.destination) {
            appState.selectDestination(destination)
        }
    }
    LaunchedEffect(detailRequest) {
        detailRequest?.let(appState::selectDetail)
    }
    LaunchedEffect(needsCatalogHome, catalogHomeStore) {
        if (needsCatalogHome) {
            catalogHomeStore.dispatch(CatalogAction.Refresh)
        }
    }
    LaunchedEffect(detailStore) {
        detailStore?.load()
    }
    LaunchedEffect(detailStore) {
        detailStore?.navigation?.collect { event ->
            when (event) {
                is com.fluxa.app.shared.feature.detail.DetailNavigationEvent.SelectSources -> {
                    appState.openSourceSelection()
                    detailStore.loadSources(event.episodeId)
                }
                is com.fluxa.app.shared.feature.detail.DetailNavigationEvent.PlayStream -> onDetailNavigationEvent(event)
            }
        }
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
    LaunchedEffect(appState.uiState.showNotifications, calendarStore) {
        if (appState.uiState.showNotifications) {
            calendarStore?.dispatch(CalendarAction.Refresh)
        }
    }
    LaunchedEffect(appState.uiState.destination, addonStoreStore) {
        if (appState.uiState.destination == FluxaDestination.AddonStore) {
            addonStoreStore?.dispatch(AddonStoreAction.Refresh)
        }
    }
    LaunchedEffect(appState.uiState.destination, pluginsStore) {
        if (appState.uiState.destination == FluxaDestination.Plugins || appState.uiState.destination == FluxaDestination.AddonStore) {
            pluginsStore?.dispatch(PluginsAction.Refresh)
        }
    }
    LaunchedEffect(authStartOnNuvio, authStore) {
        if (authStartOnNuvio) {
            authStore?.dispatch(AuthAction.ContinueWithNuvio)
        }
    }
    LaunchedEffect(appState.uiState.destination, settingsStore) {
        if (appState.uiState.destination == FluxaDestination.Settings) {
            settingsStore?.refreshContentFeeds()
        }
    }
    LaunchedEffect(appState.uiState.settingsBackStack) {
        onSettingsCanPopChanged(appState.uiState.settingsBackStack.isNotEmpty())
    }
    LaunchedEffect(settingsPopRequestId) {
        if (settingsPopRequestId > 0) {
            appState.popSettingsCategory()
        }
    }
    LaunchedEffect(
        appState.uiState.selectedDetail,
        appState.uiState.showSourceSelection,
        appState.uiState.selectedCategoryId,
        appState.uiState.showNotifications
    ) {
        val hasOverlay = appState.uiState.selectedDetail != null ||
            appState.uiState.showSourceSelection ||
            appState.uiState.selectedCategoryId != null ||
            appState.uiState.showNotifications
        onOverlayOpenChanged(hasOverlay)
    }
    LaunchedEffect(overlayPopRequestId) {
        if (overlayPopRequestId > 0) {
            when {
                appState.uiState.showSourceSelection -> appState.closeSourceSelection()
                appState.uiState.selectedDetail != null -> appState.clearDetail()
                appState.uiState.showNotifications -> appState.closeNotifications()
                appState.uiState.selectedCategoryId != null -> appState.clearCategory()
            }
        }
    }

    val catalogActionHandler: (CatalogAction) -> Unit = remember(
        appState,
        libraryState,
        libraryStore,
        catalogHomeStore,
        onCatalogAction,
        scope,
    ) {
        { action ->
            if (action is CatalogAction.ItemSelected) {
                if (action.item.type == "catalog_folder") {
                    libraryState?.collections?.flatMap { it.folders }
                        ?.firstOrNull { it.id == action.item.id }
                        ?.let { folder ->
                            scope.launch { libraryStore?.dispatch(LibraryAction.FolderSelected(folder)) }
                        }
                } else {
                    val resume = action.item.resume
                    when {
                        resume != null && !resume.streamUrl.isNullOrBlank() -> {
                            pendingAutoPlayId = null
                        }
                        resume != null -> {
                            pendingAutoPlayId = action.item.id
                            appState.selectDetail(action.item)
                        }
                        else -> {
                            pendingAutoPlayId = null
                            appState.selectDetail(action.item)
                        }
                    }
                }
            }
            if (action is CatalogAction.PlayRequested && action.item.type != "catalog_folder") {
                pendingAutoPlayId = action.item.id
                appState.selectDetail(action.item)
            }
            scope.launch { catalogHomeStore.dispatch(action) }
            onCatalogAction(action)
        }
    }

    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val widthClass = if (deviceType == com.fluxa.app.ui.catalog.DeviceType.TV) {
            com.fluxa.app.ui.catalog.WindowWidthClass.Expanded
        } else {
            com.fluxa.app.ui.catalog.widthClassFor(maxWidth)
        }
        CompositionLocalProvider(com.fluxa.app.ui.catalog.LocalWindowWidthClass provides widthClass) {
    FluxaApp(
        state = appState.uiState,
        catalogHome = catalogHome,
        features = FluxaAppFeatureStates(
            detail = detailState,
            search = searchState,
            discover = discoverState,
            calendar = calendarState,
            library = libraryState,
            profile = profileState,
            settings = settingsState,
            addonStore = addonStoreState,
            plugins = pluginsState,
            auth = authState,
        ),
        actions = FluxaAppActions(
            onDestinationSelected = appState::selectDestination,
            onCatalogAction = catalogActionHandler,
            onDetailAction = { action ->
                if (action is DetailAction.RelatedItemSelected) {
                    appState.selectDetail(action.item)
                    onCatalogAction(CatalogAction.ItemSelected(action.item))
                }
                scope.launch {
                    detailStore?.dispatch(action)
                }
            },
            onDetailBackRequested = {
                appState.clearDetail()
                onDetailBackRequested()
            },
            onSourceSelectionBackRequested = appState::closeSourceSelection,
            onCategoryBackRequested = appState::clearCategory,
            onCategoryItemSelected = { item ->
                appState.selectDetail(item)
                onCatalogAction(CatalogAction.ItemSelected(item))
            },
            onCategorySelected = { id, title -> appState.selectCategory(id, title) },
            onSearchAction = { action ->
                if (action is SearchAction.ItemSelected) {
                    appState.selectDetail(action.item)
                    onCatalogAction(CatalogAction.ItemSelected(action.item))
                }
                scope.launch {
                    searchStore?.dispatch(action)
                }
            },
            onDiscoverAction = { action ->
                if (action is DiscoverAction.ItemSelected) {
                    appState.selectDetail(action.item)
                    onCatalogAction(CatalogAction.ItemSelected(action.item))
                }
                scope.launch {
                    discoverStore?.dispatch(action)
                }
            },
            onCalendarAction = { action ->
                if (action is CalendarAction.ItemSelected) {
                    appState.selectDetail(action.item)
                    onCatalogAction(CatalogAction.ItemSelected(action.item))
                }
                scope.launch {
                    calendarStore?.dispatch(action)
                }
            },
            onNotificationsRequested = { appState.openNotifications() },
            onNotificationsBackRequested = { appState.closeNotifications() },
            onLibraryItemSelected = { item ->
                if (item.type == "catalog_folder") {
                    libraryState?.collections?.flatMap { it.folders }
                        ?.firstOrNull { it.id == item.id }
                        ?.let { folder ->
                            scope.launch { libraryStore?.dispatch(LibraryAction.FolderSelected(folder)) }
                        }
                } else {
                    appState.selectDetail(item)
                }
                onCatalogAction(CatalogAction.ItemSelected(item))
            },
            onLibraryAction = { action ->
                when (action) {
                    is LibraryAction.DownloadOpened -> onDownloadOpened(action.id)
                    is LibraryAction.LocalMediaFolderPickerRequested -> {
                        libraryCallbacks.onPickLocalMediaFolderRequested(action.kind) { picked ->
                            if (picked != null) {
                                scope.launch {
                                    libraryStore?.dispatch(
                                        LibraryAction.LocalMediaSourceAdded(
                                            com.fluxa.app.shared.feature.localmedia.LocalMediaSourceInput(
                                                kind = action.kind,
                                                sourceType = com.fluxa.app.shared.feature.localmedia.LocalMediaSourceType.LocalFolder,
                                                location = picked.location,
                                                displayName = picked.displayName,
                                            )
                                        )
                                    )
                                    libraryStore?.dispatch(LibraryAction.LocalMediaScanRequested())
                                }
                            }
                        }
                    }
                    else -> scope.launch { libraryStore?.dispatch(action) }
                }
            },
            onSettingsAction = { action ->
                when (action) {
                    SettingsAction.SwitchProfilesRequested -> appState.selectDestination(FluxaDestination.ProfileList)
                    SettingsAction.ManageAddonsRequested -> onManageAddonsRequested()
                    SettingsAction.ManagePluginsRequested -> onManagePluginsRequested()
                    SettingsAction.ConnectStremioRequested -> onConnectStremioRequested()
                    SettingsAction.ConnectNuvioRequested -> onConnectNuvioRequested()
                    is SettingsAction.ConnectStremioWithCredentials -> onConnectStremioWithCredentials(action.email, action.password)
                    is SettingsAction.ConnectNuvioWithCredentials -> onConnectNuvioWithCredentials(action.email, action.password)
                    SettingsAction.ConnectTraktRequested -> onConnectTraktRequested()
                    SettingsAction.ConnectSimklRequested -> onConnectSimklRequested()
                    is SettingsAction.SyncProviderRequested -> onSyncProviderRequested(action.provider)
                    SettingsAction.ConnectAnilistRequested -> onConnectAnilistRequested()
                    SettingsAction.CheckForUpdateRequested -> onCheckForUpdateRequested()
                    else -> scope.launch { settingsStore?.dispatch(action) }
                }
            },
            onSwitchProfilesRequested = { appState.selectDestination(FluxaDestination.ProfileList) },
            onSettingsBackRequested = onSettingsBackRequested,
            onSettingsPushCategory = appState::pushSettingsCategory,
            onSettingsPopCategory = appState::popSettingsCategory,
            onSettingsSelectCategory = appState::selectSettingsCategory,
            onAddonStoreAction = { action ->
                scope.launch {
                    addonStoreStore?.dispatch(action)
                }
            },
            onOpenUrlRequested = onOpenUrlRequested,
            onAddonStoreBackRequested = onAddonStoreBackRequested,
            onPluginsAction = { action ->
                scope.launch {
                    pluginsStore?.dispatch(action)
                }
            },
            onPluginsBackRequested = onPluginsBackRequested,
            onAuthAction = { action ->
                when (action) {
                    AuthAction.BackRequested -> onAuthBackRequested()
                    AuthAction.Completed -> onAuthCompleted()
                    else -> scope.launch { authStore?.dispatch(action) }
                }
            },
            onProfileListAction = { action ->
                when (action) {
                    ProfileAction.AddRequested -> {
                        profileAvatarUrl = null
                        appState.beginProfileEdit(ProfileEditTarget.New)
                    }
                    is ProfileAction.EditRequested -> {
                        profileAvatarUrl = action.profile.avatarUrl
                        appState.beginProfileEdit(ProfileEditTarget.Existing(action.profile.id))
                    }
                    ProfileAction.PickerSettingsRequested -> appState.openProfilePickerSettings()
                    ProfileAction.PickerSettingsClosed -> appState.closeProfilePickerSettings()
                    is ProfileAction.Selected -> scope.launch {
                        profileStore?.dispatch(action)
                        if (!action.profile.hasPin) onProfileSelectionCompleted(action.profile.id)
                    }
                    else -> scope.launch { profileStore?.dispatch(action) }
                }
            },
            onProfileBiometricRequested = { profile ->
                onBiometricAuthRequested(profile) { success ->
                    if (success) {
                        scope.launch {
                            profileDataSource?.confirmBiometricUnlock(profile.id)
                            onProfileSelectionCompleted(profile.id)
                        }
                    }
                }
            },
        ),
        profileEditor = FluxaProfileEditorBindings(
            avatarUrl = profileAvatarUrl,
            onPickAvatarClick = { onPickAvatarRequested { url -> profileAvatarUrl = url } },
            onRemoveAvatarClick = { profileAvatarUrl = null },
            onPickPackAvatarClick = { url -> profileAvatarUrl = url },
            onProfileSave = { edit ->
                scope.launch {
                    profileStore?.saveProfile(edit)
                    appState.beginProfileEdit(null)
                }
            },
            onProfileDelete = (appState.uiState.editingProfile as? ProfileEditTarget.Existing)?.let { existing ->
                { pin ->
                    val deleted = profileStore?.deleteProfile(existing.id, pin) == true
                    if (deleted) {
                        appState.beginProfileEdit(null)
                    }
                    deleted
                }
            },
            onProfileEditCancel = { appState.beginProfileEdit(null) },
            onPickBackgroundClick = {
                onPickAvatarRequested { url -> scope.launch { profileStore?.dispatch(ProfileAction.BackgroundUrlChanged(url)) } }
            },
        ),
        presentation = FluxaAppPresentation(
            deviceType = deviceType,
            isLowRamDevice = isLowRamDevice,
            settingsBrandIcons = com.fluxa.app.shared.feature.settings.SettingsBrandIcons(
                stremio = stremioIcon,
                nuvio = nuvioIcon,
                trakt = traktIcon,
                simkl = simklIcon,
                anilist = anilistIcon
            ),
            nuvioIcon = nuvioIcon,
            stremioIcon = stremioIcon,
            authBackdrop = authBackdrop,
            biometricAvailable = biometricAvailable,
            showNavigationBar = showNavigationBar,
        ),
        modifier = modifier,
    )
        }
    }
}
