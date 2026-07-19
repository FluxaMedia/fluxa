package com.fluxa.app.shared

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.chrisbanes.haze.HazeStyle
import dev.chrisbanes.haze.HazeTint
import dev.chrisbanes.haze.hazeEffect
import dev.chrisbanes.haze.hazeSource
import dev.chrisbanes.haze.rememberHazeState
import kotlinx.coroutines.delay
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.ui.draw.alpha
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogBillboardUiModel
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CategoryResultsScreen
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaIcons
import com.fluxa.app.shared.feature.addonstore.AddonStoreAction
import com.fluxa.app.shared.feature.addonstore.AddonStoreScreen
import com.fluxa.app.shared.feature.addonstore.AddonStoreUiState
import com.fluxa.app.shared.feature.auth.AuthAction
import com.fluxa.app.shared.feature.auth.AuthScreen
import com.fluxa.app.shared.feature.auth.AuthUiState
import com.fluxa.app.shared.feature.calendar.CalendarAction
import com.fluxa.app.shared.feature.calendar.CalendarScreen
import com.fluxa.app.shared.feature.calendar.CalendarUiState
import com.fluxa.app.shared.feature.calendar.NotificationsScreen
import com.fluxa.app.shared.feature.detail.DetailAction
import com.fluxa.app.shared.feature.detail.DetailRequestUiModel
import com.fluxa.app.shared.feature.detail.DetailScreen
import com.fluxa.app.shared.feature.detail.DetailUiState
import com.fluxa.app.shared.feature.detail.SourceSelectionScreen
import com.fluxa.app.shared.feature.discover.DiscoverAction
import com.fluxa.app.shared.feature.discover.DiscoverScreen
import com.fluxa.app.shared.feature.discover.DiscoverUiState
import com.fluxa.app.shared.feature.library.LibraryAction
import com.fluxa.app.shared.feature.library.LibraryFolderDetailScreen
import com.fluxa.app.shared.feature.library.LibraryScreen
import com.fluxa.app.shared.feature.library.LibraryUiState
import com.fluxa.app.shared.feature.profile.ProfileAction
import com.fluxa.app.shared.feature.profile.ProfileEditScreen
import com.fluxa.app.shared.feature.profile.ProfileEditTarget
import com.fluxa.app.shared.feature.profile.ProfileEditUiModel
import com.fluxa.app.shared.feature.profile.ProfileListScreen
import com.fluxa.app.shared.feature.profile.ProfileUiState
import com.fluxa.app.shared.feature.settings.SettingsAction
import com.fluxa.app.shared.feature.settings.SettingsScreen
import com.fluxa.app.shared.feature.settings.SettingsUiState
import com.fluxa.app.shared.feature.search.SearchAction
import com.fluxa.app.shared.feature.search.SearchScreen
import com.fluxa.app.shared.feature.search.SearchUiState
import com.fluxa.app.shared.feature.player.PlayerControlsSurface
import com.fluxa.app.shared.feature.player.PlayerRenderAction
import com.fluxa.app.shared.feature.player.PlayerRenderState
import com.fluxa.app.player.TrailerCue
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.PosterActionSheet

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
    Auth("auth.log_in"),
    ProfileList("auto.profile")
}

data class FluxaAppUiState(
    val language: String? = null,
    val destination: FluxaDestination = FluxaDestination.Home,
    val catalogHome: CatalogHomeUiState = CatalogHomeUiState(),
    val selectedDetail: DetailRequestUiModel? = null,
    val showSourceSelection: Boolean = false,
    val selectedCategoryId: String? = null,
    val selectedCategoryTitle: String? = null,
    val editingProfile: ProfileEditTarget? = null,
    val showNotifications: Boolean = false,
    val settingsBackStack: List<com.fluxa.app.shared.feature.settings.SettingsCategory> = emptyList(),
    val initialLibrarySection: com.fluxa.app.shared.feature.library.LibrarySection? = null
)

