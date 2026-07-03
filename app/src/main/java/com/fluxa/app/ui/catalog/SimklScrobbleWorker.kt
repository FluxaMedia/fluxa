package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.content.Context
import android.util.Log
import androidx.work.BackoffPolicy
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.OutOfQuotaPolicy
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import androidx.work.workDataOf
import com.fluxa.app.BuildConfig
import androidx.hilt.work.HiltWorker
import dagger.assisted.Assisted
import dagger.assisted.AssistedInject
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.RequestBody.Companion.toRequestBody
import java.util.concurrent.TimeUnit

@HiltWorker
class SimklScrobbleWorker @AssistedInject constructor(
    @Assisted appContext: Context,
    @Assisted params: WorkerParameters,
    private val profileManager: ProfileManager,
    private val api: TraktApi
) : CoroutineWorker(appContext, params) {

    override suspend fun doWork(): Result {
        val clientId = BuildConfig.SIMKL_CLIENT_ID
        if (clientId.isBlank()) return Result.success()

        val profileId = inputData.getString(KEY_PROFILE_ID)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val mediaType = inputData.getString(KEY_MEDIA_TYPE)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val mediaId = inputData.getString(KEY_MEDIA_ID)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val action = inputData.getString(KEY_ACTION)?.takeIf { it in setOf("start", "pause", "stop") } ?: return Result.failure()
        val positionMs = inputData.getLong(KEY_POSITION_MS, -1L).takeIf { it >= 0L } ?: return Result.failure()
        val durationMs = inputData.getLong(KEY_DURATION_MS, -1L).takeIf { it > 0L } ?: return Result.failure()

        val profile = profileManager.getProfiles().firstOrNull { it.id == profileId } ?: return Result.failure()
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
                        profileManager.saveProfile(profile.copy(simklLastSyncAt = System.currentTimeMillis()))
                        Result.success()
                    }
                    response.code() == 401 -> {
                        Log.w("SimklScrobbleWorker", "Simkl access revoked for profile=$profileId, clearing token")
                        profileManager.saveProfile(profile.copy(simklAccessToken = null))
                        Result.failure()
                    }
                    response.code() == 429 || response.code() >= 500 -> Result.retry()
                    else -> {
                        Log.w("SimklScrobbleWorker", "Scrobble $action failed media_id=$mediaId http=${response.code()}")
                        Result.failure()
                    }
                }
            },
            onFailure = { error ->
                Log.w("SimklScrobbleWorker", "Scrobble $action failed for $mediaId", error)
                Result.retry()
            }
        )
    }

    companion object {
        private const val KEY_PROFILE_ID = "profile_id"
        private const val KEY_MEDIA_TYPE = "media_type"
        private const val KEY_MEDIA_ID = "media_id"
        private const val KEY_ACTION = "action"
        private const val KEY_POSITION_MS = "position_ms"
        private const val KEY_DURATION_MS = "duration_ms"

        fun enqueue(
            context: Context,
            profileId: String,
            mediaType: String,
            mediaId: String,
            action: String,
            positionMs: Long,
            durationMs: Long
        ) {
            if (profileId.isBlank() || mediaId.isBlank() || durationMs <= 0L || action !in setOf("start", "pause", "stop")) return
            val work = OneTimeWorkRequestBuilder<SimklScrobbleWorker>()
                .setInputData(
                    workDataOf(
                        KEY_PROFILE_ID to profileId,
                        KEY_MEDIA_TYPE to mediaType,
                        KEY_MEDIA_ID to mediaId,
                        KEY_ACTION to action,
                        KEY_POSITION_MS to positionMs,
                        KEY_DURATION_MS to durationMs
                    )
                )
                .setConstraints(
                    Constraints.Builder()
                        .setRequiredNetworkType(NetworkType.CONNECTED)
                        .build()
                )
                .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 30, TimeUnit.SECONDS)
                .setExpedited(OutOfQuotaPolicy.RUN_AS_NON_EXPEDITED_WORK_REQUEST)
                .build()

            val uniqueName = "simkl_scrobble_${profileId}_${mediaType}_${mediaId}"
            WorkManager.getInstance(context.applicationContext)
                .enqueueUniqueWork(uniqueName, ExistingWorkPolicy.REPLACE, work)
        }
    }
}
