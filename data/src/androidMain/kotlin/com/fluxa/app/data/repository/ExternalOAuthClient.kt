package com.fluxa.app.data.repository

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
    private val traktApi: TraktApi,
    private val config: OAuthClientConfig
) {
    suspend fun exchangeTraktCode(code: String): TraktTokenResponse {
        return traktApi.exchangeCode(
            TraktTokenRequest(
                code = code,
                client_id = config.traktClientId,
                client_secret = config.traktClientSecret.orEmpty(),
                redirect_uri = TraktIntegration.MOBILE_REDIRECT_URI
            )
        )
    }

    suspend fun refreshTraktToken(refreshToken: String): TraktTokenResponse {
        return traktApi.refreshToken(
            TraktRefreshTokenRequest(
                refresh_token = refreshToken,
                client_id = config.traktClientId,
                client_secret = config.traktClientSecret.orEmpty(),
                redirect_uri = TraktIntegration.MOBILE_REDIRECT_URI
            )
        )
    }

    suspend fun createTraktDeviceCode(): TraktDeviceCodeResponse {
        return traktApi.createDeviceCode(
            TraktDeviceCodeRequest(
                client_id = config.traktClientId
            )
        )
    }

    suspend fun exchangeTraktDeviceCode(deviceCode: String): retrofit2.Response<TraktTokenResponse> {
        return traktApi.exchangeDeviceCode(
            TraktDeviceTokenRequest(
                code = deviceCode,
                client_id = config.traktClientId,
                client_secret = config.traktClientSecret.orEmpty()
            )
        )
    }

    suspend fun exchangeSimklCode(code: String): ExternalOAuthTokenResponse {
        return traktApi.exchangeSimklCode(
            clientId = config.simklClientId,
            clientSecret = config.simklClientSecret.orEmpty(),
            grantType = "authorization_code",
            code = code,
            redirectUri = SimklIntegration.REDIRECT_URI
        )
    }

    suspend fun exchangeAnilistCode(code: String): ExternalOAuthTokenResponse {
        return traktApi.exchangeAnilistCode(
            com.fluxa.app.data.remote.AnilistTokenRequest(
                client_id = config.anilistClientId,
                client_secret = config.anilistClientSecret.orEmpty(),
                redirect_uri = AnilistIntegration.REDIRECT_URI,
                code = code
            )
        )
    }

    suspend fun getTraktUsername(accessToken: String): String? = runCatching {
        traktApi.getTraktSettings(TraktIntegration.bearer(accessToken), config.traktClientId)
            .getAsJsonObject("user")
            ?.get("username")
            ?.takeUnless { it.isJsonNull }
            ?.asString
    }.getOrNull()

    suspend fun getSimklUsername(accessToken: String): String? = runCatching {
        traktApi.getSimklSettings("Bearer $accessToken", config.simklClientId)
            .getAsJsonObject("user")
            ?.get("name")
            ?.takeUnless { it.isJsonNull }
            ?.asString
    }.getOrNull()

    suspend fun getAnilistUsername(accessToken: String): String? = runCatching {
        traktApi.anilistGraphQl(
            "Bearer $accessToken",
            com.fluxa.app.data.remote.AnilistGraphQlRequest(query = "query { Viewer { name } }")
        ).getAsJsonObject("data")
            ?.getAsJsonObject("Viewer")
            ?.get("name")
            ?.takeUnless { it.isJsonNull }
            ?.asString
    }.getOrNull()
}
