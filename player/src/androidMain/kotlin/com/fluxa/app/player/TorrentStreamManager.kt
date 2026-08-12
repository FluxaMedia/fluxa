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
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.MediaType.Companion.toMediaType
import java.util.concurrent.TimeUnit

sealed class TorrentStreamResult {
    data class Success(val url: String) : TorrentStreamResult()
    data class Error(val message: String) : TorrentStreamResult()
}

class TorrentStreamManager private constructor() {
    private data class TelemetrySessionContext(
        val sessionId: String,
        val generation: Long,
        val link: String?
    )

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
    @Volatile private var activeTorrentLink: String? = null
    private var activeTelemetryGeneration: Long = 0L
    @Volatile private var activeTelemetryContext: TelemetrySessionContext? = null

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

    fun configurePreferences(speedPreset: String?, cacheLimitMb: Long? = null) {
        val newSettings = TorrentSettings(preloadSize = speedPreset.toPreloadSizeMb(), cacheLimitMb = cacheLimitMb)
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
                        fileId = fileIdx ?: plan.selectedFileIdx,
                        prewarm = true
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
                    fileStats = emptyList(),
                    durationMs = durationMs.takeIf { it > 0L }
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
                activateTorrentLink(plan.normalizedLink)
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
        engine?.setTorrentActive(false)
        _status.value = TorrentStreamStatus()
        activeTorrentLink?.let { link ->
            scope.launch {
                runCatching { api.removeTorrent(TorrentRequest(action = "deactivate", link = link)) }
            }
        }
        clearActiveTelemetry()
    }

    /**
     * Reports player-side milestones to the local streaming engine. This is
     * best-effort telemetry: a failed report must never affect playback.
     */
    @Synchronized
    fun beginPlaybackTelemetry(sessionId: String) {
        if (sessionId.isBlank()) return
        val nextGeneration = activeTelemetryGeneration + 1L
        activeTelemetryGeneration = nextGeneration
        activeTelemetryContext = TelemetrySessionContext(
            sessionId = sessionId,
            generation = nextGeneration,
            link = activeTorrentLink
        )
    }

    @Synchronized
    private fun activateTorrentLink(link: String) {
        activeTorrentLink = link
        engine?.setTorrentActive(true)
        activeTelemetryContext = activeTelemetryContext?.let { context ->
            if (context.link == null) context.copy(link = link) else context
        }
    }

    @Synchronized
    private fun clearActiveTelemetry() {
        activeTorrentLink = null
        activeTelemetryContext = null
    }

    fun recordPlaybackTelemetry(event: String, elapsedMs: Long? = null, sessionId: String) {
        val context = activeTelemetryContext ?: return
        if (context.sessionId != sessionId) return
        val link = context.link ?: return
        scope.launch {
            runCatching {
                val payload = gson.toJson(
                    mapOf(
                        "link" to link,
                        "sessionId" to sessionId,
                        "sessionGeneration" to context.generation,
                        "event" to event,
                        "elapsedMs" to elapsedMs
                    )
                )
                val request = Request.Builder()
                    .url("${Constants.LocalServer.TORRENT_SERVER_BASE_URL}/telemetry")
                    .post(payload.toRequestBody("application/json".toMediaType()))
                    .build()
                client.newCall(request).execute().use { response ->
                    if (!response.isSuccessful) {
                        Log.d(TAG, "Torrent telemetry rejected: HTTP ${response.code}")
                    }
                }
            }.onFailure { Log.d(TAG, "Torrent telemetry report failed", it) }
        }
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

    // Target ~20 seconds of media rather than a fixed fraction of a file.
    // A size-only target makes high-bitrate remuxes start with just a couple
    // of seconds buffered while over-buffering low-bitrate content.
    private fun estimatePreloadMb(fileSizeBytes: Long, durationMs: Long): Long {
        if (fileSizeBytes > 0L && durationMs > 0L) {
            val bytesPerSecond = fileSizeBytes.toDouble() / (durationMs / 1_000.0)
            return (bytesPerSecond * 20.0 / (1024.0 * 1024.0))
                .toLong()
                .coerceIn(4L, 256L)
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
