package com.fluxa.app.player


import android.content.Context
import androidx.media3.common.*
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.DefaultRenderersFactory
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.exoplayer.DefaultLoadControl
import androidx.media3.extractor.DefaultExtractorsFactory
import androidx.media3.extractor.mkv.MatroskaExtractor
import androidx.media3.datasource.DataSource
import androidx.media3.datasource.DataSpec
import androidx.media3.datasource.DefaultDataSource
import androidx.media3.datasource.TransferListener
import androidx.media3.datasource.cache.CacheDataSource
import androidx.media3.datasource.cache.CacheDataSink
import androidx.media3.datasource.cache.CacheWriter
import androidx.media3.datasource.cache.LeastRecentlyUsedCacheEvictor
import androidx.media3.datasource.cache.SimpleCache
import androidx.media3.database.StandaloneDatabaseProvider
import androidx.media3.datasource.okhttp.OkHttpDataSource
import android.net.Uri
import kotlinx.coroutines.cancel
import okhttp3.OkHttpClient
import okhttp3.ResponseBody
import okhttp3.ResponseBody.Companion.toResponseBody
import okio.Buffer
import okio.ForwardingSource
import okio.buffer

import androidx.media3.exoplayer.trackselection.DefaultTrackSelector
import androidx.media3.exoplayer.upstream.DefaultBandwidthMeter
import android.media.MediaCodecList
import java.util.Locale
import java.util.Collections
import java.util.WeakHashMap
import java.util.concurrent.TimeUnit

@UnstableApi
internal object MediaPlayerControllerFactory {
    internal class ExoRequestContext {
        @Volatile var streamHeaders: Map<String, String> = emptyMap()
        @Volatile var dolbyVisionFallbackMode: DolbyVisionFallbackMode = DolbyVisionFallbackMode.Off
        @Volatile var lastDolbyVisionDecision: String = "dv_fallback=not_checked"
        @Volatile var dvProxyPlanDebug: String = ""
        @Volatile var needsIptPqc2ToneMap: Boolean = false
        @Volatile var iptPqc2UseHdr: Boolean = false
        @Volatile var shouldConvertRpuP7: Boolean = false
        @Volatile var cachedCapabilities: DolbyVisionCapabilities? = null
        @Volatile var dvRpuMode: Int = 2
        @Volatile var dvZeroLevel5: Boolean = false
        @Volatile var dvHdr10PlusMode: String = "auto"
        @Volatile var disableDiskCache: Boolean = false
        @Volatile var videoEffectsActive: Boolean = false
    }

    private val requestContexts = Collections.synchronizedMap(WeakHashMap<ExoPlayer, ExoRequestContext>())
    private val libassRelays = Collections.synchronizedMap(WeakHashMap<ExoPlayer, LibassEventRelay>())
    private val subtitleCoordinators = Collections.synchronizedMap(WeakHashMap<ExoPlayer, com.fluxa.app.player.subtitle.SubtitleCoordinator>())
    private val subtitleClockBindings = Collections.synchronizedMap(WeakHashMap<ExoPlayer, ExoPlaybackClock>())
    private val subtitleScopes = Collections.synchronizedMap(WeakHashMap<ExoPlayer, kotlinx.coroutines.CoroutineScope>())
    fun getLibassRelay(player: ExoPlayer): LibassEventRelay? = libassRelays[player]
    fun getSubtitleCoordinator(player: ExoPlayer): com.fluxa.app.player.subtitle.SubtitleCoordinator? = subtitleCoordinators[player]
    internal fun requestContext(player: ExoPlayer): ExoRequestContext? = requestContexts[player]

    fun releaseExoPlayer(player: ExoPlayer) {
        libassRelays.remove(player)?.close()
        subtitleClockBindings.remove(player)?.release()
        subtitleCoordinators.remove(player)
        subtitleScopes.remove(player)?.cancel()
        requestContexts.remove(player)
        runCatching { player.release() }
    }

    private const val BYTES_PER_MB = 1024 * 1024
    private const val PLAYER_CACHE_FRAGMENT_BYTES = 8L * 1024L * 1024L
    private const val PREFS_PLAYER = "fluxa_player"
    private const val PREF_BW_ESTIMATE_BPS = "bw_estimate_bps"
    @Volatile private var playerDiskCache: SimpleCache? = null
    @Volatile private var lastPersistedBandwidthAtMs: Long = 0L
    @Volatile private var lastPersistedBandwidthBps: Long = 0L

