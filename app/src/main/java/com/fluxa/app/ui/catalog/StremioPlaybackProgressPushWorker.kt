package com.fluxa.app.ui.catalog

import android.content.Context
import android.util.Log
import androidx.hilt.work.HiltWorker
import androidx.work.WorkerParameters
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.providerAccountId
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
        val expectedAccountId = inputData.getString(KEY_PROVIDER_ACCOUNT_ID)?.takeIf { it.isNotBlank() }
            ?: return Result.failure()
        val metaJson = inputData.getString(KEY_META_JSON)?.takeIf { it.isNotBlank() } ?: return Result.failure()
        val timeOffset = inputData.getLong(KEY_TIME_OFFSET, -1L).takeIf { it >= 0L } ?: return Result.failure()
        val duration = inputData.getLong(KEY_DURATION, -1L).takeIf { it > 0L } ?: return Result.failure()

        val profile = requireProfile(profileId) ?: return Result.failure()
        if (profile.providerAccountId(ThirdPartyProviderId.STREMIO) != expectedAccountId) {
            // The queued write belongs to a disconnected or replaced remote account.
            return Result.success()
        }
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
        internal const val KEY_PROFILE_ID = "profile_id"
        internal const val KEY_PROVIDER_ACCOUNT_ID = "provider_account_id"
        internal const val KEY_META_JSON = "meta_json"
        internal const val KEY_TIME_OFFSET = "time_offset"
        internal const val KEY_DURATION = "duration"
    }

}
