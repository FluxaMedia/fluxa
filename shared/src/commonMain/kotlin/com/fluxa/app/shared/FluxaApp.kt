package com.fluxa.app.shared

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.focus.focusRestorer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import dev.chrisbanes.haze.hazeSource
import dev.chrisbanes.haze.rememberHazeState
import androidx.compose.ui.draw.alpha
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CategoryResultsScreen
import com.fluxa.app.shared.feature.addonstore.AddonStoreScreen
import com.fluxa.app.shared.feature.auth.AuthScreen
import com.fluxa.app.shared.feature.calendar.CalendarAction
import com.fluxa.app.shared.feature.calendar.CalendarScreen
import com.fluxa.app.shared.feature.calendar.NotificationsScreen
import com.fluxa.app.shared.feature.detail.DetailAction
import com.fluxa.app.shared.feature.detail.DetailRequestUiModel
import com.fluxa.app.shared.feature.detail.DetailPresentationOptions
import com.fluxa.app.shared.feature.detail.DetailScreenStyle
import com.fluxa.app.shared.feature.detail.DetailScreen
import com.fluxa.app.shared.feature.detail.SourceSelectionScreen
import com.fluxa.app.shared.feature.discover.DiscoverAction
import com.fluxa.app.shared.feature.discover.DiscoverScreen
import com.fluxa.app.shared.feature.library.LibraryAction
import com.fluxa.app.shared.feature.library.LibraryFolderDetailScreen
import com.fluxa.app.shared.feature.library.LibraryScreen
import com.fluxa.app.shared.feature.plugins.PluginsScreen
import com.fluxa.app.shared.feature.profile.ProfileAction
import com.fluxa.app.shared.feature.profile.ProfileEditScreen
import com.fluxa.app.shared.feature.profile.ProfileEditTarget
import com.fluxa.app.shared.feature.profile.ProfileListScreen
import com.fluxa.app.shared.feature.profile.ProfilePickerSettingsScreen
import com.fluxa.app.shared.feature.settings.SettingsScreen
import com.fluxa.app.shared.feature.search.SearchAction
import com.fluxa.app.shared.feature.search.SearchScreen
import com.fluxa.app.shared.feature.player.PlayerControlsSurface
import com.fluxa.app.ui.catalog.FluxaColors

private val FluxaColorScheme = darkColorScheme(
    background = FluxaColors.background,
    surface = FluxaColors.surface,
    surfaceVariant = FluxaColors.surfaceCard,
    primary = Color.White,
    secondary = Color.White,
    onBackground = Color.White,
    onSurface = Color.White,
    onSurfaceVariant = Color.White.copy(alpha = 0.85f),
    onPrimary = Color.Black,
    onSecondary = Color.Black,
    outline = Color.White.copy(alpha = 0.3f),
    error = FluxaColors.errorRed
)

enum class FluxaDestination(val titleKey: String) {
    Home("nav.home"),
    Search("auto.search"),
    Discover("nav.discover"),
    Calendar("nav.calendar"),
    Library("nav.library"),
    Settings("nav.settings"),
    AddonStore("auto.addons"),
    Plugins("settings.plugins.title"),
    Auth("auth.log_in"),
    ProfileList("auto.profile")
}

data class FluxaAppUiState(
    val language: String? = null,
    val destination: FluxaDestination = FluxaDestination.Home,
    val selectedDetail: DetailRequestUiModel? = null,
    val showSourceSelection: Boolean = false,
    val selectedCategoryId: String? = null,
    val selectedCategoryTitle: String? = null,
    val editingProfile: ProfileEditTarget? = null,
    val showProfilePickerSettings: Boolean = false,
    val showNotifications: Boolean = false,
    val settingsBackStack: List<com.fluxa.app.shared.feature.settings.SettingsCategory> = emptyList(),
    val initialLibrarySection: com.fluxa.app.shared.feature.library.LibrarySection? = null
)

