package com.fluxa.app.core.rust

import android.util.Log
import com.fluxa.app.data.local.*
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.TraktIntegration
import com.fluxa.app.data.repository.TraktRepository
import com.google.gson.Gson
import kotlinx.coroutines.withTimeoutOrNull

internal class AndroidAuthEffectHandler(
    private val repository: StremioRepository,
    private val traktRepository: TraktRepository,
    private val watchlistManager: WatchlistManager,
    private val nuvioAccountImportCoordinator: NuvioAccountImportCoordinator,
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
        return success(
            effect,
            mapOf(
                "snapshot" to when (effect.payload.string("provider")) {
                    "trakt" -> traktRepository.getSyncSnapshot(profile, effect.payload.string("language", profile.safeLanguage))
                    else -> repository.getExternalContinueWatching(profile, effect.payload.string("language", profile.safeLanguage))
                }
            )
        )
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
                    traktTokenExpiresAt = TraktIntegration.tokenExpiresAt(response.createdAt, response.expiresIn)
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
                    traktTokenExpiresAt = TraktIntegration.tokenExpiresAt(responseBody.createdAt, responseBody.expiresIn)
                )
            }
            "mal" -> {
                val response = repository.exchangeMalCode(payload.string("code"), payload.string("codeVerifier"))
                profile.copy(
                    malAccessToken = response.accessToken,
                    malRefreshToken = response.refreshToken,
                    malTokenExpiresAt = response.expiresIn?.let { System.currentTimeMillis() + it * 1000L }
                )
            }
            "simkl" -> profile.copy(simklAccessToken = repository.exchangeSimklCode(payload.string("code")).accessToken)
            "anilist" -> {
                val response = repository.exchangeAnilistCode(payload.string("code"))
                profile.copy(
                    anilistAccessToken = response.accessToken,
                    anilistRefreshToken = response.refreshToken,
                    anilistTokenExpiresAt = response.expiresIn?.let { System.currentTimeMillis() + it * 1000L }
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
            "mal" -> refreshMalTokenIfNeeded(profile)
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

    private suspend fun refreshMalTokenIfNeeded(profile: UserProfile): UserProfile {
        val refreshToken = profile.malRefreshToken?.takeIf(String::isNotBlank) ?: return profile
        val refreshWindowMs = 24L * 60L * 60L * 1000L
        if (!profile.malAccessToken.isNullOrBlank() && profile.safeMalTokenExpiresAt > System.currentTimeMillis() + refreshWindowMs) return profile
        return runCatching {
            val response = repository.refreshMalToken(refreshToken)
            profile.copy(
                malAccessToken = response.accessToken,
                malRefreshToken = response.refreshToken ?: refreshToken,
                malTokenExpiresAt = response.expiresIn?.let { System.currentTimeMillis() + it * 1000L }
            )
        }.getOrElse { throwable ->
            Log.w("Mal", "Token refresh failed", throwable)
            if ((throwable as? retrofit2.HttpException)?.code() in setOf(400, 401)) {
                profile.copy(malAccessToken = null, malRefreshToken = null, malTokenExpiresAt = null)
            } else profile
        }
    }

    private suspend fun syncExternalIntegration(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val profile = payload.parseProfile() ?: return failure(effect, "missing_profile")
        if (payload.string("provider") == "nuvio") {
            if (profile.nuvioAccessToken.isNullOrBlank()) return failure(effect, "missing_nuvio_token")
            val imported = nuvioAccountImportCoordinator.sync(profile) {}
            return success(
                effect,
                mapOf(
                    "profile" to imported.profile,
                    "snapshot" to mapOf("profile" to imported.profile),
                    "externalContinueWatching" to imported.externalContinueWatching
                )
            )
        }
        if (payload.string("provider") == "stremio") {
            if (profile.authKey.isBlank()) return failure(effect, "missing_stremio_token")
            val addons = repository.getUserAddons(profile.authKey, forceRefresh = true)
            val library = repository.getLibraryItems(profile.authKey)
            val updated = profile.copy(localAddons = addons.map { it.transportUrl }.distinct())
            return success(
                effect,
                mapOf(
                    "profile" to updated,
                    "snapshot" to mapOf("addons" to addons, "library" to library),
                    "externalContinueWatching" to library
                )
            )
        }
        val traktToken = profile.traktAccessToken ?: return failure(effect, "missing_trakt_token")
        val language = payload.string("language", profile.safeLanguage)
        val snapshot = traktRepository.getTraktSyncSnapshot(profile, language)
        val watchedState = withTimeoutOrNull(8_000L) { traktRepository.getTraktWatchedState(traktToken) }
        if (watchedState != null) {
            watchlistManager.replaceExternalWatchedEpisodes("trakt", watchedState.episodeIdsBySeries)
            watchlistManager.replaceExternalWatchedContentDurations("trakt", watchedState.durationRecords)
        }
        val externalItems = repository.getExternalContinueWatching(profile, language)
        val updated = profile.copy(
            traktLastSyncAt = System.currentTimeMillis(),
            traktLastSyncedItems = snapshot.syncedItems,
            traktLastContinueWatchingCount = snapshot.continueWatchingCount,
            traktLastWatchlistCount = snapshot.watchlistCount
        )
        return success(
            effect,
            mapOf(
                "profile" to updated,
                "snapshot" to snapshot,
                "watchedState" to watchedState,
                "externalContinueWatching" to externalItems
            )
        )
    }

    private fun Map<String, Any?>.parseProfile(): UserProfile? = parseProfile(gson)

    private fun success(effect: NativeHeadlessEffect, value: Any?) =
        HeadlessEffectCompletion(effectId = effect.id, status = "ok", value = value)

    private fun failure(effect: NativeHeadlessEffect, code: String) =
        HeadlessEffectCompletion(effectId = effect.id, status = "error", error = mapOf("code" to code))
}
