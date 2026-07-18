package com.fluxa.app.ui.catalog

import android.content.Context
import android.util.Log
import androidx.hilt.work.HiltWorker
import androidx.work.BackoffPolicy
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import androidx.work.workDataOf
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.NuvioSyncCoordinator
import dagger.assisted.Assisted
import dagger.assisted.AssistedInject
import java.util.concurrent.TimeUnit

@HiltWorker
class NuvioPlaybackProgressPushWorker @AssistedInject constructor(
    @Assisted appContext: Context,
    @Assisted params: WorkerParameters,
    private val profileManager: ProfileManager,
    private val nuvioSyncCoordinator: NuvioSyncCoordinator
) : CoroutineWorker(appContext, params) {

    override suspend fun doWork(): Result {
        val profileId = inputData.getString(KEY_PROFILE_ID)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val contentId = inputData.getString(KEY_CONTENT_ID)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val contentType = inputData.getString(KEY_CONTENT_TYPE).orEmpty()
        val videoId = inputData.getString(KEY_VIDEO_ID)
        val position = inputData.getLong(KEY_POSITION, -1L).takeIf { it >= 0L } ?: return Result.failure()
        val duration = inputData.getLong(KEY_DURATION, -1L).takeIf { it > 0L } ?: return Result.failure()

        val profile = profileManager.getProfiles().firstOrNull { it.id == profileId } ?: return Result.failure()
        if (profile.nuvioAccessToken.isNullOrBlank()) return Result.success()

        val meta = Meta(id = contentId, name = contentId, type = contentType)
        return runCatching {
            nuvioSyncCoordinator.pushPlaybackProgress(profile, meta, videoId, position, duration)
        }.fold(
            onSuccess = {
                Log.d("NuvioPushWorker", "Pushed playback progress content_id=$contentId video_id=$videoId position=$position duration=$duration")
                profileManager.clearExternalSyncFailure(profileId, "nuvio")
                Result.success()
            },
            onFailure = { error ->
                Log.w("NuvioPushWorker", "Failed to push playback progress content_id=$contentId video_id=$videoId", error)
                profileManager.recordExternalSyncFailure(profileId, "nuvio")
                Result.retry()
            }
        )
    }

    companion object {
        private const val KEY_PROFILE_ID = "profile_id"
        private const val KEY_CONTENT_ID = "content_id"
        private const val KEY_CONTENT_TYPE = "content_type"
        private const val KEY_VIDEO_ID = "video_id"
        private const val KEY_POSITION = "position"
        private const val KEY_DURATION = "duration"

        fun enqueue(
            context: Context,
            profileId: String,
            contentId: String,
            contentType: String,
            videoId: String?,
            position: Long,
            duration: Long
        ) {
            if (profileId.isBlank() || contentId.isBlank() || duration <= 0L) return
            val work = OneTimeWorkRequestBuilder<NuvioPlaybackProgressPushWorker>()
                .setInputData(
                    workDataOf(
                        KEY_PROFILE_ID to profileId,
                        KEY_CONTENT_ID to contentId,
                        KEY_CONTENT_TYPE to contentType,
                        KEY_VIDEO_ID to videoId,
                        KEY_POSITION to position,
                        KEY_DURATION to duration
                    )
                )
                .setConstraints(
                    Constraints.Builder()
                        .setRequiredNetworkType(NetworkType.CONNECTED)
                        .build()
                )
                .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 30, TimeUnit.SECONDS)
                .build()

            val uniqueName = "nuvio_playback_progress_${profileId}_$contentId"
            WorkManager.getInstance(context.applicationContext)
                .enqueueUniqueWork(uniqueName, ExistingWorkPolicy.REPLACE, work)
        }
    }
}
