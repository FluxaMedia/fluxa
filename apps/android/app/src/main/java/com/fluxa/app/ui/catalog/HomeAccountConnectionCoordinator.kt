package com.fluxa.app.ui.catalog

import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.data.repository.NuvioSyncCoordinator
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.LoginRequest
import com.fluxa.app.data.repository.StremioRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/** Handles credential-based provider connections and Nuvio account health. */
internal class HomeAccountConnectionCoordinator(
    private val scope: CoroutineScope,
    private val repository: StremioRepository,
    private val nuvioAccountImportCoordinator: NuvioAccountImportCoordinator,
    private val nuvioSyncCoordinator: NuvioSyncCoordinator,
    private val setProviderSyncing: (provider: String, syncing: Boolean) -> Unit,
    private val setConnectError: (provider: String, error: String?) -> Unit,
    private val syncStremio: (
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit,
    ) -> Unit,
    private val syncNuvio: (
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit,
    ) -> Unit,
) {
    fun connectStremio(
        email: String,
        password: String,
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit,
    ) {
        setProviderSyncing("stremio", true)
        setConnectError("stremio", null)
        scope.launch {
            try {
                val response = repository.login(LoginRequest(email.trim(), password))
                val result = response.body()?.result
                if (response.isSuccessful && result != null) {
                    val updated = profile.copy(
                        authKey = result.user.authKey,
                        stremioUserId = result.user.id,
                        stremioEmail = result.user.email,
                    )
                    onProfileUpdated(updated)
                    syncStremio(updated, onProfileUpdated, onComplete)
                } else {
                    setProviderSyncing("stremio", false)
                    setConnectError("stremio", "invalid_credentials")
                    onComplete(false)
                }
            } catch (error: Exception) {
                setProviderSyncing("stremio", false)
                setConnectError("stremio", error.localizedMessage ?: "network_error")
                onComplete(false)
            }
        }
    }

    fun connectNuvio(
        email: String,
        password: String,
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit,
    ) {
        setProviderSyncing("nuvio", true)
        setConnectError("nuvio", null)
        scope.launch {
            nuvioAccountImportCoordinator.signIn(email.trim(), password).fold(
                onSuccess = { session ->
                    val updated = profile.copy(
                        nuvioAccessToken = session.accessToken,
                        nuvioRefreshToken = session.refreshToken,
                        nuvioTokenExpiresAt = session.expiresIn?.let {
                            System.currentTimeMillis() + it * 1_000L
                        },
                        nuvioEmail = session.user?.email ?: email,
                    )
                    onProfileUpdated(updated)
                    syncNuvio(updated, onProfileUpdated, onComplete)
                },
                onFailure = {
                    setProviderSyncing("nuvio", false)
                    setConnectError("nuvio", "invalid_credentials")
                    onComplete(false)
                },
            )
        }
    }

    suspend fun isNuvioHealthy(): Boolean = nuvioSyncCoordinator.isHealthy()

}
