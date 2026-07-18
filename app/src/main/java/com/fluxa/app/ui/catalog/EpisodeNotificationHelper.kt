package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.core.rust.FluxaCoreUniFfi
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
import com.google.gson.JsonArray
import com.google.gson.JsonObject
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.Request

object EpisodeNotificationHelper {
    private const val CHANNEL_ID = "new_episodes"
    private const val PREFS = "fluxa_episode_notifications"
    private const val NOTIFIED_KEYS = "notified_keys"

    suspend fun notifyReleasedEpisodes(context: Context, profile: UserProfile?, items: List<CalendarUpcomingItem>, todayIso: String) {
        val appContext = context.applicationContext
        val prefs = appContext.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        val alreadyNotified = prefs.getStringSet(NOTIFIED_KEYS, emptySet()).orEmpty()

        val content = fetchNotificationContent(items, todayIso, alreadyNotified, profile)
        val releasedItems = content.getAsJsonArray("items")
        if (releasedItems.isEmpty) return

        if (!canPostNotifications(appContext)) return
        ensureChannel(appContext)
        val launchIntent = appContext.packageManager.getLaunchIntentForPackage(appContext.packageName)
        val pendingIntent = launchIntent?.let {
            PendingIntent.getActivity(
                appContext,
                0,
                it,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )
        }

        releasedItems.forEach { element ->
            val row = element.asJsonObject
            val key = row.get("key").asString
            val original = items.firstOrNull {
                it.meta.id == row.get("metaId").asString &&
                    it.dateIso == row.get("dateIso").asString &&
                    it.seasonNumber == row.get("seasonNumber")?.takeIf { v -> !v.isJsonNull }?.asInt &&
                    it.episodeNumber == row.get("episodeNumber")?.takeIf { v -> !v.isJsonNull }?.asInt
            } ?: return@forEach
            val image = loadBitmap(original.artworkUrl())
            val notificationTitle = AppStrings.t(profile?.safeLanguage, row.get("titleKey").asString)
            val text = releaseNotificationText(original, profile?.safeLanguage)
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

        val newKeys = content.getAsJsonArray("keys").map { it.asString }
        prefs.edit().putStringSet(NOTIFIED_KEYS, alreadyNotified + newKeys).apply()
    }

    private fun fetchNotificationContent(
        items: List<CalendarUpcomingItem>,
        todayIso: String,
        alreadyNotified: Set<String>,
        profile: UserProfile?
    ): JsonObject {
        val requestItems = JsonArray().apply {
            items.forEach { item ->
                add(JsonObject().apply {
                    addProperty("dateIso", item.dateIso)
                    addProperty("metaId", item.meta.id)
                    addProperty("metaType", item.meta.type)
                    addProperty("title", item.title)
                    item.subtitle?.let { addProperty("subtitle", it) }
                    item.seasonNumber?.let { addProperty("seasonNumber", it) }
                    item.episodeNumber?.let { addProperty("episodeNumber", it) }
                    item.episodeTitle?.let { addProperty("episodeTitle", it) }
                    item.artworkUrl()?.let { addProperty("artworkUrl", it) }
                })
            }
        }
        val request = JsonObject().apply {
            add("items", requestItems)
            addProperty("todayIso", todayIso)
            add("alreadyNotifiedKeys", JsonArray().apply { alreadyNotified.forEach { add(it) } })
            profile?.id?.let { addProperty("profileId", it) }
            addProperty("notificationsEnabled", profile?.safeNotificationsEnabled != false)
            addProperty("alertNewEpisodes", profile?.safeAlertNewEpisodes != false)
        }
        return FluxaCoreUniFfi.coreInvokeValue("calendarNotificationContent", request.toString()).asJsonObject
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
