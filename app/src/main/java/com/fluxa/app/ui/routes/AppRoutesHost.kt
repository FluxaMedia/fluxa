package com.fluxa.app.ui.routes

import android.content.Context
import android.content.SharedPreferences
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Modifier
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.data.local.OfflineDownloadManager
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import com.fluxa.app.ui.TraktDeviceAuthUiState
import com.fluxa.app.ui.installLocalAddonForProfile
import com.fluxa.app.ui.mobileNavDestination
import com.fluxa.app.ui.moveLocalAddonForProfile
import com.fluxa.app.ui.navDirection
import com.fluxa.app.ui.removeLocalAddonForProfile
import com.fluxa.app.ui.setLocalAddonEnabledForProfile
import com.fluxa.app.ui.catalog.AddonStoreScreen
import com.fluxa.app.ui.catalog.AppStrings
import com.fluxa.app.ui.catalog.CalendarScreen
import com.fluxa.app.ui.catalog.CategoryResultsScreen
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.ExploreScreen
import com.fluxa.app.ui.catalog.HomeViewModel
import com.fluxa.app.ui.catalog.LoginScreen
import com.fluxa.app.ui.catalog.ProfileEditScreen
import com.fluxa.app.ui.catalog.ProfileScreen
import com.fluxa.app.ui.catalog.SearchScreen
import com.fluxa.app.ui.catalog.WatchlistScreen
import kotlinx.coroutines.CoroutineScope

