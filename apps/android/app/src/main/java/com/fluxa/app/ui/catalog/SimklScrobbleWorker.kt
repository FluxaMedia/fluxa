package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.content.Context
import android.util.Log
import androidx.work.WorkerParameters
import com.fluxa.app.BuildConfig
import androidx.hilt.work.HiltWorker
import dagger.assisted.Assisted
import dagger.assisted.AssistedInject
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.RequestBody.Companion.toRequestBody

@HiltWorker
class SimklScrobbleWorker @AssistedInject constructor(
    @Assisted appContext: Context,
    @Assisted params: WorkerParameters,
    profileManager: ProfileManager,
    private val api: ExternalSyncApi
) : ProviderSyncPushWorker(appContext, params, profileManager) {

    override val providerName = "simkl"

    override suspend fun doWork(): Result {
        val clientId = BuildConfig.SIMKL_CLIENT_ID
        if (clientId.isBlank()) return Result.success()

        val profileId = inputData.getString(KEY_PROFILE_ID)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val expectedAccountId = inputData.getString(KEY_PROVIDER_ACCOUNT_ID)?.takeIf { it.isNotBlank() }
            ?: return Result.failure()
        val mediaType = inputData.getString(KEY_MEDIA_TYPE)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val mediaId = inputData.getString(KEY_MEDIA_ID)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val action = inputData.getString(KEY_ACTION)?.takeIf { it in setOf("start", "pause", "stop") } ?: return Result.failure()
        val positionMs = inputData.getLong(KEY_POSITION_MS, -1L).takeIf { it >= 0L } ?: return Result.failure()
        val durationMs = inputData.getLong(KEY_DURATION_MS, -1L).takeIf { it > 0L } ?: return Result.failure()

        val profile = requireProfile(profileId) ?: return Result.failure()
        if (profile.providerAccountId(ThirdPartyProviderId.SIMKL) != expectedAccountId) {
            // The queued write belongs to a disconnected or replaced remote account.
            return Result.success()
        }
        val token = profile.simklAccessToken?.takeIf { it.isNotBlank() } ?: return Result.success()

        val imdbId = SimklIntegration.imdbIdFrom(mediaId)
        if (imdbId.isNullOrBlank()) return Result.success()

        val isEpisode = mediaType == "series"
        val episode = if (isEpisode) TraktIntegration.episodeLocator(mediaId) else null
        if (isEpisode && episode == null) return Result.success()

        val bearer = "Bearer $token"
        val wantType = if (isEpisode) "tv" else "movie"

        return runCatching {
            val lookup = api.simklSearchById(imdbId, clientId, bearer)
            val simklId = if (lookup.isSuccessful) {
                lookup.body()?.firstOrNull { it.type == wantType }?.ids?.simkl
            } else null

            val body = SimklIntegration.scrobbleBody(
                imdbId = imdbId,
                simklId = simklId,
                isEpisode = isEpisode,
                season = episode?.season ?: 1,
                episode = episode?.episode ?: 1,
                timePosSec = positionMs / 1000.0,
                durationSec = durationMs / 1000.0
            ) ?: return@runCatching null

            val requestBody = body.toRequestBody("application/json".toMediaType())
            when (action) {
                "start" -> api.simklScrobbleStart(clientId, bearer, body = requestBody)
                "pause" -> api.simklScrobblePause(clientId, bearer, body = requestBody)
                else -> api.simklScrobbleStop(clientId, bearer, body = requestBody)
            }
        }.fold(
            onSuccess = { response ->
                when {
                    response == null -> Result.success()
                    response.isSuccessful -> {
                        profileManager.updateProfile(profile.id) { it.copy(simklLastSyncAt = System.currentTimeMillis()) }
                        onSyncSuccess(profileId)
                        Result.success()
                    }
                    response.code() == 401 -> {
                        Log.w("SimklScrobbleWorker", "Simkl access revoked for profile=$profileId, clearing token")
                        profileManager.updateProfile(profile.id) { it.copy(simklAccessToken = null) }
                        Result.failure()
                    }
                    response.code() == 429 || response.code() >= 500 -> {
                        onSyncFailure(profileId)
                        Result.retry()
                    }
                    else -> {
                        Log.w("SimklScrobbleWorker", "Scrobble $action failed media_id=$mediaId http=${response.code()}")
                        onSyncFailure(profileId)
                        Result.failure()
                    }
                }
            },
            onFailure = { error ->
                Log.w("SimklScrobbleWorker", "Scrobble $action failed for $mediaId", error)
                onSyncFailure(profileId)
                Result.retry()
            }
        )
    }

    companion object {
        internal const val KEY_PROFILE_ID = "profile_id"
        internal const val KEY_PROVIDER_ACCOUNT_ID = "provider_account_id"
        internal const val KEY_MEDIA_TYPE = "media_type"
        internal const val KEY_MEDIA_ID = "media_id"
        internal const val KEY_ACTION = "action"
        internal const val KEY_POSITION_MS = "position_ms"
        internal const val KEY_DURATION_MS = "duration_ms"
    }

}
