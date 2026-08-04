package com.fluxa.app.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import android.content.Context
import android.widget.Toast
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.HomeViewModel
import com.fluxa.app.ui.catalog.PlayerPipSuppression
import com.fluxa.app.ui.catalog.UpdateManager
import com.fluxa.app.ui.catalog.exchangeAnilistCode
import com.fluxa.app.ui.catalog.exchangeSimklCode
import com.fluxa.app.ui.catalog.exchangeTraktCode
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive

@Composable
internal fun NuvioHealthSyncEffect(
    profile: UserProfile?,
    homeViewModel: HomeViewModel,
    onProfileUpdated: (UserProfile) -> Unit
) {
    val latestProfile by rememberUpdatedState(profile)
    LaunchedEffect(profile?.id, profile?.nuvioAccessToken) {
        if (profile?.nuvioAccessToken.isNullOrBlank()) return@LaunchedEffect
        var wasHealthy: Boolean? = null
        while (isActive) {
            val currentProfile = latestProfile
            if (currentProfile != null && !currentProfile.nuvioAccessToken.isNullOrBlank()) {
                val isHealthy = homeViewModel.isNuvioHealthy()
                if (isHealthy && wasHealthy != true) {
                    homeViewModel.syncNuvioIntegration(
                        profile = currentProfile,
                        onProfileUpdated = onProfileUpdated,
                        onComplete = {}
                    )
                }
                wasHealthy = isHealthy
                delay(if (isHealthy) 60_000L else 30_000L)
            } else {
                delay(30_000L)
            }
        }
    }
}

@Composable
internal fun AppUpdateCheckEffect(
    automaticUpdatesEnabled: Boolean,
    isDebugBuild: Boolean,
    onUpdateFound: (UpdateManager.UpdateInfo) -> Unit
) {
    LaunchedEffect(automaticUpdatesEnabled) {
        if (!automaticUpdatesEnabled) return@LaunchedEffect
        while (isActive) {
            val update = UpdateManager.checkUpdate()
            if (update != null) {
                onUpdateFound(update)
                break
            }
            delay(if (isDebugBuild) 120_000L else 21_600_000L)
        }
    }
}

@Composable
internal fun OAuthRedirectEffect(
    redirectHandler: OAuthRedirectHandler,
    context: Context,
    homeViewModel: HomeViewModel,
    profileManager: ProfileManager,
    activeProfile: UserProfile?,
    onProfileUpdated: (UserProfile) -> Unit,
    onTraktSyncingChanged: (Boolean) -> Unit
) {
    val latestOnProfileUpdated by rememberUpdatedState(onProfileUpdated)
    val latestOnTraktSyncingChanged by rememberUpdatedState(onTraktSyncingChanged)
    val latestActiveProfile by rememberUpdatedState(activeProfile)

    LaunchedEffect(redirectHandler) {
        redirectHandler.trakt.collect { code ->
            var exchangedProfile: UserProfile? = null
            homeViewModel.exchangeTraktCode(code, onProfileUpdated = { updated ->
                profileManager.persistOAuthUpdate("trakt", updated).also {
                    exchangedProfile = it
                    latestOnProfileUpdated(it)
                }
            }) { success ->
                val profile = exchangedProfile ?: latestActiveProfile
                Toast.makeText(
                    context,
                    AppStrings.t(profile?.safeLanguage, if (success) "toast.trakt_connected" else "toast.trakt_connect_failed"),
                    Toast.LENGTH_SHORT
                ).show()
                profile?.let { homeViewModel.loadInitialData(it, force = true) }
                if (success && profile != null) {
                    latestOnTraktSyncingChanged(true)
                    homeViewModel.syncTraktIntegration(
                        profile = profile,
                        onProfileUpdated = { updated ->
                            val persisted = profileManager.persistOAuthUpdate("trakt", updated)
                            latestOnProfileUpdated(persisted)
                            profileManager.setLastActiveProfile(persisted)
                        }
                    ) { latestOnTraktSyncingChanged(false) }
                }
            }
        }
    }

    LaunchedEffect(redirectHandler) {
        redirectHandler.simkl.collect { code ->
            var exchangedProfile: UserProfile? = null
            homeViewModel.exchangeSimklCode(code, onProfileUpdated = { updated ->
                profileManager.persistOAuthUpdate("simkl", updated).also {
                    exchangedProfile = it
                    latestOnProfileUpdated(it)
                }
            }) { success ->
                val profile = exchangedProfile ?: latestActiveProfile
                Toast.makeText(
                    context,
                    AppStrings.t(profile?.safeLanguage, if (success) "toast.simkl_connected" else "toast.simkl_connect_failed"),
                    Toast.LENGTH_SHORT
                ).show()
                profile?.let {
                    homeViewModel.loadInitialData(it, force = true)
                    homeViewModel.loadLibraryItems(it)
                }
            }
        }
    }

    LaunchedEffect(redirectHandler) {
        redirectHandler.anilist.collect { code ->
            var exchangedProfile: UserProfile? = null
            homeViewModel.exchangeAnilistCode(code, onProfileUpdated = { updated ->
                profileManager.persistOAuthUpdate("anilist", updated).also {
                    exchangedProfile = it
                    latestOnProfileUpdated(it)
                }
            }) { success ->
                val profile = exchangedProfile ?: latestActiveProfile
                Toast.makeText(
                    context,
                    AppStrings.t(profile?.safeLanguage, if (success) "toast.anilist_connected" else "toast.anilist_connect_failed"),
                    Toast.LENGTH_SHORT
                ).show()
                profile?.let {
                    homeViewModel.loadInitialData(it, force = true)
                    homeViewModel.loadLibraryItems(it)
                }
            }
        }
    }
}

