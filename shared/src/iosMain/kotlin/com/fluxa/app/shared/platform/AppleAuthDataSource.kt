package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.auth.AuthDataSource
import com.fluxa.app.shared.feature.auth.AuthStage
import com.fluxa.app.shared.feature.auth.AuthUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import platform.Foundation.NSNotificationCenter

class AppleAuthDataSource : AuthDataSource {
    private val state = MutableStateFlow(AuthUiState())

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
        postAction("FluxaAppleAuthSubmitRequested") {
            put("email", state.value.email)
            put("password", state.value.password)
            put("isSignup", state.value.isSignupTab)
        }
    }

    override suspend fun confirmImport() {
        postAction("FluxaAppleAuthConfirmImportRequested") { }
    }

    private fun postAction(name: String, extra: kotlinx.serialization.json.JsonObjectBuilder.() -> Unit) {
        NSNotificationCenter.defaultCenter.postNotificationName(
            aName = name,
            `object` = buildJsonObject(extra).toString(),
            userInfo = null
        )
    }

    fun updateJson(authJson: String) {
        state.value = runCatching {
            val root = Json.parseToJsonElement(authJson).jsonObject
            val current = state.value
            AuthUiState(
                stage = root.string("stage")?.let { name -> runCatching { AuthStage.valueOf(name) }.getOrNull() } ?: current.stage,
                showProviderActions = root.string("showProviderActions")?.toBooleanStrictOrNull() ?: true,
                allowSignup = root.string("allowSignup")?.toBooleanStrictOrNull() ?: true,
                isSignupTab = root.boolean("isSignupTab"),
                email = current.email,
                password = current.password,
                confirmPassword = current.confirmPassword,
                emailError = root.string("emailError"),
                passwordError = root.string("passwordError"),
                confirmError = root.string("confirmError"),
                globalError = root.string("globalError"),
                isSubmitting = root.boolean("isSubmitting"),
                importDone = root.boolean("importDone"),
                isAuthenticated = root.boolean("isAuthenticated")
            )
        }.getOrElse { state.value }
    }
}

private fun Map<String, JsonElement>.string(key: String): String? =
    get(key)?.jsonPrimitive?.contentOrNull

private fun Map<String, JsonElement>.boolean(key: String): Boolean =
    string(key)?.toBooleanStrictOrNull() ?: false
