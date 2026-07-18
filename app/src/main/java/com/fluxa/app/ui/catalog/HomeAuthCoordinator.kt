package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.NativeHeadlessEngineResult
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.TraktDeviceCodeResponse
import com.google.gson.Gson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

internal class HomeAuthCoordinator(
    private val scope: CoroutineScope,
    private val gson: Gson,
    private val dispatch: suspend (Any) -> NativeHeadlessEngineResult,
    private val activeProfile: () -> UserProfile?,
    private val updateActiveProfile: (UserProfile) -> Unit,
    private val invalidateHome: () -> Unit
) {
    fun refreshTokenIfNeeded(
        provider: String,
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit
    ) {
        scope.launch {
            val result = dispatch(
                mapOf(
                    "type" to "authRefreshRequested",
                    "provider" to provider,
                    "profile" to profile
                )
            )
            updatedProfile(result)?.takeIf { it != profile }?.let { updated ->
                accept(profile, updated, onProfileUpdated)
            }
        }
    }

    fun exchangeCode(
        provider: String,
        code: String,
        codeVerifier: String?,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        scope.launch {
            val profile = activeProfile()
            if (profile == null) {
                onComplete(false)
                return@launch
            }
            val result = dispatch(
                mapOf(
                    "type" to "authExchangeRequested",
                    "provider" to provider,
                    "code" to code,
                    "codeVerifier" to codeVerifier,
                    "profile" to profile
                )
            )
            val updated = updatedProfile(result)
            if (updated == null) {
                onComplete(false)
                return@launch
            }
            accept(profile, updated, onProfileUpdated)
            onComplete(true)
        }
    }

    fun startTraktDeviceAuthorization(
        onCodeReady: (TraktDeviceCodeResponse) -> Unit,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean, String?) -> Unit
    ) {
        scope.launch {
            val profile = activeProfile()
            if (profile == null) {
                onComplete(false, "toast.trakt_connect_failed")
                return@launch
            }
            val codeResult = dispatch(
                mapOf(
                    "type" to "authFlowRequested",
                    "provider" to "trakt",
                    "mode" to "deviceCode"
                )
            )
            val codeResponse = decode<TraktDeviceCodeResponse>(
                (codeResult.state["auth"] as? Map<*, *>)?.get("result")
            )
            if (codeResponse == null) {
                onComplete(false, "toast.trakt_connect_failed")
                return@launch
            }
            onCodeReady(codeResponse)
            val expiresAt = System.currentTimeMillis() + codeResponse.expiresIn * 1000L
            var intervalMs = codeResponse.interval.coerceAtLeast(5) * 1000L
            var failureMessageKey = "toast.trakt_connect_failed"
            while (System.currentTimeMillis() < expiresAt) {
                delay(intervalMs)
                val tokenResult = dispatch(
                    mapOf(
                        "type" to "authExchangeRequested",
                        "provider" to "traktDevice",
                        "code" to codeResponse.deviceCode,
                        "profile" to (activeProfile() ?: profile)
                    )
                )
                val authResult = (tokenResult.state["auth"] as? Map<*, *>)?.get("result") as? Map<*, *>
                val updated = decode<UserProfile>(authResult?.get("profile"))
                if (updated != null) {
                    accept(activeProfile() ?: profile, updated, onProfileUpdated)
                    onComplete(true, null)
                    return@launch
                }
                val errorCode = authResult?.get("errorCode") as? String
                val httpCode = (authResult?.get("httpCode") as? Number)?.toInt()
                when {
                    errorCode == "slow_down" || httpCode == 429 -> {
                        val retryAfterMs = (authResult.get("retryAfterSeconds") as? Number)?.toLong()?.times(1000L)
                        intervalMs = (retryAfterMs ?: intervalMs + 5_000L).coerceAtMost(60_000L)
                    }
                    errorCode == "expired_token" || errorCode == "invalid_grant" -> {
                        failureMessageKey = "toast.trakt_device_code_expired"
                        break
                    }
                    errorCode == "authorization_pending" || httpCode in setOf(400, 404, 409, 428) -> Unit
                    else -> break
                }
            }
            if (System.currentTimeMillis() >= expiresAt) {
                failureMessageKey = "toast.trakt_device_code_expired"
            }
            onComplete(false, failureMessageKey)
        }
    }

    private fun accept(base: UserProfile, updated: UserProfile, onProfileUpdated: (UserProfile) -> Unit) {
        val merged = mergeSyncedProfile(gson, base, updated, activeProfile())
        updateActiveProfile(merged)
        onProfileUpdated(merged)
        invalidateHome()
    }

    private fun updatedProfile(result: NativeHeadlessEngineResult): UserProfile? {
        val auth = result.state["auth"] as? Map<*, *>
        val authResult = auth?.get("result")
        val value = (authResult as? Map<*, *>)?.get("profile")
            ?: authResult
            ?: (result.state["profile"] as? Map<*, *>)?.get("active")
        return decode(value)
    }

    private inline fun <reified T> decode(value: Any?): T? {
        if (value == null) return null
        return runCatching { gson.fromJson(gson.toJsonTree(value), T::class.java) }.getOrNull()
    }
}
