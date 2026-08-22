package com.fluxa.app.ui.catalog

import android.content.Context
import android.util.Log
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import java.time.LocalDate
import androidx.hilt.work.HiltWorker
import dagger.assisted.Assisted
import dagger.assisted.AssistedInject
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.library.ProviderContinueWatchingRepository
import com.fluxa.app.data.repository.library.ThirdPartyProviderRepository

@HiltWorker
class EpisodeReleaseWorker @AssistedInject constructor(
    @Assisted appContext: Context,
    @Assisted params: WorkerParameters,
    private val profileManager: ProfileManager,
    private val repository: StremioRepository,
    private val watchlistManager: WatchlistManager,
    private val providerContinueWatchingRepository: ProviderContinueWatchingRepository,
    private val thirdPartyProviderRepository: ThirdPartyProviderRepository
) : CoroutineWorker(appContext, params) {
    override suspend fun doWork(): Result {
        return try {
            val profiles = profileManager.getProfiles()
                .filter { it.safeNotificationsEnabled && it.safeAlertNewEpisodes }
            if (profiles.isEmpty()) return Result.success()

            val today = LocalDate.now()
            val loader = EpisodeCalendarLoader(
                repository,
                watchlistManager,
                providerContinueWatchingRepository,
                thirdPartyProviderRepository
            )
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
        internal const val UNIQUE_PERIODIC_WORK_NAME = "episode_release_notifications"
        internal const val UNIQUE_IMMEDIATE_WORK_NAME = "episode_release_notifications_now"
    }

}
