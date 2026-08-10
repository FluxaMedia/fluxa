package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.ExternalOAuthTokenResponse
import com.fluxa.app.data.remote.ExternalSyncApi
import com.fluxa.app.data.remote.TraktDeviceCodeRequest
import com.fluxa.app.data.remote.TraktDeviceCodeResponse
import com.fluxa.app.data.remote.TraktDeviceTokenRequest
import com.fluxa.app.data.remote.TraktRefreshTokenRequest
import com.fluxa.app.data.remote.TraktTokenRequest
import com.fluxa.app.data.remote.TraktTokenResponse
import javax.inject.Inject

class ExternalOAuthClient @Inject constructor(
    private val externalSyncApi: ExternalSyncApi,
    private val config: OAuthClientConfig
) {
    suspend fun exchangeTraktCode(code: String): TraktTokenResponse {
        return externalSyncApi.exchangeCode(
            TraktTokenRequest(
                code = code,
                client_id = config.traktClientId,
                client_secret = config.traktClientSecret.orEmpty(),
                redirect_uri = TraktIntegration.MOBILE_REDIRECT_URI
            )
        )
    }

    suspend fun refreshTraktToken(refreshToken: String): TraktTokenResponse {
        return externalSyncApi.refreshToken(
            TraktRefreshTokenRequest(
                refresh_token = refreshToken,
                client_id = config.traktClientId,
                client_secret = config.traktClientSecret.orEmpty(),
                redirect_uri = TraktIntegration.MOBILE_REDIRECT_URI
            )
        )
    }

    suspend fun createTraktDeviceCode(): TraktDeviceCodeResponse {
        return externalSyncApi.createDeviceCode(
            TraktDeviceCodeRequest(
                client_id = config.traktClientId
            )
        )
    }

    suspend fun exchangeTraktDeviceCode(deviceCode: String): retrofit2.Response<TraktTokenResponse> {
        return externalSyncApi.exchangeDeviceCode(
            TraktDeviceTokenRequest(
                code = deviceCode,
                client_id = config.traktClientId,
                client_secret = config.traktClientSecret.orEmpty()
            )
        )
    }

    suspend fun exchangeSimklCode(
        code: String,
        codeVerifier: String,
        redirectUri: String = SimklIntegration.REDIRECT_URI,
    ): ExternalOAuthTokenResponse {
        require(codeVerifier.length in 43..128) { "Invalid Simkl PKCE code verifier" }
        return externalSyncApi.exchangeSimklCode(
            clientId = config.simklClientId,
            codeVerifier = codeVerifier,
            grantType = "authorization_code",
            code = code,
            redirectUri = redirectUri,
        )
    }

    suspend fun getTraktUsername(accessToken: String): String? = runCatching {
        externalSyncApi.getTraktSettings(TraktIntegration.bearer(accessToken), config.traktClientId)
            .getAsJsonObject("user")
            ?.get("username")
            ?.takeUnless { it.isJsonNull }
            ?.asString
    }.getOrNull()

    suspend fun getSimklUsername(accessToken: String): String? = runCatching {
        externalSyncApi.getSimklSettings("Bearer $accessToken", config.simklClientId)
            .getAsJsonObject("user")
            ?.get("name")
            ?.takeUnless { it.isJsonNull }
            ?.asString
    }.getOrNull()

    suspend fun getAnilistUsername(accessToken: String): String? = runCatching {
        externalSyncApi.anilistGraphQl(
            "Bearer $accessToken",
            com.fluxa.app.data.remote.AnilistGraphQlRequest(query = "query { Viewer { name } }")
        ).getAsJsonObject("data")
            ?.getAsJsonObject("Viewer")
            ?.get("name")
            ?.takeUnless { it.isJsonNull }
            ?.asString
    }.getOrNull()
}
