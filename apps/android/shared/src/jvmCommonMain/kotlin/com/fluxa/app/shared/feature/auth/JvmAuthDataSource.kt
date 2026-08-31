package com.fluxa.app.shared.feature.auth

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.NuvioSession
import com.fluxa.app.data.remote.NuvioUser
import com.fluxa.app.data.remote.DeviceAuthAdapter
import com.fluxa.app.data.remote.DeviceAuthResult
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.common.PlatformLog
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.delay

/**
 * Shared JVM authentication state machine used by Android and desktop.
 * Platform wrappers only provide ID/email validation and optional import hooks.
 */
class JvmAuthDataSource(
    private val nuvioCoordinator: NuvioAccountImportCoordinator,
    private val profileManager: ProfileManager,
    private val language: () -> String,
    private val idGenerator: () -> String,
    private val emailValidator: (String) -> Boolean,
    private val onAuthenticated: (UserProfile) -> Unit,
    private val deviceAuthAdapters: Map<String, DeviceAuthAdapter> = emptyMap(),
    private val deviceAuthEnabled: Boolean = false,
) : AuthDataSource {
    private val state = MutableStateFlow(AuthUiState())
    private var pendingNuvioEmail: String = ""

    override fun observeAuth(): Flow<AuthUiState> = state.asStateFlow()

    override suspend fun continueWithNuvio() {
        if (deviceAuthEnabled && deviceAuthAdapters.containsKey("nuvio")) {
            startDeviceAuth("nuvio")
            return
        }
        state.value = AuthUiState(stage = AuthStage.Nuvio)
    }

    override suspend fun continueWithFluxa() {
        if (deviceAuthEnabled && deviceAuthAdapters.containsKey("fluxa")) startDeviceAuth("fluxa")
    }

    private suspend fun startDeviceAuth(provider: String) {
        val adapter = deviceAuthAdapters[provider] ?: return
        state.value = AuthUiState(stage = AuthStage.DeviceQr, showProviderActions = false, allowSignup = false, qrProvider = provider, qrStatus = "starting")
        try {
            val session = adapter.start("Fluxa TV")
            state.value = AuthUiState(
                stage = AuthStage.DeviceQr,
                showProviderActions = false,
                allowSignup = false,
                qrProvider = provider,
                qrCode = session.userCode,
                qrUrl = session.verificationUri,
                qrExpiresAtMillis = System.currentTimeMillis() + session.expiresInSeconds * 1000,
                qrStatus = "pending",
            )
            while (System.currentTimeMillis() < (state.value.qrExpiresAtMillis ?: 0L)) {
                delay(session.pollIntervalSeconds.coerceAtLeast(1) * 1000)
                if (adapter.poll(session)) {
                    val result = adapter.exchange(session)
                    if (provider == "nuvio") {
                        val accessToken = result.accessToken ?: error("Nuvio device login returned no access token")
                        pendingNuvioEmail = result.email.orEmpty()
                        connectNuvio(
                            NuvioSession(
                                accessToken = accessToken,
                                refreshToken = result.refreshToken.orEmpty(),
                                expiresIn = 3600L,
                                user = result.userId?.let { NuvioUser(it, result.email.orEmpty()) },
                            )
                        )
                    } else {
                        persistDeviceResult(provider, result)
                        state.update { it.copy(isAuthenticated = true, qrStatus = "authorized") }
                    }
                    return
                }
            }
            state.update { it.copy(qrStatus = "expired", globalError = AppStrings.t(language(), "auth.device_code_expired")) }
        } catch (error: Exception) {
            state.update { it.copy(qrStatus = "error", globalError = error.localizedMessage ?: AppStrings.t(language(), "auth.device_login_failed")) }
        }
    }

    private suspend fun persistDeviceResult(provider: String, result: DeviceAuthResult) {
        val profile = UserProfile(
            id = result.userId ?: idGenerator(),
            email = result.email ?: "$provider TV",
            authKey = result.authKey ?: result.accessToken.orEmpty(),
            nuvioAccessToken = result.accessToken.takeIf { provider == "nuvio" },
            nuvioRefreshToken = result.refreshToken.takeIf { provider == "nuvio" },
        )
        persistAndAuthenticate(profile)
    }

    override suspend fun continueWithoutAccount() {
        val profile = UserProfile(
            id = idGenerator(),
            email = AppStrings.t(language(), "auth.primary_profile_name"),
            authKey = "",
        )
        persistAndAuthenticate(profile)
        state.update { it.copy(isAuthenticated = true) }
    }

    override suspend fun backToRoot() {
        state.value = AuthUiState()
    }

    override suspend fun updateEmail(value: String) {
        state.update { it.copy(email = value, emailError = null, globalError = null) }
    }

    override suspend fun updatePassword(value: String) {
        state.update { it.copy(password = value, passwordError = null, globalError = null) }
    }

    override suspend fun updateConfirmPassword(value: String) {
        state.update { it.copy(confirmPassword = value, confirmError = null) }
    }

    override suspend fun setSignupMode(signup: Boolean) {
        state.update {
            it.copy(
                isSignupTab = signup,
                password = "",
                confirmPassword = "",
                passwordError = null,
                confirmError = null,
                globalError = null,
            )
        }
    }

    override suspend fun submit() {
        when (state.value.stage) {
            AuthStage.Credentials -> submitCredentials()
            AuthStage.Nuvio -> submitNuvio()
            AuthStage.NuvioImporting -> Unit
            AuthStage.DeviceQr -> Unit
        }
    }

    override suspend fun confirmImport() {
        Unit
    }

    private fun validateCredentials(): Boolean {
        val current = state.value
        val lang = language()
        var emailError: String? = null
        var passwordError: String? = null
        var confirmError: String? = null

        if (current.email.isBlank()) {
            emailError = AppStrings.t(lang, "auth.error.email_required")
        } else if (!emailValidator(current.email)) {
            emailError = AppStrings.t(lang, "auth.error.email_invalid")
        }

        if (current.password.isEmpty()) {
            passwordError = AppStrings.t(lang, "auth.error.password_required")
        } else if (current.password.length < 8) {
            passwordError = AppStrings.t(lang, "auth.error.password_too_short")
        }

        if (current.allowSignup && current.isSignupTab && current.password != current.confirmPassword) {
            confirmError = AppStrings.t(lang, "auth.error.passwords_mismatch")
        }

        state.update {
            it.copy(
                emailError = emailError,
                passwordError = passwordError,
                confirmError = confirmError,
            )
        }
        return emailError == null && passwordError == null && confirmError == null
    }

    private suspend fun submitCredentials() {
        if (!validateCredentials()) return
        val lang = language()
        state.update { it.copy(isSubmitting = true, globalError = null) }
        try {
            error("Credential login is not available in the app")
        } catch (error: Exception) {
            state.update {
                it.copy(
                    isSubmitting = false,
                    globalError = AppStrings.format(
                        lang,
                        "login.connection_error",
                        error.localizedMessage ?: error.message.orEmpty(),
                    ),
                )
            }
        }
    }

    private suspend fun submitNuvio() {
        val current = state.value
        val lang = language()
        if (current.email.isBlank() || current.password.isBlank()) {
            state.update { it.copy(globalError = AppStrings.t(lang, "auth.error.fill_required")) }
            return
        }
        state.update { it.copy(isSubmitting = true, globalError = null) }
        nuvioCoordinator.signIn(current.email.trim(), current.password).fold(
            onSuccess = { session ->
                pendingNuvioEmail = current.email
                connectNuvio(session)
            },
            onFailure = {
                state.update {
                    it.copy(
                        isSubmitting = false,
                        globalError = AppStrings.t(lang, "auth.error.invalid_credentials"),
                    )
                }
            },
        )
    }

    private suspend fun connectNuvio(session: NuvioSession) {
        val lang = language()
        try {
            val baseProfile = UserProfile(
                id = idGenerator(),
                email = session.user?.email ?: pendingNuvioEmail,
                authKey = "",
            )
            val profile = nuvioCoordinator.connect(baseProfile, session)
            onAuthenticated(profile)
            state.update {
                it.copy(
                    isSubmitting = false,
                    isAuthenticated = true,
                    qrStatus = "authorized",
                )
            }
        } catch (error: Exception) {
            PlatformLog.w("NuvioAuth", "Nuvio account connection failed after authentication", error)
            state.update {
                it.copy(
                    stage = if (it.qrProvider == "nuvio") AuthStage.DeviceQr else AuthStage.Nuvio,
                    isSubmitting = false,
                    globalError = AppStrings.t(lang, "auth.error.network"),
                    qrStatus = if (it.qrProvider == "nuvio") "error" else it.qrStatus,
                )
            }
        }
    }

    private suspend fun persistAndAuthenticate(profile: UserProfile) {
        profileManager.saveProfile(profile)
        profileManager.setLastActiveProfile(profile)
        onAuthenticated(profile)
    }
}
