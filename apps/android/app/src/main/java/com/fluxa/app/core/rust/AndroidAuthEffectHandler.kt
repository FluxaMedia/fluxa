package com.fluxa.app.core.rust

import android.util.Log
import com.fluxa.app.data.local.*
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.TraktIntegration
import com.fluxa.app.data.repository.TraktRepository
import com.fluxa.app.data.repository.library.ProviderAdapters
import com.fluxa.app.data.repository.library.ThirdPartyProviderRepository
import com.google.gson.Gson
import kotlinx.coroutines.withTimeoutOrNull

private const val ANILIST_TOKEN_LIFETIME_MS = 365L * 24L * 60L * 60L * 1000L

internal class AndroidAuthEffectHandler(
    private val repository: StremioRepository,
    private val traktRepository: TraktRepository,
    private val nuvioAccountImportCoordinator: NuvioAccountImportCoordinator,
    private val providerAdapters: ProviderAdapters,
    private val thirdPartyProviderRepository: ThirdPartyProviderRepository,
    private val gson: Gson
) {
    suspend fun execute(effect: NativeHeadlessEffect): HeadlessEffectCompletion = when (effect.type) {
        "runExternalSync" -> runExternalSync(effect)
        "runAuthFlow" -> runAuthFlow(effect)
        "exchangeAuthCode" -> exchangeAuthCode(effect)
        "refreshAuthToken" -> refreshAuthToken(effect)
        "syncExternalIntegration" -> syncExternalIntegration(effect)
        else -> failure(effect, "unsupported_auth_effect")
    }

    private suspend fun runExternalSync(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val profile = effect.payload.parseProfile() ?: return success(effect, emptyMap<String, Any?>())
        val providerId = ThirdPartyProviderId.from(effect.payload.string("provider"))
            ?: return failure(effect, "unsupported_sync_provider")
        val snapshot = thirdPartyProviderRepository.load(profile, providerId, refresh = true)
            ?: return failure(effect, "provider_not_connected")
        return success(effect, mapOf("snapshot" to snapshot))
    }

    private suspend fun runAuthFlow(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        return when (effect.payload.string("provider")) {
            "trakt" -> when (effect.payload.string("mode")) {
                "deviceCode" -> success(effect, repository.createTraktDeviceCode())
                else -> failure(effect, "unsupported_auth_mode")
            }
            else -> failure(effect, "unsupported_auth_provider")
        }
    }

    private suspend fun exchangeAuthCode(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val profile = payload.parseProfile() ?: return failure(effect, "missing_profile")
        val updated = when (payload.string("provider")) {
            "trakt" -> {
                val response = repository.exchangeTraktCode(payload.string("code"))
                profile.copy(
                    traktAccessToken = response.accessToken,
                    traktRefreshToken = response.refreshToken,
                    traktTokenExpiresAt = TraktIntegration.tokenExpiresAt(response.createdAt, response.expiresIn),
                    traktUsername = repository.getTraktUsername(response.accessToken)
                )
            }
            "traktDevice" -> {
                val response = repository.exchangeTraktDeviceCode(payload.string("code"))
                if (!response.isSuccessful) {
                    val errorCode = response.errorBody()?.string()?.let(FluxaCoreNative::traktOAuthErrorCode)
                    return success(
                        effect,
                        mapOf(
                            "status" to "pending",
                            "errorCode" to (errorCode ?: "http_${response.code()}"),
                            "httpCode" to response.code(),
                            "retryAfterSeconds" to response.headers()["Retry-After"]?.toLongOrNull()
                        )
                    )
                }
                val responseBody = response.body() ?: return failure(effect, "empty_device_token")
                profile.copy(
                    traktAccessToken = responseBody.accessToken,
                    traktRefreshToken = responseBody.refreshToken,
                    traktTokenExpiresAt = TraktIntegration.tokenExpiresAt(responseBody.createdAt, responseBody.expiresIn),
                    traktUsername = repository.getTraktUsername(responseBody.accessToken)
                )
            }
            "simkl" -> {
                val codeVerifier = payload.string("codeVerifier").takeIf(String::isNotBlank)
                    ?: return failure(effect, "missing_simkl_pkce_verifier")
                val response = repository.exchangeSimklCode(payload.string("code"), codeVerifier)
                profile.copy(
                    simklAccessToken = response.accessToken,
                    simklUsername = repository.getSimklUsername(response.accessToken),
                    simklLastSyncAt = System.currentTimeMillis()
                )
            }
            "anilist" -> {
                val accessToken = payload.string("code").takeIf(String::isNotBlank)
                    ?: return failure(effect, "missing_anilist_access_token")
                profile.copy(
                    anilistAccessToken = accessToken,
                    anilistRefreshToken = null,
                    anilistTokenExpiresAt = System.currentTimeMillis() + ANILIST_TOKEN_LIFETIME_MS,
                    anilistUsername = repository.getAnilistUsername(accessToken)
                )
            }
            else -> return failure(effect, "unsupported_auth_provider")
        }
        return success(effect, mapOf("profile" to updated))
    }

    private suspend fun refreshAuthToken(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val profile = effect.payload.parseProfile() ?: return failure(effect, "missing_profile")
        val updated = when (effect.payload.string("provider")) {
            "trakt" -> refreshTraktTokenIfNeeded(profile)
            else -> return failure(effect, "unsupported_auth_provider")
        }
        return success(effect, mapOf("profile" to updated))
    }

    private suspend fun refreshTraktTokenIfNeeded(profile: UserProfile): UserProfile {
        val refreshToken = profile.traktRefreshToken?.takeIf(String::isNotBlank) ?: return profile
        val refreshWindowMs = 24L * 60L * 60L * 1000L
        if (!profile.traktAccessToken.isNullOrBlank() && profile.safeTraktTokenExpiresAt > System.currentTimeMillis() + refreshWindowMs) return profile
        return runCatching {
            val response = traktRepository.refreshTraktToken(refreshToken)
            profile.copy(
                traktAccessToken = response.accessToken,
                traktRefreshToken = response.refreshToken,
                traktTokenExpiresAt = TraktIntegration.tokenExpiresAt(response.createdAt, response.expiresIn)
            )
        }.getOrElse { throwable ->
            Log.w("Trakt", "Token refresh failed", throwable)
            if ((throwable as? retrofit2.HttpException)?.code() in setOf(400, 401)) {
                profile.copy(traktAccessToken = null, traktRefreshToken = null, traktTokenExpiresAt = null)
            } else profile
        }
    }

    private suspend fun syncExternalIntegration(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val originalProfile = payload.parseProfile() ?: return failure(effect, "missing_profile")
        val providerId = ThirdPartyProviderId.from(payload.string("provider"))
            ?: return failure(effect, "unsupported_sync_provider")

        val profile = if (providerId == ThirdPartyProviderId.NUVIO) {
            if (originalProfile.nuvioAccessToken.isNullOrBlank()) return failure(effect, "missing_nuvio_token")
            nuvioAccountImportCoordinator.sync(originalProfile, onStep = {}).profile
        } else {
            originalProfile
        }

        val adapter = providerAdapters.byId(providerId.key)
            ?: return failure(effect, "unsupported_sync_provider")
        if (!adapter.isConnected(profile)) return failure(effect, "provider_not_connected")

        val snapshot = thirdPartyProviderRepository.load(profile, providerId, refresh = true)
            ?: return failure(effect, "provider_sync_failed")
        if (snapshot.fromCache || profile.providerAccountId(providerId) != snapshot.accountId) {
            return failure(effect, "provider_sync_not_fresh")
        }

        val updated = profile.withProviderLastSyncAt(providerId, snapshot.syncedAt)
        val providerExtras = when (providerId) {
            ThirdPartyProviderId.STREMIO -> mapOf(
                "addons" to repository.getUserAddons(profile.authKey, forceRefresh = true)
            )
            ThirdPartyProviderId.TRAKT -> mapOf(
                "syncedItems" to snapshot.itemCount,
                "continueWatchingCount" to snapshot.continueWatching.size,
                "watchlistCount" to snapshot.libraryCount
            )
            else -> emptyMap()
        }
        return success(
            effect,
            mapOf(
                "profile" to updated,
                "snapshot" to mapOf(
                    "profile" to updated,
                    "provider" to snapshot,
                    "extras" to providerExtras
                ),
                "externalContinueWatching" to snapshot.continueWatching
            )
        )
    }

    private fun Map<String, Any?>.parseProfile(): UserProfile? = parseProfile(gson)

    private fun success(effect: NativeHeadlessEffect, value: Any?) =
        HeadlessEffectCompletion(effectId = effect.id, status = "ok", value = value)

    private fun failure(effect: NativeHeadlessEffect, code: String) =
        HeadlessEffectCompletion(effectId = effect.id, status = "error", error = mapOf("code" to code))
}