    private fun savedBandwidthEstimate(context: Context): Long =
        context.getSharedPreferences(PREFS_PLAYER, Context.MODE_PRIVATE)
            .getLong(PREF_BW_ESTIMATE_BPS, 0L)

    internal fun saveBandwidthEstimate(context: Context, bps: Long) {
        if (bps <= 0L) return
        val now = android.os.SystemClock.elapsedRealtime()
        val previous = lastPersistedBandwidthBps
        val relativeChange = if (previous > 0L) kotlin.math.abs(bps - previous).toDouble() / previous.toDouble() else 1.0
        if (now - lastPersistedBandwidthAtMs < 30_000L && relativeChange < 0.25) return
        lastPersistedBandwidthAtMs = now
        lastPersistedBandwidthBps = bps
        context.applicationContext.getSharedPreferences(PREFS_PLAYER, Context.MODE_PRIVATE)
            .edit().putLong(PREF_BW_ESTIMATE_BPS, bps).apply()
    }

    internal fun deviceResourceBudget(context: Context): com.fluxa.app.core.rust.models.NativeDeviceResourceBudget {
        val activityManager = context.getSystemService(android.app.ActivityManager::class.java)
        val memInfo = android.app.ActivityManager.MemoryInfo()
        activityManager?.getMemoryInfo(memInfo)
        return com.fluxa.app.core.rust.FluxaCoreNative.deviceResourceBudget(
            totalRamMb = memInfo.totalMem / (1024L * 1024L),
            heapMaxMb = Runtime.getRuntime().maxMemory() / (1024L * 1024L),
            isLowRamDevice = activityManager?.isLowRamDevice == true,
            isTelevision = context.packageManager.hasSystemFeature(android.content.pm.PackageManager.FEATURE_LEANBACK)
        )
    }

    private fun safePlayerTargetBufferBytes(
        requestedMb: Int,
        budget: com.fluxa.app.core.rust.models.NativeDeviceResourceBudget
    ): Int {
        val requestedBytes = requestedMb.coerceIn(16, 2000).toLong() * BYTES_PER_MB
        return requestedBytes.coerceAtMost(budget.playerTargetBufferBytes).toInt()
    }

    private fun playerDiskCacheBytes(context: Context): Long {
        val activityManager = context.getSystemService(android.app.ActivityManager::class.java)
        return when {
            activityManager?.isLowRamDevice == true -> 128L * 1024L * 1024L
            context.packageManager.hasSystemFeature(android.content.pm.PackageManager.FEATURE_LEANBACK) -> 256L * 1024L * 1024L
            else -> 512L * 1024L * 1024L
        }
    }

    internal fun shouldScanEmbeddedAssFonts(url: String, title: String?): Boolean {
        fun String.looksMatroska(): Boolean {
            val value = substringBefore('?').substringBefore('#').lowercase(Locale.ROOT)
            return value.endsWith(".mkv") ||
                value.endsWith(".mks") ||
                value.endsWith(".mk3d") ||
                value.contains(".mkv")
        }
        return url.looksMatroska() || title?.looksMatroska() == true
    }

    internal fun playerCache(context: Context): SimpleCache {
        return playerDiskCache ?: synchronized(this) {
            playerDiskCache ?: SimpleCache(
                context.applicationContext.cacheDir.resolve("player_http_cache"),
                LeastRecentlyUsedCacheEvictor(playerDiskCacheBytes(context)),
                StandaloneDatabaseProvider(context.applicationContext)
            ).also { playerDiskCache = it }
        }
    }

    private fun shouldUsePlayerDiskCache(uri: Uri): Boolean {
        val scheme = uri.scheme?.lowercase(Locale.ROOT) ?: return false
        if (scheme != "http" && scheme != "https") return false
        val host = uri.host?.lowercase(Locale.ROOT) ?: return false
        if (host == "localhost" || host == "127.0.0.1" || host == "::1") return false
        return true
    }

