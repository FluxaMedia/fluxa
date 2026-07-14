package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.auth.AuthDataSource
import com.fluxa.app.shared.feature.auth.AuthStage
import com.fluxa.app.shared.feature.auth.AuthUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

data class AppleAuthSubmitSnapshot(
    val email: String,
    val password: String,
    val isSignup: Boolean
)

data class AppleAuthSnapshot(
    val isSubmitting: Boolean = false,
    val isAuthenticated: Boolean = false,
    val globalError: String? = null
)

class AppleAuthDataSource : AuthDataSource {
    private val state = MutableStateFlow(AuthUiState())
    private var onSubmitRequested: (AppleAuthSubmitSnapshot) -> Unit = {}

    override fun observeAuth(): Flow<AuthUiState> = state.asStateFlow()

    override suspend fun continueWithNuvio() {
        state.value = AuthUiState(stage = AuthStage.Nuvio)
    }

    override suspend fun continueWithStremio() {
        state.value = AuthUiState(stage = AuthStage.Credentials, showProviderActions = false, allowSignup = false)
    }

    override suspend fun continueWithoutAccount() {
        state.value = state.value.copy(isAuthenticated = true)
    }

    override suspend fun backToRoot() {
        state.value = AuthUiState()
    }

    override suspend fun updateEmail(value: String) {
        state.value = state.value.copy(email = value, emailError = null, globalError = null)
    }

    override suspend fun updatePassword(value: String) {
        state.value = state.value.copy(password = value, passwordError = null, globalError = null)
    }

    override suspend fun updateConfirmPassword(value: String) {
        state.value = state.value.copy(confirmPassword = value, confirmError = null, globalError = null)
    }

    override suspend fun setSignupMode(signup: Boolean) {
        state.value = state.value.copy(isSignupTab = signup)
    }

    override suspend fun submit() {
        state.value = state.value.copy(isSubmitting = true)
        onSubmitRequested(AppleAuthSubmitSnapshot(state.value.email, state.value.password, state.value.isSignupTab))
    }

    override suspend fun confirmImport() {
        Unit
    }

    fun setOnSubmitRequested(handler: (AppleAuthSubmitSnapshot) -> Unit) {
        onSubmitRequested = handler
    }

    fun update(snapshot: AppleAuthSnapshot) {
        state.value = state.value.copy(
            globalError = snapshot.globalError,
            isSubmitting = snapshot.isSubmitting,
            isAuthenticated = snapshot.isAuthenticated
        )
    }
}
