package com.fluxa.app.plugins

import android.annotation.SuppressLint
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat
import androidx.work.BackoffPolicy
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import java.util.concurrent.TimeUnit
import androidx.hilt.work.HiltWorker
import com.fluxa.app.R
import com.fluxa.app.common.AppStrings
import dagger.assisted.Assisted
import dagger.assisted.AssistedInject

@HiltWorker
class PluginAutoUpdateWorker @AssistedInject constructor(
    @Assisted appContext: Context,
    @Assisted params: WorkerParameters,
    private val pluginManager: PluginManager
) : CoroutineWorker(appContext, params) {

    override suspend fun doWork(): Result {
        return try {
            Log.d(TAG, "[AutoUpdate] Starting background plugin update check...")
            val report = pluginManager.checkAndAutoUpdatePlugins()
            if (report.updatedPlugins.isNotEmpty()) {
                Log.i(TAG, "[AutoUpdate] Plugins auto-updated: ${report.updatedPlugins}")
                postUpdateNotification(applicationContext, report.updatedPlugins)
            }
            if (report.failedPlugins.isNotEmpty()) {
                Log.w(TAG, "[AutoUpdate] Plugin updates failed: ${report.failedPlugins}")
                postFailureNotification(applicationContext, report.failedPlugins)
            }
            if (report.updatedPlugins.isEmpty() && report.failedPlugins.isEmpty()) {
                Log.d(TAG, "[AutoUpdate] All plugins are up to date")
            }
            Result.success()
        } catch (e: Exception) {
            Log.w(TAG, "[AutoUpdate] Plugin update check skipped after error", e)
            Result.retry()
        }
    }

    @SuppressLint("MissingPermission")
    private fun postUpdateNotification(context: Context, updatedPlugins: List<String>) {
        postPluginNotification(
            context = context,
            notificationId = PLUGIN_UPDATE_NOTIFICATION_ID,
            title = AppStrings.t("en", "notification.plugins_updated_title"),
            body = AppStrings.format(
                "en",
                "notification.plugins_updated_body",
                updatedPlugins.size.toString(),
                updatedPlugins.joinToString(", ")
            )
        )
    }

    @SuppressLint("MissingPermission")
    private fun postFailureNotification(context: Context, failedPlugins: List<String>) {
        postPluginNotification(
            context = context,
            notificationId = PLUGIN_UPDATE_FAILED_NOTIFICATION_ID,
            title = AppStrings.t("en", "notification.plugins_update_failed_title"),
            body = AppStrings.format(
                "en",
                "notification.plugins_update_failed_body",
                failedPlugins.size.toString(),
                failedPlugins.joinToString(", ")
            )
        )
    }

    @SuppressLint("MissingPermission")
    private fun postPluginNotification(
        context: Context,
        notificationId: Int,
        title: String,
        body: String
    ) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
            ContextCompat.checkSelfPermission(context, android.Manifest.permission.POST_NOTIFICATIONS)
            != PackageManager.PERMISSION_GRANTED
        ) return

        val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        if (manager.getNotificationChannel(PLUGIN_UPDATE_CHANNEL_ID) == null) {
            manager.createNotificationChannel(
                NotificationChannel(
                    PLUGIN_UPDATE_CHANNEL_ID,
                    AppStrings.t("en", "notification.plugin_updates_channel"),
                    NotificationManager.IMPORTANCE_DEFAULT
                )
            )
        }

        val notification = NotificationCompat.Builder(context, PLUGIN_UPDATE_CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_launcher_foreground)
            .setContentTitle(title)
            .setContentText(body)
            .setStyle(NotificationCompat.BigTextStyle().bigText(body))
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .setAutoCancel(true)
            .build()

        runCatching {
            NotificationManagerCompat.from(context).notify(notificationId, notification)
        }
    }

    companion object {
        private const val TAG = "PluginAutoUpdateWorker"
        private const val UNIQUE_WORK_NAME = "plugin_auto_update"
        private const val PLUGIN_UPDATE_CHANNEL_ID = "plugin_updates"
        private const val PLUGIN_UPDATE_NOTIFICATION_ID = 0x50_00
        private const val PLUGIN_UPDATE_FAILED_NOTIFICATION_ID = 0x50_01

        fun enqueue(context: Context) {
            val work = OneTimeWorkRequestBuilder<PluginAutoUpdateWorker>()
                .setInitialDelay(5, TimeUnit.SECONDS)
                .setConstraints(
                    Constraints.Builder()
                        .setRequiredNetworkType(NetworkType.CONNECTED)
                        .build()
                )
                .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 30, TimeUnit.SECONDS)
                .build()

            WorkManager.getInstance(context.applicationContext)
                .enqueueUniqueWork(UNIQUE_WORK_NAME, ExistingWorkPolicy.KEEP, work)
        }
    }
}