    private class SelectiveCacheDataSource(
        private val cachedFactory: DataSource.Factory,
        private val uncachedFactory: DataSource.Factory,
        private val shouldUseCache: () -> Boolean
    ) : DataSource {
        private var active: DataSource? = null
        private val transferListeners = mutableListOf<TransferListener>()

        override fun addTransferListener(transferListener: TransferListener) {
            transferListeners += transferListener
            active?.addTransferListener(transferListener)
        }

        override fun open(dataSpec: DataSpec): Long {
            val selected = if (shouldUseCache() && shouldUsePlayerDiskCache(dataSpec.uri)) {
                cachedFactory.createDataSource()
            } else {
                uncachedFactory.createDataSource()
            }
            transferListeners.forEach(selected::addTransferListener)
            active = selected
            return selected.open(dataSpec)
        }

        override fun read(buffer: ByteArray, offset: Int, length: Int): Int {
            return active?.read(buffer, offset, length) ?: C.RESULT_END_OF_INPUT
        }

        override fun getUri(): Uri? = active?.uri

        override fun getResponseHeaders(): Map<String, List<String>> {
            return active?.responseHeaders ?: emptyMap()
        }

        override fun close() {
            active?.close()
            active = null
        }
    }

    // Pre-warm ExoPlayer's disk cache with the first bytes of an HTTP stream URL so
    // the next-episode transition starts from cache rather than a cold network open.
    // Must be called from a background thread; failures are silently ignored.
    fun primeHttpStream(
        context: Context,
        url: String,
        headers: Map<String, String>,
        primeBytes: Long = 2L * 1024L * 1024L
    ) {
        val uri = Uri.parse(url)
        if (!shouldUsePlayerDiskCache(uri)) return
        val okHttp = OkHttpClient.Builder()
            .callTimeout(10, java.util.concurrent.TimeUnit.SECONDS)
            .apply { cronetTransportInterceptor(context)?.let { addInterceptor(it) } }
            .build()
        val upstream = OkHttpDataSource.Factory(okHttp)
            .setUserAgent(StreamRequestPolicy.DEFAULT_USER_AGENT)
            .apply { if (headers.isNotEmpty()) setDefaultRequestProperties(headers) }
        val cacheDataSource = CacheDataSource.Factory()
            .setCache(playerCache(context))
            .setUpstreamDataSourceFactory(upstream)
            .setFlags(CacheDataSource.FLAG_IGNORE_CACHE_ON_ERROR)
            .createDataSource()
        val dataSpec = DataSpec(uri, 0L, primeBytes)
        runCatching { CacheWriter(cacheDataSource, dataSpec, null, null).cache() }
    }

