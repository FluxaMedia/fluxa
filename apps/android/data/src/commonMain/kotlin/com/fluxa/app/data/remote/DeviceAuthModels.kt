package com.fluxa.app.data.remote

data class DeviceAuthSession(
    val provider: String,
    val sessionId: String,
    val userCode: String,
    val verificationUri: String,
    val pollSecret: String? = null,
    val expiresInSeconds: Long = 300,
    val pollIntervalSeconds: Long = 5,
)

data class DeviceAuthResult(
    val authKey: String? = null,
    val accessToken: String? = null,
    val refreshToken: String? = null,
    val email: String? = null,
    val userId: String? = null,
)

interface DeviceAuthAdapter {
    suspend fun start(deviceName: String): DeviceAuthSession
    suspend fun poll(session: DeviceAuthSession): Boolean
    suspend fun exchange(session: DeviceAuthSession): DeviceAuthResult
}
