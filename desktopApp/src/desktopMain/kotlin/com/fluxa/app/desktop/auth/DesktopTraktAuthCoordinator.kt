package com.fluxa.app.desktop.auth

import com.fluxa.app.data.PlatformSecrets
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.ExternalSyncApi
import com.fluxa.app.data.repository.ExternalOAuthClient
import com.fluxa.app.data.repository.OAuthClientConfig
import com.fluxa.app.data.repository.TraktIntegration
import java.awt.Desktop
import java.net.URI
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

class DesktopTraktAuthCoordinator(
    private val profileManager: ProfileManager,
    private val onProfileUpdated: (UserProfile) -> Unit
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

    fun connect(profileId: String, scope: CoroutineScope) {
        if (PlatformSecrets.traktClientId.isBlank()) return
        scope.launch(Dispatchers.IO) {
            val codeResponse = runCatching { oauthClient.createTraktDeviceCode() }.getOrNull() ?: return@launch
            runCatching {
                Desktop.getDesktop().browse(URI("https://trakt.tv/activate/${codeResponse.userCode}"))
            }
            val expiresAt = System.currentTimeMillis() + codeResponse.expiresIn * 1000L
            var intervalMs = codeResponse.interval.coerceAtLeast(5) * 1000L
            while (System.currentTimeMillis() < expiresAt) {
                delay(intervalMs)
                val response = runCatching { oauthClient.exchangeTraktDeviceCode(codeResponse.deviceCode) }.getOrNull()
                val tokens = response?.body()
                if (response?.isSuccessful == true && tokens != null) {
                    val username = oauthClient.getTraktUsername(tokens.accessToken)
                    val profile = profileManager.getProfiles().firstOrNull { it.id == profileId } ?: return@launch
                    val updated = profile.copy(
                        traktAccessToken = tokens.accessToken,
                        traktRefreshToken = tokens.refreshToken,
                        traktTokenExpiresAt = TraktIntegration.tokenExpiresAt(tokens.created_at, tokens.expiresIn),
                        traktUsername = username
                    )
                    profileManager.saveProfile(updated)
                    onProfileUpdated(updated)
                    return@launch
                }
                when (response?.code()) {
                    429 -> intervalMs = (intervalMs + 5_000L).coerceAtMost(60_000L)
                    400, 404, 409, 428 -> Unit
                    else -> return@launch
                }
            }
        }
    }
}
