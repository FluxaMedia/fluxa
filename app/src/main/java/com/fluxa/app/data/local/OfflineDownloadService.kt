package com.fluxa.app.data.local

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import com.fluxa.app.R
import com.fluxa.app.ui.MainActivity
import com.fluxa.app.ui.catalog.AppStrings
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import okhttp3.Call
import okhttp3.OkHttpClient
import okhttp3.Request
import java.io.File
import java.util.concurrent.ConcurrentHashMap
import kotlin.math.max

class OfflineDownloadService : Service() {
    private val gson = Gson()
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val client = OfflineDownloadManager.sharedHttpClient
    private val activeCalls = ConcurrentHashMap<String, Call>()
    private lateinit var notificationManager: NotificationManager

    override fun onCreate() {
        super.onCreate()
        notificationManager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        ensureChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START -> {
                val itemId = intent.getStringExtra(EXTRA_ITEM_ID) ?: return START_NOT_STICKY
                val url = intent.getStringExtra(EXTRA_URL) ?: return START_NOT_STICKY
                val headersJson = intent.getStringExtra(EXTRA_HEADERS_JSON).orEmpty()
                val headers = runCatching {
                    gson.fromJson<Map<String, String>>(headersJson, object : TypeToken<Map<String, String>>() {}.type)
                }.getOrNull().orEmpty()
                startForeground(notificationId(itemId), buildNotification(loadItem(itemId), 0, 0L, -1L, null, true))
                scope.launch { download(itemId, url, headers) }
            }
            ACTION_CANCEL -> {
                val itemId = intent.getStringExtra(EXTRA_ITEM_ID) ?: return START_NOT_STICKY
                activeCalls.remove(itemId)?.cancel()
                loadItem(itemId)?.let { item ->
                    runCatching { File(item.videoPath).delete() }
                    saveItem(item.copy(status = "failed", error = "cancelled", speedBytesPerSecond = 0L, etaSeconds = -1L))
                }
                notificationManager.cancel(notificationId(itemId))
                stopSelf(startId)
            }
        }
        return START_NOT_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        activeCalls.values.forEach { it.cancel() }
        scope.cancel()
        super.onDestroy()
    }

    private fun download(itemId: String, url: String, headers: Map<String, String>) {
        val item = loadItem(itemId) ?: return
        val artwork = loadNotificationBitmap(item)
        val requestBuilder = Request.Builder().url(url)
        headers.forEach { (key, value) -> requestBuilder.addHeader(key, value) }
        val call = client.newCall(requestBuilder.build())
        activeCalls[itemId] = call
        try {
            call.execute().use { response ->
                if (!response.isSuccessful) error("HTTP ${response.code}")
                val body = response.body
                val total = body.contentLength().coerceAtLeast(0L)
                File(item.videoPath).parentFile?.mkdirs()
                body.byteStream().use { input ->
                    File(item.videoPath).outputStream().use { output ->
                        val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
                        var downloaded = 0L
                        var sinceUpdate = 0L
                        var lastUpdateAt = System.currentTimeMillis()
                        saveItem(item.copy(status = "downloading", totalBytes = total))
                        while (true) {
                            val read = input.read(buffer)
                            if (read == -1) break
                            output.write(buffer, 0, read)
                            downloaded += read
                            sinceUpdate += read
                            val now = System.currentTimeMillis()
                            if (now - lastUpdateAt >= 1000L) {
                                val speed = (sinceUpdate * 1000L) / max(1L, now - lastUpdateAt)
                                val eta = if (speed > 0L && total > downloaded) (total - downloaded) / speed else -1L
                                val progress = if (total > 0L) ((downloaded * 100L) / total).toInt().coerceIn(0, 100) else 0
                                val updated = item.copy(
                                    status = "downloading",
                                    progress = progress,
                                    downloadedBytes = downloaded,
                                    totalBytes = total,
                                    speedBytesPerSecond = speed,
                                    etaSeconds = eta
                                )
                                saveItem(updated)
                                notificationManager.notify(notificationId(itemId), buildNotification(updated, progress, speed, eta, artwork, true))
                                sinceUpdate = 0L
                                lastUpdateAt = now
                            }
                        }
                        output.flush()
                        val complete = item.copy(
                            status = "downloaded",
                            progress = 100,
                            downloadedBytes = downloaded,
                            totalBytes = if (total > 0L) total else downloaded,
                            speedBytesPerSecond = 0L,
                            etaSeconds = -1L,
                            error = null
                        )
                        saveItem(complete)
                        notificationManager.notify(notificationId(itemId), buildNotification(complete, 100, 0L, -1L, artwork, false))
                    }
                }
            }
        } catch (t: Throwable) {
            if (activeCalls[itemId]?.isCanceled() != true) {
                saveItem(item.copy(status = "failed", error = t.message, speedBytesPerSecond = 0L, etaSeconds = -1L))
                notificationManager.notify(notificationId(itemId), buildNotification(item.copy(status = "failed"), item.progress, 0L, -1L, artwork, false))
            }
        } finally {
            activeCalls.remove(itemId)
            stopForeground(STOP_FOREGROUND_DETACH)
            stopSelf()
        }
    }

    private fun buildNotification(
        item: OfflineDownloadItem?,
        progress: Int,
        speed: Long,
        eta: Long,
        poster: Bitmap?,
        ongoing: Boolean
    ) = NotificationCompat.Builder(this, CHANNEL_ID)
        .setSmallIcon(R.drawable.ic_launcher_foreground)
        .setLargeIcon(poster)
        .setContentTitle(item?.let { listOfNotNull(it.title, it.episodeTitle).joinToString(" - ") } ?: getString(R.string.app_name))
        .setContentText(notificationText(item, speed, eta))
        .setStyle(poster?.let { NotificationCompat.BigPictureStyle().bigPicture(it).bigLargeIcon(null as Bitmap?) })
        .setContentIntent(openAppIntent())
        .setOngoing(ongoing)
        .setOnlyAlertOnce(true)
        .setSilent(false)
        .setProgress(100, progress.coerceIn(0, 100), item?.totalBytes?.let { it <= 0L } ?: true)
        .apply {
            if (ongoing && item != null) {
                addAction(0, AppStrings.t(item.language, "downloads.cancel"), cancelIntent(item.id))
            }
        }
        .build()

    private fun notificationText(item: OfflineDownloadItem?, speed: Long, eta: Long): String {
        val lang = item?.language
        if (item?.status == "downloaded") return AppStrings.t(lang, "downloads.notification_complete")
        if (item?.status == "failed") return AppStrings.t(lang, "downloads.notification_failed")
        val sizeText = when {
            item == null -> AppStrings.t(lang, "downloads.notification_downloading")
            item.totalBytes > 0L -> AppStrings.format(
                lang,
                "downloads.notification_downloading_size",
                item.downloadedBytes.formatBytes(),
                item.totalBytes.formatBytes()
            )
            item.downloadedBytes > 0L -> AppStrings.format(
                lang,
                "downloads.notification_downloading_size_unknown",
                item.downloadedBytes.formatBytes()
            )
            else -> AppStrings.t(lang, "downloads.notification_downloading")
        }
        val speedText = if (speed > 0L) "${speed.formatBytes()}/s" else null
        val etaText = if (eta >= 0L) AppStrings.format(lang, "format.remaining_minutes", eta / 60L + 1L) else null
        return listOfNotNull(sizeText, speedText, etaText).joinToString(" • ")
    }

    private fun openAppIntent(): PendingIntent {
        val intent = Intent(this, MainActivity::class.java)
        return PendingIntent.getActivity(this, 0, intent, PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT)
    }

    private fun cancelIntent(itemId: String): PendingIntent {
        val intent = Intent(this, OfflineDownloadService::class.java)
            .setAction(ACTION_CANCEL)
            .putExtra(EXTRA_ITEM_ID, itemId)
        return PendingIntent.getService(this, itemId.hashCode(), intent, PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT)
    }

    private fun loadNotificationBitmap(item: OfflineDownloadItem): Bitmap? {
        val candidates = listOf(
            item.localBackgroundPath,
            item.background,
            item.localPosterPath,
            item.poster
        )
        return candidates.firstNotNullOfOrNull { loadBitmap(it) }
    }

    private fun loadBitmap(url: String?): Bitmap? {
        if (url.isNullOrBlank()) return null
        return runCatching {
            val localFile = File(url)
            if (localFile.exists()) {
                return@runCatching BitmapFactory.decodeFile(
                    localFile.absolutePath,
                    BitmapFactory.Options().apply {
                        inPreferredConfig = Bitmap.Config.RGB_565
                        inSampleSize = 2
                    }
                )
            }
            client.newCall(Request.Builder().url(url).build()).execute().use { response ->
                if (!response.isSuccessful) return@use null
                BitmapFactory.decodeStream(
                    response.body.byteStream(),
                    null,
                    BitmapFactory.Options().apply {
                        inPreferredConfig = Bitmap.Config.RGB_565
                        inSampleSize = 2
                    }
                )
            }
        }.getOrNull()
    }

    private fun loadItem(itemId: String): OfflineDownloadItem? {
        return loadItems().firstOrNull { it.id == itemId }
    }

    private fun saveItem(item: OfflineDownloadItem) {
        val updated = (loadItems().filterNot { it.id == item.id } + item).sortedByDescending { it.createdAt }
        prefs().edit().putString("items", gson.toJson(updated)).apply()
    }

    private fun loadItems(): List<OfflineDownloadItem> {
        val json = prefs().getString("items", "[]").orEmpty()
        return runCatching {
            gson.fromJson<List<OfflineDownloadItem>>(json, object : TypeToken<List<OfflineDownloadItem>>() {}.type)
        }.getOrNull().orEmpty()
    }

    private fun prefs() = getSharedPreferences("fluxa_offline_downloads", Context.MODE_PRIVATE)

    private fun ensureChannel() {
        val channel = NotificationChannel(CHANNEL_ID, AppStrings.t(null, "downloads.notification_channel"), NotificationManager.IMPORTANCE_DEFAULT)
        channel.description = AppStrings.t(null, "downloads.notification_channel_desc")
        notificationManager.createNotificationChannel(channel)
    }

    private fun notificationId(itemId: String): Int = itemId.hashCode()

    companion object {
        const val ACTION_START = "com.fluxa.app.action.DOWNLOAD_START"
        const val ACTION_CANCEL = "com.fluxa.app.action.DOWNLOAD_CANCEL"
        const val EXTRA_ITEM_ID = "item_id"
        const val EXTRA_URL = "url"
        const val EXTRA_HEADERS_JSON = "headers_json"
        private const val CHANNEL_ID = "fluxa_downloads_v2"
    }
}

private fun Long.formatBytes(): String {
    val value = this.toDouble()
    fun oneDecimal(amount: Double): String = String.format(java.util.Locale.US, "%.1f", amount).removeSuffix(".0")
    return when {
        this >= 1024L * 1024L * 1024L -> "${oneDecimal(value / (1024.0 * 1024.0 * 1024.0))} GB"
        this >= 1024L * 1024L -> "${oneDecimal(value / (1024.0 * 1024.0))} MB"
        this >= 1024L -> String.format(java.util.Locale.US, "%.0f KB", value / 1024.0)
        else -> "$this B"
    }
}
