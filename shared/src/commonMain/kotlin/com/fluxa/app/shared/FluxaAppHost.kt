package com.fluxa.app.shared

import androidx.compose.runtime.Composable
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
    onConnectStremioRequested: () -> Unit = {},
    onConnectNuvioRequested: () -> Unit = {},
    onConnectTraktRequested: () -> Unit = {},
    onConnectSimklRequested: () -> Unit = {},
    onConnectAnilistRequested: () -> Unit = {},
    onCheckForUpdateRequested: () -> Unit = {},
    onDownloadOpened: (String) -> Unit = {},
    onSettingsBackRequested: () -> Unit = {},
    settingsPopRequestId: Int = 0,
    onSettingsCanPopChanged: (Boolean) -> Unit = {},
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
        onConnectStremioRequested = onConnectStremioRequested,
        onConnectNuvioRequested = onConnectNuvioRequested,
        onConnectTraktRequested = onConnectTraktRequested,
        onConnectSimklRequested = onConnectSimklRequested,
        onConnectAnilistRequested = onConnectAnilistRequested,
        onCheckForUpdateRequested = onCheckForUpdateRequested,
        onDownloadOpened = onDownloadOpened,
        onSettingsBackRequested = onSettingsBackRequested,
        settingsPopRequestId = settingsPopRequestId,
        onSettingsCanPopChanged = onSettingsCanPopChanged,
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
    onConnectStremioRequested: () -> Unit = {},
    onConnectNuvioRequested: () -> Unit = {},
    onConnectTraktRequested: () -> Unit = {},
    onConnectSimklRequested: () -> Unit = {},
    onConnectAnilistRequested: () -> Unit = {},
    onCheckForUpdateRequested: () -> Unit = {},
    onDownloadOpened: (String) -> Unit = {},
    onSettingsBackRequested: () -> Unit = {},
    settingsPopRequestId: Int = 0,
    onSettingsCanPopChanged: (Boolean) -> Unit = {},
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
    val settingsStore = settingsDataSource?.let { source ->
        remember(source) { SettingsStore(source, scope) }
    }
    val settingsState = settingsStore?.state?.collectAsState()?.value
    val addonStoreStore = addonStoreDataSource?.let { source ->
        remember(source) { AddonStoreStore(source, scope) }
    }
    val addonStoreState = addonStoreStore?.state?.collectAsState()?.value
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
        destination?.let(appState::selectDestination)
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
                SettingsAction.ConnectStremioRequested -> onConnectStremioRequested()
                SettingsAction.ConnectNuvioRequested -> onConnectNuvioRequested()
                SettingsAction.ConnectTraktRequested -> onConnectTraktRequested()
                SettingsAction.ConnectSimklRequested -> onConnectSimklRequested()
                SettingsAction.ConnectAnilistRequested -> onConnectAnilistRequested()
                SettingsAction.CheckForUpdateRequested -> onCheckForUpdateRequested()
                SettingsAction.ManageDownloadsRequested -> appState.openLibraryDownloads()
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
        onProfileSave = { edit ->
            scope.launch {
                profileStore?.saveProfile(edit)
                appState.beginProfileEdit(null)
            }
        },
        onProfileDelete = (appState.uiState.editingProfile as? ProfileEditTarget.Existing)?.let { existing ->
            {
                scope.launch {
                    profileStore?.deleteProfile(existing.id)
                    appState.beginProfileEdit(null)
                }
                Unit
            }
        },
        onProfileEditCancel = { appState.beginProfileEdit(null) },
        biometricAvailable = biometricAvailable,
        showNavigationBar = showNavigationBar,
        modifier = modifier
    )
}
