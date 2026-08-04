package com.fluxa.app.shared

import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import com.fluxa.app.shared.feature.addonstore.AddonStoreAction
import com.fluxa.app.shared.feature.addonstore.AddonStoreDataSource
import com.fluxa.app.shared.feature.addonstore.AddonStoreStore
import com.fluxa.app.shared.feature.plugins.PluginsAction
import com.fluxa.app.shared.feature.plugins.PluginsDataSource
import com.fluxa.app.shared.feature.plugins.PluginsStore
import com.fluxa.app.shared.feature.auth.AuthAction
import com.fluxa.app.shared.feature.auth.AuthDataSource
import com.fluxa.app.shared.feature.auth.AuthStore
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
import com.fluxa.app.shared.feature.profile.ProfileAction
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.profile.ProfileEditTarget
import com.fluxa.app.shared.feature.profile.ProfileEditUiModel
import com.fluxa.app.shared.feature.profile.ProfileStore
import com.fluxa.app.shared.feature.profile.ProfileUiModel
import com.fluxa.app.shared.feature.settings.SettingsAction
import com.fluxa.app.shared.feature.settings.SettingsDataSource
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
import kotlinx.coroutines.launch

@Composable
fun FluxaAppHost(
    platformServices: FluxaPlatformServices,
    deviceType: com.fluxa.app.ui.catalog.DeviceType = com.fluxa.app.ui.catalog.DeviceType.Mobile,
    language: String? = null,
    onCatalogAction: (CatalogAction) -> Unit = {},
    destination: FluxaDestination? = null,
    detailRequest: DetailRequestUiModel? = null,
    onDetailNavigationEvent: (com.fluxa.app.shared.feature.detail.DetailNavigationEvent) -> Unit = {},
    onDetailBackRequested: () -> Unit = {},
    showNavigationBar: Boolean = true,
    onOpenUrlRequested: (String) -> Unit = {},
    onAddonStoreBackRequested: () -> Unit = {},
    onPluginsBackRequested: () -> Unit = {},
    onAuthBackRequested: () -> Unit = {},
    onAuthCompleted: () -> Unit = {},
    authStartOnNuvio: Boolean = false,
    nuvioIcon: @Composable () -> Unit = {},
    stremioIcon: @Composable () -> Unit = {},
    traktIcon: @Composable () -> Unit = {},
    simklIcon: @Composable () -> Unit = {},
    anilistIcon: @Composable () -> Unit = {},
    biometricAvailable: Boolean = false,
    onPickAvatarRequested: (onPicked: (String?) -> Unit) -> Unit = {},
    onBiometricAuthRequested: (ProfileUiModel, onResult: (Boolean) -> Unit) -> Unit = { _, _ -> },
    onProfileSelectionCompleted: (String) -> Unit = {},
    onManageAddonsRequested: () -> Unit = {},
    onManagePluginsRequested: () -> Unit = {},
    onConnectStremioRequested: () -> Unit = {},
    onConnectNuvioRequested: () -> Unit = {},
    onConnectStremioWithCredentials: (String, String) -> Unit = { _, _ -> },
    onConnectNuvioWithCredentials: (String, String) -> Unit = { _, _ -> },
    onConnectTraktRequested: () -> Unit = {},
    onConnectSimklRequested: () -> Unit = {},
    onConnectAnilistRequested: () -> Unit = {},
    onCheckForUpdateRequested: () -> Unit = {},
    onDownloadOpened: (String) -> Unit = {},
    onSettingsBackRequested: () -> Unit = {},
    settingsPopRequestId: Int = 0,
    onSettingsCanPopChanged: (Boolean) -> Unit = {},
    overlayPopRequestId: Int = 0,
    onOverlayOpenChanged: (Boolean) -> Unit = {},
    onDestinationChanged: (FluxaDestination) -> Unit = {},
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
        pluginsDataSource = (platformServices as? FluxaPluginsServices)?.pluginsDataSource,
        authDataSource = (platformServices as? FluxaAuthServices)?.authDataSource,
        settingsDataSource = (platformServices as? FluxaSettingsServices)?.settingsDataSource,
        deviceType = deviceType,
        language = language,
        onCatalogAction = onCatalogAction,
        destination = destination,
        detailRequest = detailRequest,
        onDetailNavigationEvent = onDetailNavigationEvent,
        onDetailBackRequested = onDetailBackRequested,
        showNavigationBar = showNavigationBar,
        onOpenUrlRequested = onOpenUrlRequested,
        onAddonStoreBackRequested = onAddonStoreBackRequested,
        onPluginsBackRequested = onPluginsBackRequested,
        onAuthBackRequested = onAuthBackRequested,
        onAuthCompleted = onAuthCompleted,
        authStartOnNuvio = authStartOnNuvio,
        nuvioIcon = nuvioIcon,
        stremioIcon = stremioIcon,
        traktIcon = traktIcon,
        simklIcon = simklIcon,
        anilistIcon = anilistIcon,
        biometricAvailable = biometricAvailable,
        onPickAvatarRequested = onPickAvatarRequested,
        onBiometricAuthRequested = onBiometricAuthRequested,
        onProfileSelectionCompleted = onProfileSelectionCompleted,
        onManageAddonsRequested = onManageAddonsRequested,
        onManagePluginsRequested = onManagePluginsRequested,
        onConnectStremioRequested = onConnectStremioRequested,
        onConnectNuvioRequested = onConnectNuvioRequested,
        onConnectStremioWithCredentials = onConnectStremioWithCredentials,
        onConnectNuvioWithCredentials = onConnectNuvioWithCredentials,
        onConnectTraktRequested = onConnectTraktRequested,
        onConnectSimklRequested = onConnectSimklRequested,
        onConnectAnilistRequested = onConnectAnilistRequested,
        onCheckForUpdateRequested = onCheckForUpdateRequested,
        onDownloadOpened = onDownloadOpened,
        onSettingsBackRequested = onSettingsBackRequested,
        settingsPopRequestId = settingsPopRequestId,
        onSettingsCanPopChanged = onSettingsCanPopChanged,
        overlayPopRequestId = overlayPopRequestId,
        onOverlayOpenChanged = onOverlayOpenChanged,
        onDestinationChanged = onDestinationChanged,
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
    pluginsDataSource: PluginsDataSource? = null,
    authDataSource: AuthDataSource? = null,
    settingsDataSource: SettingsDataSource? = null,
    deviceType: com.fluxa.app.ui.catalog.DeviceType = com.fluxa.app.ui.catalog.DeviceType.Mobile,
    language: String? = null,
    onCatalogAction: (CatalogAction) -> Unit = {},
    destination: FluxaDestination? = null,
    detailRequest: DetailRequestUiModel? = null,
    onDetailNavigationEvent: (com.fluxa.app.shared.feature.detail.DetailNavigationEvent) -> Unit = {},
    onDetailBackRequested: () -> Unit = {},
    showNavigationBar: Boolean = true,
    onOpenUrlRequested: (String) -> Unit = {},
    onAddonStoreBackRequested: () -> Unit = {},
    onPluginsBackRequested: () -> Unit = {},
    onAuthBackRequested: () -> Unit = {},
    onAuthCompleted: () -> Unit = {},
    authStartOnNuvio: Boolean = false,
    nuvioIcon: @Composable () -> Unit = {},
    stremioIcon: @Composable () -> Unit = {},
    traktIcon: @Composable () -> Unit = {},
    simklIcon: @Composable () -> Unit = {},
    anilistIcon: @Composable () -> Unit = {},
    biometricAvailable: Boolean = false,
    onPickAvatarRequested: (onPicked: (String?) -> Unit) -> Unit = {},
    onBiometricAuthRequested: (ProfileUiModel, onResult: (Boolean) -> Unit) -> Unit = { _, _ -> },
    onProfileSelectionCompleted: (String) -> Unit = {},
    onManageAddonsRequested: () -> Unit = {},
    onManagePluginsRequested: () -> Unit = {},
    onConnectStremioRequested: () -> Unit = {},
    onConnectNuvioRequested: () -> Unit = {},
    onConnectStremioWithCredentials: (String, String) -> Unit = { _, _ -> },
    onConnectNuvioWithCredentials: (String, String) -> Unit = { _, _ -> },
    onConnectTraktRequested: () -> Unit = {},
    onConnectSimklRequested: () -> Unit = {},
    onConnectAnilistRequested: () -> Unit = {},
    onCheckForUpdateRequested: () -> Unit = {},
    onDownloadOpened: (String) -> Unit = {},
    onSettingsBackRequested: () -> Unit = {},
    settingsPopRequestId: Int = 0,
    onSettingsCanPopChanged: (Boolean) -> Unit = {},
    overlayPopRequestId: Int = 0,
    onOverlayOpenChanged: (Boolean) -> Unit = {},
    onDestinationChanged: (FluxaDestination) -> Unit = {},
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
    LaunchedEffect(profileDataSource) {
        profileDataSource?.refreshAllAvatarPacks()
    }
    val settingsStore = settingsDataSource?.let { source ->
        remember(source) { SettingsStore(source, scope) }
    }
    val settingsState = settingsStore?.state?.collectAsState()?.value
    val addonStoreStore = addonStoreDataSource?.let { source ->
        remember(source) { AddonStoreStore(source, scope) }
    }
    val addonStoreState = addonStoreStore?.state?.collectAsState()?.value
    val pluginsStore = pluginsDataSource?.let { source ->
        remember(source) { PluginsStore(source, scope) }
    }
    val pluginsState = pluginsStore?.state?.collectAsState()?.value
    val authStore = authDataSource?.let { source ->
        remember(source) { AuthStore(source, scope) }
    }
    val authState = authStore?.state?.collectAsState()?.value
    val appState = rememberFluxaAppState(
        FluxaAppUiState(
            language = language,
            destination = destination ?: FluxaDestination.Home,
            catalogHome = catalogHome
        )
    )
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

    LaunchedEffect(catalogHome) {
        appState.updateCatalogHome(catalogHome)
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
    LaunchedEffect(catalogHomeStore) {
        catalogHomeStore.dispatch(CatalogAction.Refresh)
    }
    LaunchedEffect(calendarStore) {
        calendarStore?.dispatch(CalendarAction.Refresh)
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
        if (appState.uiState.destination == FluxaDestination.Plugins) {
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

    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val widthClass = if (deviceType == com.fluxa.app.ui.catalog.DeviceType.TV) {
            com.fluxa.app.ui.catalog.WindowWidthClass.Expanded
        } else {
            com.fluxa.app.ui.catalog.widthClassFor(maxWidth)
        }
        CompositionLocalProvider(com.fluxa.app.ui.catalog.LocalWindowWidthClass provides widthClass) {
    FluxaApp(
        state = appState.uiState,
        deviceType = deviceType,
        onDestinationSelected = appState::selectDestination,
        onCatalogAction = { action ->
            if (action is CatalogAction.ItemSelected) {
                if (action.item.type == "catalog_folder") {
                    libraryState?.collections?.flatMap { it.folders }
                        ?.firstOrNull { it.id == action.item.id }
                        ?.let { folder ->
                            scope.launch { libraryStore.dispatch(LibraryAction.FolderSelected(folder)) }
                        }
                } else {
                    pendingAutoPlayId = null
                    appState.selectDetail(action.item)
                }
            }
            if (action is CatalogAction.PlayRequested && action.item.type != "catalog_folder") {
                pendingAutoPlayId = action.item.id
                appState.selectDetail(action.item)
            }
            scope.launch {
                catalogHomeStore.dispatch(action)
            }
            onCatalogAction(action)
        },
        onCategorySelected = { id, title -> appState.selectCategory(id, title) },
        detailState = detailState,
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
        onNotificationsRequested = { appState.openNotifications() },
        onNotificationsBackRequested = { appState.closeNotifications() },
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
            if (item.type == "catalog_folder") {
                libraryState?.collections?.flatMap { it.folders }
                    ?.firstOrNull { it.id == item.id }
                    ?.let { folder ->
                        scope.launch { libraryStore.dispatch(LibraryAction.FolderSelected(folder)) }
                    }
            } else {
                appState.selectDetail(item)
            }
            onCatalogAction(CatalogAction.ItemSelected(item))
        },
        onLibraryAction = { action ->
            if (action is LibraryAction.DownloadOpened) {
                onDownloadOpened(action.id)
            } else {
                scope.launch { libraryStore?.dispatch(action) }
            }
        },
        profileState = profileState,
        settingsState = settingsState,
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
        settingsBrandIcons = com.fluxa.app.shared.feature.settings.SettingsBrandIcons(
            stremio = stremioIcon,
            nuvio = nuvioIcon,
            trakt = traktIcon,
            simkl = simklIcon,
            anilist = anilistIcon
        ),
        addonStoreState = addonStoreState,
        onAddonStoreAction = { action ->
            scope.launch {
                addonStoreStore?.dispatch(action)
            }
        },
        onOpenUrlRequested = onOpenUrlRequested,
        onAddonStoreBackRequested = onAddonStoreBackRequested,
        pluginsState = pluginsState,
        onPluginsAction = { action ->
            scope.launch {
                pluginsStore?.dispatch(action)
            }
        },
        onPluginsBackRequested = onPluginsBackRequested,
        authState = authState,
        onAuthAction = { action ->
            when (action) {
                AuthAction.BackRequested -> onAuthBackRequested()
                AuthAction.Completed -> onAuthCompleted()
                else -> scope.launch { authStore?.dispatch(action) }
            }
        },
        nuvioIcon = nuvioIcon,
        stremioIcon = stremioIcon,
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
        profileEditAvatarUrl = profileAvatarUrl,
        onPickAvatarClick = { onPickAvatarRequested { url -> profileAvatarUrl = url } },
        onRemoveAvatarClick = { profileAvatarUrl = null },
        onPickPackAvatarClick = { url -> profileAvatarUrl = url },
        onPickBackgroundClick = {
            onPickAvatarRequested { url -> scope.launch { profileStore?.dispatch(ProfileAction.BackgroundUrlChanged(url)) } }
        },
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
        biometricAvailable = biometricAvailable,
        showNavigationBar = showNavigationBar,
        modifier = modifier
    )
        }
    }
}
