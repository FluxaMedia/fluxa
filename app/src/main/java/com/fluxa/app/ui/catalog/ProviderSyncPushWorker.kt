package com.fluxa.app.ui.catalog

import android.content.Context
import androidx.work.BackoffPolicy
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.Data
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.OutOfQuotaPolicy
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import java.util.concurrent.TimeUnit

abstract class ProviderSyncPushWorker(
    appContext: Context,
    params: WorkerParameters,
    protected val profileManager: ProfileManager
) : CoroutineWorker(appContext, params) {

    protected abstract val providerName: String

    protected suspend fun requireProfile(profileId: String): UserProfile? =
        profileManager.getProfiles().firstOrNull { it.id == profileId }

    protected fun onSyncSuccess(profileId: String) {
        profileManager.clearExternalSyncFailure(profileId, providerName)
    }

    protected fun onSyncFailure(profileId: String) {
        profileManager.recordExternalSyncFailure(profileId, providerName)
    }

    companion object {
        const val BACKOFF_DELAY_SECONDS = 30L

        fun defaultConstraints(): Constraints =
            Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build()

        inline fun <reified W : CoroutineWorker> buildPushWork(
            inputData: Data,
            expedited: Boolean = false
        ) = OneTimeWorkRequestBuilder<W>()
            .setInputData(inputData)
            .setConstraints(defaultConstraints())
            .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, BACKOFF_DELAY_SECONDS, TimeUnit.SECONDS)
            .apply { if (expedited) setExpedited(OutOfQuotaPolicy.RUN_AS_NON_EXPEDITED_WORK_REQUEST) }
            .build()

        inline fun <reified W : CoroutineWorker> enqueueUnique(
            context: Context,
            uniqueName: String,
            inputData: Data,
            expedited: Boolean = false
        ) {
            WorkManager.getInstance(context.applicationContext)
                .enqueueUniqueWork(uniqueName, ExistingWorkPolicy.REPLACE, buildPushWork<W>(inputData, expedited))
        }
    }
}
