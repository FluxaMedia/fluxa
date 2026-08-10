package com.fluxa.app.data.repository

import java.net.URLEncoder
import java.nio.charset.StandardCharsets
import java.security.MessageDigest
import java.security.SecureRandom
import java.util.Base64

/** PKCE helpers for Simkl public clients. No client secret is required. */
data class SimklPkceRequest(
    val codeVerifier: String,
    val codeChallenge: String,
    val state: String,
    val createdAtMs: Long = System.currentTimeMillis(),
)

object SimklPkce {
    private val secureRandom = SecureRandom()

    fun create(): SimklPkceRequest {
        val verifierBytes = ByteArray(32).also(secureRandom::nextBytes)
        val verifier = base64Url(verifierBytes)
        val challenge = base64Url(
            MessageDigest.getInstance("SHA-256")
                .digest(verifier.toByteArray(StandardCharsets.US_ASCII))
        )
        val stateBytes = ByteArray(24).also(secureRandom::nextBytes)
        return SimklPkceRequest(
            codeVerifier = verifier,
            codeChallenge = challenge,
            state = base64Url(stateBytes),
        )
    }

    fun authorizationUrl(
        clientId: String,
        request: SimklPkceRequest,
        redirectUri: String = SimklIntegration.REDIRECT_URI,
    ): String {
        fun encode(value: String): String = URLEncoder.encode(value, StandardCharsets.UTF_8.name())
        return buildString {
            append("https://simkl.com/oauth/authorize")
            append("?response_type=code")
            append("&client_id=").append(encode(clientId))
            append("&redirect_uri=").append(encode(redirectUri))
            append("&code_challenge=").append(encode(request.codeChallenge))
            append("&code_challenge_method=S256")
            append("&state=").append(encode(request.state))
            append("&app-name=Fluxa")
        }
    }

    fun isFresh(request: SimklPkceRequest, nowMs: Long = System.currentTimeMillis()): Boolean {
        return nowMs - request.createdAtMs in 0..MAX_SESSION_AGE_MS
    }

    private fun base64Url(bytes: ByteArray): String =
        Base64.getUrlEncoder().withoutPadding().encodeToString(bytes)

    private const val MAX_SESSION_AGE_MS = 10L * 60L * 1000L
}