    fun createExoPlayer(
        context: Context,
        audioDecoderMode: String = "hw_prefer",
        preferredAudioLanguage: String = "en",
        bufferCacheMb: Int = 100,
        forwardBufferSeconds: Int = 120,
        backBufferSeconds: Int = 30,
        tunneledPlayback: Boolean = false,
        minBufferSeconds: Int = 8,
        playbackBufferMs: Int = 3000,
        rebufferBufferMs: Int = 5000,
        enableLibassRelay: Boolean = true
    ): ExoPlayer {
        val requestContext = ExoRequestContext()
        val savedBps = savedBandwidthEstimate(context)
        val bandwidthMeter = DefaultBandwidthMeter.Builder(context)
            .apply { if (savedBps > 0L) setInitialBitrateEstimate(savedBps) }
            .build()
        val subtitleScope = kotlinx.coroutines.CoroutineScope(kotlinx.coroutines.SupervisorJob() + kotlinx.coroutines.Dispatchers.Default)
        val subtitleClock = com.fluxa.app.player.subtitle.MonotonicPlaybackClock()
        val subtitleFetcher = OkHttpSubtitleFetcher(
            OkHttpClient.Builder()
                .connectTimeout(10, TimeUnit.SECONDS)
                .readTimeout(15, TimeUnit.SECONDS)
                .build()
        )
        val subtitleCoordinator = com.fluxa.app.player.subtitle.SubtitleCoordinator(
            sidecar = com.fluxa.app.player.subtitle.SidecarTextEngineImpl(subtitleClock, subtitleScope, subtitleFetcher),
            embedded = com.fluxa.app.player.subtitle.EmbeddedTextEngine(subtitleClock, subtitleScope),
            scope = subtitleScope
        )
        val renderersFactory = AppAudioRendererFactory(context, audioDecoderMode, subtitleCoordinator).apply {
            setExtensionRendererMode(
                when (audioDecoderMode) {
                    "hw_only" -> DefaultRenderersFactory.EXTENSION_RENDERER_MODE_OFF
                    "sw_only" -> DefaultRenderersFactory.EXTENSION_RENDERER_MODE_PREFER
                    else -> DefaultRenderersFactory.EXTENSION_RENDERER_MODE_ON
                }
            )
            setEnableDecoderFallback(true)
        }

        val okHttpClient = OkHttpClient.Builder()
            .addInterceptor { chain ->
                val request = chain.request()
                val host = request.url.host.lowercase()
                val builder = request.newBuilder()

                //  CRITICAL FIX: ABSOLUTELY DO NOT TOUCH LOCALHOST HEADERS
                if (host != "127.0.0.1" && host != "localhost") {
                    StreamRequestPolicy.headersFor(request.url.toString(), requestContext.streamHeaders).forEach { (k, v) ->
                        builder.header(k, v)
                    }
                }

                val response = chain.proceed(builder.build())
                rewriteDolbyVisionManifestResponse(context, response, requestContext)
            }
            .connectTimeout(15, java.util.concurrent.TimeUnit.SECONDS)
            .readTimeout(15, java.util.concurrent.TimeUnit.SECONDS)
            // Local proxy: fast connect (2s) + unlimited read (torrent seeks block on piece download).
            .addInterceptor { chain ->
                val host = chain.request().url.host.lowercase()
                if (host == "127.0.0.1" || host == "localhost") {
                    chain.withConnectTimeout(2, java.util.concurrent.TimeUnit.SECONDS)
                        .withReadTimeout(0, java.util.concurrent.TimeUnit.SECONDS)
                        .proceed(chain.request())
                } else {
                    chain.proceed(chain.request())
                }
            }
            .apply { cronetTransportInterceptor(context)?.let { addInterceptor(it) } }
            .build()

        val httpDataSourceFactory = OkHttpDataSource.Factory(okHttpClient)
            .setUserAgent(StreamRequestPolicy.DEFAULT_USER_AGENT)
        val uncachedDataSourceFactory = DefaultDataSource.Factory(context, httpDataSourceFactory)
        val cachedDataSourceFactory = CacheDataSource.Factory()
            .setCache(playerCache(context))
            .setUpstreamDataSourceFactory(uncachedDataSourceFactory)
            .setCacheWriteDataSinkFactory(
                CacheDataSink.Factory()
                    .setCache(playerCache(context))
                    .setFragmentSize(PLAYER_CACHE_FRAGMENT_BYTES)
            )
            .setFlags(CacheDataSource.FLAG_IGNORE_CACHE_ON_ERROR)
        val dataSourceFactory = DataSource.Factory {
            SelectiveCacheDataSource(
                cachedFactory = cachedDataSourceFactory,
                uncachedFactory = uncachedDataSourceFactory,
                shouldUseCache = { !requestContext.disableDiskCache }
            )
        }

        // Always probe MKV cues for seekability. rqbit/TorrentServer supports range requests
        // and will download the tail pieces on demand. Previously this was disabled for
        // localhost (torrent proxy) to avoid a seek-to-end probe, but that made torrent
        // MKV streams non-seekable — ExoPlayer reverts to position 0 on seekTo() without cues.
        val resourceBudget = deviceResourceBudget(context)
        val relay = if (enableLibassRelay) LibassEventRelay(resourceBudget.subtitleGlyphCacheBytes) else null
        val fontsDir = context.applicationContext.filesDir.resolve("fonts").absolutePath
        val baseExtractorsFactory = DefaultExtractorsFactory().let { factory ->
            if (enableLibassRelay) {
                factory.setMatroskaExtractorFlags(MatroskaExtractor.FLAG_EMIT_RAW_SUBTITLE_DATA)
            } else {
                factory
            }
        }
        val extractorsFactory = relay?.let { activeRelay ->
            LibassInjectingExtractorsFactory(
                activeRelay,
                baseExtractorsFactory,
                fontsDir,
                fontAttachmentsProvider = null
            )
        } ?: baseExtractorsFactory
        val mediaSourceFactory = DefaultMediaSourceFactory(context, extractorsFactory)
            .setDataSourceFactory(dataSourceFactory)

        val loadControl = DefaultLoadControl.Builder()
            .setBufferDurationsMs(
                minBufferSeconds.coerceIn(2, 30) * 1000,
                forwardBufferSeconds.coerceIn(10, 600) * 1000,
                playbackBufferMs.coerceIn(500, 5000),
                rebufferBufferMs.coerceIn(1000, 10000)
            )
            .setBackBuffer(backBufferSeconds.coerceIn(0, 60) * 1000, true)
            .setTargetBufferBytes(safePlayerTargetBufferBytes(bufferCacheMb, resourceBudget))
            .setPrioritizeTimeOverSizeThresholds(true)
            .build()

        val preferredVideoMimeType = detectPreferredVideoMimeType(context)
        val trackSelector = DefaultTrackSelector(context).apply {
            parameters = buildUponParameters()
                .setMaxAudioChannelCount(8)
                .setAllowVideoMixedMimeTypeAdaptiveness(true)
                .apply {
                    if (preferredAudioLanguage.isNotBlank() && preferredAudioLanguage != "none") {
                        setPreferredAudioLanguage(preferredAudioLanguage)
                    }
                    setTunnelingEnabled(tunneledPlayback)
                    if (preferredVideoMimeType != null) setPreferredVideoMimeType(preferredVideoMimeType)
                }
                .build()
        }

        return ExoPlayer.Builder(context, renderersFactory)
            .setBandwidthMeter(bandwidthMeter)
            .setMediaSourceFactory(mediaSourceFactory)
            .setLoadControl(loadControl)
            .setTrackSelector(trackSelector)
            .setUsePlatformDiagnostics(false)
            .setVideoChangeFrameRateStrategy(androidx.media3.common.C.VIDEO_CHANGE_FRAME_RATE_STRATEGY_ONLY_IF_SEAMLESS)
            .setAudioAttributes(
                AudioAttributes.Builder()
                    .setUsage(C.USAGE_MEDIA)
                    .setContentType(C.AUDIO_CONTENT_TYPE_MOVIE)
                    .build(),
                true
            )
            .setHandleAudioBecomingNoisy(true)
            .setSeekBackIncrementMs(10000)
            .setSeekForwardIncrementMs(10000)
            .setReleaseTimeoutMs(500)
            .build().apply {
                playWhenReady = true
                setWakeMode(C.WAKE_MODE_NETWORK)
                requestContexts[this] = requestContext
                relay?.let { libassRelays[this] = it }
                subtitleCoordinators[this] = subtitleCoordinator
                subtitleScopes[this] = subtitleScope
                subtitleClockBindings[this] = ExoPlaybackClock(this, subtitleClock)
                if (relay != null) {
                    LibassDebugLog.d("created ExoPlayer with libass relay registered player=${System.identityHashCode(this)}")
                }
            }
    }

