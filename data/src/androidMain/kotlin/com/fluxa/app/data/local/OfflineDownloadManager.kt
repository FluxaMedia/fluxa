package com.fluxa.app.data.local

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Environment
import androidx.core.content.ContextCompat
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.SubtitleAttributes
import com.fluxa.app.data.remote.SubtitleData
import com.fluxa.app.data.remote.Video
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import java.io.File
import java.util.UUID

val OfflineDownloadItem.isPlayable: Boolean
    get() = status == "downloaded" && File(videoPath).exists()

class OfflineDownloadManager private constructor(private val context: Context) {
    private val appContext = context.applicationContext
    private val gson = Gson()
    private val prefs = appContext.getSharedPreferences("fluxa_offline_downloads", Context.MODE_PRIVATE)
    private val httpClient = sharedHttpClient
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val _items = MutableStateFlow<List<OfflineDownloadItem>>(emptyList())
    val items: StateFlow<List<OfflineDownloadItem>> = _items

    init {
        scope.launch { _items.value = loadItems() }
    }

    suspend fun enqueue(
        profileId: String?,
        meta: Meta,
        video: Video?,
        videoId: String?,
        stream: Stream,
        subtitle: OfflineSubtitleOption?,
        profileLanguage: String? = null
    ): Result<OfflineDownloadItem> = withContext(Dispatchers.IO) {
        val id = UUID.randomUUID().toString()
        val plan = FluxaCoreNative.offlineDownloadPlan(
            meta = meta,
            video = video,
            videoId = videoId,
            stream = stream,
            subtitleUrl = subtitle?.url,
            downloadId = id
        )
        if (!plan.supported) {
            return@withContext Result.failure(IllegalArgumentException(plan.reason ?: "unsupported_source"))
        }
        val playbackUrl = plan.playbackUrl
        val folder = File(appContext.getExternalFilesDir(Environment.DIRECTORY_MOVIES), "offline").apply { mkdirs() }
        val videoFile = File(folder, plan.videoFileName)
        val subtitleFile = plan.subtitleFileName?.let { File(folder, it) }
        val localPoster = downloadArtwork(meta.poster, File(folder, plan.posterFileName))
        val localBackground = downloadArtwork(video?.thumbnail ?: meta.background, File(folder, plan.backgroundFileName))
        val localLogo = downloadArtwork(meta.logo, File(folder, plan.logoFileName))

        val headers = FluxaCoreNative.streamRequestHeaders(stream.getHeaders())

        val downloadedSubtitle = subtitle?.let { option ->
            subtitleFile?.let { file ->
                runCatching { downloadSubtitle(option.url, file) }.getOrNull()
            }
        }
        val item = OfflineDownloadItem(
            id = id,
            profileId = profileId,
            language = profileLanguage,
            metaId = meta.id,
            metaType = meta.type,
            title = meta.name,
            episodeTitle = video?.name,
            videoId = plan.videoId,
            poster = meta.poster,
            background = video?.thumbnail ?: meta.background,
            logo = meta.logo,
            localPosterPath = localPoster?.absolutePath,
            localBackgroundPath = localBackground?.absolutePath,
            localLogoPath = localLogo?.absolutePath,
            streamTitle = plan.streamTitle,
            videoPath = videoFile.absolutePath,
            subtitlePath = downloadedSubtitle?.absolutePath,
            subtitleLabel = subtitle?.label,
            subtitleLanguage = subtitle?.language,
            downloadId = 0L,
            createdAt = System.currentTimeMillis()
        )
        upsert(item)
        val intent = Intent(appContext, OfflineDownloadService::class.java)
            .setAction(OfflineDownloadService.ACTION_START)
            .putExtra(OfflineDownloadService.EXTRA_ITEM_ID, item.id)
            .putExtra(OfflineDownloadService.EXTRA_URL, playbackUrl)
            .putExtra(OfflineDownloadService.EXTRA_HEADERS_JSON, gson.toJson(headers))
        ContextCompat.startForegroundService(appContext, intent)
        Result.success(item)
    }