@Composable
fun FluxaApp(
    state: FluxaAppUiState,
    deviceType: com.fluxa.app.ui.catalog.DeviceType = com.fluxa.app.ui.catalog.DeviceType.Mobile,
    onDestinationSelected: (FluxaDestination) -> Unit,
    onCatalogAction: (CatalogAction) -> Unit,
    detailState: DetailUiState? = null,
    onDetailAction: (DetailAction) -> Unit = {},
    onDetailBackRequested: () -> Unit = {},
    onSourceSelectionBackRequested: () -> Unit = {},
    onCategoryBackRequested: () -> Unit = {},
    onCategoryItemSelected: (CatalogItemUiModel) -> Unit = {},
    onCategorySelected: (id: String, title: String) -> Unit = { _, _ -> },
    searchState: SearchUiState? = null,
    onSearchAction: (SearchAction) -> Unit = {},
    discoverState: DiscoverUiState? = null,
    onDiscoverAction: (DiscoverAction) -> Unit = {},
    calendarState: CalendarUiState? = null,
    onCalendarAction: (CalendarAction) -> Unit = {},
    onNotificationsRequested: () -> Unit = {},
    onNotificationsBackRequested: () -> Unit = {},
    libraryState: LibraryUiState? = null,
    onLibraryItemSelected: (CatalogItemUiModel) -> Unit = {},
    onLibraryAction: (LibraryAction) -> Unit = {},
    profileState: ProfileUiState? = null,
    settingsState: SettingsUiState? = null,
    onSettingsAction: (SettingsAction) -> Unit = {},
    onSwitchProfilesRequested: () -> Unit = {},
    onSettingsBackRequested: () -> Unit = {},
    onSettingsPushCategory: (com.fluxa.app.shared.feature.settings.SettingsCategory) -> Unit = {},
    onSettingsPopCategory: () -> Unit = {},
    onSettingsSelectCategory: (com.fluxa.app.shared.feature.settings.SettingsCategory) -> Unit = {},
    settingsBrandIcons: com.fluxa.app.shared.feature.settings.SettingsBrandIcons = com.fluxa.app.shared.feature.settings.SettingsBrandIcons(),
    addonStoreState: AddonStoreUiState? = null,
    onAddonStoreAction: (AddonStoreAction) -> Unit = {},
    onOpenUrlRequested: (String) -> Unit = {},
    onAddonStoreBackRequested: () -> Unit = {},
    authState: AuthUiState? = null,
    onAuthAction: (AuthAction) -> Unit = {},
    nuvioIcon: @Composable () -> Unit = {},
    stremioIcon: @Composable () -> Unit = {},
    onProfileListAction: (ProfileAction) -> Unit = {},
    onProfileBiometricRequested: (com.fluxa.app.shared.feature.profile.ProfileUiModel) -> Unit = {},
    profileEditAvatarUrl: String? = null,
    onPickAvatarClick: () -> Unit = {},
    onRemoveAvatarClick: () -> Unit = {},
    onProfileSave: (ProfileEditUiModel) -> Unit = {},
    onProfileDelete: (() -> Unit)? = null,
    onProfileEditCancel: () -> Unit = {},
    playerState: PlayerRenderState? = null,
    onPlayerAction: (PlayerRenderAction) -> Unit = {},
    biometricAvailable: Boolean = false,
    showNavigationBar: Boolean = true,
    modifier: Modifier = Modifier
) {
    MaterialTheme(colorScheme = FluxaColorScheme) {
        val liquidGlassMode = settingsState?.appearance?.liquidGlassMode == true
        val hazeState = rememberHazeState()
        Box(
            modifier = modifier
                .fillMaxSize()
                .background(FluxaColors.background)
        ) {
            val screenKey = when {
                playerState?.content != null -> "player:${playerState.content?.id.orEmpty()}"
                state.editingProfile != null -> "profileEdit"
                state.showSourceSelection -> "sources"
                state.selectedDetail != null -> "detail:${state.selectedDetail.id}"
                libraryState?.folderDetail?.folder != null -> "folder:${libraryState.folderDetail.folder.id}"
                state.selectedCategoryId != null -> "category:${state.selectedCategoryId}"
                state.showNotifications -> "notifications"
                else -> "dest:${state.destination}"
            }
            val isTv = deviceType == com.fluxa.app.ui.catalog.DeviceType.TV
            val isHomeActive = screenKey == "dest:${FluxaDestination.Home}" && !isTv
            val isPreAuthDestination = state.destination == FluxaDestination.Auth ||
                state.destination == FluxaDestination.ProfileList
            val navChromeVisible = showNavigationBar && !isPreAuthDestination
            var navBarHeightPx by remember { mutableIntStateOf(0) }
            var tvSidebarWidthPx by remember { mutableIntStateOf(0) }
            val density = LocalDensity.current
            val navBarHeightDp = with(density) { navBarHeightPx.toDp() }
            val tvSidebarWidthDp = with(density) { tvSidebarWidthPx.toDp() }
            val saveableStateHolder = androidx.compose.runtime.saveable.rememberSaveableStateHolder()
            AnimatedContent(
                targetState = screenKey,
                transitionSpec = {
                    (fadeIn(tween(220)) + scaleIn(initialScale = 0.98f, animationSpec = tween(220)))
                        .togetherWith(fadeOut(tween(120)))
                },
                label = "fluxa-screen-transition",
                modifier = (
                    if (!navChromeVisible) {
                        Modifier.fillMaxSize()
                    } else if (isTv) {
                        Modifier.fillMaxSize().padding(start = tvSidebarWidthDp)
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
                    biometricAvailable = biometricAvailable,
                    language = state.language,
                    onPickAvatarClick = onPickAvatarClick,
                    onRemoveAvatarClick = onRemoveAvatarClick,
                    onSave = onProfileSave,
                    onDelete = onProfileDelete,
                    onCancel = onProfileEditCancel,
                    modifier = Modifier.fillMaxSize()
                )
                state.showNotifications -> NotificationsScreen(
                    state = calendarState,
                    language = state.language,
                    onBack = onNotificationsBackRequested,
                    onItemSelected = { release -> onCalendarAction(CalendarAction.ItemSelected(release.item)) },
                    modifier = Modifier.fillMaxSize()
                )
                state.showSourceSelection && detailState?.content != null -> SourceSelectionScreen(
                    content = detailState.content,
                    language = state.language,
                    onBack = onSourceSelectionBackRequested,
                    onStreamSelected = { stream ->
                        onDetailAction(DetailAction.StreamSelected(stream, detailState.content.selectedEpisodeId))
                    },
                    modifier = Modifier.fillMaxSize()
                )
                state.selectedDetail != null && detailState != null -> DetailScreen(
                    state = detailState,
                    language = state.language,
                    onAction = onDetailAction,
                    onBack = onDetailBackRequested,
                    modifier = Modifier.fillMaxSize()
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
                    items = state.catalogHome.rows.firstOrNull { it.id == state.selectedCategoryId }?.items.orEmpty(),
                    language = state.language,
                    onBack = onCategoryBackRequested,
                    onItemSelected = onCategoryItemSelected,
                    deviceType = deviceType,
                    modifier = Modifier.fillMaxSize()
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
                    onBackRequested = onSettingsBackRequested,
                    onPushCategory = onSettingsPushCategory,
                    onPopCategory = onSettingsPopCategory,
                    onSelectCategory = onSettingsSelectCategory,
                    deviceType = deviceType,
                    brandIcons = settingsBrandIcons,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.AddonStore && addonStoreState != null -> AddonStoreScreen(
                    state = addonStoreState,
                    language = state.language,
                    onAction = onAddonStoreAction,
                    onConfigureRequested = onOpenUrlRequested,
                    onBackRequested = onAddonStoreBackRequested,
                    modifier = Modifier.fillMaxSize()
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
                    state = state.catalogHome,
                    onAction = onCatalogAction,
                    language = state.language,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Home -> FluxaHomeContent(
                    state = state,
                    onCatalogAction = onCatalogAction,
                    onNotificationsRequested = onNotificationsRequested,
                    onAvatarClick = { onDestinationSelected(FluxaDestination.Settings) },
                    onCategorySelected = onCategorySelected,
                    profileAvatarUrl = profileState?.activeProfile?.avatarUrl,
                    topBarEnabled = settingsState?.appearanceHome?.topBarEnabled != false,
                    bottomContentInset = if (liquidGlassMode && navChromeVisible) navBarHeightDp + 20.dp else 24.dp,
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
                if (isTv) {
                    TvSidebarNav(
                        destination = state.destination,
                        language = state.language,
                        onDestinationSelected = onDestinationSelected,
                        modifier = Modifier
                            .align(Alignment.CenterStart)
                            .onGloballyPositioned { tvSidebarWidthPx = it.size.width }
                    )
                } else {
                    FluxaNavigationBar(
                        destination = state.destination,
                        accentColorArgb = profileState?.activeProfile?.accentColorArgb,
                        floating = settingsState?.appearance?.floatingBottomBar == true,
                        liquidGlass = liquidGlassMode,
                        hazeState = hazeState,
                        showLabels = settingsState?.appearance?.bottomBarLabels == true,
                        showProfile = settingsState?.appearanceHome?.topBarEnabled == false,
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

private data class FluxaBottomNavItem(
    val destination: FluxaDestination,
    val selectedIcon: ImageVector,
    val icon: ImageVector
)

private val FluxaBottomNavItems = listOf(
    FluxaBottomNavItem(FluxaDestination.Home, FluxaIcons.BottomHome, FluxaIcons.BottomHomeOutline),
    FluxaBottomNavItem(FluxaDestination.Discover, FluxaIcons.BottomDiscover, FluxaIcons.BottomDiscoverOutline),
    FluxaBottomNavItem(FluxaDestination.Calendar, FluxaIcons.BottomCalendar, FluxaIcons.BottomCalendarOutline),
    FluxaBottomNavItem(FluxaDestination.Library, FluxaIcons.BottomLibrary, FluxaIcons.BottomLibraryOutline)
)

@Composable
private fun FluxaNavigationBar(
    destination: FluxaDestination,
    accentColorArgb: Long?,
    floating: Boolean,
    liquidGlass: Boolean,
    hazeState: dev.chrisbanes.haze.HazeState,
    showLabels: Boolean,
    showProfile: Boolean,
    profileAvatarUrl: String?,
    language: String?,
    onDestinationSelected: (FluxaDestination) -> Unit,
    modifier: Modifier = Modifier
) {
    val selectedColor = accentColorArgb?.let { Color(it) } ?: Color.White
    val inactiveColor = if (liquidGlass) Color.White.copy(alpha = 0.78f) else Color(0xFFA0A5AD)
    val barShape = RoundedCornerShape(if (floating) 28.dp else 0.dp)
    val items = if (showProfile) {
        FluxaBottomNavItems + FluxaBottomNavItem(FluxaDestination.Settings, FluxaIcons.BottomSettings, FluxaIcons.BottomSettingsOutline)
    } else {
        FluxaBottomNavItems
    }
    Box(
        modifier = modifier
            .fillMaxWidth()
            .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Bottom))
            .padding(horizontal = if (floating) 12.dp else 0.dp)
            .padding(bottom = if (floating) 10.dp else 0.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .then(
                    if (liquidGlass) {
                        Modifier.shadow(
                            elevation = 18.dp,
                            shape = barShape,
                            ambientColor = Color.Black.copy(alpha = 0.35f),
                            spotColor = Color.Black.copy(alpha = 0.45f)
                        )
                    } else {
                        Modifier
                    }
                )
                .clip(barShape)
                .then(
                    if (liquidGlass) {
                        Modifier
                            .hazeEffect(
                                state = hazeState,
                                style = HazeStyle(
                                    backgroundColor = if (floating) Color(0xFF1C1C1E) else Color(0xFF101012),
                                    tints = listOf(
                                        HazeTint(Color.Black.copy(alpha = 0.4f)),
                                        HazeTint(Color.White.copy(alpha = 0.05f))
                                    ),
                                    blurRadius = 26.dp,
                                    noiseFactor = 0.1f
                                )
                            ) {
                                progressive = dev.chrisbanes.haze.HazeProgressive.RadialGradient(
                                    centerIntensity = 1f,
                                    radiusIntensity = 0.55f
                                )
                            }
                            .border(
                                width = 1.dp,
                                brush = Brush.verticalGradient(
                                    listOf(Color(0x73FFFFFF), Color.Transparent, Color(0x59000000))
                                ),
                                shape = barShape
                            )
                            .drawWithContent {
                                drawRect(
                                    Brush.verticalGradient(
                                        0f to Color.White.copy(alpha = 0.10f),
                                        0.22f to Color.Transparent,
                                        0.78f to Color.Transparent,
                                        1f to Color.Black.copy(alpha = 0.16f)
                                    )
                                )
                                drawContent()
                            }
                    } else {
                        Modifier.background(if (floating) Color(0xF2222222) else Color(0xFF111111))
                    }
                )
                .padding(horizontal = 12.dp)
                .height(if (showLabels) 68.dp else 64.dp),
            horizontalArrangement = Arrangement.SpaceEvenly,
            verticalAlignment = Alignment.CenterVertically
        ) {
            items.forEach { item ->
                val isSelected = item.destination == destination
                val tint by animateColorAsState(
                    targetValue = if (isSelected) selectedColor else inactiveColor,
                    label = "nav-item-tint"
                )
                Column(
                    modifier = Modifier
                        .weight(1f)
                        .clip(RoundedCornerShape(18.dp))
                        .clickable { onDestinationSelected(item.destination) }
                        .padding(vertical = 8.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(2.dp)
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        if (liquidGlass && isSelected) {
                            Box(
                                modifier = Modifier
                                    .size(50.dp)
                                    .background(
                                        Brush.radialGradient(
                                            listOf(selectedColor.copy(alpha = 0.28f), Color.Transparent)
                                        ),
                                        CircleShape
                                    )
                            )
                            Box(
                                modifier = Modifier
                                    .size(38.dp)
                                    .background(Color.White.copy(alpha = 0.1f), CircleShape)
                                    .border(1.dp, selectedColor.copy(alpha = 0.4f), CircleShape)
                            )
                        }
                        if (item.destination == FluxaDestination.Settings && !profileAvatarUrl.isNullOrBlank()) {
                            FluxaRemoteImage(
                                imageUrl = profileAvatarUrl,
                                cacheKey = "bottom-profile:$profileAvatarUrl",
                                contentDescription = null,
                                modifier = Modifier
                                    .size(32.dp)
                                    .then(
                                        if (isSelected) {
                                            Modifier.border(2.dp, tint, CircleShape).padding(2.dp)
                                        } else {
                                            Modifier
                                        }
                                    )
                                    .clip(CircleShape),
                                contentScale = ContentScale.Crop
                            )
                        } else {
                            Icon(
                                if (isSelected) item.selectedIcon else item.icon,
                                contentDescription = null,
                                tint = tint,
                                modifier = Modifier.size(29.dp)
                            )
                        }
                    }
                    if (showLabels) {
                        Text(
                            text = AppStrings.t(language, item.destination.titleKey),
                            color = tint,
                            fontSize = 10.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                }
            }
        }
    }
}

private enum class FluxaHomeFilter(val value: String, val titleKey: String) {
    All("all", "nav.home"),
    Series("series", "auto.series"),
    Movies("movie", "auto.movie")
}

@Composable
private fun FluxaHomeOverlayBar(
    activeFilter: String,
    language: String?,
    profileAvatarUrl: String?,
    scrimAlpha: Float,
    onFilterSelected: (String) -> Unit,
    onNotificationsRequested: () -> Unit,
    onAvatarClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    val backgroundColor = androidx.compose.ui.graphics.lerp(
        Color.Black.copy(alpha = 0.4f),
        FluxaColors.background,
        scrimAlpha
    )
    Row(
        modifier = modifier
            .fillMaxWidth()
            .background(backgroundColor)
            .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Top))
            .padding(horizontal = 20.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
            FluxaHomeFilter.entries.forEach { filter ->
                val selected = filter.value == activeFilter || (filter == FluxaHomeFilter.All && activeFilter.isBlank())
                val underlineWidth by animateDpAsState(if (selected) 20.dp else 0.dp, label = "filter-underline")
                Column(modifier = Modifier.clickable { onFilterSelected(filter.value) }) {
                    Text(
                        text = AppStrings.t(language, filter.titleKey),
                        color = if (selected) Color.White else Color.White.copy(alpha = 0.6f),
                        fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
                        fontSize = 15.sp
                    )
                    Box(
                        modifier = Modifier
                            .padding(top = 8.dp)
                            .height(2.dp)
                            .width(underlineWidth)
                            .background(Color.White)
                    )
                }
            }
        }
        Row(verticalAlignment = Alignment.CenterVertically) {
            Icon(
                imageVector = FluxaIcons.Notifications,
                contentDescription = AppStrings.t(language, "auto.notifications"),
                tint = Color.White,
                modifier = Modifier.size(24.dp).clickable(onClick = onNotificationsRequested)
            )
            Box(
                modifier = Modifier
                    .padding(start = 16.dp)
                    .size(32.dp)
                    .clip(CircleShape)
                    .background(FluxaColors.surfaceRaised)
                    .clickable(onClick = onAvatarClick)
            ) {
                if (profileAvatarUrl != null) {
                    FluxaRemoteImage(
                        imageUrl = profileAvatarUrl,
                        cacheKey = "profile-avatar:$profileAvatarUrl",
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop
                    )
                }
            }
        }
    }
}

@Composable
private fun FluxaHomeContent(
    state: FluxaAppUiState,
    onCatalogAction: (CatalogAction) -> Unit,
    onNotificationsRequested: () -> Unit,
    onAvatarClick: () -> Unit,
    onCategorySelected: (id: String, title: String) -> Unit,
    profileAvatarUrl: String?,
    topBarEnabled: Boolean,
    bottomContentInset: androidx.compose.ui.unit.Dp = 24.dp,
    modifier: Modifier
) {
    if (state.catalogHome.isLoading && state.catalogHome.rows.isEmpty()) {
        FluxaHomeSkeleton(modifier = modifier)
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

    val heroItems = state.catalogHome.heroItems
    val showHero = state.catalogHome.showHeroSection && heroItems.isNotEmpty()
    val listState = rememberLazyListState()
    var heroHeightPx by remember { mutableIntStateOf(0) }
    var posterActionItem by remember { mutableStateOf<CatalogItemUiModel?>(null) }
    val scrimAlpha by remember {
        androidx.compose.runtime.derivedStateOf {
            when {
                !showHero -> 1f
                listState.firstVisibleItemIndex > 0 -> 1f
                heroHeightPx <= 0 -> 0f
                else -> (listState.firstVisibleItemScrollOffset.toFloat() / heroHeightPx).coerceIn(0f, 1f)
            }
        }
    }

    Box(modifier = modifier) {
        LazyColumn(
            state = listState,
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = bottomContentInset),
            verticalArrangement = Arrangement.spacedBy(24.dp)
        ) {
            if (showHero) {
                item(key = "hero") {
                    FluxaHomeHero(
                        items = heroItems,
                        billboard = state.catalogHome.billboard,
                        language = state.language,
                        onCatalogAction = onCatalogAction,
                        modifier = Modifier.onGloballyPositioned { heroHeightPx = it.size.height }
                    )
                }
            }
            items(
                state.catalogHome.rows.filterNot { it.categoryType == "collection_folder" },
                key = { it.id }
            ) { row ->
                val rowState = rememberLazyListState()
                val shouldLoadMore by remember(row.id, row.canLoadMore) {
                    derivedStateOf {
                        val lastVisible = rowState.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0
                        row.canLoadMore && row.items.isNotEmpty() && lastVisible >= row.items.lastIndex - 4
                    }
                }
                LaunchedEffect(shouldLoadMore, row.items.size) {
                    if (shouldLoadMore) {
                        onCatalogAction(CatalogAction.LoadMore(row.id))
                    }
                }
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .then(
                                if (row.canLoadMore) {
                                    Modifier.clickable { onCategorySelected(row.id, row.title) }
                                } else {
                                    Modifier
                                }
                            )
                            .padding(horizontal = 20.dp, vertical = 4.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = row.title,
                            color = Color.White,
                            fontWeight = FontWeight.SemiBold,
                            fontSize = 16.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f)
                        )
                        if (row.canLoadMore) {
                            Icon(
                                imageVector = Icons.AutoMirrored.Filled.ArrowForward,
                                contentDescription = AppStrings.t(state.language, "common.view_all"),
                                tint = Color.White.copy(alpha = 0.7f),
                                modifier = Modifier
                                    .padding(start = 12.dp)
                                    .size(20.dp)
                            )
                        }
                    }
                    LazyRow(
                        state = rowState,
                        contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 20.dp),
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        items(row.items, key = { it.id }) { item ->
                            CatalogCard(
                                model = item.card,
                                onClick = { onCatalogAction(CatalogAction.ItemSelected(item)) },
                                onLongClick = { posterActionItem = item }
                            )
                        }
                    }
                }
            }
        }
        if (showHero && topBarEnabled) {
            FluxaHomeOverlayBar(
                activeFilter = state.catalogHome.activeFilter,
                language = state.language,
                profileAvatarUrl = profileAvatarUrl,
                scrimAlpha = scrimAlpha,
                onFilterSelected = { filter -> onCatalogAction(CatalogAction.FilterChanged(filter)) },
                onNotificationsRequested = onNotificationsRequested,
                onAvatarClick = onAvatarClick,
                modifier = Modifier.align(Alignment.TopCenter)
            )
        }
    }

    posterActionItem?.let { item ->
        PosterActionSheet(
            item = item,
            language = state.language,
            onDismiss = { posterActionItem = null },
            onAddToLibrary = {
                posterActionItem = null
                onCatalogAction(CatalogAction.AddToLibraryRequested(item))
            }
        )
    }
}

@Composable
private fun FluxaHomeSkeleton(modifier: Modifier = Modifier) {
    LazyColumn(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(24.dp)
    ) {
        item(key = "hero-skeleton") {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .aspectRatio(3f / 4f)
                    .skeletonShimmer(shape = androidx.compose.ui.graphics.RectangleShape)
            )
        }
        items(3, key = { "row-skeleton-$it" }) {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Box(
                    modifier = Modifier
                        .padding(horizontal = 20.dp)
                        .width(120.dp)
                        .height(16.dp)
                        .skeletonShimmer()
                )
                LazyRow(
                    contentPadding = PaddingValues(horizontal = 20.dp),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    items(6, key = { skeletonIndex -> skeletonIndex }) {
                        Box(
                            modifier = Modifier
                                .width(110.dp)
                                .aspectRatio(2f / 3f)
                                .skeletonShimmer()
                        )
                    }
                }
            }
        }
    }
}

private const val HERO_AUTO_SWIPE_DELAY_MS = 5000L

@Composable
private fun FluxaHomeHero(
    items: List<CatalogItemUiModel>,
    billboard: CatalogBillboardUiModel?,
    language: String?,
    onCatalogAction: (CatalogAction) -> Unit,
    modifier: Modifier = Modifier
) {
    val loop = items.size > 1
    val virtualPageCount = if (loop) Int.MAX_VALUE else items.size
    val startPage = if (loop) (Int.MAX_VALUE / 2) - (Int.MAX_VALUE / 2) % items.size else 0
    val pagerState = rememberPagerState(initialPage = startPage, pageCount = { virtualPageCount })

    LaunchedEffect(pagerState.currentPage, pagerState.isScrollInProgress) {
        if (!pagerState.isScrollInProgress) {
            onCatalogAction(CatalogAction.HeroPageChanged(items[pagerState.currentPage % items.size]))
            if (loop) {
                delay(HERO_AUTO_SWIPE_DELAY_MS)
                if (!pagerState.isScrollInProgress) {
                    pagerState.animateScrollToPage(pagerState.currentPage + 1)
                }
            }
        }
    }

    Box(modifier = modifier.fillMaxWidth()) {
        HorizontalPager(
            state = pagerState,
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(3f / 4f)
                .heightIn(max = 560.dp)
        ) { page ->
            val item = items[page % items.size]
            val activeBillboard = billboard?.takeIf { it.item.id == item.id && it.item.type == item.type }
            FluxaHomeHeroSlide(
                item = item,
                trailerUrl = activeBillboard?.trailerUrl,
                trailerSubtitleCues = activeBillboard?.trailerSubtitleCues.orEmpty(),
                language = language,
                onCatalogAction = onCatalogAction
            )
        }
        if (items.size > 1) {
            val current = pagerState.currentPage % items.size
            Row(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(bottom = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                items.indices.forEach { index ->
                    val selected = index == current
                    Box(
                        modifier = Modifier
                            .size(if (selected) 8.dp else 6.dp)
                            .background(
                                color = Color.White.copy(alpha = if (selected) 0.95f else 0.4f),
                                shape = CircleShape
                            )
                    )
                }
            }
        }
    }
}

@Composable
private fun FluxaHomeHeroSlide(
    item: CatalogItemUiModel,
    trailerUrl: String?,
    trailerSubtitleCues: List<TrailerCue>,
    language: String?,
    onCatalogAction: (CatalogAction) -> Unit
) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .clickable { onCatalogAction(CatalogAction.ItemSelected(item)) }
    ) {
        val heroArtworkUrl = item.backdropUrl ?: item.card.artworkUrl
        FluxaRemoteImage(
            imageUrl = heroArtworkUrl,
            cacheKey = heroArtworkUrl?.let { "home-hero:$it" },
            contentDescription = item.card.title,
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Crop
        )
        val trailerSurface = LocalHeroTrailerSurface.current
        val trailerActive = trailerUrl != null && trailerSurface != null
        var activeSubtitle by remember(item.id, item.type) { mutableStateOf("") }
        if (trailerUrl != null && trailerSurface != null) {
            val trailerAlpha by animateFloatAsState(targetValue = 1f, label = "hero-trailer-fade")
            trailerSurface(trailerUrl, trailerSubtitleCues, { activeSubtitle = it }, Modifier.fillMaxSize().alpha(trailerAlpha))
        }
        val gradientStartY by animateFloatAsState(targetValue = if (trailerActive) 0.94f else 0.35f, label = "hero-gradient-start")
        val gradientMaxAlpha by animateFloatAsState(targetValue = if (trailerActive) 0.55f else 1f, label = "hero-gradient-alpha")
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.verticalGradient(
                        colors = listOf(Color.Transparent, FluxaColors.background.copy(alpha = gradientMaxAlpha)),
                        startY = gradientStartY
                    )
                )
        )
        val logoMaxHeight by animateDpAsState(targetValue = if (trailerActive) 34.dp else 64.dp, label = "hero-logo-height")
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .padding(start = 20.dp, end = 20.dp, top = 20.dp, bottom = if (trailerActive) 24.dp else 44.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            if (item.card.logoUrl != null) {
                FluxaRemoteImage(
                    imageUrl = item.card.logoUrl,
                    cacheKey = "home-hero-logo:${item.card.logoUrl}",
                    contentDescription = item.card.title,
                    modifier = Modifier.heightIn(max = logoMaxHeight).fillMaxWidth(0.65f),
                    contentScale = ContentScale.Fit
                )
            } else if (!trailerActive) {
                Text(
                    text = item.card.title,
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                    style = MaterialTheme.typography.headlineSmall
                )
            }
            val badgeParts = buildList {
                item.ageRating?.takeIf { it.isNotBlank() }?.let { add(it) }
                if (item.type == "series" && (item.seasonsCount ?: 0) > 0) {
                    add("${item.seasonsCount} ${AppStrings.t(language, "auto.seasons")}")
                } else {
                    item.runtimeLabel?.takeIf { it.isNotBlank() }?.let { add(it) }
                }
            }
            if (!trailerActive) {
                if (badgeParts.isNotEmpty()) {
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                        badgeParts.forEach { part ->
                            Text(text = part, color = Color.White.copy(alpha = 0.8f), fontSize = 13.sp)
                        }
                    }
                }
                item.description?.takeIf { it.isNotBlank() }?.let {
                    Text(
                        text = it,
                        color = Color.White.copy(alpha = 0.85f),
                        fontSize = 13.sp,
                        lineHeight = 15.sp,
                        maxLines = 3,
                        overflow = TextOverflow.Ellipsis,
                        textAlign = androidx.compose.ui.text.style.TextAlign.Center
                    )
                }
                Row(
                    modifier = Modifier
                        .padding(top = 6.dp)
                        .height(48.dp)
                        .clip(RoundedCornerShape(14.dp))
                        .background(Color.White)
                        .clickable { onCatalogAction(CatalogAction.PlayRequested(item)) }
                        .padding(horizontal = 24.dp),
                    horizontalArrangement = Arrangement.Center,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        imageVector = Icons.Rounded.PlayArrow,
                        contentDescription = null,
                        tint = Color.Black,
                        modifier = Modifier.size(26.dp)
                    )
                    Text(
                        text = AppStrings.t(language, "common.play"),
                        color = Color.Black,
                        fontWeight = FontWeight.ExtraBold,
                        fontSize = 15.sp,
                        modifier = Modifier.padding(start = 7.dp)
                    )
                }
            }
        }
        if (trailerActive && activeSubtitle.isNotBlank()) {
            Text(
                text = activeSubtitle,
                color = Color.White,
                fontWeight = FontWeight.Bold,
                fontSize = 16.sp,
                textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(bottom = 82.dp, start = 32.dp, end = 32.dp)
                    .clip(RoundedCornerShape(4.dp))
                    .background(Color.Black.copy(alpha = 0.58f))
                    .padding(horizontal = 10.dp, vertical = 4.dp)
            )
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