@Composable
internal fun FluxaApp(
    state: FluxaAppUiState,
    catalogHome: CatalogHomeUiState,
    features: FluxaAppFeatureStates,
    actions: FluxaAppActions,
    profileEditor: FluxaProfileEditorBindings = FluxaProfileEditorBindings(),
    presentation: FluxaAppPresentation = FluxaAppPresentation(),
    modifier: Modifier = Modifier,
) {
    val deviceType = presentation.deviceType
    val isLowRamDevice = presentation.isLowRamDevice
    val settingsBrandIcons = presentation.settingsBrandIcons
    val nuvioIcon = presentation.nuvioIcon
    val stremioIcon = presentation.stremioIcon
    val authBackdrop = presentation.authBackdrop
    val biometricAvailable = presentation.biometricAvailable
    val showNavigationBar = presentation.showNavigationBar

    val detailState = features.detail
    val searchState = features.search
    val discoverState = features.discover
    val calendarState = features.calendar
    val libraryState = features.library
    val profileState = features.profile
    val settingsState = features.settings
    val addonStoreState = features.addonStore
    val pluginsState = features.plugins
    val authState = features.auth
    val playerState = features.player

    val onDestinationSelected = actions.onDestinationSelected
    val onCatalogAction = actions.onCatalogAction
    val onDetailAction = actions.onDetailAction
    val onDetailBackRequested = actions.onDetailBackRequested
    val onSourceSelectionBackRequested = actions.onSourceSelectionBackRequested
    val onCategoryBackRequested = actions.onCategoryBackRequested
    val onCategoryItemSelected = actions.onCategoryItemSelected
    val onCategorySelected = actions.onCategorySelected
    val onSearchAction = actions.onSearchAction
    val onDiscoverAction = actions.onDiscoverAction
    val onCalendarAction = actions.onCalendarAction
    val onNotificationsRequested = actions.onNotificationsRequested
    val onNotificationsBackRequested = actions.onNotificationsBackRequested
    val onLibraryItemSelected = actions.onLibraryItemSelected
    val onLibraryAction = actions.onLibraryAction
    val onSettingsAction = actions.onSettingsAction
    val onSwitchProfilesRequested = actions.onSwitchProfilesRequested
    val onSettingsBackRequested = actions.onSettingsBackRequested
    val onSettingsPushCategory = actions.onSettingsPushCategory
    val onSettingsPopCategory = actions.onSettingsPopCategory
    val onSettingsSelectCategory = actions.onSettingsSelectCategory
    val onAddonStoreAction = actions.onAddonStoreAction
    val onOpenUrlRequested = actions.onOpenUrlRequested
    val onAddonStoreBackRequested = actions.onAddonStoreBackRequested
    val onPluginsAction = actions.onPluginsAction
    val onPluginsBackRequested = actions.onPluginsBackRequested
    val onAuthAction = actions.onAuthAction
    val onProfileListAction = actions.onProfileListAction
    val onProfileBiometricRequested = actions.onProfileBiometricRequested
    val onPlayerAction = actions.onPlayerAction

    val profileEditAvatarUrl = profileEditor.avatarUrl
    val onPickAvatarClick = profileEditor.onPickAvatarClick
    val onRemoveAvatarClick = profileEditor.onRemoveAvatarClick
    val onPickPackAvatarClick = profileEditor.onPickPackAvatarClick
    val onProfileSave = profileEditor.onProfileSave
    val onProfileDelete = profileEditor.onProfileDelete
    val onProfileEditCancel = profileEditor.onProfileEditCancel
    val onPickBackgroundClick = profileEditor.onPickBackgroundClick

    MaterialTheme(colorScheme = FluxaColorScheme) {
        val isTv = deviceType == com.fluxa.app.ui.catalog.DeviceType.TV
        val isDesktop = deviceType == com.fluxa.app.ui.catalog.DeviceType.Desktop
        // TV never captures a full-screen blur source. Mobile only does it while the
        // bottom navigation is actually visible and consuming the haze source.
        val liquidGlassRequested = !isTv && settingsState?.appearance?.liquidGlassMode == true
        val hazeState = rememberHazeState()
        val accentColor = profileState?.activeProfile?.accentColorArgb?.let { Color(it) } ?: FluxaColors.accent
        androidx.compose.runtime.CompositionLocalProvider(com.fluxa.app.ui.catalog.LocalAccentColor provides accentColor) {
        Box(
            modifier = modifier
                .fillMaxSize()
                .background(FluxaColors.background)
        ) {
            val screenKey = when {
                playerState?.content != null -> "player:${playerState.content?.id.orEmpty()}"
                state.editingProfile != null -> "profileEdit"
                state.showProfilePickerSettings -> "profilePickerSettings"
                state.showSourceSelection -> "sources"
                state.selectedDetail != null -> "detail:${state.selectedDetail.id}"
                libraryState?.folderDetail?.folder != null -> "folder:${libraryState.folderDetail.folder.id}"
                state.selectedCategoryId != null -> "category:${state.selectedCategoryId}"
                state.showNotifications -> "notifications"
                else -> "dest:${state.destination}"
            }
            val tvRouteModifier = if (isTv) {
                Modifier
                    .padding(horizontal = 40.dp, vertical = 28.dp)
                    .focusRestorer()
            } else {
                Modifier
            }
            val widthClass = com.fluxa.app.ui.catalog.LocalWindowWidthClass.current
            val topNavEnabled = settingsState?.appearance?.topNavigationBar == true &&
                (isTv || widthClass != com.fluxa.app.ui.catalog.WindowWidthClass.Compact)
            val useRail = !isTv && !topNavEnabled &&
                widthClass == com.fluxa.app.ui.catalog.WindowWidthClass.Expanded
            val isHomeActive = screenKey == "dest:${FluxaDestination.Home}" && !isTv
            val isPreAuthDestination = state.destination == FluxaDestination.Auth ||
                state.destination == FluxaDestination.ProfileList
            val navChromeVisible = showNavigationBar && !isPreAuthDestination
            val useTopNav = navChromeVisible && topNavEnabled
            val useBottomNavigation = navChromeVisible && !isTv && !useTopNav && !useRail
            val liquidGlassMode = liquidGlassRequested && useBottomNavigation
            var navBarHeightPx by remember { mutableIntStateOf(0) }
            var tvSidebarWidthPx by remember { mutableIntStateOf(0) }
            var navRailWidthPx by remember { mutableIntStateOf(0) }
            var topNavHeightPx by remember { mutableIntStateOf(0) }
            val density = LocalDensity.current
            val navBarHeightDp = with(density) { navBarHeightPx.toDp() }
            val tvSidebarWidthDp = with(density) { tvSidebarWidthPx.toDp() }
            val navRailWidthDp = with(density) { navRailWidthPx.toDp() }
            val topNavHeightDp = with(density) { topNavHeightPx.toDp() }
            val saveableStateHolder = androidx.compose.runtime.saveable.rememberSaveableStateHolder()
            AnimatedContent(
                targetState = screenKey,
                transitionSpec = {
                    when {
                        isTv && isLowRamDevice -> EnterTransition.None togetherWith ExitTransition.None
                        isTv -> fadeIn(tween(100)).togetherWith(fadeOut(tween(80)))
                        isDesktop -> {
                            // Desktop windows can contain several bitmap-heavy rows at once. Keep
                            // route overlap short so Skia does not render two full scenes for long.
                            fadeIn(tween(90)).togetherWith(fadeOut(tween(70)))
                        }
                        else -> {
                            // Avoid scaling two full bitmap-heavy routes at once on mobile.
                            fadeIn(tween(140)).togetherWith(fadeOut(tween(100)))
                        }
                    }
                },
                label = "fluxa-screen-transition",
                modifier = (
                    if (!navChromeVisible) {
                        Modifier.fillMaxSize()
                    } else if (useTopNav) {
                        Modifier.fillMaxSize().padding(top = topNavHeightDp)
                    } else if (isTv) {
                        Modifier.fillMaxSize().padding(start = tvSidebarWidthDp)
                    } else if (useRail) {
                        Modifier.fillMaxSize().padding(start = navRailWidthDp)
                    } else if (isHomeActive) {
                        Modifier.fillMaxSize()
                    } else {
                        Modifier.fillMaxSize().padding(bottom = navBarHeightDp)
                    }
                    ).then(if (liquidGlassMode) Modifier.hazeSource(hazeState) else Modifier)
            ) { key ->
            saveableStateHolder.SaveableStateProvider(key) {
            when {
                playerState?.content != null -> PlayerControlsSurface(
                    state = playerState,
                    language = state.language,
                    onAction = onPlayerAction,
                    modifier = Modifier.fillMaxSize()
                )
                state.editingProfile != null && profileState != null -> ProfileEditScreen(
                    initialProfile = (state.editingProfile as? ProfileEditTarget.Existing)?.let { target ->
                        profileState.profiles.firstOrNull { it.id == target.id }
                    },
                    avatarUrl = profileEditAvatarUrl,
                    avatarPacks = profileState.avatarPacks,
                    biometricAvailable = biometricAvailable,
                    language = state.language,
                    onPickAvatarClick = onPickAvatarClick,
                    onRemoveAvatarClick = onRemoveAvatarClick,
                    onPickPackAvatarClick = onPickPackAvatarClick,
                    onSave = onProfileSave,
                    onDelete = onProfileDelete,
                    onCancel = onProfileEditCancel,
                    modifier = Modifier.fillMaxSize()
                )
                state.showProfilePickerSettings && profileState != null -> ProfilePickerSettingsScreen(
                    state = profileState,
                    language = state.language,
                    onAction = onProfileListAction,
                    onPickBackgroundClick = onPickBackgroundClick,
                    onBack = { onProfileListAction(ProfileAction.PickerSettingsClosed) },
                    modifier = Modifier.fillMaxSize()
                )
                state.showNotifications -> NotificationsScreen(
                    state = calendarState,
                    language = state.language,
                    onBack = onNotificationsBackRequested,
                    onItemSelected = { release -> onCalendarAction(CalendarAction.ItemSelected(release.item)) },
                    modifier = Modifier.fillMaxSize().then(tvRouteModifier)
                )
                state.showSourceSelection && detailState?.content != null -> SourceSelectionScreen(
                    content = detailState.content,
                    language = state.language,
                    onBack = onSourceSelectionBackRequested,
                    onStreamSelected = { stream ->
                        onDetailAction(DetailAction.StreamSelected(stream, detailState.content.selectedEpisodeId))
                    },
                    onRetry = { onDetailAction(DetailAction.RetrySourcesRequested) },
                    modifier = Modifier.fillMaxSize().then(tvRouteModifier)
                )
                state.selectedDetail != null && detailState != null -> DetailScreen(
                    state = detailState,
                    language = state.language,
                    onAction = onDetailAction,
                    onBack = onDetailBackRequested,
                    presentation = settingsState?.appearanceDetail?.let { detailSettings ->
                        DetailPresentationOptions(
                            screenStyle = DetailScreenStyle.from(detailSettings.detailScreenStyle),
                            preferClearlogo = detailSettings.detailPreferClearlogo,
                            showEpisodeDescriptions = detailSettings.detailShowEpisodeDescriptions,
                            showCast = detailSettings.detailShowCast,
                            showRecommendations = detailSettings.detailShowRecommendations,
                            collapsingHero = detailSettings.detailCollapsingHero,
                            blurUnwatchedEpisodes = detailSettings.blurUnwatchedEpisodes,
                            seasonSelectorMode = detailSettings.detailSeasonSelectorMode,
                            episodeCardsLayout = detailSettings.episodeCardsLayout
                        )
                    } ?: DetailPresentationOptions(),
                    modifier = Modifier.fillMaxSize().then(tvRouteModifier)
                )
                libraryState?.folderDetail?.folder != null -> LibraryFolderDetailScreen(
                    state = libraryState.folderDetail,
                    language = state.language,
                    onBack = { onLibraryAction(LibraryAction.FolderClosed) },
                    onItemSelected = onLibraryItemSelected,
                    modifier = Modifier.fillMaxSize()
                )
                state.selectedCategoryId != null -> CategoryResultsScreen(
                    title = state.selectedCategoryTitle.orEmpty(),
                    items = catalogHome.rows.firstOrNull { it.id == state.selectedCategoryId }?.items.orEmpty(),
                    language = state.language,
                    onBack = onCategoryBackRequested,
                    onItemSelected = onCategoryItemSelected,
                    deviceType = deviceType,
                    modifier = Modifier.fillMaxSize().then(tvRouteModifier)
                )
                state.destination == FluxaDestination.Search && searchState != null && deviceType == com.fluxa.app.ui.catalog.DeviceType.TV -> com.fluxa.app.shared.feature.search.TvSearchScreen(
                    state = searchState,
                    language = state.language,
                    onQueryChanged = { value -> onSearchAction(SearchAction.QueryChanged(value)) },
                    onItemSelected = { item -> onSearchAction(SearchAction.ItemSelected(item)) },
                    onAddToLibrary = { item -> onCatalogAction(CatalogAction.AddToLibraryRequested(item)) },
                    onClearHistory = { onSearchAction(SearchAction.ClearHistory) },
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Search && searchState != null -> SearchScreen(
                    state = searchState,
                    language = state.language,
                    onQueryChanged = { value -> onSearchAction(SearchAction.QueryChanged(value)) },
                    onItemSelected = { item -> onSearchAction(SearchAction.ItemSelected(item)) },
                    onAddToLibrary = { item -> onCatalogAction(CatalogAction.AddToLibraryRequested(item)) },
                    onClearHistory = { onSearchAction(SearchAction.ClearHistory) },
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Discover && discoverState != null && deviceType == com.fluxa.app.ui.catalog.DeviceType.TV -> com.fluxa.app.shared.feature.discover.TvDiscoverScreen(
                    state = discoverState,
                    language = state.language,
                    onFiltersChanged = { filters -> onDiscoverAction(DiscoverAction.FiltersChanged(filters)) },
                    onItemSelected = { item -> onDiscoverAction(DiscoverAction.ItemSelected(item)) },
                    onLoadMore = { onDiscoverAction(DiscoverAction.LoadMore) },
                    searchQuery = searchState?.query.orEmpty(),
                    onSearchQueryChanged = { value -> onSearchAction(SearchAction.QueryChanged(value)) },
                    searchResultRows = searchState?.resultRows.orEmpty(),
                    searchResults = searchState?.results.orEmpty(),
                    isSearching = searchState?.isLoading == true,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Discover && discoverState != null -> DiscoverScreen(
                    state = discoverState,
                    language = state.language,
                    onFiltersChanged = { filters -> onDiscoverAction(DiscoverAction.FiltersChanged(filters)) },
                    onItemSelected = { item -> onDiscoverAction(DiscoverAction.ItemSelected(item)) },
                    onLoadMore = { onDiscoverAction(DiscoverAction.LoadMore) },
                    searchQuery = searchState?.query.orEmpty(),
                    onSearchQueryChanged = { value -> onSearchAction(SearchAction.QueryChanged(value)) },
                    searchResultRows = searchState?.resultRows.orEmpty(),
                    searchResults = searchState?.results.orEmpty(),
                    isSearching = searchState?.isLoading == true,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Calendar && calendarState != null && deviceType == com.fluxa.app.ui.catalog.DeviceType.TV -> com.fluxa.app.shared.feature.calendar.TvCalendarScreen(
                    state = calendarState,
                    language = state.language,
                    onAction = onCalendarAction,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Calendar && calendarState != null -> CalendarScreen(
                    state = calendarState,
                    language = state.language,
                    onAction = onCalendarAction,
                    onBack = { onDestinationSelected(FluxaDestination.Home) },
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Library && libraryState != null && deviceType == com.fluxa.app.ui.catalog.DeviceType.TV -> com.fluxa.app.shared.feature.library.TvLibraryScreen(
                    state = libraryState,
                    language = state.language,
                    onAction = onLibraryAction,
                    onItemSelected = onLibraryItemSelected,
                    initialSection = state.initialLibrarySection ?: com.fluxa.app.shared.feature.library.LibrarySection.Planned,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Library && libraryState != null -> LibraryScreen(
                    state = libraryState,
                    language = state.language,
                    onAction = onLibraryAction,
                    onItemSelected = onLibraryItemSelected,
                    initialSection = state.initialLibrarySection ?: com.fluxa.app.shared.feature.library.LibrarySection.Planned,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Settings && settingsState != null -> SettingsScreen(
                    state = settingsState,
                    language = state.language,
                    backStack = state.settingsBackStack,
                    onAction = onSettingsAction,
                    onSwitchProfilesRequested = onSwitchProfilesRequested,
                    profileState = profileState,
                    onProfileAction = { action ->
                        if (action is ProfileAction.Selected && action.profile.hasPin) {
                            onSwitchProfilesRequested()
                        }
                        onProfileListAction(action)
                    },
                    onBackRequested = onSettingsBackRequested,
                    onPushCategory = onSettingsPushCategory,
                    onPopCategory = onSettingsPopCategory,
                    onSelectCategory = onSettingsSelectCategory,
                    deviceType = deviceType,
                    brandIcons = settingsBrandIcons,
                    modifier = Modifier.fillMaxSize().then(tvRouteModifier)
                )
                state.destination == FluxaDestination.AddonStore && addonStoreState != null -> AddonStoreScreen(
                    state = addonStoreState,
                    language = state.language,
                    onAction = onAddonStoreAction,
                    onConfigureRequested = onOpenUrlRequested,
                    onBackRequested = onAddonStoreBackRequested,
                    modifier = Modifier.fillMaxSize().then(tvRouteModifier)
                )
                state.destination == FluxaDestination.Plugins && pluginsState != null -> PluginsScreen(
                    state = pluginsState,
                    language = state.language,
                    onAction = onPluginsAction,
                    onBackRequested = onPluginsBackRequested,
                    modifier = Modifier.fillMaxSize().then(tvRouteModifier)
                )
                state.destination == FluxaDestination.Auth && authState != null && deviceType == com.fluxa.app.ui.catalog.DeviceType.TV -> com.fluxa.app.shared.feature.auth.TvAuthScreen(
                    state = authState,
                    language = state.language,
                    onAction = onAuthAction,
                    nuvioIcon = nuvioIcon,
                    stremioIcon = stremioIcon,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Auth && authState != null -> AuthScreen(
                    state = authState,
                    language = state.language,
                    onAction = onAuthAction,
                    nuvioIcon = nuvioIcon,
                    stremioIcon = stremioIcon,
                    backdrop = authBackdrop,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.ProfileList && profileState != null -> ProfileListScreen(
                    state = profileState,
                    language = state.language,
                    biometricAvailable = biometricAvailable,
                    onAction = onProfileListAction,
                    onBiometricRequested = onProfileBiometricRequested,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Home && deviceType == com.fluxa.app.ui.catalog.DeviceType.TV -> com.fluxa.app.shared.feature.catalog.TvCatalogHomeScreen(
                    state = catalogHome,
                    onAction = onCatalogAction,
                    language = state.language,
                    hideContinueWatchingLabels = settingsState?.appearanceHome?.continueWatchingHideTitles == true,
                    continueWatchingWidthPreset = settingsState?.appearanceHome?.continueWatchingWidthPreset ?: "medium",
                    continueWatchingCornerPreset = settingsState?.appearanceHome?.continueWatchingCardCornerPreset ?: "medium",
                    continueWatchingDensity = settingsState?.appearanceHome?.continueWatchingInterfaceDensity ?: "medium",
                    continueWatchingLandscapeMode = settingsState?.appearanceHome?.continueWatchingHorizontal ?: true,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Home -> FluxaHomeContent(
                    state = state,
                    catalogHome = catalogHome,
                    onCatalogAction = onCatalogAction,
                    onCategorySelected = onCategorySelected,
                    hideContinueWatchingLabels = settingsState?.appearanceHome?.continueWatchingHideTitles == true,
                    continueWatchingWidthPreset = settingsState?.appearanceHome?.continueWatchingWidthPreset ?: "medium",
                    continueWatchingCornerPreset = settingsState?.appearanceHome?.continueWatchingCardCornerPreset ?: "medium",
                    continueWatchingDensity = settingsState?.appearanceHome?.continueWatchingInterfaceDensity ?: "medium",
                    continueWatchingLandscapeMode = settingsState?.appearanceHome?.continueWatchingHorizontal ?: true,
                    bottomContentInset = if (navChromeVisible) navBarHeightDp + 20.dp else 24.dp,
                    modifier = Modifier.fillMaxSize()
                )
                else -> FluxaDestinationPlaceholder(
                    language = state.language,
                    destination = state.destination,
                    modifier = Modifier.fillMaxSize()
                )
            }
            }
            }
            if (navChromeVisible) {
                if (useTopNav) {
                    FluxaTopNavBar(
                        destination = state.destination,
                        accentColorArgb = profileState?.activeProfile?.accentColorArgb,
                        showProfile = true,
                        showLabels = settingsState?.appearance?.bottomBarLabels == true,
                        isTv = isTv,
                        language = state.language,
                        onDestinationSelected = onDestinationSelected,
                        modifier = Modifier
                            .align(Alignment.TopCenter)
                            .onGloballyPositioned { topNavHeightPx = it.size.height }
                    )
                } else if (isTv) {
                    TvSidebarNav(
                        destination = state.destination,
                        language = state.language,
                        onDestinationSelected = onDestinationSelected,
                        modifier = Modifier
                            .align(Alignment.CenterStart)
                            .onGloballyPositioned { tvSidebarWidthPx = it.size.width }
                    )
                } else if (useRail) {
                    FluxaNavigationRail(
                        destination = state.destination,
                        accentColorArgb = profileState?.activeProfile?.accentColorArgb,
                        showProfile = true,
                        showLabels = settingsState?.appearance?.bottomBarLabels == true,
                        language = state.language,
                        onDestinationSelected = onDestinationSelected,
                        modifier = Modifier
                            .align(Alignment.CenterStart)
                            .onGloballyPositioned { navRailWidthPx = it.size.width },
                        profileAvatarUrl = profileState?.activeProfile?.avatarUrl,
                        activeProfileName = profileState?.activeProfile?.name,
                        activeProfileId = profileState?.activeProfile?.id,
                        profiles = profileState?.profiles.orEmpty(),
                        onProfileSelected = { profile ->
                            if (profile.hasPin) onSwitchProfilesRequested()
                            onProfileListAction(ProfileAction.Selected(profile))
                        },
                        onAddProfileRequested = { onProfileListAction(ProfileAction.AddRequested) },
                        onManageProfilesRequested = onSwitchProfilesRequested
                    )
                } else {
                    FluxaNavigationBar(
                        destination = state.destination,
                        accentColorArgb = profileState?.activeProfile?.accentColorArgb,
                        floating = settingsState?.appearance?.floatingBottomBar == true,
                        liquidGlass = liquidGlassMode,
                        hazeState = hazeState,
                        showLabels = settingsState?.appearance?.bottomBarLabels == true,
                        showProfile = true,
                        profileAvatarUrl = profileState?.activeProfile?.avatarUrl,
                        language = state.language,
                        onDestinationSelected = onDestinationSelected,
                        modifier = Modifier
                            .align(Alignment.BottomCenter)
                            .onGloballyPositioned { navBarHeightPx = it.size.height }
                    )
                }
            }
        }
        }
    }
}



@Composable
internal fun FluxaDestinationPlaceholder(
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
