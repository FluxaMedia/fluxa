package com.fluxa.app.shared.feature.auth

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn

class AuthStore(
    private val dataSource: AuthDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<AuthUiState> = dataSource.observeAuth()
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), AuthUiState())

    suspend fun dispatch(action: AuthAction) {
        when (action) {
            AuthAction.ContinueWithNuvio -> dataSource.continueWithNuvio()
            AuthAction.ContinueWithStremio -> dataSource.continueWithStremio()
            AuthAction.ContinueWithoutAccount -> dataSource.continueWithoutAccount()
            AuthAction.BackToRoot -> dataSource.backToRoot()
            AuthAction.BackRequested -> Unit
            is AuthAction.EmailChanged -> dataSource.updateEmail(action.value)
            is AuthAction.PasswordChanged -> dataSource.updatePassword(action.value)
            is AuthAction.ConfirmPasswordChanged -> dataSource.updateConfirmPassword(action.value)
            is AuthAction.TabChanged -> dataSource.setSignupMode(action.signup)
            AuthAction.Submit -> dataSource.submit()
            AuthAction.ContinueAfterImport -> dataSource.confirmImport()
            AuthAction.Completed -> Unit
        }
    }
}