@Composable
internal fun AppRoutesHost(
    context: Context,
    currentScreen: Screen,
    deviceType: DeviceType,
    activeProfile: UserProfile?,
    onActiveProfileChanged: (UserProfile?) -> Unit,
    navigator: AppNavigator,
    profileManager: ProfileManager,
    homeViewModel: HomeViewModel,
    previewPlayer: ExoPlayer,
    mainPlayer: ExoPlayer,
    coroutineScope: CoroutineScope,
    offlineDownloadManager: OfflineDownloadManager,
    oauthPrefs: SharedPreferences,
    onShowTraktSheet: () -> Unit,
    onTraktDeviceAuthChanged: (TraktDeviceAuthUiState?) -> Unit,
    onPendingMalCodeVerifierChanged: (String?) -> Unit,
    navigateBackSafely: () -> Unit
) {
    AnimatedContent(
        targetState = currentScreen,
        modifier = Modifier.then(
            if (deviceType == DeviceType.Mobile && currentScreen !is Screen.Player) {
                Modifier.windowInsetsPadding(
                    WindowInsets.safeDrawing.only(WindowInsetsSides.Top + WindowInsetsSides.Horizontal)
                )
            } else {
                Modifier
            }
        ),
        transitionSpec = {
            if (activeProfile?.safeAnimationsEnabled == false || activeProfile?.safeReduceMotion == true) {
                EnterTransition.None togetherWith ExitTransition.None
            } else {
                val direction = navDirection(initialState, targetState)
                val distance = if (deviceType == DeviceType.Mobile) 92 else 54
                (
                    fadeIn(animationSpec = tween(220, easing = FastOutSlowInEasing)) +
                        slideInHorizontally(
                            animationSpec = tween(260, easing = FastOutSlowInEasing),
                            initialOffsetX = { direction * distance }
                        ) +
                        scaleIn(
                            animationSpec = tween(260, easing = FastOutSlowInEasing),
                            initialScale = 0.985f
                        )
                    ) togetherWith (
                    fadeOut(animationSpec = tween(160, easing = LinearOutSlowInEasing)) +
                        slideOutHorizontally(
                            animationSpec = tween(220, easing = FastOutSlowInEasing),
                            targetOffsetX = { -direction * distance }
                        ) +
                        scaleOut(
                            animationSpec = tween(220, easing = FastOutSlowInEasing),
                            targetScale = 0.992f
                        )
                    )
            }.using(null)
        },
        label = "nav"
    ) { screen ->
        when (screen) {
            is Screen.Profiles -> ProfileScreen(
                context,
                onProfileSelected = {
                    onActiveProfileChanged(it)
                    profileManager.setLastActiveProfile(it)
                    navigator.navigateTo(Screen.Home, true)
                },
                onAddProfileClick = { navigator.navigateTo(Screen.ProfileEdit()) },
                onEditProfileClick = { navigator.navigateTo(Screen.ProfileEdit(it)) },
                viewModel = homeViewModel
            )
            is Screen.ProfileEdit -> ProfileEditScreen(
                initialProfile = screen.profile,
                onSave = {
                    profileManager.saveProfile(it)
                    onActiveProfileChanged(it)
                    profileManager.setLastActiveProfile(it)
                    navigator.navigateBack()
                },
                onDelete = {
                    profileManager.deleteProfileById(it.id)
                    if (activeProfile?.id == it.id) {
                        onActiveProfileChanged(null)
                        profileManager.setLastActiveProfile(null)
                        navigator.navigateTo(Screen.Profiles, true)
                    } else {
                        navigator.navigateBack()
                    }
                },
                onCancel = { navigator.navigateBack() }
            )
            is Screen.Login -> LoginScreen(
                context,
                onLoginSuccess = {
                    onActiveProfileChanged(it)
                    navigator.navigateTo(Screen.Home, true)
                },
                onCancel = navigateBackSafely
            )
            is Screen.Home -> HomeRoute(activeProfile, navigator, homeViewModel, previewPlayer, coroutineScope)
            is Screen.CategoryResults -> CategoryResultsScreen(
                activeProfile = activeProfile,
                categoryId = screen.categoryId,
                title = screen.title,
                onMovieClick = { meta, sourceAddonTransportUrl, sourceAddonCatalogType -> navigator.navigateTo(meta.detailScreen(sourceAddonTransportUrl, sourceAddonCatalogType)) },
                onBack = navigateBackSafely,
                viewModel = homeViewModel
            )
            is Screen.Explore -> ExploreScreen(
                activeProfile,
                { meta, sourceAddonTransportUrl, sourceAddonCatalogType ->
                    navigator.navigateTo(meta.detailScreen(sourceAddonTransportUrl, sourceAddonCatalogType))
                },
                navigateBackSafely,
                homeViewModel,
                initialType = screen.initialType,
                initialGenre = screen.initialGenre
            )
            is Screen.Search -> SearchScreen(
                activeProfile,
                homeViewModel.searchResults.collectAsState().value,
                { homeViewModel.search(it) },
                { meta, sourceAddonTransportUrl, sourceAddonCatalogType ->
                    navigator.navigateTo(meta.detailScreen(sourceAddonTransportUrl, sourceAddonCatalogType))
                },
                navigateBackSafely,
                homeViewModel
            )
            is Screen.Calendar -> CalendarScreen(
                activeProfile = activeProfile,
                viewModel = homeViewModel,
                onMovieClick = { navigator.navigateTo(it.detailScreen()) }
            )
            is Screen.Watchlist -> WatchlistScreen(
                activeProfile,
                {
                    if (it.type == "catalog_folder") {
                        navigator.navigateTo(Screen.CategoryResults(it.id, it.name))
                    } else {
                        navigator.navigateTo(it.detailScreen())
                    }
                },
                {
                    val canResumeDirect = it.lastVideoId != null || (it.timeOffset ?: 0L) > 0L
                    if (canResumeDirect) {
                        navigator.navigateTo(it.sourcesScreen())
                    } else if (it.type == "movie") {
                        navigator.navigateTo(Screen.Sources(it))
                    } else {
                        navigator.navigateTo(Screen.Detail(it.type, it.id))
                    }
                },
                navigateBackSafely,
                homeViewModel,
                offlineDownloadManager = offlineDownloadManager,
                onOpenDownload = { item ->
                    offlineDownloadManager.refresh()
                    if (item.isPlayable) {
                        navigator.navigateTo(
                            Screen.Player(
                                meta = offlineDownloadManager.asPlayableMeta(item),
                                videoId = item.videoId,
                                initialProgress = 0L,
                                streamIndex = 0,
                                initialStreams = listOf(offlineDownloadManager.asPlayableStream(item))
                            )
                        )
                    }
                },
                onUpdateProfile = {
                    onActiveProfileChanged(it)
                    profileManager.saveProfile(it)
                    homeViewModel.applyUpdatedProfile(it)
                }
            )
            is Screen.Detail -> DetailRoute(screen, activeProfile, navigator, navigateBackSafely)
            is Screen.Sources -> SourcesRoute(screen, activeProfile, navigator, navigateBackSafely)
            is Screen.Player -> PlayerRoute(
                screen = screen,
                activeProfile = activeProfile,
                profileManager = profileManager,
                homeViewModel = homeViewModel,
                mainPlayer = mainPlayer,
                navigator = navigator,
                onBack = navigateBackSafely,
                onProfileChanged = onActiveProfileChanged
            )
            is Screen.AddonStore -> AddonStoreScreen(
                activeProfile = activeProfile,
                onBack = navigateBackSafely,
                onInstallAddon = { addonUrl ->
                    installLocalAddonForProfile(activeProfile, addonUrl, profileManager, homeViewModel, onActiveProfileChanged)
                },
                onRemoveAddon = { addonUrl ->
                    removeLocalAddonForProfile(activeProfile, addonUrl, profileManager, homeViewModel, onActiveProfileChanged)
                },
                onMoveAddon = { addonUrl, direction ->
                    moveLocalAddonForProfile(activeProfile, addonUrl, direction, profileManager, homeViewModel, onActiveProfileChanged)
                },
                onToggleAddon = { addonUrl, enabled ->
                    setLocalAddonEnabledForProfile(activeProfile, addonUrl, enabled, profileManager, homeViewModel, onActiveProfileChanged)
                }
            )
            is Screen.Settings -> SettingsRoute(
                context = context,
                activeProfile = activeProfile,
                profileManager = profileManager,
                homeViewModel = homeViewModel,
                navigator = navigator,
                offlineDownloadManager = offlineDownloadManager,
                oauthPrefs = oauthPrefs,
                onBack = navigateBackSafely,
                onProfileChanged = onActiveProfileChanged,
                onShowTraktSheet = onShowTraktSheet,
                onTraktDeviceAuthChanged = onTraktDeviceAuthChanged,
                onPendingMalCodeVerifierChanged = onPendingMalCodeVerifierChanged
            )
        }
    }
}
