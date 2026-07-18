package com.fluxa.app.ui.catalog

import android.util.Patterns
import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.LoginRequest
import com.fluxa.app.data.remote.NuvioSession
import com.fluxa.app.data.remote.StremioService
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.data.repository.NuvioImportStep
import com.fluxa.app.shared.feature.auth.AuthDataSource
import com.fluxa.app.shared.feature.auth.AuthImportStep
import com.fluxa.app.shared.feature.auth.AuthStage
import com.fluxa.app.shared.feature.auth.AuthUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import java.util.UUID

private val NUVIO_STEP_MAP = mapOf(
    NuvioImportStep.PROFILE to AuthImportStep.PROFILE,
    NuvioImportStep.ADDONS to AuthImportStep.ADDONS,
    NuvioImportStep.LIBRARY to AuthImportStep.LIBRARY,
    NuvioImportStep.PROGRESS to AuthImportStep.PROGRESS,
    NuvioImportStep.HISTORY to AuthImportStep.HISTORY,
    NuvioImportStep.COLLECTIONS to AuthImportStep.COLLECTIONS
)

class AndroidAuthDataSource(
    private val authService: StremioService,
    private val nuvioCoordinator: NuvioAccountImportCoordinator,
    private val profileManager: ProfileManager,
    private val language: () -> String,
    private val onAuthenticated: (UserProfile) -> Unit
) : AuthDataSource {

    private val state = MutableStateFlow(AuthUiState())
    private var pendingNuvioSession: NuvioSession? = null
    private var pendingNuvioEmail: String = ""
    private var pendingImportedProfile: UserProfile? = null

    override fun observeAuth(): Flow<AuthUiState> = state.asStateFlow()

    override suspend fun continueWithNuvio() {
        state.value = AuthUiState(stage = AuthStage.Nuvio)
    }

    override suspend fun continueWithStremio() {
        state.value = AuthUiState(stage = AuthStage.Credentials, showProviderActions = false, allowSignup = false)
    }

    override suspend fun continueWithoutAccount() {
        val guest = UserProfile(
            id = UUID.randomUUID().toString(),
            email = AppStrings.t(language(), "auth.primary_profile_name"),
            authKey = "",
            isGuest = false
        )
        profileManager.saveProfile(guest)
        profileManager.setLastActiveProfile(guest)
        onAuthenticated(guest)
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
                globalError = null
            )
        }
    }

    override suspend fun submit() {
        when (state.value.stage) {
            AuthStage.Credentials -> submitCredentials()
            AuthStage.Nuvio -> submitNuvio()
            AuthStage.NuvioImporting -> Unit
        }
    }

    override suspend fun confirmImport() {
        val profile = pendingImportedProfile ?: return
        onAuthenticated(profile)
        state.update { it.copy(isAuthenticated = true) }
    }

    private fun validateCredentials(): Boolean {
        val s = state.value
        val lang = language()
        var valid = true
        var emailError: String? = null
        var passwordError: String? = null
        var confirmError: String? = null

        if (s.email.isBlank()) {
            emailError = AppStrings.t(lang, "auth.error.email_required")
            valid = false
        } else if (!Patterns.EMAIL_ADDRESS.matcher(s.email).matches()) {
            emailError = AppStrings.t(lang, "auth.error.email_invalid")
            valid = false
        }

        if (s.password.isEmpty()) {
            passwordError = AppStrings.t(lang, "auth.error.password_required")
            valid = false
        } else if (s.password.length < 8) {
            passwordError = AppStrings.t(lang, "auth.error.password_too_short")
            valid = false
        }

        if (s.allowSignup && s.isSignupTab && s.password != s.confirmPassword) {
            confirmError = AppStrings.t(lang, "auth.error.passwords_mismatch")
            valid = false
        }

        state.update { it.copy(emailError = emailError, passwordError = passwordError, confirmError = confirmError) }
        return valid
    }

    private suspend fun submitCredentials() {
        if (!validateCredentials()) return
        val lang = language()
        state.update { it.copy(isSubmitting = true, globalError = null) }
        try {
            val s = state.value
            val request = LoginRequest(s.email.trim(), s.password)
            val response = if (s.allowSignup && s.isSignupTab) authService.register(request) else authService.login(request)
            val result = response.body()?.result
            if (response.isSuccessful && result != null) {
                val existing = profileManager.getProfiles().firstOrNull {
                    it.id == result.user.id || (!it.isGuest && it.email == result.user.email)
                }
                val profile = existing?.copy(
                    id = result.user.id,
                    email = result.user.email,
                    authKey = result.user.authKey,
                    isGuest = false
                ) ?: UserProfile(result.user.id, result.user.email, result.user.authKey)
                profileManager.saveProfile(profile)
                profileManager.setLastActiveProfile(profile)
                onAuthenticated(profile)
                state.update { it.copy(isSubmitting = false, isAuthenticated = true) }
            } else {
                state.update { it.copy(isSubmitting = false, globalError = AppStrings.t(lang, "login.stremio_failed")) }
            }
        } catch (e: Exception) {
            state.update {
                it.copy(isSubmitting = false, globalError = AppStrings.format(lang, "login.connection_error", e.localizedMessage ?: ""))
            }
        }
    }

    private suspend fun submitNuvio() {
        val s = state.value
        val lang = language()
        if (s.email.isBlank() || s.password.isBlank()) {
            state.update { it.copy(globalError = AppStrings.t(lang, "auth.error.fill_required")) }
            return
        }
        state.update { it.copy(isSubmitting = true, globalError = null) }
        val result = nuvioCoordinator.signIn(s.email.trim(), s.password)
        result.fold(
            onSuccess = { session ->
                pendingNuvioSession = session
                pendingNuvioEmail = s.email
                state.update {
                    it.copy(stage = AuthStage.NuvioImporting, isSubmitting = false, importSteps = emptySet(), importDone = false)
                }
                runImport(session)
            },
            onFailure = {
                state.update { it.copy(isSubmitting = false, globalError = AppStrings.t(lang, "auth.error.invalid_credentials")) }
            }
        )
    }

    private suspend fun runImport(session: NuvioSession) {
        val lang = language()
        try {
            val baseProfile = UserProfile(
                id = UUID.randomUUID().toString(),
                email = session.user?.email ?: pendingNuvioEmail,
                authKey = ""
            )
            val imported = nuvioCoordinator.import(baseProfile, session) { step ->
                NUVIO_STEP_MAP[step]?.let { mapped -> state.update { it.copy(importSteps = it.importSteps + mapped) } }
            }
            pendingImportedProfile = imported.profile
            state.update { it.copy(importDone = true) }
        } catch (e: Exception) {
            state.update { it.copy(stage = AuthStage.Nuvio, globalError = AppStrings.t(lang, "auth.error.network")) }
        }
    }
}
