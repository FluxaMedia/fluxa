package com.fluxa.app.player

import com.fluxa.app.shared.feature.player.TorrentStreamStatus

import android.content.Context
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
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
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import java.util.concurrent.TimeUnit

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
    private val healthClient = OkHttpClient.Builder()
        .connectTimeout(2, TimeUnit.SECONDS)
        .readTimeout(2, TimeUnit.SECONDS)
        .writeTimeout(2, TimeUnit.SECONDS)
        .retryOnConnectionFailure(false)
        .build()
    private val gson = Gson()
    private val api = TorrentServerApi.create()
    private val engineLock = Mutex()
    private var statusJob: Job? = null
    @Volatile private var appliedSettings: TorrentSettings? = null
    @Volatile private var pendingSettings = defaultSettings()
    @Volatile private var appContext: Context? = null
    @Volatile private var engine: TorrentServerEngine? = null

    private val _status = MutableStateFlow(TorrentStreamStatus())
    val status: StateFlow<TorrentStreamStatus> = _status.asStateFlow()

    // Called once from Application.onCreate() — starts engine in app process, no service, no notification.
    fun startEngineEarly(context: Context) {
        attachContext(context)
        scope.launch {
            runCatching {
                ensureEngineReady()
                Log.d(TAG, "Torrent engine started early (in-process, no notification)")
            }.onFailure { Log.w(TAG, "Early engine start failed", it) }
        }
    }

    fun configurePreferences(speedPreset: String?) {
        val newSettings = TorrentSettings(preloadSize = speedPreset.toPreloadSizeMb())
        if (newSettings == pendingSettings) return
        pendingSettings = newSettings
        scope.launch {
            applySettingsIfChanged()
        }
    }

    fun preWarm(link: String, title: String, fileIdx: Int? = null) {
        scope.launch {
            runCatching {
                ensureEngineReady()
                val plan = TorrentCorePolicy.plan(
                    link = link,
                    title = title,
                    requestedFileIdx = fileIdx,
                    preferredFilename = null,
                    sources = emptyList(),
                    fileStats = emptyList()
                )
                api.addTorrent(
                    TorrentRequest(
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
        wifiOnly: Boolean = false,
        callback: (TorrentStreamResult) -> Unit
    ) {
        scope.launch {
            try {
                if (wifiOnly && !isOnUnmeteredNetwork()) {
                    callback(TorrentStreamResult.Error("Torrent streaming is restricted to Wi-Fi in Settings"))
                    return@launch
                }
                ensureEngineReady()
                applySettingsIfChanged()
                val smartPreload = estimatePreloadMb(fileSizeBytes, durationMs)
                if (smartPreload != appliedSettings?.preloadSize) {
                    runCatching { api.updateSettings(TorrentSettings(preloadSize = smartPreload)) }
                        .onSuccess { appliedSettings = TorrentSettings(preloadSize = smartPreload) }
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
                            TorrentRequest(
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
            runCatching { api.updateSettings(TorrentSettings(preloadSize = 0L)) }
        }
    }

    private suspend fun applySettingsIfChanged() {
        val desired = pendingSettings
        if (desired == appliedSettings) return
        ensureEngineReady()
        runCatching { api.updateSettings(desired) }
            .onSuccess { appliedSettings = desired }
            .onFailure { Log.w(TAG, "TorrentServer settings update failed", it) }
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
        val torrentStatus = runCatching { gson.fromJson(body, AndroidTorrentStatus::class.java).toShared() }.getOrNull() ?: return
        val statusInfo = TorrentCorePolicy.statusInfo(torrentStatus)
        _status.value = TorrentStreamStatus(
            bufferProgress = statusInfo.bufferProgress,
            detailedStatus = torrentStatus.statString.ifBlank { statusInfo.statusKey },
            downloadSpeed = torrentStatus.downloadSpeed,
            activePeers = torrentStatus.activePeers,
            totalPeers = torrentStatus.totalPeers
        )
    }

    private suspend fun requestUrl(url: String): String = withContext(Dispatchers.IO) {
        client.newCall(Request.Builder().url(url).build()).execute().use { response ->
            if (!response.isSuccessful) error("TorrentServer HTTP ${response.code}")
            response.body.string()
        }
    }

    private fun attachContext(context: Context) {
        appContext = context.applicationContext
    }

    private fun isOnUnmeteredNetwork(): Boolean {
        val context = appContext ?: return true
        val connectivityManager = context.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager ?: return true
        val network = connectivityManager.activeNetwork ?: return false
        val capabilities = connectivityManager.getNetworkCapabilities(network) ?: return false
        return capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) ||
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET)
    }

    private suspend fun ensureEngineReady() {
        engineLock.withLock {
            if (isEngineHealthy()) return@withLock
            val context = appContext ?: error("Torrent engine context is unavailable")
            val current = engine ?: TorrentServerEngine(context).also { engine = it }
            if (!current.isRunning()) {
                current.start()
                appliedSettings = null
            }
            if (isEngineHealthy()) return@withLock

            Log.w(TAG, "Torrent engine health check failed. Restarting engine.")
            current.stop()
            val restarted = TorrentServerEngine(context).also { engine = it }
            restarted.start()
            appliedSettings = null
            if (!isEngineHealthy()) {
                error("Torrent engine health check failed after restart")
            }
        }
    }

    private suspend fun isEngineHealthy(): Boolean = withContext(Dispatchers.IO) {
        requestHealth("${Constants.LocalServer.TORRENT_SERVER_BASE_URL}/health") ||
            requestHealth(Constants.LocalServer.TORRENT_SERVER_BASE_URL)
    }

    private fun requestHealth(url: String): Boolean {
        return runCatching {
            healthClient.newCall(
                Request.Builder()
                    .url(url)
                    .get()
                    .build()
            ).execute().use { response ->
                response.isSuccessful
            }
        }.getOrDefault(false)
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
            (instance ?: synchronized(this) {
                instance ?: TorrentStreamManager().also { instance = it }
            }).also { it.attachContext(context) }

        // Overload for callers that don't have a context (engine already started)
        fun getInstance(): TorrentStreamManager =
            instance ?: error("TorrentStreamManager not initialized — call getInstance(context) first")

        private fun defaultSettings(): TorrentSettings = TorrentSettings(preloadSize = 3L)
    }
}
