package com.fluxa.app.ui.catalog

import android.content.Context
import android.util.Log
import androidx.hilt.work.HiltWorker
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import androidx.work.workDataOf
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.StremioRepository
import com.google.gson.Gson
import dagger.assisted.Assisted
import dagger.assisted.AssistedInject

@HiltWorker
class StremioPlaybackProgressPushWorker @AssistedInject constructor(
    @Assisted appContext: Context,
    @Assisted params: WorkerParameters,
    profileManager: ProfileManager,
    private val repository: StremioRepository,
    private val gson: Gson
) : ProviderSyncPushWorker(appContext, params, profileManager) {

    override val providerName = "stremio"

    override suspend fun doWork(): Result {
        val profileId = inputData.getString(KEY_PROFILE_ID)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val metaJson = inputData.getString(KEY_META_JSON)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val timeOffset = inputData.getLong(KEY_TIME_OFFSET, -1L).takeIf { it >= 0L } ?: return Result.failure()
        val duration = inputData.getLong(KEY_DURATION, -1L).takeIf { it > 0L } ?: return Result.failure()

        val profile = requireProfile(profileId) ?: return Result.failure()
        val meta = runCatching { gson.fromJson(metaJson, Meta::class.java) }.getOrNull() ?: return Result.failure()

        val success = repository.savePlaybackProgress(profile.authKey, meta, timeOffset, duration)
        return if (success) {
            onSyncSuccess(profileId)
            Result.success()
        } else {
            Log.w("StremioPushWorker", "Failed to push playback progress content_id=${meta.id}")
            onSyncFailure(profileId)
            Result.retry()
        }
    }

    companion object {
        private const val KEY_PROFILE_ID = "profile_id"
        private const val KEY_META_JSON = "meta_json"
        private const val KEY_TIME_OFFSET = "time_offset"
        private const val KEY_DURATION = "duration"

        fun enqueue(context: Context, gson: Gson, profileId: String, meta: Meta, timeOffset: Long, duration: Long) {
            if (profileId.isBlank() || meta.id.isBlank() || duration <= 0L) return
            val inputData = workDataOf(
                KEY_PROFILE_ID to profileId,
                KEY_META_JSON to gson.toJson(meta),
                KEY_TIME_OFFSET to timeOffset,
                KEY_DURATION to duration
            )
            ProviderSyncPushWorker.enqueueUnique<StremioPlaybackProgressPushWorker>(
                context,
                "stremio_playback_progress_${profileId}_${meta.id}",
                inputData
            )
        }
    }
}
