package com.fluxa.app.ui.catalog

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile

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
    }
}
