package com.fluxa.app.data.remote

import retrofit2.Response
import com.google.gson.annotations.SerializedName
import java.security.SecureRandom
import java.util.Base64

data class NuvioTvLoginStartDto(
    val code: String,
    @SerializedName("web_url") val webUrl: String,
    @SerializedName("expires_at") val expiresAt: String,
    @SerializedName("poll_interval_seconds") val pollIntervalSeconds: Long = 3,
)

data class NuvioTvLoginPollDto(
    val approved: Boolean = false,
    val status: String? = null,
)

data class FluxaDeviceStartRequest(@SerializedName("device_name") val deviceName: String? = null)
data class FluxaDevicePollRequest(@SerializedName("session_id") val sessionId: String, @SerializedName("poll_secret") val pollSecret: String)
data class FluxaDeviceExchangeRequest(@SerializedName("session_id") val sessionId: String, @SerializedName("poll_secret") val pollSecret: String)
data class FluxaDeviceStartResponse(
    @SerializedName("session_id") val sessionId: String,
    @SerializedName("user_code") val userCode: String,
    @SerializedName("verification_uri") val verificationUri: String,
    @SerializedName("poll_secret") val pollSecret: String,
    @SerializedName("expires_in") val expiresIn: Long,
    @SerializedName("poll_interval") val pollInterval: Long,
)
data class FluxaDeviceStatusResponse(val status: String)
data class FluxaDeviceExchangeResponse(
    val status: String,
    @SerializedName("access_token") val accessToken: String? = null,
    @SerializedName("refresh_token") val refreshToken: String? = null,
    @SerializedName("token_type") val tokenType: String? = null,
    @SerializedName("expires_in") val expiresIn: Long? = null,
    val user: FluxaDeviceUser? = null,
)
data class FluxaDeviceUser(val id: String? = null, val email: String? = null)

interface FluxaSyncService {
    @retrofit2.http.POST("api/v1/auth/device/start")
    suspend fun startDevice(@retrofit2.http.Body body: FluxaDeviceStartRequest): retrofit2.Response<FluxaDeviceStartResponse>

    @retrofit2.http.POST("api/v1/auth/device/poll")
    suspend fun pollDevice(@retrofit2.http.Body body: FluxaDevicePollRequest): retrofit2.Response<FluxaDeviceStatusResponse>

    @retrofit2.http.POST("api/v1/auth/device/exchange")
    suspend fun exchangeDevice(@retrofit2.http.Body body: FluxaDeviceExchangeRequest): retrofit2.Response<FluxaDeviceExchangeResponse>
}

class NuvioDeviceAuthAdapter(
    private val service: NuvioService,
    private val redirectBaseUrl: String,
) : DeviceAuthAdapter {
    private var nonce: String = ""

    override suspend fun start(deviceName: String): DeviceAuthSession {
        nonce = createDeviceNonce()
        val request = buildMap {
            put("p_device_nonce", nonce)
            put("p_redirect_base_url", redirectBaseUrl)
            if (deviceName.isNotBlank()) put("p_device_name", deviceName)
        }
        val response = service.startTvLogin(request)
        val compatibleResponse = if (response.code() == 400 && deviceName.isNotBlank()) {
            service.startTvLogin(request - "p_device_name")
        } else {
            response
        }
        val body = compatibleResponse.requireBody("Nuvio TV login could not start")
            .firstOrNull() ?: error("Nuvio TV login returned an empty response")
        return DeviceAuthSession(
            provider = "nuvio",
            sessionId = body.code,
            userCode = body.code,
            verificationUri = body.webUrl,
            pollSecret = nonce,
            expiresInSeconds = secondsUntil(body.expiresAt),
            pollIntervalSeconds = body.pollIntervalSeconds.coerceAtLeast(1),
        )
    }

    override suspend fun poll(session: DeviceAuthSession): Boolean {
        val body = service.pollTvLogin(mapOf("p_code" to session.userCode, "p_device_nonce" to (session.pollSecret ?: nonce)))
            .requireBody("Nuvio TV login poll failed")
            .firstOrNull() ?: error("Nuvio TV login poll returned an empty response")
        return body.approved || body.status.equals("approved", ignoreCase = true)
    }

    override suspend fun exchange(session: DeviceAuthSession): DeviceAuthResult {
        val body = service.exchangeTvLogin(mapOf("code" to session.userCode, "device_nonce" to (session.pollSecret ?: nonce))).requireBody("Nuvio TV login exchange failed")
        val user = body.user ?: body.accessToken.let { accessToken ->
            service.getUser("Bearer $accessToken").body()
        }
        return DeviceAuthResult(
            accessToken = body.accessToken,
            refreshToken = body.refreshToken,
            email = user?.email,
            userId = user?.id,
        )
    }
}

private fun createDeviceNonce(): String {
    val bytes = ByteArray(24)
    SecureRandom().nextBytes(bytes)
    return Base64.getUrlEncoder().withoutPadding().encodeToString(bytes)
}

private fun secondsUntil(expiresAt: String): Long = runCatching {
    ((java.time.Instant.parse(expiresAt).toEpochMilli() - System.currentTimeMillis()) / 1000L)
        .coerceIn(1L, 300L)
}.getOrDefault(300L)

class FluxaDeviceAuthAdapter(private val service: FluxaSyncService) : DeviceAuthAdapter {
    override suspend fun start(deviceName: String): DeviceAuthSession {
        val body = service.startDevice(FluxaDeviceStartRequest(deviceName)).requireBody("Fluxa device login could not start")
        return DeviceAuthSession("fluxa", body.sessionId, body.userCode, body.verificationUri, body.pollSecret, body.expiresIn, body.pollInterval)
    }

    override suspend fun poll(session: DeviceAuthSession): Boolean {
        return service.pollDevice(FluxaDevicePollRequest(session.sessionId, session.pollSecret.orEmpty())).requireBody("Fluxa device login poll failed").status == "approved"
    }

    override suspend fun exchange(session: DeviceAuthSession): DeviceAuthResult {
        val body = service.exchangeDevice(FluxaDeviceExchangeRequest(session.sessionId, session.pollSecret.orEmpty())).requireBody("Fluxa device login exchange failed")
        return DeviceAuthResult(accessToken = body.accessToken, refreshToken = body.refreshToken, email = body.user?.email, userId = body.user?.id)
    }
}

private fun <T> Response<T>.requireBody(message: String): T {
    if (!isSuccessful) {
        val detail = errorBody()?.string()?.replace(Regex("\\s+"), " ")?.take(240).orEmpty()
        error("$message (${code()})${detail.takeIf { it.isNotBlank() }?.let { ": $it"}.orEmpty()}")
    }
    return body() ?: error(message)
}
