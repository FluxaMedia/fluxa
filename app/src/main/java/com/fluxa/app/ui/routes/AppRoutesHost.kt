package com.fluxa.app.ui.routes

import android.content.Context
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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.OfflineDownloadManager
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import com.fluxa.app.ui.TraktDeviceAuthUiState
import com.fluxa.app.ui.navDirection
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.HomeViewModel
import com.fluxa.app.ui.catalog.UpdateManager
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
    mainPlayer: ExoPlayer,
    coroutineScope: CoroutineScope,
    offlineDownloadManager: OfflineDownloadManager,
    onShowTraktSheet: () -> Unit,
    onShowSimklSheet: () -> Unit,
    onTraktDeviceAuthChanged: (TraktDeviceAuthUiState?) -> Unit,
    onUpdateInfoChanged: (UpdateManager.UpdateInfo?) -> Unit,
    navigateBackSafely: () -> Unit
) {
    val mobileSharedDestination = when {
        deviceType == DeviceType.Mobile -> when (currentScreen) {
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
        deviceType == DeviceType.TV -> when (currentScreen) {
            is Screen.Home -> com.fluxa.app.shared.FluxaDestination.Home
            is Screen.Search -> com.fluxa.app.shared.FluxaDestination.Search
            is Screen.Explore -> com.fluxa.app.shared.FluxaDestination.Discover
            is Screen.Calendar -> com.fluxa.app.shared.FluxaDestination.Calendar
            is Screen.Watchlist -> com.fluxa.app.shared.FluxaDestination.Library
            is Screen.Settings -> com.fluxa.app.shared.FluxaDestination.Settings
            is Screen.AddonStore -> com.fluxa.app.shared.FluxaDestination.AddonStore
            is Screen.Profiles -> com.fluxa.app.shared.FluxaDestination.ProfileList
            is Screen.Welcome, is Screen.Login -> com.fluxa.app.shared.FluxaDestination.Auth
            else -> null
        }
        else -> null
    }

    val mobileDetailScreen = (currentScreen as? Screen.Detail).takeIf {
        deviceType == DeviceType.Mobile || deviceType == DeviceType.TV
    }
    val mobileDetailRequest = mobileDetailScreen?.let { screen ->
        androidFluxaPlatformServices?.detailDataSource?.setInitialMeta(screen.initialMeta)
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
            lastStreamTitle = screen.lastStreamTitle,
            initialContent = screen.initialMeta?.let { meta ->
                com.fluxa.app.shared.feature.detail.DetailUiModel(
                    id = meta.id,
                    type = meta.type,
                    title = meta.name,
                    description = meta.description.orEmpty(),
                    posterUrl = meta.poster,
                    backgroundUrl = meta.background,
                    logoUrl = meta.logo,
                    releaseLabel = meta.releaseInfo.orEmpty(),
                    ratingLabel = meta.imdbRating.orEmpty(),
                    runtimeLabel = meta.runtime,
                    ageRating = meta.ageRating,
                    isInWatchlist = false,
                    relatedItems = emptyList(),
                    availableSeasons = meta.videos
                        ?.mapNotNull { it.season }
                        ?.filter { it >= 0 }
                        ?.distinct()
                        ?.sorted()
                        .orEmpty()
                )
            }
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

    if (mobileSharedDestination != null || mobileDetailRequest != null) {
        com.fluxa.app.shared.FluxaAppHost(
            platformServices = androidFluxaPlatformServices!!,
            deviceType = deviceType,
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
                    is com.fluxa.app.shared.feature.detail.DetailNavigationEvent.SelectSources -> Unit
                }
            },
            onDetailBackRequested = { if (mobileDetailScreen != null) navigateBackSafely() },
            showNavigationBar = mobileDetailScreen == null,
            onCatalogAction = { action ->
            },
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
            onProfileSelectionCompleted = { profileId ->
                profileManager.getProfiles().firstOrNull { it.id == profileId }?.let { profile ->
                    onActiveProfileChanged(profile)
                    profileManager.setLastActiveProfile(profile)
                    navigator.navigateTo(Screen.Home, true)
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
            is Screen.Profiles,
            is Screen.Welcome,
            is Screen.Login,
            is Screen.Home,
            is Screen.Explore,
            is Screen.Search,
            is Screen.Calendar,
            is Screen.Watchlist,
            is Screen.Detail,
            is Screen.AddonStore,
            is Screen.Settings -> error("Shared route was not rendered")
        }
    }
}
