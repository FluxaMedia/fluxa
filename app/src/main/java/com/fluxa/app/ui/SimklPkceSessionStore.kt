package com.fluxa.app.ui

import android.content.Context
import com.fluxa.app.data.repository.SimklPkceRequest

internal data class AndroidSimklPkceSession(
    val request: SimklPkceRequest,
    val profileId: String?,
)

/**
 * Persists the one-time Simkl PKCE verifier while the system browser owns the foreground.
 * The verifier is short-lived and consumed exactly once when the OAuth redirect returns.
 */
internal class SimklPkceSessionStore(context: Context) {
    private val prefs = context.applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    fun save(request: SimklPkceRequest, profileId: String?) {
        prefs.edit()
            .putString(KEY_VERIFIER, request.codeVerifier)
            .putString(KEY_STATE, request.state)
            .putLong(KEY_CREATED_AT, request.createdAtMs)
            .putString(KEY_PROFILE_ID, profileId)
            .apply()
    }

    fun consume(returnedState: String?): AndroidSimklPkceSession? {
        val verifier = prefs.getString(KEY_VERIFIER, null)
        val expectedState = prefs.getString(KEY_STATE, null)
        val createdAt = prefs.getLong(KEY_CREATED_AT, 0L)
        val profileId = prefs.getString(KEY_PROFILE_ID, null)

        if (verifier.isNullOrBlank() || expectedState.isNullOrBlank()) return null
        val request = SimklPkceRequest(
            codeVerifier = verifier,
            codeChallenge = "",
            state = expectedState,
            createdAtMs = createdAt,
        )
        if (!com.fluxa.app.data.repository.SimklPkce.isFresh(request)) {
            clear()
            return null
        }
        if (returnedState.isNullOrBlank() || returnedState != expectedState) return null

        clear()
        return AndroidSimklPkceSession(request = request, profileId = profileId)
    }

    fun clear() {
        prefs.edit()
            .remove(KEY_VERIFIER)
            .remove(KEY_STATE)
            .remove(KEY_CREATED_AT)
            .remove(KEY_PROFILE_ID)
            .apply()
    }

    private companion object {
        const val PREFS_NAME = "fluxa_simkl_pkce"
        const val KEY_VERIFIER = "verifier"
        const val KEY_STATE = "state"
        const val KEY_CREATED_AT = "created_at"
        const val KEY_PROFILE_ID = "profile_id"
    }
}
