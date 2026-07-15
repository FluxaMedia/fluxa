package com.fluxa.app.ui.routes

import android.content.Context
import android.net.Uri
import android.widget.Toast
import com.fluxa.app.BuildConfig
import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.repository.TraktIntegration
import com.fluxa.app.ui.TraktDeviceAuthUiState
import com.fluxa.app.ui.catalog.HomeViewModel
import java.net.URLEncoder

internal fun connectTrakt(
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

internal fun connectSimkl(context: Context, activeProfile: UserProfile?) {
    val clientId = BuildConfig.SIMKL_CLIENT_ID
    if (clientId.isBlank()) {
        Toast.makeText(context, AppStrings.t(activeProfile?.safeLanguage, "toast.simkl_client_missing"), Toast.LENGTH_SHORT).show()
    } else {
        val redirect = URLEncoder.encode("app://simkl", "UTF-8")
        val url = "https://simkl.com/oauth/authorize?response_type=code&client_id=$clientId&redirect_uri=$redirect"
        context.startActivity(android.content.Intent(android.content.Intent.ACTION_VIEW, Uri.parse(url)))
    }
}

internal fun connectAnilist(context: Context, activeProfile: UserProfile?) {
    val clientId = BuildConfig.ANILIST_CLIENT_ID
    if (clientId.isBlank()) {
        Toast.makeText(context, AppStrings.t(activeProfile?.safeLanguage, "toast.anilist_client_missing"), Toast.LENGTH_SHORT).show()
    } else {
        val redirect = URLEncoder.encode(com.fluxa.app.data.repository.AnilistIntegration.REDIRECT_URI, "UTF-8")
        val url = "https://anilist.co/api/v2/oauth/authorize?response_type=code&client_id=$clientId&redirect_uri=$redirect"
        context.startActivity(android.content.Intent(android.content.Intent.ACTION_VIEW, Uri.parse(url)))
    }
}
