package com.fluxa.app.ui

import android.content.Context
import android.widget.Toast
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.ime
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.zIndex
import androidx.lifecycle.viewModelScope
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.HomeViewModel
import com.fluxa.app.ui.catalog.UpdateManager
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
internal fun AppChromeOverlays(
    context: Context,
    applicationContext: Context,
    currentScreen: Screen,
    deviceType: DeviceType,
    activeProfile: UserProfile?,
    profiles: List<UserProfile>,
    onActiveProfileChanged: (UserProfile?) -> Unit,
    onQuickProfileSelected: (UserProfile) -> Unit,
    profileManager: ProfileManager,
    navigator: AppNavigator,
    homeViewModel: HomeViewModel,
    updateInfo: UpdateManager.UpdateInfo?,
    isDownloading: Boolean,
    downloadProgress: Float,
    isDirectLoading: Boolean,
    showTraktSheet: Boolean,
    isTraktSyncing: Boolean,
    traktContinueWatchingLastUpdatedAt: Long,
    showMalSheet: Boolean,
    showSimklSheet: Boolean,
    onUpdateInfoChanged: (UpdateManager.UpdateInfo?) -> Unit,
    onDownloadingChanged: (Boolean) -> Unit,
    onDownloadProgressChanged: (Float) -> Unit,
    onShowTraktSheetChanged: (Boolean) -> Unit,
    onTraktSyncingChanged: (Boolean) -> Unit,
    onShowMalSheetChanged: (Boolean) -> Unit,
    onShowSimklSheetChanged: (Boolean) -> Unit
) {
    AppUpdateOverlay(
        update = updateInfo,
        deviceType = deviceType,
        activeProfile = activeProfile,
        isDownloading = isDownloading,
        downloadProgress = downloadProgress,
        onUpdateNow = {
            val update = updateInfo ?: return@AppUpdateOverlay
            onDownloadingChanged(true)
            homeViewModel.viewModelScope.launch {
                val success = UpdateManager.downloadAndInstall(
                    context = applicationContext,
                    updateUrl = update.url,
                    expectedSha256 = update.sha256,
                    onProgress = onDownloadProgressChanged
                )
                if (!success) {
                    withContext(Dispatchers.Main) {
                        Toast.makeText(
                            applicationContext,
                            AppStrings.t(activeProfile?.safeLanguage, "update.download_failed"),
                            Toast.LENGTH_SHORT
                        ).show()
                        onDownloadingChanged(false)
                    }
                }
            }
        },
        onSkip = { onUpdateInfoChanged(null) }
    )
    DirectLoadingOverlay(visible = isDirectLoading)
    if (showTraktSheet && activeProfile != null) {
        TraktIntegrationSheet(
            profile = activeProfile,
            lastContinueWatchingUpdatedAt = traktContinueWatchingLastUpdatedAt,
            syncing = isTraktSyncing,
            onDismiss = { onShowTraktSheetChanged(false) },
            onSyncNow = {
                onTraktSyncingChanged(true)
                homeViewModel.syncTraktIntegration(
                    profile = activeProfile,
                    onProfileUpdated = { updated ->
                        onActiveProfileChanged(updated)
                        profileManager.saveProfile(updated)
                        profileManager.setLastActiveProfile(updated)
                    }
                ) { success ->
                    onTraktSyncingChanged(false)
                    Toast.makeText(
                        context,
                        AppStrings.t(activeProfile.safeLanguage, if (success) "toast.trakt_synced" else "toast.trakt_sync_failed"),
                        Toast.LENGTH_SHORT
                    ).show()
                }
            },
            onDisconnect = {
                val updated = activeProfile.copy(
                    traktAccessToken = null,
                    traktRefreshToken = null,
                    traktTokenExpiresAt = null,
                    traktLastSyncAt = null,
                    traktLastSyncedItems = null,
                    traktLastContinueWatchingCount = null,
                    traktLastWatchlistCount = null
                )
                onActiveProfileChanged(updated)
                profileManager.saveProfile(updated)
                profileManager.setLastActiveProfile(updated)
                homeViewModel.loadInitialData(updated, force = true)
                onShowTraktSheetChanged(false)
            }
        )
    }
    if (showMalSheet && activeProfile != null) {
        com.fluxa.app.ui.SimpleIntegrationSheet(
            titleKey = "brand.myanimelist",
            iconRes = com.fluxa.app.R.drawable.ic_myanimelist,
            lang = activeProfile.safeLanguage,
            onDismiss = { onShowMalSheetChanged(false) },
            onDisconnect = {
                val updated = activeProfile.copy(
                    malAccessToken = null,
                    malRefreshToken = null
                )
                onActiveProfileChanged(updated)
                profileManager.saveProfile(updated)
                profileManager.setLastActiveProfile(updated)
                homeViewModel.loadInitialData(updated, force = true)
                onShowMalSheetChanged(false)
            }
        )
    }
    if (showSimklSheet && activeProfile != null) {
        com.fluxa.app.ui.SimpleIntegrationSheet(
            titleKey = "brand.simkl",
            iconRes = com.fluxa.app.R.drawable.ic_simkl,
            lang = activeProfile.safeLanguage,
            onDismiss = { onShowSimklSheetChanged(false) },
            onDisconnect = {
                val updated = activeProfile.copy(
                    simklAccessToken = null
                )
                onActiveProfileChanged(updated)
                profileManager.saveProfile(updated)
                profileManager.setLastActiveProfile(updated)
                homeViewModel.loadInitialData(updated, force = true)
                onShowSimklSheetChanged(false)
            }
        )
    }
    val density = LocalDensity.current
    val isKeyboardVisible = WindowInsets.ime.getBottom(density) > 0
    if (deviceType == DeviceType.Mobile && currentScreen.mobileNavDestination() != null && !isKeyboardVisible) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .zIndex(120f),
            contentAlignment = Alignment.BottomCenter
        ) {
            MobileBottomNav(
                currentScreen = currentScreen,
                activeProfile = activeProfile,
                profiles = profiles,
                onNavigate = { destination ->
                    if (currentScreen.mobileNavDestination() != destination) {
                        when (destination) {
                            MobileNavDestination.Home -> navigator.navigateTo(Screen.Home, clearStack = true)
                            MobileNavDestination.Discover -> navigator.navigateTo(Screen.Explore(), clearStack = true)
                            MobileNavDestination.Calendar -> navigator.navigateTo(Screen.Calendar, clearStack = true)
                            MobileNavDestination.Library -> navigator.navigateTo(Screen.Watchlist, clearStack = true)
                            MobileNavDestination.Settings -> navigator.navigateTo(Screen.Settings(), clearStack = true)
                        }
                    }
                },
                onQuickProfileSelected = onQuickProfileSelected
            )
        }
    }
}
