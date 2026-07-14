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
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.ui.unit.dp
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.data.local.OfflineDownloadManager
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import com.fluxa.app.ui.asNavigationMeta
import com.fluxa.app.ui.TraktDeviceAuthUiState
import com.fluxa.app.ui.installLocalAddonForProfile
import com.fluxa.app.ui.mobileNavDestination
import com.fluxa.app.ui.moveLocalAddonForProfile
import com.fluxa.app.ui.navDirection
import com.fluxa.app.ui.removeLocalAddonForProfile
import com.fluxa.app.ui.setLocalAddonEnabledForProfile
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.AddonStoreScreen
import com.fluxa.app.ui.catalog.CalendarScreen
import com.fluxa.app.ui.catalog.CategoryResultsScreen
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.ExploreScreen
import com.fluxa.app.ui.catalog.HomeViewModel
import com.fluxa.app.ui.catalog.LoginScreen
import com.fluxa.app.ui.catalog.ProfileEditScreen
import com.fluxa.app.ui.catalog.ProfileScreen
import com.fluxa.app.ui.catalog.SearchScreen
import com.fluxa.app.ui.catalog.UpdateManager
import com.fluxa.app.ui.catalog.tvNavDestination
import com.fluxa.app.ui.catalog.WatchlistScreen
import com.fluxa.app.ui.catalog.WelcomeScreen
import com.fluxa.app.ui.catalog.FluxaDimensions
import com.fluxa.app.ui.catalog.BiometricLockHelper
import com.fluxa.app.ui.catalog.copyProfileImageToLocalUri
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
internal fun AppRoutesHost(
    context: Context,
    currentScreen: Screen,
    deviceType: DeviceType,
    androidFluxaPlatformServices: com.fluxa.app.ui.AndroidFluxaPlatformServices?,
    sharedDetailViewModel: com.fluxa.app.ui.catalog.DetailViewModel,
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
    onShowMalSheet: () -> Unit,
    onShowSimklSheet: () -> Unit,
    onTraktDeviceAuthChanged: (TraktDeviceAuthUiState?) -> Unit,
    onPendingMalCodeVerifierChanged: (String?) -> Unit,
    onUpdateInfoChanged: (UpdateManager.UpdateInfo?) -> Unit,
    navigateBackSafely: () -> Unit
) {
    val onSharedPlayRequested: () -> Unit = {
        sharedDetailViewModel.uiState.value.detail?.let { meta ->
            val savedPlayback = sharedDetailViewModel.uiState.value.savedPlayback
            val targetVideoId = if (meta.type == "series") savedPlayback?.lastVideoId else null
            val resumeProgress = if (targetVideoId != null) savedPlayback?.timeOffset ?: 0L else 0L
            if (meta.id.startsWith("cs3:") || targetVideoId?.startsWith("cs3:") == true) {
                navigator.navigateTo(
                    Screen.Player(
                        meta = meta.asNavigationMeta(),
                        videoId = targetVideoId,
                        initialProgress = resumeProgress
                    )
                )
            } else {
                navigator.navigateTo(
                    Screen.Sources(
                        meta = meta.asNavigationMeta(),
                        videoId = targetVideoId,
                        initialProgress = resumeProgress
                    )
                )
            }
        }
    }
    val tvNavActions = com.fluxa.app.ui.catalog.TvNavActions(
        onHome = { if (currentScreen.tvNavDestination() != com.fluxa.app.ui.catalog.TvNavDestination.Home) navigator.navigateTo(Screen.Home, clearStack = true) },
        onSearch = { if (currentScreen.tvNavDestination() != com.fluxa.app.ui.catalog.TvNavDestination.Search) navigator.navigateTo(Screen.Search, clearStack = true) },
        onWatchlist = { if (currentScreen.tvNavDestination() != com.fluxa.app.ui.catalog.TvNavDestination.Library) navigator.navigateTo(Screen.Watchlist, clearStack = true) },
        onExplore = { if (currentScreen.tvNavDestination() != com.fluxa.app.ui.catalog.TvNavDestination.Discover) navigator.navigateTo(Screen.Explore(), clearStack = true) },
        onSettings = { if (currentScreen.tvNavDestination() != com.fluxa.app.ui.catalog.TvNavDestination.Settings) navigator.navigateTo(Screen.Settings(), clearStack = true) }
    )

    val mobileSharedDestination = if (deviceType == DeviceType.Mobile) {
        when (currentScreen) {
            is Screen.Home -> com.fluxa.app.shared.FluxaDestination.Home
            is Screen.Search -> com.fluxa.app.shared.FluxaDestination.Search
            is Screen.Explore -> com.fluxa.app.shared.FluxaDestination.Discover
            is Screen.Calendar -> com.fluxa.app.shared.FluxaDestination.Calendar
            is Screen.AddonStore -> com.fluxa.app.shared.FluxaDestination.AddonStore
            is Screen.Welcome, is Screen.Login -> com.fluxa.app.shared.FluxaDestination.Auth
            is Screen.Profiles -> com.fluxa.app.shared.FluxaDestination.ProfileList
            is Screen.Settings -> com.fluxa.app.shared.FluxaDestination.Settings
            is Screen.Watchlist -> com.fluxa.app.shared.FluxaDestination.Library
            else -> null
        }
    } else {
        null
    }

    val mobileDetailScreen = (currentScreen as? Screen.Detail).takeIf { deviceType == DeviceType.Mobile }
    val mobileDetailRequest = mobileDetailScreen?.let { screen ->
        com.fluxa.app.shared.feature.detail.DetailRequestUiModel(
            id = screen.id,
            type = screen.type,
            source = com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel(
                addonTransportUrl = screen.sourceAddonTransportUrl,
                catalogType = screen.sourceAddonCatalogType
            ),
            initialProgress = screen.initialProgress,
            lastVideoId = screen.lastVideoId,
            lastStreamIndex = screen.lastStreamIndex,
            autoPlay = screen.autoPlay,
            targetSeason = screen.targetSeason,
            targetEpisode = screen.targetEpisode,
            lastStreamUrl = screen.lastStreamUrl,
            lastStreamTitle = screen.lastStreamTitle
        )
    }

    var pendingAvatarPicked by remember { mutableStateOf<((String?) -> Unit)?>(null) }
    val avatarPicker = rememberLauncherForActivityResult(ActivityResultContracts.GetContent()) { uri ->
        val callback = pendingAvatarPicked
        pendingAvatarPicked = null
        if (uri == null || callback == null) return@rememberLauncherForActivityResult
        coroutineScope.launch {
            val copied = withContext(Dispatchers.IO) { copyProfileImageToLocalUri(context, uri) ?: uri.toString() }
            callback(copied)
        }
    }

    if (deviceType == DeviceType.Mobile) {
        com.fluxa.app.shared.FluxaAppHost(
            platformServices = androidFluxaPlatformServices!!,
            language = activeProfile?.language,
            destination = mobileSharedDestination,
            detailRequest = mobileDetailRequest,
            onDetailNavigationEvent = onDetailNavigationEvent@{ event ->
                val screen = mobileDetailScreen ?: return@onDetailNavigationEvent
                val androidDetailSource = androidFluxaPlatformServices.detailDataSource
                val detail = androidDetailSource.detailViewModel.uiState.value.detail
                val meta = detail?.let {
                    com.fluxa.app.data.remote.Meta(
                        id = it.id, name = it.name, type = it.type, poster = it.poster, background = it.background,
                        logo = it.logo, description = it.description, imdbRating = it.imdbRating, releaseInfo = it.releaseInfo,
                        released = it.released, originalLanguage = it.originalLanguage, originalName = it.originalName,
                        videos = it.videos, trailers = it.trailers
                    )
                } ?: com.fluxa.app.data.remote.Meta(screen.id, "", screen.type, "", "")
                when (event) {
                    is com.fluxa.app.shared.feature.detail.DetailNavigationEvent.PlayStream -> {
                        val resolvedStream = androidDetailSource.resolveStream(event.stream.playableUrl)
                        val streams = androidDetailSource.detailViewModel.uiState.value.filteredStreams
                        val index = resolvedStream?.let { streams.indexOf(it) }?.coerceAtLeast(0) ?: 0
                        navigator.navigateTo(
                            Screen.Player(
                                meta = meta,
                                videoId = event.episodeId,
                                initialProgress = event.resumeProgress,
                                streamIndex = index,
                                initialStreams = streams,
                                lastStreamUrl = event.stream.playableUrl,
                                lastStreamTitle = event.stream.title
                            )
                        )
                    }
                    is com.fluxa.app.shared.feature.detail.DetailNavigationEvent.SelectSources -> {
                        val episode = androidDetailSource.detailViewModel.uiState.value.seasonEpisodes
                            .firstOrNull { it.id == event.episodeId }
                        navigator.navigateTo(
                            Screen.Sources(
                                meta = meta,
                                video = episode,
                                videoId = event.episodeId,
                                initialProgress = event.resumeProgress,
                                lastStreamIndex = screen.lastStreamIndex,
                                lastStreamUrl = screen.lastStreamUrl,
                                lastStreamTitle = screen.lastStreamTitle
                            )
                        )
                    }
                }
            },
            onDetailBackRequested = { if (mobileDetailScreen != null) navigateBackSafely() },
            showNavigationBar = false,
            onPlayRequested = onSharedPlayRequested,
            onOpenUrlRequested = { url ->
                context.startActivity(android.content.Intent(android.content.Intent.ACTION_VIEW, android.net.Uri.parse(url)))
            },
            onAddonStoreBackRequested = navigateBackSafely,
            onAuthBackRequested = navigateBackSafely,
            onAuthCompleted = { navigator.navigateTo(Screen.Home, true) },
            authStartOnNuvio = (currentScreen as? Screen.Login)?.startOnNuvio == true,
            biometricAvailable = BiometricLockHelper.isAvailable(context),
            onPickAvatarRequested = { onPicked ->
                pendingAvatarPicked = onPicked
                avatarPicker.launch("image/*")
            },
            onBiometricAuthRequested = { profile, onResult ->
                val activity = context as? androidx.fragment.app.FragmentActivity
                if (activity != null) {
                    BiometricLockHelper.authenticate(
                        activity = activity,
                        lang = profile.language,
                        onSuccess = { onResult(true) },
                        onFailure = { onResult(false) }
                    )
                } else {
                    onResult(false)
                }
            },
            nuvioIcon = {
                androidx.compose.foundation.Image(
                    painter = androidx.compose.ui.res.painterResource(id = com.fluxa.app.R.drawable.ic_nuvio),
                    contentDescription = null,
                    modifier = Modifier.size(26.dp)
                )
            },
            stremioIcon = {
                androidx.compose.foundation.Image(
                    painter = androidx.compose.ui.res.painterResource(id = com.fluxa.app.R.drawable.ic_stremio),
                    contentDescription = null,
                    modifier = Modifier.size(26.dp)
                )
            },
            traktIcon = {
                androidx.compose.foundation.Image(
                    painter = androidx.compose.ui.res.painterResource(id = com.fluxa.app.R.drawable.ic_trakt),
                    contentDescription = null,
                    modifier = Modifier.size(26.dp)
                )
            },
            simklIcon = {
                androidx.compose.foundation.Image(
                    painter = androidx.compose.ui.res.painterResource(id = com.fluxa.app.R.drawable.ic_simkl),
                    contentDescription = null,
                    modifier = Modifier.size(26.dp)
                )
            },
            anilistIcon = {
                androidx.compose.foundation.Image(
                    painter = androidx.compose.ui.res.painterResource(id = com.fluxa.app.R.drawable.ic_anilist),
                    contentDescription = null,
                    modifier = Modifier.size(26.dp)
                )
            },
            onSettingsBackRequested = navigateBackSafely,
            onManageAddonsRequested = { navigator.navigateTo(Screen.AddonStore) },
            onConnectStremioRequested = {
                val profile = activeProfile
                if (profile?.authKey.isNullOrBlank()) {
                    navigator.navigateTo(Screen.Login())
                } else {
                    homeViewModel.syncStremioIntegration(
                        profile = profile!!,
                        onProfileUpdated = { updated ->
                            onActiveProfileChanged(updated)
                            profileManager.saveProfile(updated)
                            profileManager.setLastActiveProfile(updated)
                            homeViewModel.applyUpdatedProfile(updated, refreshHomeSideEffects = true)
                        },
                        onComplete = { }
                    )
                }
            },
            onConnectNuvioRequested = {
                val profile = activeProfile
                if (profile?.nuvioAccessToken.isNullOrBlank()) {
                    navigator.navigateTo(Screen.Login(startOnNuvio = true))
                } else {
                    homeViewModel.syncNuvioIntegration(
                        profile = profile!!,
                        onProfileUpdated = { updated ->
                            onActiveProfileChanged(updated)
                            profileManager.saveProfile(updated)
                            profileManager.setLastActiveProfile(updated)
                            homeViewModel.applyUpdatedProfile(updated, refreshHomeSideEffects = true)
                        },
                        onComplete = { }
                    )
                }
            },
            onConnectTraktRequested = {
                if (!activeProfile?.traktAccessToken.isNullOrBlank()) {
                    onShowTraktSheet()
                } else {
                    connectTrakt(context, activeProfile, profileManager, homeViewModel, onActiveProfileChanged, onTraktDeviceAuthChanged)
                }
            },
            onConnectSimklRequested = {
                if (!activeProfile?.simklAccessToken.isNullOrBlank()) {
                    onShowSimklSheet()
                } else {
                    connectSimkl(context, activeProfile)
                }
            },
            onConnectAnilistRequested = { connectAnilist(context, activeProfile) },
            onCheckForUpdateRequested = {
                coroutineScope.launch {
                    val update = com.fluxa.app.ui.catalog.UpdateManager.checkUpdate()
                    onUpdateInfoChanged(update)
                }
            },
            onDownloadOpened = { id ->
                offlineDownloadManager.items.value.firstOrNull { it.id == id }?.let { item ->
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
                }
            },
            modifier = Modifier
                .fillMaxSize()
                .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Top + WindowInsetsSides.Horizontal))
        )
    }

    if (mobileSharedDestination != null || mobileDetailRequest != null) {
        return
    }

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
                    fadeIn(animationSpec = tween(FluxaDimensions.AnimDuration.contentExpand, easing = FastOutSlowInEasing)) +
                        slideInHorizontally(
                            animationSpec = tween(FluxaDimensions.AnimDuration.settingsExpandAlt, easing = FastOutSlowInEasing),
                            initialOffsetX = { direction * distance }
                        ) +
                        scaleIn(
                            animationSpec = tween(FluxaDimensions.AnimDuration.settingsExpandAlt, easing = FastOutSlowInEasing),
                            initialScale = 0.985f
                        )
                    ) togetherWith (
                    fadeOut(animationSpec = tween(FluxaDimensions.AnimDuration.routeExit, easing = LinearOutSlowInEasing)) +
                        slideOutHorizontally(
                            animationSpec = tween(FluxaDimensions.AnimDuration.contentExpand, easing = FastOutSlowInEasing),
                            targetOffsetX = { -direction * distance }
                        ) +
                        scaleOut(
                            animationSpec = tween(FluxaDimensions.AnimDuration.contentExpand, easing = FastOutSlowInEasing),
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
            is Screen.Welcome -> WelcomeScreen(
                onContinueWithNuvio = { navigator.navigateTo(Screen.Login(startOnNuvio = true)) },
                onLoginWithStremio = { navigator.navigateTo(Screen.Login()) },
                onContinueWithoutAccount = {
                    val guest = UserProfile(
                        id = java.util.UUID.randomUUID().toString(),
                        email = AppStrings.t("en", "auth.primary_profile_name"),
                        authKey = "",
                        isGuest = false
                    )
                    profileManager.saveProfile(guest)
                    profileManager.setLastActiveProfile(guest)
                    onActiveProfileChanged(guest)
                    navigator.navigateTo(Screen.Home, true)
                }
            )
            is Screen.Login -> LoginScreen(
                context,
                onLoginSuccess = {
                    onActiveProfileChanged(it)
                    navigator.navigateTo(Screen.Home, true)
                },
                onCancel = navigateBackSafely,
                startOnNuvio = screen.startOnNuvio
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
                initialGenre = screen.initialGenre,
                tvNavActions = tvNavActions
            )
            is Screen.Search -> SearchScreen(
                activeProfile,
                homeViewModel.searchResults.collectAsState().value,
                { homeViewModel.search(it) },
                { meta, sourceAddonTransportUrl, sourceAddonCatalogType ->
                    navigator.navigateTo(meta.detailScreen(sourceAddonTransportUrl, sourceAddonCatalogType))
                },
                navigateBackSafely,
                homeViewModel,
                tvNavActions
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
                },
                tvNavActions = tvNavActions
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
                onShowMalSheet = onShowMalSheet,
                onShowSimklSheet = onShowSimklSheet,
                onTraktDeviceAuthChanged = onTraktDeviceAuthChanged,
                onPendingMalCodeVerifierChanged = onPendingMalCodeVerifierChanged,
                onUpdateInfoChanged = onUpdateInfoChanged,
                tvNavActions = tvNavActions
            )
        }
    }
}
