package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.annotation.SuppressLint
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat
import com.fluxa.app.R
import com.fluxa.app.data.remote.StremioService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.Request

object EpisodeNotificationHelper {
    private const val CHANNEL_ID = "new_episodes"
    private const val PREFS = "fluxa_episode_notifications"
    private const val NOTIFIED_KEYS = "notified_keys"

    suspend fun notifyReleasedEpisodes(context: Context, profile: UserProfile?, items: List<CalendarUpcomingItem>, todayIso: String) {
        if (profile?.safeNotificationsEnabled == false || profile?.safeAlertNewEpisodes == false) return
        val releasedToday = items.filter { it.dateIso == todayIso && it.meta.type == "series" }
        if (releasedToday.isEmpty()) return

        val appContext = context.applicationContext
        if (!canPostNotifications(appContext)) return
        ensureChannel(appContext)
        val prefs = appContext.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        val notified = prefs.getStringSet(NOTIFIED_KEYS, emptySet()).orEmpty().toMutableSet()
        val launchIntent = appContext.packageManager.getLaunchIntentForPackage(appContext.packageName)
        val pendingIntent = launchIntent?.let {
            PendingIntent.getActivity(
                appContext,
                0,
                it,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )
        }

        releasedToday.forEach { item ->
            val key = "${profile?.id.orEmpty()}:${item.dateIso}:${item.meta.id}:${item.subtitle.orEmpty()}"
            if (!notified.add(key)) return@forEach
            val image = loadBitmap(item.artworkUrl())
            val notificationTitle = AppStrings.t(
                profile?.safeLanguage,
                if (item.episodeNumber == 1) "notification.new_season_released" else "notification.new_episode_released"
            )
            val text = releaseNotificationText(item, profile?.safeLanguage)
            val notification = NotificationCompat.Builder(appContext, CHANNEL_ID)
                .setSmallIcon(R.drawable.ic_launcher_foreground)
                .setContentTitle(notificationTitle)
                .setContentText(text)
                .setStyle(
                    if (image != null) {
                        NotificationCompat.BigPictureStyle()
                            .bigPicture(image)
                            .bigLargeIcon(null as Bitmap?)
                            .setSummaryText(text)
                    } else {
                        NotificationCompat.BigTextStyle().bigText(text)
                    }
                )
                .apply { image?.let(::setLargeIcon) }
                .setPriority(NotificationCompat.PRIORITY_DEFAULT)
                .setAutoCancel(true)
                .apply { pendingIntent?.let(::setContentIntent) }
                .build()
            postNotification(appContext, key.hashCode(), notification)
        }
        prefs.edit().putStringSet(NOTIFIED_KEYS, notified).apply()
    }

    private fun releaseNotificationText(item: CalendarUpcomingItem, language: String?): String {
        val season = item.seasonNumber
        val episode = item.episodeNumber
        return if (season != null && episode != null) {
            AppStrings.format(language, "notification.release_now_episode", item.title, season, episode)
        } else {
            listOf(item.title, item.subtitle).filter { !it.isNullOrBlank() }.joinToString(" - ")
        }
    }

    private suspend fun loadBitmap(url: String?): Bitmap? = withContext(Dispatchers.IO) {
        val cleanUrl = url?.takeIf { it.isNotBlank() } ?: return@withContext null
        runCatching {
            val request = Request.Builder().url(cleanUrl).build()
            StremioService.sharedClient.newCall(request).execute().use { response ->
                if (!response.isSuccessful) return@use null
                response.body.byteStream().use(BitmapFactory::decodeStream)
            }
        }.getOrNull()
    }

    @SuppressLint("MissingPermission")
    private fun postNotification(context: Context, id: Int, notification: android.app.Notification) {
        if (!canPostNotifications(context)) return
        runCatching {
            NotificationManagerCompat.from(context).notify(id, notification)
        }
    }

    private fun canPostNotifications(context: Context): Boolean {
        return Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU ||
            ContextCompat.checkSelfPermission(
                context,
                android.Manifest.permission.POST_NOTIFICATIONS
            ) == PackageManager.PERMISSION_GRANTED
    }

    private fun ensureChannel(context: Context) {
        val manager = context.getSystemService(NotificationManager::class.java) ?: return
        if (manager.getNotificationChannel(CHANNEL_ID) != null) return
        manager.createNotificationChannel(
            NotificationChannel(
                CHANNEL_ID,
                AppStrings.t("en", "settings.notifications"),
                NotificationManager.IMPORTANCE_DEFAULT
            )
        )
    }
}
