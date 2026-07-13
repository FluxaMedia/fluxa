package com.fluxa.app.shared.feature.auth

import kotlinx.coroutines.flow.Flow

enum class AuthStage { Credentials, Nuvio, NuvioImporting }

enum class AuthImportStep { PROFILE, ADDONS, LIBRARY, PROGRESS, HISTORY, COLLECTIONS }

data class AuthUiState(
    val stage: AuthStage = AuthStage.Credentials,
    val showProviderActions: Boolean = true,
    val allowSignup: Boolean = true,
    val isSignupTab: Boolean = false,
    val email: String = "",
    val password: String = "",
    val confirmPassword: String = "",
    val emailError: String? = null,
    val passwordError: String? = null,
    val confirmError: String? = null,
    val globalError: String? = null,
    val isSubmitting: Boolean = false,
    val importSteps: Set<AuthImportStep> = emptySet(),
    val importDone: Boolean = false,
    val isAuthenticated: Boolean = false
)

sealed interface AuthAction {
    data object ContinueWithNuvio : AuthAction
    data object ContinueWithStremio : AuthAction
    data object ContinueWithoutAccount : AuthAction
    data object BackToRoot : AuthAction
    data object BackRequested : AuthAction
    data class EmailChanged(val value: String) : AuthAction
    data class PasswordChanged(val value: String) : AuthAction
    data class ConfirmPasswordChanged(val value: String) : AuthAction
    data class TabChanged(val signup: Boolean) : AuthAction
    data object Submit : AuthAction
    data object ContinueAfterImport : AuthAction
    data object Completed : AuthAction
}

interface AuthDataSource {
    fun observeAuth(): Flow<AuthUiState>
    suspend fun continueWithNuvio()
    suspend fun continueWithStremio()
    suspend fun continueWithoutAccount()
    suspend fun backToRoot()
    suspend fun updateEmail(value: String)
    suspend fun updatePassword(value: String)
    suspend fun updateConfirmPassword(value: String)
    suspend fun setSignupMode(signup: Boolean)
    suspend fun submit()
    suspend fun confirmImport()
}
