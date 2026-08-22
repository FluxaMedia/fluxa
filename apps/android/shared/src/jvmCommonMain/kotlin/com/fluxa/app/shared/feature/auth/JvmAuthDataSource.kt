package com.fluxa.app.shared.feature.auth

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.LoginRequest
import com.fluxa.app.data.remote.NuvioPluginDto
import com.fluxa.app.data.remote.NuvioSession
import com.fluxa.app.data.remote.StremioService
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.data.repository.NuvioImportStep
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

private val NUVIO_STEP_MAP = mapOf(
    NuvioImportStep.PROFILE to AuthImportStep.PROFILE,
    NuvioImportStep.ADDONS to AuthImportStep.ADDONS,
    NuvioImportStep.LIBRARY to AuthImportStep.LIBRARY,
    NuvioImportStep.PROGRESS to AuthImportStep.PROGRESS,
    NuvioImportStep.HISTORY to AuthImportStep.HISTORY,
    NuvioImportStep.COLLECTIONS to AuthImportStep.COLLECTIONS,
)

/**
 * Shared JVM authentication state machine used by Android and desktop.
 * Platform wrappers only provide ID/email validation and optional import hooks.
 */
class JvmAuthDataSource(
    private val authService: StremioService,
    private val nuvioCoordinator: NuvioAccountImportCoordinator,
    private val profileManager: ProfileManager,
    private val language: () -> String,
    private val idGenerator: () -> String,
    private val emailValidator: (String) -> Boolean,
    private val onPluginsImported: suspend (List<NuvioPluginDto>) -> Unit = {},
    private val onAuthenticated: (UserProfile) -> Unit,
) : AuthDataSource {
    private val state = MutableStateFlow(AuthUiState())
    private var pendingNuvioEmail: String = ""
    private var pendingImportedProfile: UserProfile? = null

    override fun observeAuth(): Flow<AuthUiState> = state.asStateFlow()

    override suspend fun continueWithNuvio() {
        state.value = AuthUiState(stage = AuthStage.Nuvio)
    }

    override suspend fun continueWithStremio() {
        state.value = AuthUiState(
            stage = AuthStage.Credentials,
            showProviderActions = false,
            allowSignup = false,
        )
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
        }
    }

    override suspend fun confirmImport() {
        val profile = pendingImportedProfile ?: return
        onAuthenticated(profile)
        state.update { it.copy(isAuthenticated = true) }
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
            val current = state.value
            val request = LoginRequest(current.email.trim(), current.password)
            val response = if (current.allowSignup && current.isSignupTab) {
                authService.register(request)
            } else {
                authService.login(request)
            }
            val result = response.body()?.result
            if (response.isSuccessful && result != null) {
                val existing = profileManager.getProfiles().firstOrNull { profile ->
                    profile.id == result.user.id ||
                        profile.stremioUserId == result.user.id ||
                        profile.stremioEmail.equals(result.user.email, ignoreCase = true)
                }
                val profile = existing?.copy(
                    id = result.user.id,
                    email = result.user.email,
                    authKey = result.user.authKey,
                    stremioUserId = result.user.id,
                    stremioEmail = result.user.email,
                ) ?: UserProfile(
                    id = result.user.id,
                    email = result.user.email,
                    authKey = result.user.authKey,
                    stremioUserId = result.user.id,
                    stremioEmail = result.user.email,
                )
                persistAndAuthenticate(profile)
                state.update { it.copy(isSubmitting = false, isAuthenticated = true) }
            } else {
                state.update {
                    it.copy(
                        isSubmitting = false,
                        globalError = AppStrings.t(lang, "login.stremio_failed"),
                    )
                }
            }
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
                state.update {
                    it.copy(
                        stage = AuthStage.NuvioImporting,
                        isSubmitting = false,
                        importSteps = emptySet(),
                        importDone = false,
                    )
                }
                runImport(session)
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

    private suspend fun runImport(session: NuvioSession) {
        val lang = language()
        try {
            val baseProfile = UserProfile(
                id = idGenerator(),
                email = session.user?.email ?: pendingNuvioEmail,
                authKey = "",
            )
            val imported = nuvioCoordinator.import(
                baseProfile,
                session,
                onStep = { step ->
                    NUVIO_STEP_MAP[step]?.let { mapped ->
                        state.update { it.copy(importSteps = it.importSteps + mapped) }
                    }
                },
                onItemProgress = { index, total, title ->
                    state.update {
                        it.copy(
                            importItemIndex = index,
                            importItemTotal = total,
                            importItemTitle = title,
                        )
                    }
                },
            )
            onPluginsImported(imported.plugins)
            pendingImportedProfile = imported.profile
            state.update {
                it.copy(
                    importDone = true,
                    importItemIndex = null,
                    importItemTotal = null,
                    importItemTitle = null,
                )
            }
        } catch (_: Exception) {
            state.update {
                it.copy(
                    stage = AuthStage.Nuvio,
                    isSubmitting = false,
                    globalError = AppStrings.t(lang, "auth.error.network"),
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