private fun ProfileManager.persistOAuthUpdate(provider: String, updated: UserProfile): UserProfile {
    return updateProfile(updated.id) { current ->
        when (provider) {
            "trakt" -> current.copy(
                traktAccessToken = updated.traktAccessToken,
                traktRefreshToken = updated.traktRefreshToken,
                traktTokenExpiresAt = updated.traktTokenExpiresAt,
                traktUsername = updated.traktUsername,
                traktLastSyncAt = updated.traktLastSyncAt,
                traktLastSyncedItems = updated.traktLastSyncedItems,
                traktLastContinueWatchingCount = updated.traktLastContinueWatchingCount,
                traktLastWatchlistCount = updated.traktLastWatchlistCount
            )
            "simkl" -> current.copy(
                simklAccessToken = updated.simklAccessToken,
                simklUsername = updated.simklUsername,
                simklLastSyncAt = updated.simklLastSyncAt
            )
            "anilist" -> current.copy(
                anilistAccessToken = updated.anilistAccessToken,
                anilistRefreshToken = updated.anilistRefreshToken,
                anilistTokenExpiresAt = updated.anilistTokenExpiresAt,
                anilistUsername = updated.anilistUsername
            )
            else -> current
        }
    } ?: updated
}

@Composable
internal fun TraktDeviceAuthDialog(
    state: TraktDeviceAuthUiState?,
    lang: String?,
    onDismiss: () -> Unit
) {
    state ?: return
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(AppStrings.t(lang, "brand.trakt")) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text(AppStrings.t(lang, "trakt.device.open_url"))
                Text(
                    text = state.verificationUrl,
                    color = Color.White,
                    fontWeight = FontWeight.SemiBold
                )
                Text(AppStrings.t(lang, "trakt.device.enter_code"))
                Text(
                    text = state.userCode,
                    fontSize = 30.sp,
                    fontWeight = FontWeight.Black,
                    letterSpacing = 2.sp
                )
                if (state.isWaiting) {
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        CircularProgressIndicator(modifier = Modifier.size(18.dp), strokeWidth = 2.dp)
                        Text(AppStrings.t(lang, "trakt.device.waiting"))
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = onDismiss) {
                Text(AppStrings.t(lang, "common.close"))
            }
        }
    )
}

@Composable
internal fun PlayerLifecycleEffect(
    isPlayerActive: Boolean,
    activeProfile: UserProfile?,
    mainPlayer: ExoPlayer,
    previewPlayer: ExoPlayer,
    homeViewModel: HomeViewModel,
    enterPictureInPicture: () -> Unit
) {
    val lifecycleOwner = LocalLifecycleOwner.current
    val latestIsPlayerActive by rememberUpdatedState(isPlayerActive)
    val latestActiveProfile by rememberUpdatedState(activeProfile)
    val latestBackgroundPlayback by rememberUpdatedState(activeProfile?.safeBackgroundPlayback == true)
    val latestPictureInPicture by rememberUpdatedState(activeProfile?.safePictureInPicture == true)
    DisposableEffect(lifecycleOwner, mainPlayer, previewPlayer) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_RESUME) {
                PlayerPipSuppression.suppressAutoEnter = false
                if (latestActiveProfile != null) {
                    homeViewModel.refreshInstalledAddons(forceRefresh = true)
                }
                homeViewModel.refreshExternalContinueWatching()
            } else if (event == Lifecycle.Event.ON_PAUSE) {
                previewPlayer.pause()
                val isPlayerScreen = latestIsPlayerActive
                val shouldEnterPip = isPlayerScreen &&
                    latestPictureInPicture &&
                    !PlayerPipSuppression.suppressAutoEnter

                if (shouldEnterPip) {
                    runCatching { enterPictureInPicture() }
                }

                if (!isPlayerScreen || (!latestBackgroundPlayback && !shouldEnterPip)) {
                    mainPlayer.pause()
                }
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose {
            lifecycleOwner.lifecycle.removeObserver(observer)
            mainPlayer.release()
            previewPlayer.release()
        }
    }
}