    private fun detectPreferredVideoMimeType(context: Context): String? {
        val caps = AndroidDolbyVisionCapabilities.detect(context)
        if (caps.displaySupportsDolbyVision && caps.decoderAnyDv) {
            return MimeTypes.VIDEO_DOLBY_VISION
        }
        val codecInfos = MediaCodecList(MediaCodecList.REGULAR_CODECS).codecInfos
            .filter { !it.isEncoder }
        fun hasHardware(mimeType: String) = codecInfos.any { info ->
            info.supportedTypes.any { it.equals(mimeType, ignoreCase = true) } &&
                !info.name.startsWith("OMX.google") &&
                !info.name.startsWith("c2.android")
        }
        if (hasHardware(MimeTypes.VIDEO_AV1)) return null
        if (hasHardware(MimeTypes.VIDEO_H265)) return MimeTypes.VIDEO_H265
        return MimeTypes.VIDEO_H264
    }

    private fun rewriteDolbyVisionManifestResponse(
        context: Context,
        response: okhttp3.Response,
        requestContext: ExoRequestContext
    ): okhttp3.Response {
        val mode = requestContext.dolbyVisionFallbackMode
        if (mode == DolbyVisionFallbackMode.Off || !response.isSuccessful) return response
        val body = response.body
        val contentType = body.contentType()
        val path = response.request.url.encodedPath.lowercase(Locale.ROOT)
        val mediaType = contentType?.toString().orEmpty().lowercase(Locale.ROOT)
        val isManifest = path.endsWith(".m3u8") ||
            path.endsWith(".mpd") ||
            mediaType.contains("mpegurl") ||
            mediaType.contains("dash+xml") ||
            mediaType.contains("application/vnd.apple.mpegurl")
        if (!isManifest) {
            val host = response.request.url.host.lowercase()
            val isProxy = host == "127.0.0.1" || host == "localhost"
            if (!isProxy) {
                if (requestContext.shouldConvertRpuP7) {
                    val isHlsSegment = path.endsWith(".m4s") || path.endsWith(".ts") ||
                        mediaType.contains("video/mp4") || mediaType.contains("video/mp2t")
                    if (isHlsSegment) {
                        return rewriteHlsSegmentBytes(response, requestContext)
                    }
                }
                // For direct MKV/MP4/M4V streams, strip the DVCC fourcc in the first 64 KiB.
                if (path.contains(".mkv") || path.contains(".mp4") || path.contains(".m4v")) {
                    val caps = requestContext.cachedCapabilities
                        ?: AndroidDolbyVisionCapabilities.detect(context).also { requestContext.cachedCapabilities = it }
                    return maybeStripContainerDvcc(response, requestContext, caps)
                }
            }
            return response
        }

        val original = runCatching { body.string() }.getOrElse { return response }
        val caps = requestContext.cachedCapabilities
            ?: AndroidDolbyVisionCapabilities.detect(context).also { requestContext.cachedCapabilities = it }
        val rewrite = DolbyVisionFallbackPolicy.rewriteManifest(
            manifest = original,
            mode = mode,
            capabilities = caps
        )
        requestContext.lastDolbyVisionDecision = rewrite.decision
        if (rewrite.requiresIptPqc2ToneMap) requestContext.needsIptPqc2ToneMap = true
        if (rewrite.hasP81Conversion) requestContext.shouldConvertRpuP7 = true
        if (rewrite.manifest == original) {
            return response.newBuilder().body(original.toResponseBody(contentType)).build()
        }
        return response.newBuilder()
            .removeHeader("Content-Length")
            .body(rewrite.manifest.toResponseBody(contentType))
            .build()
    }

