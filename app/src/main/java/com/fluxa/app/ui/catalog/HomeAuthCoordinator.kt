package com.fluxa.app.ui.catalog

import android.util.Log
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.TraktDeviceCodeResponse
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.TraktIntegration
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

internal class HomeAuthCoordinator(
    private val repository: StremioRepository,
    private val scope: CoroutineScope,
    private val activeProfile: () -> UserProfile?,
    private val invalidateHome: () -> Unit
) {
    fun exchangeTraktCode(
        code: String,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        scope.launch(Dispatchers.IO) {
            try {
                val response = repository.exchangeTraktCode(code)
                activeProfile()?.let { profile ->
                    val updated = profile.copy(
                        traktAccessToken = response.accessToken,
                        traktRefreshToken = response.refreshToken,
                        traktTokenExpiresAt = TraktIntegration.tokenExpiresAt(response.createdAt, response.expiresIn)
                    )
                    withContext(Dispatchers.Main) {
                        onProfileUpdated(updated)
                        invalidateHome()
                        onComplete(true)
                    }
                }
            } catch (e: Exception) {
                e.printStackTrace()
                withContext(Dispatchers.Main) { onComplete(false) }
            }
        }
    }

    fun startTraktDeviceAuthorization(
        onCodeReady: (TraktDeviceCodeResponse) -> Unit,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean, String?) -> Unit
    ) {
        scope.launch(Dispatchers.IO) {
            try {
                val codeResponse = repository.createTraktDeviceCode()
                withContext(Dispatchers.Main) { onCodeReady(codeResponse) }
                val startedAt = System.currentTimeMillis()
                val expiresAt = startedAt + codeResponse.expiresIn * 1000L
                var intervalMs = codeResponse.interval.coerceAtLeast(5) * 1000L
                var failureMessageKey: String? = "toast.trakt_connect_failed"
                while (System.currentTimeMillis() < expiresAt) {
                    delay(intervalMs)
                    val response = repository.exchangeTraktDeviceCode(codeResponse.deviceCode)
                    if (response.isSuccessful) {
                        val tokenResponse = response.body() ?: break
                        activeProfile()?.let { profile ->
                            val updated = profile.copy(
                                traktAccessToken = tokenResponse.accessToken,
                                traktRefreshToken = tokenResponse.refreshToken,
                                traktTokenExpiresAt = TraktIntegration.tokenExpiresAt(tokenResponse.createdAt, tokenResponse.expiresIn)
                            )
                            withContext(Dispatchers.Main) {
                                onProfileUpdated(updated)
                                invalidateHome()
                                onComplete(true, null)
                            }
                            return@launch
                        }
                        break
                    }
                    val errorCode = response.errorBody()?.string()?.let(FluxaCoreNative::traktOAuthErrorCode)
                    when {
                        errorCode == "slow_down" || response.code() == 429 -> {
                            val retryAfterMs = response.headers()["Retry-After"]?.toLongOrNull()?.let { it * 1000L }
                            intervalMs = (retryAfterMs ?: (intervalMs + 5_000L)).coerceAtMost(60_000L)
                            continue
                        }
                        errorCode == "expired_token" || errorCode == "invalid_grant" -> {
                            Log.w("Trakt", "Device auth expired: $errorCode")
                            failureMessageKey = "toast.trakt_device_code_expired"
                            break
                        }
                        errorCode == "authorization_pending" || response.code() in setOf(400, 404, 409, 428) -> {
                            continue
                        }
                        else -> {
                            Log.w("Trakt", "Device auth failed: HTTP ${response.code()} ${errorCode.orEmpty()}")
                            break
                        }
                    }
                }
                if (System.currentTimeMillis() >= expiresAt) {
                    failureMessageKey = "toast.trakt_device_code_expired"
                }
                withContext(Dispatchers.Main) { onComplete(false, failureMessageKey) }
            } catch (e: Exception) {
                Log.w("Trakt", "Device authorization failed", e)
                withContext(Dispatchers.Main) { onComplete(false, "toast.trakt_connect_failed") }
            }
        }
    }

    fun exchangeMalCode(
        code: String,
        codeVerifier: String,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        scope.launch(Dispatchers.IO) {
            try {
                val response = repository.exchangeMalCode(code, codeVerifier)
                activeProfile()?.let { profile ->
                    val updated = profile.copy(
                        malAccessToken = response.accessToken,
                        malRefreshToken = response.refreshToken
                    )
                    withContext(Dispatchers.Main) {
                        onProfileUpdated(updated)
                        onComplete(true)
                    }
                }
            } catch (e: Exception) {
                e.printStackTrace()
                withContext(Dispatchers.Main) { onComplete(false) }
            }
        }
    }

    fun exchangeSimklCode(
        code: String,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        scope.launch(Dispatchers.IO) {
            try {
                val response = repository.exchangeSimklCode(code)
                activeProfile()?.let { profile ->
                    val updated = profile.copy(simklAccessToken = response.accessToken)
                    withContext(Dispatchers.Main) {
                        onProfileUpdated(updated)
                        onComplete(true)
                    }
                }
            } catch (e: Exception) {
                e.printStackTrace()
                withContext(Dispatchers.Main) { onComplete(false) }
            }
        }
    }

}
