package com.fluxa.app.data.repository

import com.fluxa.app.data.BuildConfig
import com.fluxa.app.data.remote.ExternalOAuthTokenResponse
import com.fluxa.app.data.remote.TraktApi
import com.fluxa.app.data.remote.TraktDeviceCodeRequest
import com.fluxa.app.data.remote.TraktDeviceCodeResponse
import com.fluxa.app.data.remote.TraktDeviceTokenRequest
import com.fluxa.app.data.remote.TraktRefreshTokenRequest
import com.fluxa.app.data.remote.TraktTokenRequest
import com.fluxa.app.data.remote.TraktTokenResponse
import javax.inject.Inject

class ExternalOAuthClient @Inject constructor(
    private val traktApi: TraktApi
) {
    suspend fun exchangeTraktCode(code: String): TraktTokenResponse {
        return traktApi.exchangeCode(
            TraktTokenRequest(
                code = code,
                client_id = BuildConfig.TRAKT_CLIENT_ID,
                client_secret = BuildConfig.TRAKT_CLIENT_SECRET,
                redirect_uri = TraktIntegration.MOBILE_REDIRECT_URI
            )
        )
    }

    suspend fun refreshTraktToken(refreshToken: String): TraktTokenResponse {
        return traktApi.refreshToken(
            TraktRefreshTokenRequest(
                refresh_token = refreshToken,
                client_id = BuildConfig.TRAKT_CLIENT_ID,
                client_secret = BuildConfig.TRAKT_CLIENT_SECRET,
                redirect_uri = TraktIntegration.MOBILE_REDIRECT_URI
            )
        )
    }

    suspend fun createTraktDeviceCode(): TraktDeviceCodeResponse {
        return traktApi.createDeviceCode(
            TraktDeviceCodeRequest(
                client_id = BuildConfig.TRAKT_CLIENT_ID
            )
        )
    }

    suspend fun exchangeTraktDeviceCode(deviceCode: String): retrofit2.Response<TraktTokenResponse> {
        return traktApi.exchangeDeviceCode(
            TraktDeviceTokenRequest(
                code = deviceCode,
                client_id = BuildConfig.TRAKT_CLIENT_ID,
                client_secret = BuildConfig.TRAKT_CLIENT_SECRET
            )
        )
    }

    suspend fun exchangeMalCode(code: String, codeVerifier: String): ExternalOAuthTokenResponse {
        return traktApi.exchangeMalCode(
            clientId = BuildConfig.MAL_CLIENT_ID,
            clientSecret = BuildConfig.MAL_CLIENT_SECRET.takeIf { it.isNotBlank() },
            grantType = "authorization_code",
            code = code,
            redirectUri = "fluxa://oauth/mal",
            codeVerifier = codeVerifier
        )
    }

    suspend fun refreshMalToken(refreshToken: String): ExternalOAuthTokenResponse {
        return traktApi.refreshMalToken(
            clientId = BuildConfig.MAL_CLIENT_ID,
            clientSecret = BuildConfig.MAL_CLIENT_SECRET.takeIf { it.isNotBlank() },
            grantType = "refresh_token",
            refreshToken = refreshToken
        )
    }

    suspend fun exchangeSimklCode(code: String): ExternalOAuthTokenResponse {
        return traktApi.exchangeSimklCode(
            clientId = BuildConfig.SIMKL_CLIENT_ID,
            clientSecret = BuildConfig.SIMKL_CLIENT_SECRET,
            grantType = "authorization_code",
            code = code,
            redirectUri = "fluxa://oauth/simkl"
        )
    }
}
