package com.fluxa.app.desktop.auth

import com.fluxa.app.data.PlatformSecrets
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.remote.ExternalSyncApi
import com.fluxa.app.data.repository.AnilistIntegration
import com.fluxa.app.data.repository.ExternalOAuthClient
import com.fluxa.app.data.repository.OAuthClientConfig
import com.fluxa.app.data.repository.SimklIntegration
import java.awt.Desktop
import java.net.URI
import java.net.URLEncoder
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

class DesktopSimklAnilistAuthCoordinator(
    private val profileManager: ProfileManager
) {
    private val oauthClient = ExternalOAuthClient(
        ExternalSyncApi.create(),
        OAuthClientConfig(
            traktClientId = PlatformSecrets.traktClientId,
            traktClientSecret = PlatformSecrets.traktClientSecret,
            simklClientId = PlatformSecrets.simklClientId,
            simklClientSecret = PlatformSecrets.simklClientSecret,
            anilistClientId = PlatformSecrets.anilistClientId,
            anilistClientSecret = PlatformSecrets.anilistClientSecret
        )
    )

    fun connectSimkl() {
        val clientId = PlatformSecrets.simklClientId
        if (clientId.isBlank()) return
        val redirect = URLEncoder.encode(SimklIntegration.REDIRECT_URI, "UTF-8")
        val url = "https://simkl.com/oauth/authorize?response_type=code&client_id=$clientId&redirect_uri=$redirect"
        runCatching { Desktop.getDesktop().browse(URI(url)) }
    }

    fun connectAnilist() {
        val clientId = PlatformSecrets.anilistClientId
        if (clientId.isBlank()) return
        val redirect = URLEncoder.encode(AnilistIntegration.REDIRECT_URI, "UTF-8")
        val url = "https://anilist.co/api/v2/oauth/authorize?response_type=code&client_id=$clientId&redirect_uri=$redirect"
        runCatching { Desktop.getDesktop().browse(URI(url)) }
    }

    fun handleCallback(service: String, code: String, profileId: String?, scope: CoroutineScope) {
        val id = profileId ?: return
        when (service) {
            "simkl" -> scope.launch(Dispatchers.IO) { exchangeSimkl(code, id) }
            "anilist" -> scope.launch(Dispatchers.IO) { exchangeAnilist(code, id) }
        }
    }

    private suspend fun exchangeSimkl(code: String, profileId: String) {
        val tokens = runCatching { oauthClient.exchangeSimklCode(code) }.getOrNull() ?: return
        val username = oauthClient.getSimklUsername(tokens.accessToken)
        val profile = profileManager.getProfiles().firstOrNull { it.id == profileId } ?: return
        profileManager.saveProfile(
            profile.copy(
                simklAccessToken = tokens.accessToken,
                simklUsername = username
            )
        )
    }

    private suspend fun exchangeAnilist(code: String, profileId: String) {
        val tokens = runCatching { oauthClient.exchangeAnilistCode(code) }.getOrNull() ?: return
        val username = oauthClient.getAnilistUsername(tokens.accessToken)
        val profile = profileManager.getProfiles().firstOrNull { it.id == profileId } ?: return
        profileManager.saveProfile(
            profile.copy(
                anilistAccessToken = tokens.accessToken,
                anilistRefreshToken = tokens.refreshToken,
                anilistTokenExpiresAt = tokens.expiresIn?.let { System.currentTimeMillis() + it * 1000L },
                anilistUsername = username
            )
        )
    }
}
