package com.fluxa.app.player

import android.content.Context
import android.util.Log
import com.fluxa.app.common.Constants
import com.google.gson.Gson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import java.util.concurrent.TimeUnit

data class TorrentStreamStatus(
    val bufferProgress: Int = 0,
    val detailedStatus: String = "",
    val downloadSpeed: Double = 0.0,
    val activePeers: Int = 0,
    val totalPeers: Int = 0
)

sealed class TorrentStreamResult {
    data class Success(val url: String) : TorrentStreamResult()
    data class Error(val message: String) : TorrentStreamResult()
}

class TorrentStreamManager private constructor() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val client = OkHttpClient.Builder()
        .connectTimeout(5, TimeUnit.SECONDS)
        .readTimeout(5, TimeUnit.SECONDS)
        .build()
    private val gson = Gson()
    private val api = TorrServerApi.create()
    private var statusJob: Job? = null
    @Volatile private var appliedSettings: TorrSettings? = null
    @Volatile private var pendingSettings = defaultSettings()

    private val _status = MutableStateFlow(TorrentStreamStatus())
    val status: StateFlow<TorrentStreamStatus> = _status.asStateFlow()

    // Called once from Application.onCreate() — starts engine in app process, no service, no notification.
    fun startEngineEarly(context: Context) {
        scope.launch {
            runCatching {
                TorrServerEngine(context.applicationContext).start()
                Log.d(TAG, "Torrent engine started early (in-process, no notification)")
            }.onFailure { Log.w(TAG, "Early engine start failed", it) }
        }
    }

    fun configurePreferences(speedPreset: String?) {
        val newSettings = TorrSettings(preloadSize = speedPreset.toPreloadSizeMb())
        if (newSettings == pendingSettings) return
        pendingSettings = newSettings
        scope.launch {
            applySettingsIfChanged()
        }
    }

    fun preWarm(link: String, title: String, fileIdx: Int? = null) {
        scope.launch {
            runCatching {
                val plan = TorrentCorePolicy.plan(
                    link = link,
                    title = title,
                    requestedFileIdx = fileIdx,
                    preferredFilename = null,
                    sources = emptyList(),
                    fileStats = emptyList()
                )
                api.addTorrent(
                    TorrRequest(
                        action = "add",
                        link = plan.normalizedLink,
                        title = title,
                        saveToDb = false,
                        fileId = fileIdx ?: plan.selectedFileIdx
                    )
                )
                Log.d(TAG, "preWarm registered: ${plan.normalizedLink} fileIdx=${fileIdx ?: plan.selectedFileIdx}")
            }.onFailure { Log.w(TAG, "Torrent preWarm failed", it) }
        }
    }

    fun startStream(
        link: String,
        videoId: String,
        playbackTitle: String,
        fileIdx: Int?,
        preferredFilename: String?,
        sources: List<String>?,
        fileSizeBytes: Long = 0L,
        durationMs: Long = 0L,
        callback: (TorrentStreamResult) -> Unit
    ) {
        scope.launch {
            try {
                applySettingsIfChanged()
                val smartPreload = estimatePreloadMb(fileSizeBytes, durationMs)
                if (smartPreload != appliedSettings?.preloadSize) {
                    runCatching { api.updateSettings(TorrSettings(preloadSize = smartPreload)) }
                        .onSuccess { appliedSettings = TorrSettings(preloadSize = smartPreload) }
                }
                val plan = TorrentCorePolicy.plan(
                    link = link,
                    title = videoId,
                    requestedFileIdx = fileIdx,
                    preferredFilename = preferredFilename,
                    sources = sources.orEmpty(),
                    fileStats = emptyList()
                )
                // Hand the player the URL immediately — stream_fname will
                // ensure_torrent on its own. Doing /torrents add here in a
                // blocking way costs 5–30s while rqbit fetches metadata,
                // which is exactly the delay we're trying to eliminate.
                // Fire-and-forget the add so the focused file is registered
                // server-side as soon as metadata arrives.
                scope.launch {
                    runCatching {
                        api.addTorrent(
                            TorrRequest(
                                action = "add",
                                link = plan.normalizedLink,
                                title = videoId,
                                saveToDb = false,
                                fileId = plan.selectedFileIdx ?: fileIdx
                            )
                        )
                    }.onFailure { Log.w(TAG, "background addTorrent failed", it) }
                }
                startStatusPolling(plan.normalizedLink, videoId)
                callback(TorrentStreamResult.Success(plan.streamUrl))
            } catch (e: Exception) {
                Log.e(TAG, "Torrent stream failed", e)
                callback(TorrentStreamResult.Error(e.message ?: "Torrent stream failed"))
            }
        }
    }

    fun stop() {
        statusJob?.cancel()
        statusJob = null
        _status.value = TorrentStreamStatus()
    }

    fun shutdown() {
        stop()
        scope.launch {
            runCatching { api.updateSettings(TorrSettings(preloadSize = 0L)) }
        }
    }

    private suspend fun applySettingsIfChanged() {
        val desired = pendingSettings
        if (desired == appliedSettings) return
        runCatching { api.updateSettings(desired) }
            .onSuccess { appliedSettings = desired }
            .onFailure { Log.w(TAG, "TorrServer settings update failed", it) }
    }

    private fun startStatusPolling(link: String, title: String) {
        statusJob?.cancel()
        statusJob = scope.launch {
            while (isActive) {
                updateStatus(link, title)
                delay(1000)
            }
        }
    }

    private suspend fun updateStatus(link: String, title: String) {
        val statusUrl = TorrentCorePolicy.plan(
            link = link,
            title = title,
            requestedFileIdx = null,
            preferredFilename = null,
            sources = emptyList(),
            fileStats = emptyList(),
            play = false,
            stat = true
        ).streamUrl
        val body = runCatching { requestUrl(statusUrl) }.getOrNull() ?: return
        val torrStatus = runCatching { gson.fromJson(body, TorrStatus::class.java) }.getOrNull() ?: return
        val statusInfo = TorrentCorePolicy.statusInfo(torrStatus)
        _status.value = TorrentStreamStatus(
            bufferProgress = statusInfo.bufferProgress,
            detailedStatus = torrStatus.statString.ifBlank { statusInfo.statusKey },
            downloadSpeed = torrStatus.downloadSpeed,
            activePeers = torrStatus.activePeers,
            totalPeers = torrStatus.totalPeers
        )
    }

    private suspend fun requestUrl(url: String): String = withContext(Dispatchers.IO) {
        client.newCall(Request.Builder().url(url).build()).execute().use { response ->
            if (!response.isSuccessful) error("TorrServer HTTP ${response.code}")
            response.body.string()
        }
    }

    // 1% of file size, clamped between 3 MB and 24 MB.
    // Falls back to the speed-preset default when file size is unknown.
    private fun estimatePreloadMb(fileSizeBytes: Long, durationMs: Long): Long {
        if (fileSizeBytes > 0L) {
            return (fileSizeBytes / 100L / (1024L * 1024L)).coerceIn(3L, 24L)
        }
        return pendingSettings.preloadSize
    }

    private fun String?.toPreloadSizeMb(): Long = when (this) {
        "fast" -> 8L; "ultra_fast" -> 16L; else -> 3L
    }

    companion object {
        private const val TAG = "TorrentStreamManager"

        @Volatile private var instance: TorrentStreamManager? = null

        fun getInstance(context: Context): TorrentStreamManager =
            instance ?: synchronized(this) {
                instance ?: TorrentStreamManager().also { instance = it }
            }

        // Overload for callers that don't have a context (engine already started)
        fun getInstance(): TorrentStreamManager =
            instance ?: error("TorrentStreamManager not initialized — call getInstance(context) first")

        private fun defaultSettings(): TorrSettings = TorrSettings(preloadSize = 3L)
    }
}
