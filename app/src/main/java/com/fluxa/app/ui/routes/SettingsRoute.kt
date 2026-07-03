package com.fluxa.app.ui.routes

import android.content.Context
import android.content.SharedPreferences
import android.net.Uri
import android.widget.Toast
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import com.fluxa.app.BuildConfig
import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.OfflineDownloadManager
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import com.fluxa.app.ui.TraktDeviceAuthUiState
import com.fluxa.app.ui.generateOAuthCodeVerifier
import com.fluxa.app.ui.requiresHomeReload
import com.fluxa.app.ui.catalog.HomeViewModel
import com.fluxa.app.ui.catalog.SettingsScreen
import com.fluxa.app.ui.catalog.UpdateManager
import com.fluxa.app.data.repository.TraktIntegration
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.net.URLEncoder

@Composable
internal fun SettingsRoute(
    context: Context,
    activeProfile: UserProfile?,
    profileManager: ProfileManager,
    homeViewModel: HomeViewModel,
    navigator: AppNavigator,
    offlineDownloadManager: OfflineDownloadManager,
    oauthPrefs: SharedPreferences,
    onBack: () -> Unit,
    onProfileChanged: (UserProfile?) -> Unit,
    onShowTraktSheet: () -> Unit,
    onShowMalSheet: () -> Unit,
    onShowSimklSheet: () -> Unit,
    onTraktDeviceAuthChanged: (TraktDeviceAuthUiState?) -> Unit,
    onPendingMalCodeVerifierChanged: (String?) -> Unit,
    onUpdateInfoChanged: (UpdateManager.UpdateInfo?) -> Unit
) {
    val scope = rememberCoroutineScope()
    SettingsScreen(
        activeProfile,
        onBack = onBack,
        onLogout = {
            onProfileChanged(null)
            profileManager.setLastActiveProfile(null)
            navigator.navigateTo(Screen.Profiles, true)
        },
        onConnectStremio = { navigator.navigateTo(Screen.Login()) },
        onConnectTrakt = {
            if (!activeProfile?.traktAccessToken.isNullOrBlank()) {
                onShowTraktSheet()
            } else {
                connectTrakt(context, activeProfile, profileManager, homeViewModel, onProfileChanged, onTraktDeviceAuthChanged)
            }
        },
        onConnectMal = {
            if (!activeProfile?.malAccessToken.isNullOrBlank()) {
                onShowMalSheet()
            } else {
                connectMal(context, activeProfile, oauthPrefs, onPendingMalCodeVerifierChanged)
            }
        },
        onConnectSimkl = {
            if (!activeProfile?.simklAccessToken.isNullOrBlank()) {
                onShowSimklSheet()
            } else {
                connectSimkl(context, activeProfile)
            }
        },
        onManageAddons = { navigator.navigateTo(Screen.AddonStore) },
        onWatchlistClick = { navigator.navigateTo(Screen.Watchlist) },
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
        onReboot = {},
        onUpdateProfile = {
            val previousProfile = activeProfile
            val reloadHome = previousProfile.requiresHomeReload(it)
            onProfileChanged(it)
            homeViewModel.applyUpdatedProfile(it, refreshHomeSideEffects = reloadHome)
            scope.launch(Dispatchers.IO) {
                profileManager.saveProfile(it)
                profileManager.setLastActiveProfile(it)
            }
            if (reloadHome) {
                homeViewModel.loadInitialData(it, force = true)
            }
        },
        onUpdateInfoChanged = onUpdateInfoChanged,
        viewModel = homeViewModel
    )
}

private fun connectTrakt(
    context: Context,
    activeProfile: UserProfile?,
    profileManager: ProfileManager,
    homeViewModel: HomeViewModel,
    onProfileChanged: (UserProfile?) -> Unit,
    onTraktDeviceAuthChanged: (TraktDeviceAuthUiState?) -> Unit
) {
    val traktClientId = BuildConfig.TRAKT_CLIENT_ID
    if (traktClientId.isBlank()) {
        Toast.makeText(
            context,
            AppStrings.t(activeProfile?.safeLanguage, "auto.trakt_api_key_is_not_configured"),
            Toast.LENGTH_SHORT
        ).show()
    } else if (BuildConfig.IS_TV) {
        homeViewModel.startTraktDeviceAuthorization(
            onCodeReady = { code ->
                onTraktDeviceAuthChanged(
                    TraktDeviceAuthUiState(
                        userCode = code.userCode,
                        verificationUrl = code.verificationUrl.ifBlank { "https://trakt.tv/activate" }
                    )
                )
            },
            onProfileUpdated = { updated ->
                onProfileChanged(updated)
                profileManager.saveProfile(updated)
            }
        ) { success, messageKey ->
            onTraktDeviceAuthChanged(null)
            Toast.makeText(
                context,
                AppStrings.t(activeProfile?.safeLanguage, if (success) "toast.trakt_connected" else (messageKey ?: "toast.trakt_connect_failed")),
                Toast.LENGTH_SHORT
            ).show()
            activeProfile?.let { homeViewModel.loadInitialData(it, force = true) }
        }
    } else {
        val redirect = URLEncoder.encode(TraktIntegration.MOBILE_REDIRECT_URI, "UTF-8")
        val url = "https://trakt.tv/oauth/authorize?response_type=code&client_id=$traktClientId&redirect_uri=$redirect"
        context.startActivity(android.content.Intent(android.content.Intent.ACTION_VIEW, Uri.parse(url)))
    }
}

private fun connectMal(
    context: Context,
    activeProfile: UserProfile?,
    oauthPrefs: SharedPreferences,
    onPendingMalCodeVerifierChanged: (String?) -> Unit
) {
    val clientId = BuildConfig.MAL_CLIENT_ID
    if (clientId.isBlank()) {
        Toast.makeText(context, AppStrings.t(activeProfile?.safeLanguage, "toast.mal_client_missing"), Toast.LENGTH_SHORT).show()
    } else {
        val verifier = generateOAuthCodeVerifier()
        onPendingMalCodeVerifierChanged(verifier)
        oauthPrefs.edit().putString("mal_code_verifier", verifier).apply()
        val redirect = URLEncoder.encode("app://mal", "UTF-8")
        val challenge = URLEncoder.encode(verifier, "UTF-8")
        val url = "https://myanimelist.net/v1/oauth2/authorize?response_type=code&client_id=$clientId&redirect_uri=$redirect&code_challenge=$challenge&code_challenge_method=plain"
        context.startActivity(android.content.Intent(android.content.Intent.ACTION_VIEW, Uri.parse(url)))
    }
}

private fun connectSimkl(context: Context, activeProfile: UserProfile?) {
    val clientId = BuildConfig.SIMKL_CLIENT_ID
    if (clientId.isBlank()) {
        Toast.makeText(context, AppStrings.t(activeProfile?.safeLanguage, "toast.simkl_client_missing"), Toast.LENGTH_SHORT).show()
    } else {
        val redirect = URLEncoder.encode("app://simkl", "UTF-8")
        val url = "https://simkl.com/oauth/authorize?response_type=code&client_id=$clientId&redirect_uri=$redirect"
        context.startActivity(android.content.Intent(android.content.Intent.ACTION_VIEW, Uri.parse(url)))
    }
}
