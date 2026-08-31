package com.fluxa.app.ui.catalog

import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.data.repository.NuvioSyncCoordinator
import com.fluxa.app.data.local.UserProfile
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
