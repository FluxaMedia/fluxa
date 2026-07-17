package com.fluxa.app.ui.routes

import android.content.Context
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
import com.fluxa.app.shared.FluxaDestination
import com.fluxa.app.ui.PlayerLaunchRequest
import com.fluxa.app.ui.TraktDeviceAuthUiState
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.HomeViewModel
import com.fluxa.app.ui.catalog.UpdateManager
import com.fluxa.app.ui.catalog.BiometricLockHelper
import com.fluxa.app.ui.catalog.copyProfileImageToLocalUri
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
internal fun AppRoutesHost(
    context: Context,
    currentDestination: FluxaDestination,
    authStartOnNuvio: Boolean,
    playerRequest: PlayerLaunchRequest?,
    deviceType: DeviceType,
    androidFluxaPlatformServices: com.fluxa.app.ui.AndroidFluxaPlatformServices?,
    activeProfile: UserProfile?,
    onActiveProfileChanged: (UserProfile?) -> Unit,
    onNavigateToDestination: (FluxaDestination) -> Unit,
    onPlayerRequestChanged: (PlayerLaunchRequest?) -> Unit,
    profileManager: ProfileManager,
    homeViewModel: HomeViewModel,
    mainPlayer: ExoPlayer,
    coroutineScope: CoroutineScope,
    offlineDownloadManager: OfflineDownloadManager,
    onShowTraktSheet: () -> Unit,
    onShowSimklSheet: () -> Unit,
    onTraktDeviceAuthChanged: (TraktDeviceAuthUiState?) -> Unit,
    onUpdateInfoChanged: (UpdateManager.UpdateInfo?) -> Unit,
    navigateBackSafely: () -> Unit,
    settingsPopRequestId: Int,
    onSettingsCanPopChanged: (Boolean) -> Unit
) {
    if (playerRequest != null) {
        PlayerRoute(
            request = playerRequest,
            activeProfile = activeProfile,
            profileManager = profileManager,
            homeViewModel = homeViewModel,
            mainPlayer = mainPlayer,
            onUpdatePlayerRequest = onPlayerRequestChanged,
            onBack = navigateBackSafely,
            onProfileChanged = onActiveProfileChanged
        )
        return
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

    com.fluxa.app.shared.FluxaAppHost(
        platformServices = androidFluxaPlatformServices!!,
        deviceType = deviceType,
        language = activeProfile?.language,
        destination = currentDestination,
        onDetailNavigationEvent = onDetailNavigationEvent@{ event ->
            val androidDetailSource = androidFluxaPlatformServices.detailDataSource
            val detail = androidDetailSource.detailViewModel.uiState.value.detail ?: return@onDetailNavigationEvent
            val meta = com.fluxa.app.data.remote.Meta(
                id = detail.id, name = detail.name, type = detail.type, poster = detail.poster, background = detail.background,
                logo = detail.logo, description = detail.description, imdbRating = detail.imdbRating, releaseInfo = detail.releaseInfo,
                released = detail.released, originalLanguage = detail.originalLanguage, originalName = detail.originalName,
                videos = detail.videos, trailers = detail.trailers
            )
            when (event) {
                is com.fluxa.app.shared.feature.detail.DetailNavigationEvent.PlayStream -> {
                    val resolvedStream = androidDetailSource.resolveStream(event.stream.playableUrl)
                    val streams = androidDetailSource.detailViewModel.uiState.value.filteredStreams
                    val index = resolvedStream?.let { streams.indexOf(it) }?.coerceAtLeast(0) ?: 0
                    onPlayerRequestChanged(
                        PlayerLaunchRequest(
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
        onDetailBackRequested = navigateBackSafely,
        showNavigationBar = true,
        onCatalogAction = { },
        onOpenUrlRequested = { url ->
            context.startActivity(android.content.Intent(android.content.Intent.ACTION_VIEW, android.net.Uri.parse(url)))
        },
        onAddonStoreBackRequested = navigateBackSafely,
        onAuthBackRequested = navigateBackSafely,
        onAuthCompleted = { onNavigateToDestination(FluxaDestination.Home) },
        authStartOnNuvio = authStartOnNuvio,
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
                onNavigateToDestination(FluxaDestination.Home)
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
        settingsPopRequestId = settingsPopRequestId,
        onSettingsCanPopChanged = onSettingsCanPopChanged,
        onManageAddonsRequested = { onNavigateToDestination(FluxaDestination.AddonStore) },
        onConnectStremioRequested = {
            val profile = activeProfile
            if (profile?.authKey.isNullOrBlank()) {
                onNavigateToDestination(FluxaDestination.Auth)
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
                onNavigateToDestination(FluxaDestination.Auth)
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
                    onPlayerRequestChanged(
                        PlayerLaunchRequest(
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
