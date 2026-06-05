package com.fluxa.app.ui.catalog

import android.content.Context
import android.util.Log
import androidx.work.BackoffPolicy
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import java.time.LocalDate
import java.util.concurrent.TimeUnit
import androidx.hilt.work.HiltWorker
import dagger.assisted.Assisted
import dagger.assisted.AssistedInject
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.repository.StremioRepository

@HiltWorker
class EpisodeReleaseWorker @AssistedInject constructor(
    @Assisted appContext: Context,
    @Assisted params: WorkerParameters,
    private val profileManager: ProfileManager,
    private val repository: StremioRepository,
    private val watchlistManager: WatchlistManager
) : CoroutineWorker(appContext, params) {
    override suspend fun doWork(): Result {
        return try {
            val profiles = profileManager.getProfiles()
                .filter { it.safeNotificationsEnabled && it.safeAlertNewEpisodes }
            if (profiles.isEmpty()) return Result.success()

            val today = LocalDate.now()
            val loader = EpisodeCalendarLoader(repository, watchlistManager)
            profiles.forEach { profile ->
                val result = loader.loadMonth(profile, today.year, today.monthValue)
                CalendarWidgetProvider.updateCalendar(
                    context = applicationContext,
                    items = result.items,
                    language = profile.safeLanguage,
                    accentColorArgb = profile.safeAccentColorArgb
                )
                EpisodeNotificationHelper.notifyReleasedEpisodes(
                    context = applicationContext,
                    profile = profile,
                    items = result.items,
                    todayIso = today.toString()
                )
            }
            Result.success()
        } catch (e: Exception) {
            Log.w(TAG, "Episode release check failed", e)
            Result.retry()
        }
    }

    companion object {
        private const val TAG = "EpisodeReleaseWorker"
        private const val UNIQUE_PERIODIC_WORK_NAME = "episode_release_notifications"
        private const val UNIQUE_IMMEDIATE_WORK_NAME = "episode_release_notifications_now"

        fun enqueue(context: Context) {
            val constraints = Constraints.Builder()
                .setRequiredNetworkType(NetworkType.CONNECTED)
                .build()
            val periodic = PeriodicWorkRequestBuilder<EpisodeReleaseWorker>(6, TimeUnit.HOURS)
                .setConstraints(constraints)
                .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 30, TimeUnit.SECONDS)
                .build()
            val immediate = OneTimeWorkRequestBuilder<EpisodeReleaseWorker>()
                .setInitialDelay(20, TimeUnit.SECONDS)
                .setConstraints(constraints)
                .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 30, TimeUnit.SECONDS)
                .build()

            val workManager = WorkManager.getInstance(context.applicationContext)
            workManager.enqueueUniquePeriodicWork(
                UNIQUE_PERIODIC_WORK_NAME,
                ExistingPeriodicWorkPolicy.UPDATE,
                periodic
            )
            workManager.enqueueUniqueWork(
                UNIQUE_IMMEDIATE_WORK_NAME,
                ExistingWorkPolicy.KEEP,
                immediate
            )
        }
    }
}
