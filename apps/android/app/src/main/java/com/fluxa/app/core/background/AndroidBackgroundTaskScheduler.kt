package com.fluxa.app.core.background

import android.content.Context
import androidx.work.BackoffPolicy
import androidx.work.Constraints
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.OutOfQuotaPolicy
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.workDataOf
import com.fluxa.app.domain.background.BackgroundTaskScheduler
import com.fluxa.app.domain.playback.NuvioPlaybackProgressSchedule
import com.fluxa.app.domain.playback.PlaybackProgressScheduler
import com.fluxa.app.domain.playback.SimklScrobbleSchedule
import com.fluxa.app.domain.playback.StremioPlaybackProgressSchedule
import com.fluxa.app.domain.playback.TraktScrobbleSchedule
import com.fluxa.app.plugins.PluginAutoUpdateWorker
import com.fluxa.app.ui.catalog.EpisodeReleaseWorker
import com.fluxa.app.ui.catalog.NuvioPlaybackProgressPushWorker
import com.fluxa.app.ui.catalog.ProviderSyncPushWorker
import com.fluxa.app.ui.catalog.SimklScrobbleWorker
import com.fluxa.app.ui.catalog.StremioPlaybackProgressPushWorker
import com.fluxa.app.ui.catalog.TraktScrobbleWorker
import com.google.gson.Gson
import dagger.hilt.android.qualifiers.ApplicationContext
import java.util.concurrent.TimeUnit
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AndroidBackgroundTaskScheduler @Inject constructor(
    @ApplicationContext context: Context,
    private val gson: Gson,
) : BackgroundTaskScheduler, PlaybackProgressScheduler {
    private val applicationContext = context.applicationContext
    private val workManager: WorkManager
        get() = WorkManager.getInstance(applicationContext)
    private val networkConstraints = Constraints.Builder()
        .setRequiredNetworkType(NetworkType.CONNECTED)
        .build()

    override fun scheduleEpisodeReleaseChecks() {
        val periodic = PeriodicWorkRequestBuilder<EpisodeReleaseWorker>(6, TimeUnit.HOURS)
            .setConstraints(networkConstraints)
            .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, ProviderSyncPushWorker.BACKOFF_DELAY_SECONDS, TimeUnit.SECONDS)
            .build()
        val immediate = OneTimeWorkRequestBuilder<EpisodeReleaseWorker>()
            .setInitialDelay(20, TimeUnit.SECONDS)
            .setConstraints(networkConstraints)
            .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, ProviderSyncPushWorker.BACKOFF_DELAY_SECONDS, TimeUnit.SECONDS)
            .build()

        workManager.enqueueUniquePeriodicWork(
            EpisodeReleaseWorker.UNIQUE_PERIODIC_WORK_NAME,
            ExistingPeriodicWorkPolicy.UPDATE,
            periodic,
        )
        workManager.enqueueUniqueWork(
            EpisodeReleaseWorker.UNIQUE_IMMEDIATE_WORK_NAME,
            ExistingWorkPolicy.KEEP,
            immediate,
        )
    }

    override fun schedulePluginAutoUpdate() {
        val request = OneTimeWorkRequestBuilder<PluginAutoUpdateWorker>()
            .setInitialDelay(5, TimeUnit.SECONDS)
            .setConstraints(networkConstraints)
            .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, ProviderSyncPushWorker.BACKOFF_DELAY_SECONDS, TimeUnit.SECONDS)
            .build()
        workManager.enqueueUniqueWork(
            PluginAutoUpdateWorker.UNIQUE_WORK_NAME,
            ExistingWorkPolicy.KEEP,
            request,
        )
    }

    override fun cancelPluginAutoUpdate() {
        workManager.cancelUniqueWork(PluginAutoUpdateWorker.UNIQUE_WORK_NAME)
    }

    override fun scheduleTraktScrobble(request: TraktScrobbleSchedule): Boolean {
        if (
            request.profileId.isBlank() ||
            request.providerAccountId.isBlank() ||
            request.mediaId.isBlank() ||
            request.action !in SCROBBLE_ACTIONS
        ) return false

        enqueueProviderWork<TraktScrobbleWorker>(
            uniqueName = "trakt_scrobble_${request.profileId}_${request.providerAccountId.hashCode()}_${request.mediaType}_${request.mediaId}",
            expedited = true,
            inputData = workDataOf(
                TraktScrobbleWorker.KEY_PROFILE_ID to request.profileId,
                TraktScrobbleWorker.KEY_PROVIDER_ACCOUNT_ID to request.providerAccountId,
                TraktScrobbleWorker.KEY_MEDIA_TYPE to request.mediaType,
                TraktScrobbleWorker.KEY_MEDIA_ID to request.mediaId,
                TraktScrobbleWorker.KEY_PROGRESS to request.progress.coerceIn(0f, 100f),
                TraktScrobbleWorker.KEY_ACTION to request.action,
            ),
        )
        return true
    }

    override fun scheduleSimklScrobble(request: SimklScrobbleSchedule): Boolean {
        if (
            request.profileId.isBlank() ||
            request.providerAccountId.isBlank() ||
            request.mediaId.isBlank() ||
            request.durationMs <= 0L ||
            request.action !in SCROBBLE_ACTIONS
        ) return false

        enqueueProviderWork<SimklScrobbleWorker>(
            uniqueName = "simkl_scrobble_${request.profileId}_${request.providerAccountId.hashCode()}_${request.mediaType}_${request.mediaId}",
            expedited = true,
            inputData = workDataOf(
                SimklScrobbleWorker.KEY_PROFILE_ID to request.profileId,
                SimklScrobbleWorker.KEY_PROVIDER_ACCOUNT_ID to request.providerAccountId,
                SimklScrobbleWorker.KEY_MEDIA_TYPE to request.mediaType,
                SimklScrobbleWorker.KEY_MEDIA_ID to request.mediaId,
                SimklScrobbleWorker.KEY_ACTION to request.action,
                SimklScrobbleWorker.KEY_POSITION_MS to request.positionMs,
                SimklScrobbleWorker.KEY_DURATION_MS to request.durationMs,
            ),
        )
        return true
    }

    override fun scheduleStremioProgress(request: StremioPlaybackProgressSchedule): Boolean {
        if (request.profileId.isBlank() || request.providerAccountId.isBlank() || request.meta.id.isBlank() || request.duration <= 0L) return false

        enqueueProviderWork<StremioPlaybackProgressPushWorker>(
            uniqueName = "stremio_playback_progress_${request.profileId}_${request.providerAccountId.hashCode()}_${request.meta.id}",
            inputData = workDataOf(
                StremioPlaybackProgressPushWorker.KEY_PROFILE_ID to request.profileId,
                StremioPlaybackProgressPushWorker.KEY_PROVIDER_ACCOUNT_ID to request.providerAccountId,
                StremioPlaybackProgressPushWorker.KEY_META_JSON to gson.toJson(request.meta),
                StremioPlaybackProgressPushWorker.KEY_TIME_OFFSET to request.timeOffset,
                StremioPlaybackProgressPushWorker.KEY_DURATION to request.duration,
            ),
        )
        return true
    }

    override fun scheduleNuvioProgress(request: NuvioPlaybackProgressSchedule): Boolean {
        if (request.profileId.isBlank() || request.providerAccountId.isBlank() || request.contentId.isBlank() || request.duration <= 0L) return false

        enqueueProviderWork<NuvioPlaybackProgressPushWorker>(
            uniqueName = "nuvio_playback_progress_${request.profileId}_${request.providerAccountId.hashCode()}_${request.contentId}",
            inputData = workDataOf(
                NuvioPlaybackProgressPushWorker.KEY_PROFILE_ID to request.profileId,
                NuvioPlaybackProgressPushWorker.KEY_PROVIDER_ACCOUNT_ID to request.providerAccountId,
                NuvioPlaybackProgressPushWorker.KEY_CONTENT_ID to request.contentId,
                NuvioPlaybackProgressPushWorker.KEY_CONTENT_TYPE to request.contentType,
                NuvioPlaybackProgressPushWorker.KEY_VIDEO_ID to request.videoId,
                NuvioPlaybackProgressPushWorker.KEY_POSITION to request.position,
                NuvioPlaybackProgressPushWorker.KEY_DURATION to request.duration,
            ),
        )
        return true
    }

    private inline fun <reified W : androidx.work.CoroutineWorker> enqueueProviderWork(
        uniqueName: String,
        inputData: androidx.work.Data,
        expedited: Boolean = false,
    ) {
        val request = OneTimeWorkRequestBuilder<W>()
            .setInputData(inputData)
            .setConstraints(networkConstraints)
            .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, ProviderSyncPushWorker.BACKOFF_DELAY_SECONDS, TimeUnit.SECONDS)
            .apply {
                if (expedited) {
                    setExpedited(OutOfQuotaPolicy.RUN_AS_NON_EXPEDITED_WORK_REQUEST)
                }
            }
            .build()
        workManager.enqueueUniqueWork(uniqueName, ExistingWorkPolicy.REPLACE, request)
    }

    private companion object {
        val SCROBBLE_ACTIONS = setOf("start", "pause", "stop")
    }
}
