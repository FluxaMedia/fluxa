package com.fluxa.app.shared

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Notifications
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.image.FluxaRemoteImage
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
import com.fluxa.app.shared.feature.discover.DiscoverAction
import com.fluxa.app.shared.feature.discover.DiscoverScreen
import com.fluxa.app.shared.feature.discover.DiscoverUiState
import com.fluxa.app.shared.feature.library.LibraryAction
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
import com.fluxa.app.ui.catalog.CatalogCard
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

private val FluxaBottomNavDestinations = listOf(
    FluxaDestination.Home,
    FluxaDestination.Search,
    FluxaDestination.Discover,
    FluxaDestination.Calendar,
    FluxaDestination.Library,
    FluxaDestination.Settings
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
    val editingProfile: ProfileEditTarget? = null,
    val showNotifications: Boolean = false
)

@Composable
fun FluxaApp(
    state: FluxaAppUiState,
    onDestinationSelected: (FluxaDestination) -> Unit,
    onCatalogAction: (CatalogAction) -> Unit,
    detailState: DetailUiState? = null,
    onDetailAction: (DetailAction) -> Unit = {},
    onDetailBackRequested: () -> Unit = {},
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
    biometricAvailable: Boolean = false,
    showNavigationBar: Boolean = true,
    modifier: Modifier = Modifier
) {
    MaterialTheme(colorScheme = FluxaColorScheme) {
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
            val screenKey = when {
                state.editingProfile != null -> "profileEdit"
                state.selectedDetail != null -> "detail:${state.selectedDetail.id}"
                state.showNotifications -> "notifications"
                else -> "dest:${state.destination}"
            }
            AnimatedContent(
                targetState = screenKey,
                transitionSpec = {
                    (fadeIn(tween(220)) + scaleIn(initialScale = 0.98f, animationSpec = tween(220)))
                        .togetherWith(fadeOut(tween(120)))
                },
                label = "fluxa-screen-transition",
                modifier = Modifier.weight(1f)
            ) { _ ->
            when {
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
                state.selectedDetail != null && detailState != null -> DetailScreen(
                    state = detailState,
                    language = state.language,
                    onAction = onDetailAction,
                    onBack = onDetailBackRequested,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Search && searchState != null -> SearchScreen(
                    state = searchState,
                    language = state.language,
                    onQueryChanged = { value -> onSearchAction(SearchAction.QueryChanged(value)) },
                    onItemSelected = { item -> onSearchAction(SearchAction.ItemSelected(item)) },
                    onClearHistory = { onSearchAction(SearchAction.ClearHistory) },
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
                state.destination == FluxaDestination.Calendar && calendarState != null -> CalendarScreen(
                    state = calendarState,
                    language = state.language,
                    onAction = onCalendarAction,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Library && libraryState != null -> LibraryScreen(
                    state = libraryState,
                    language = state.language,
                    onAction = onLibraryAction,
                    onItemSelected = onLibraryItemSelected,
                    modifier = Modifier.fillMaxSize()
                )
                state.destination == FluxaDestination.Settings && settingsState != null -> SettingsScreen(
                    state = settingsState,
                    language = state.language,
                    onAction = onSettingsAction,
                    onSwitchProfilesRequested = onSwitchProfilesRequested,
                    onBackRequested = onSettingsBackRequested,
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
                state.destination == FluxaDestination.Home -> FluxaHomeContent(
                    state = state,
                    onCatalogAction = onCatalogAction,
                    onNotificationsRequested = onNotificationsRequested,
                    onAvatarClick = { onDestinationSelected(FluxaDestination.Settings) },
                    profileAvatarUrl = profileState?.activeProfile?.avatarUrl,
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
        FluxaBottomNavDestinations.forEach { item ->
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
    onFilterSelected: (String) -> Unit,
    onNotificationsRequested: () -> Unit,
    onAvatarClick: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(
                Brush.verticalGradient(
                    colorStops = arrayOf(0f to Color.Black.copy(alpha = 0.55f), 1f to Color.Transparent)
                )
            )
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp, vertical = 12.dp),
            horizontalArrangement = Arrangement.End,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = Icons.Filled.Notifications,
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
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp),
            horizontalArrangement = Arrangement.spacedBy(20.dp)
        ) {
            FluxaHomeFilter.entries.forEach { filter ->
                val selected = filter.value == activeFilter || (filter == FluxaHomeFilter.All && activeFilter.isBlank())
                Column(modifier = Modifier.clickable { onFilterSelected(filter.value) }) {
                    Text(
                        text = AppStrings.t(language, filter.titleKey),
                        color = if (selected) Color.White else Color.White.copy(alpha = 0.6f),
                        fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
                        fontSize = 15.sp
                    )
                    Box(
                        modifier = Modifier
                            .padding(top = 8.dp, bottom = 12.dp)
                            .height(2.dp)
                            .width(if (selected) 20.dp else 0.dp)
                            .background(if (selected) Color.White else Color.Transparent)
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
    profileAvatarUrl: String?,
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

    val heroItems = state.catalogHome.rows.firstOrNull()?.items?.take(5).orEmpty()

    LazyColumn(
        modifier = modifier,
        contentPadding = PaddingValues(bottom = 120.dp),
        verticalArrangement = Arrangement.spacedBy(24.dp)
    ) {
        if (heroItems.isNotEmpty()) {
            item(key = "hero") {
                Box {
                    FluxaHomeHero(
                        items = heroItems,
                        language = state.language,
                        onCatalogAction = onCatalogAction
                    )
                    FluxaHomeOverlayBar(
                        activeFilter = state.catalogHome.activeFilter,
                        language = state.language,
                        profileAvatarUrl = profileAvatarUrl,
                        onFilterSelected = { filter -> onCatalogAction(CatalogAction.FilterChanged(filter)) },
                        onNotificationsRequested = onNotificationsRequested,
                        onAvatarClick = onAvatarClick
                    )
                }
            }
        }
        items(state.catalogHome.rows, key = { it.id }) { row ->
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
                        fontWeight = FontWeight.Bold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f)
                    )
                    if (row.canLoadMore) {
                        Text(
                            text = AppStrings.t(state.language, "common.view_all"),
                            color = Color.White.copy(alpha = 0.7f),
                            maxLines = 1,
                            modifier = Modifier
                                .padding(start = 12.dp)
                                .clickable {
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
    language: String?,
    onCatalogAction: (CatalogAction) -> Unit
) {
    val loop = items.size > 1
    val virtualPageCount = if (loop) Int.MAX_VALUE else items.size
    val startPage = if (loop) (Int.MAX_VALUE / 2) - (Int.MAX_VALUE / 2) % items.size else 0
    val pagerState = rememberPagerState(initialPage = startPage, pageCount = { virtualPageCount })

    if (loop) {
        LaunchedEffect(pagerState) {
            while (true) {
                delay(HERO_AUTO_SWIPE_DELAY_MS)
                pagerState.animateScrollToPage(pagerState.currentPage + 1)
            }
        }
    }

    Box(modifier = Modifier.fillMaxWidth()) {
        HorizontalPager(
            state = pagerState,
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(3f / 4f)
        ) { page ->
            val item = items[page % items.size]
            FluxaHomeHeroSlide(item = item, language = language, onCatalogAction = onCatalogAction)
        }
        if (items.size > 1) {
            Row(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(bottom = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                items.indices.forEach { index ->
                    val selected = index == pagerState.currentPage % items.size
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
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.verticalGradient(
                        colors = listOf(Color.Transparent, FluxaColors.background),
                        startY = 0.35f
                    )
                )
        )
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .padding(horizontal = 20.dp, vertical = 20.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            if (item.card.logoUrl != null) {
                FluxaRemoteImage(
                    imageUrl = item.card.logoUrl,
                    cacheKey = "home-hero-logo:${item.card.logoUrl}",
                    contentDescription = item.card.title,
                    modifier = Modifier.heightIn(max = 64.dp).fillMaxWidth(0.65f),
                    contentScale = ContentScale.Fit
                )
            } else {
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
            if (badgeParts.isNotEmpty()) {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                    badgeParts.forEachIndexed { index, part ->
                        if (index == 0) {
                            Text(
                                text = part,
                                color = Color.White,
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier
                                    .background(FluxaColors.accent, RoundedCornerShape(3.dp))
                                    .padding(horizontal = 6.dp, vertical = 2.dp)
                            )
                        } else {
                            Text(text = part, color = Color.White.copy(alpha = 0.8f), fontSize = 13.sp)
                        }
                    }
                }
            }
            item.description?.takeIf { it.isNotBlank() }?.let {
                Text(
                    text = it,
                    color = Color.White.copy(alpha = 0.85f),
                    fontSize = 13.sp,
                    lineHeight = 15.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    textAlign = androidx.compose.ui.text.style.TextAlign.Center
                )
            }
            Button(
                onClick = { onCatalogAction(CatalogAction.ItemSelected(item)) },
                colors = ButtonDefaults.buttonColors(containerColor = Color.White),
                modifier = Modifier.padding(top = 4.dp)
            ) {
                Text(text = AppStrings.t(language, "common.play"), color = Color.Black)
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