    fun cancel(id: String) {
        val item = _items.value.firstOrNull { it.id == id } ?: return
        appContext.startService(
            Intent(appContext, OfflineDownloadService::class.java)
                .setAction(OfflineDownloadService.ACTION_CANCEL)
                .putExtra(OfflineDownloadService.EXTRA_ITEM_ID, id)
        )
        _items.value = _items.value.filterNot { it.id == id }
        scope.launch {
            runCatching { File(item.videoPath).delete() }
            item.subtitlePath?.let { path -> runCatching { File(path).delete() } }
            item.localPosterPath?.let { path -> runCatching { File(path).delete() } }
            item.localBackgroundPath?.let { path -> runCatching { File(path).delete() } }
            item.localLogoPath?.let { path -> runCatching { File(path).delete() } }
            saveItems(_items.value)
        }
    }

    fun refresh() {
        scope.launch { _items.value = loadItems() }
    }

    fun asPlayableMeta(item: OfflineDownloadItem): Meta = Meta(
        id = "offline:${item.id}",
        name = item.title,
        type = item.metaType,
        poster = item.localPosterPath?.let { Uri.fromFile(File(it)).toString() } ?: item.poster,
        background = item.localBackgroundPath?.let { Uri.fromFile(File(it)).toString() } ?: item.background,
        logo = item.localLogoPath?.let { Uri.fromFile(File(it)).toString() } ?: item.logo,
        description = null,
        lastVideoId = item.videoId,
        lastEpisodeName = item.continueWatchingEpisodeName()
    )

    fun asPlayableStream(item: OfflineDownloadItem): Stream = Stream(
        name = item.streamTitle ?: item.title,
        title = item.streamTitle,
        url = Uri.fromFile(File(item.videoPath)).toString(),
        subtitles = item.subtitlePath?.let { path ->
            listOf(
                SubtitleData(
                    url = Uri.fromFile(File(path)).toString(),
                    lang = item.subtitleLanguage,
                    attributes = SubtitleAttributes(
                        url = Uri.fromFile(File(path)).toString(),
                        languages = listOfNotNull(item.subtitleLanguage)
                    )
                )
            )
        },
        addonName = "Offline"
    )

    private fun upsert(item: OfflineDownloadItem) {
        _items.value = (_items.value.filterNot { it.id == item.id } + item).sortedByDescending { it.createdAt }
        saveItems(_items.value)
    }

    private fun loadItems(): List<OfflineDownloadItem> {
        val json = prefs.getString("items", "[]").orEmpty()
        return runCatching {
            gson.fromJson<List<OfflineDownloadItem>>(json, object : TypeToken<List<OfflineDownloadItem>>() {}.type)
        }.getOrNull().orEmpty().sortedByDescending { it.createdAt }
    }

    private fun saveItems(items: List<OfflineDownloadItem>) {
        prefs.edit().putString("items", gson.toJson(items)).apply()
    }

    private fun downloadSubtitle(url: String, target: File): File {
        val request = Request.Builder()
            .url(url)
            .build()
        httpClient.newCall(request).execute().use { response ->
            if (!response.isSuccessful) error("Subtitle download failed")
            target.outputStream().use { output ->
                response.body.byteStream().use { input -> input.copyTo(output) }
            }
        }
        return target
    }

    private fun downloadArtwork(url: String?, target: File): File? {
        if (url.isNullOrBlank()) return null
        return runCatching {
            val request = Request.Builder().url(url).build()
            httpClient.newCall(request).execute().use { response ->
                if (!response.isSuccessful) return@use null
                target.outputStream().use { output ->
                    response.body.byteStream().use { input -> input.copyTo(output) }
                }
                target
            }
        }.getOrNull()
    }

    private fun OfflineDownloadItem.continueWatchingEpisodeName(): String? {
        val title = episodeTitle?.trim()?.takeIf { it.isNotBlank() }
        val parts = videoId?.split(":").orEmpty()
        val season = parts.getOrNull(parts.size - 2)?.toIntOrNull()
        val episode = parts.getOrNull(parts.size - 1)?.toIntOrNull()
        val code = if (season != null && episode != null) "S$season:E$episode" else null
        return listOfNotNull(code, title).joinToString(" ").takeIf { it.isNotBlank() }
    }

    companion object {
        internal val sharedHttpClient: OkHttpClient by lazy { OkHttpClient() }

        @Volatile private var INSTANCE: OfflineDownloadManager? = null
        fun getInstance(context: Context): OfflineDownloadManager =
            INSTANCE ?: synchronized(this) {
                INSTANCE ?: OfflineDownloadManager(context).also { INSTANCE = it }
            }
    }
}