    private fun maybeStripContainerDvcc(
        response: okhttp3.Response,
        requestContext: ExoRequestContext,
        caps: DolbyVisionCapabilities
    ): okhttp3.Response {
        val body = response.body
        val rangeHeader = response.header("Content-Range")
        val fileOffset = DolbyVisionFallbackPolicy.parseContentRangeStart(rangeHeader) ?: 0L
        val scanWindow = DolbyVisionFallbackPolicy.containerDvccScanWindow().toLong()
        if (fileOffset >= scanWindow) return response

        val scanLen = (scanWindow - fileOffset).toInt()
        val source = body.source()
        source.request(scanLen.toLong())
        val available = minOf(source.buffer.size, scanLen.toLong())
        if (available <= 0L) return response
        val headerBytes = source.buffer.readByteArray(available)

        val dvInfo = DolbyVisionFallbackPolicy.scanDvContainerInfo(headerBytes)
            ?: return reconstructResponse(response, body, headerBytes, source)

        val deviceSupportsDv = DolbyVisionFallbackPolicy.containerDvSupportedForCaps(dvInfo, caps)

        if (!deviceSupportsDv) {
            when {
                dvInfo.profile == 5 -> {
                    DolbyVisionFallbackPolicy.mangleDvccFourcc(headerBytes)
                    if (dvInfo.compatId != 1) requestContext.needsIptPqc2ToneMap = true
                }
                !dvInfo.notHasHdrFallback -> DolbyVisionFallbackPolicy.mangleDvccFourcc(headerBytes)
            }
        }
        return reconstructResponse(response, body, headerBytes, source)
    }

    private fun rewriteHlsSegmentBytes(
        response: okhttp3.Response,
        requestContext: ExoRequestContext
    ): okhttp3.Response {
        val body = response.body
        val contentType = body.contentType()
        val bytes = runCatching { body.bytes() }.getOrElse { return response }
        val removeHdr10Plus = when (requestContext.dvHdr10PlusMode) {
            "always" -> true
            "never" -> false
            else -> true // auto: strip HDR10+ SEIs when converting DV RPU
        }
        val rewritten = runCatching {
            com.fluxa.app.core.rust.FluxaStreamingNative.dvRewriteSegmentBytes(
                data = bytes,
                rpuMode = requestContext.dvRpuMode,
                zeroLevel5 = requestContext.dvZeroLevel5,
                removeHdr10Plus = removeHdr10Plus
            )
        }.getOrElse { return response }
        return response.newBuilder()
            .removeHeader("Content-Length")
            .body(rewritten.toResponseBody(contentType))
            .build()
    }

    private fun reconstructResponse(
        response: okhttp3.Response,
        originalBody: okhttp3.ResponseBody,
        headerBytes: ByteArray,
        remainingSource: okio.BufferedSource
    ): okhttp3.Response {
        val prefix = Buffer().write(headerBytes)
        val combined = object : ForwardingSource(remainingSource) {
            override fun read(sink: Buffer, byteCount: Long): Long {
                if (prefix.size > 0L) return prefix.read(sink, byteCount)
                return super.read(sink, byteCount)
            }
        }.buffer()
        val newBody = object : ResponseBody() {
            override fun contentType() = originalBody.contentType()
            override fun contentLength() = originalBody.contentLength()
            override fun source() = combined
        }
        return response.newBuilder().body(newBody).build()
    }


}
