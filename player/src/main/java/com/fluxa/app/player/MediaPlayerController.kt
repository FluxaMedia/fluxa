package com.fluxa.app.player

import android.content.Context
import android.util.Log
import androidx.media3.common.*
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.DefaultRenderersFactory
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.exoplayer.DefaultLoadControl
import androidx.media3.extractor.DefaultExtractorsFactory
import androidx.media3.extractor.Extractor
import androidx.media3.extractor.ExtractorsFactory
import androidx.media3.extractor.mkv.MatroskaExtractor
import androidx.media3.exoplayer.analytics.AnalyticsListener
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
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import okhttp3.OkHttpClient
import okhttp3.ResponseBody
import okhttp3.ResponseBody.Companion.toResponseBody
import okio.Buffer
import okio.ForwardingSource
import okio.buffer

import androidx.media3.common.TrackSelectionOverride
import androidx.media3.common.TrackSelectionParameters
import androidx.media3.exoplayer.trackselection.DefaultTrackSelector
import androidx.media3.exoplayer.upstream.DefaultBandwidthMeter
import com.fluxa.app.core.rust.models.NativeDvProxyPlan
import android.media.MediaCodecList
import java.util.Locale
import java.util.Collections
import java.util.WeakHashMap
import java.util.concurrent.TimeUnit

@UnstableApi
class MediaPlayerController(internal val context: Context, val exoPlayer: ExoPlayer) {
    private var preferredAudioLanguageCode: String = "en"
    var audioDecoderMode: String = "hw_prefer"
    private var currentUrl: String? = null
    private var currentStream: com.fluxa.app.data.remote.Stream? = null
    private var currentExternalSubtitles: List<ExternalSubtitleTrack> = emptyList()
    
    private val _availableAudios = MutableStateFlow<List<MediaTrack>>(emptyList())
    val availableAudios: StateFlow<List<MediaTrack>> = _availableAudios

    private val _availableSubtitles = MutableStateFlow<List<MediaTrack>>(emptyList())
    val availableSubtitles: StateFlow<List<MediaTrack>> = _availableSubtitles

    private val _currentAudio = MutableStateFlow<MediaTrack?>(null)
    val currentAudio: StateFlow<MediaTrack?> = _currentAudio

    private val _currentSubtitle = MutableStateFlow<MediaTrack?>(null)
    val currentSubtitle: StateFlow<MediaTrack?> = _currentSubtitle

    private val _technicalInfo = MutableStateFlow<String?>(null)
    val technicalInfo: StateFlow<String?> = _technicalInfo

    companion object {
        private class ExoRequestContext {
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
        fun getLibassRelay(player: ExoPlayer): LibassEventRelay? = libassRelays[player]

        private const val BYTES_PER_MB = 1024 * 1024
        private const val PLAYER_DISK_CACHE_BYTES = 512L * 1024L * 1024L
        private const val PLAYER_CACHE_FRAGMENT_BYTES = 8L * 1024L * 1024L
        private const val PREFS_PLAYER = "fluxa_player"
        private const val PREF_BW_ESTIMATE_BPS = "bw_estimate_bps"
        @Volatile private var playerDiskCache: SimpleCache? = null

        private fun savedBandwidthEstimate(context: Context): Long =
            context.getSharedPreferences(PREFS_PLAYER, Context.MODE_PRIVATE)
                .getLong(PREF_BW_ESTIMATE_BPS, 0L)

        private fun saveBandwidthEstimate(context: Context, bps: Long) {
            if (bps <= 0L) return
            context.applicationContext.getSharedPreferences(PREFS_PLAYER, Context.MODE_PRIVATE)
                .edit().putLong(PREF_BW_ESTIMATE_BPS, bps).apply()
        }

        private fun safePlayerTargetBufferBytes(requestedMb: Int): Int {
            val heapMb = (Runtime.getRuntime().maxMemory() / BYTES_PER_MB).toInt().coerceAtLeast(64)
            // Allow up to 1/4 of heap; cap at 150 MB for modern devices, minimum 32 MB.
            val heapBoundMb = (heapMb / 4).coerceIn(32, 150)
            return requestedMb.coerceIn(32, 2000)
                .coerceAtMost(heapBoundMb) * BYTES_PER_MB
        }

        private fun shouldScanEmbeddedAssFonts(url: String, title: String?): Boolean {
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
                    LeastRecentlyUsedCacheEvictor(PLAYER_DISK_CACHE_BYTES),
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
            rebufferBufferMs: Int = 5000
        ): ExoPlayer {
            val requestContext = ExoRequestContext()
            val savedBps = savedBandwidthEstimate(context)
            val bandwidthMeter = DefaultBandwidthMeter.Builder(context)
                .apply { if (savedBps > 0L) setInitialBitrateEstimate(savedBps) }
                .build()
            val renderersFactory = AppAudioRendererFactory(context, audioDecoderMode).apply {
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
            val relay = LibassEventRelay()
            val fontsDir = context.applicationContext.filesDir.resolve("fonts").absolutePath
            val extractorsFactory = LibassInjectingExtractorsFactory(
                relay,
                DefaultExtractorsFactory().setMatroskaExtractorFlags(MatroskaExtractor.FLAG_EMIT_RAW_SUBTITLE_DATA),
                fontsDir,
                fontAttachmentsProvider = null
            )
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
                .setTargetBufferBytes(safePlayerTargetBufferBytes(bufferCacheMb))
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
                .setReleaseTimeoutMs(0)
                .build().apply {
                    playWhenReady = true
                    setWakeMode(C.WAKE_MODE_NETWORK)
                    requestContexts[this] = requestContext
                    libassRelays[this] = relay
                    LibassDebugLog.d("created ExoPlayer with libass relay registered player=${System.identityHashCode(this)}")
                }
        }

        private fun detectPreferredVideoMimeType(context: Context): String? {
            val caps = DolbyVisionFallbackPolicy.capabilities(context)
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
                            ?: DolbyVisionFallbackPolicy.capabilities(context).also { requestContext.cachedCapabilities = it }
                        return maybeStripContainerDvcc(response, requestContext, caps)
                    }
                }
                return response
            }

            val original = runCatching { body.string() }.getOrElse { return response }
            val caps = requestContext.cachedCapabilities
                ?: DolbyVisionFallbackPolicy.capabilities(context).also { requestContext.cachedCapabilities = it }
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

    init {
        exoPlayer.addListener(object : Player.Listener {
            override fun onTracksChanged(tracks: Tracks) { updateTracks(tracks); updateTechnicalInfo() }
            override fun onPlaybackStateChanged(state: Int) { 
                if (state == Player.STATE_READY) updateTracks(exoPlayer.currentTracks)
                updateTechnicalInfo()
            }
            override fun onVideoSizeChanged(videoSize: VideoSize) { updateTechnicalInfo() }
            override fun onVolumeChanged(volume: Float) { updateTechnicalInfo() }
            override fun onDeviceVolumeChanged(volume: Int, muted: Boolean) { updateTechnicalInfo() }
        })
        exoPlayer.addAnalyticsListener(object : AnalyticsListener {
            override fun onVideoInputFormatChanged(
                eventTime: AnalyticsListener.EventTime,
                format: Format,
                decoderReuseEvaluation: androidx.media3.exoplayer.DecoderReuseEvaluation?
            ) {
                updateTechnicalInfo()
                val ctx = requestContexts[exoPlayer] ?: return
                if (!ctx.needsIptPqc2ToneMap) {
                    if (runCatching { com.fluxa.app.core.rust.FluxaStreamingNative.dvAutoDetectWasIptPqc2() }.getOrDefault(false)) {
                        ctx.needsIptPqc2ToneMap = true
                    }
                }
                if (ctx.needsIptPqc2ToneMap) {
                    updateIptPqc2L1State()
                    ctx.videoEffectsActive = true
                    exoPlayer.setVideoEffects(listOf(IptPqc2ToneMapEffect(ctx.iptPqc2UseHdr)))
                }
            }

            override fun onDroppedVideoFrames(eventTime: AnalyticsListener.EventTime, droppedFrames: Int, elapsedMs: Long) {
                updateTechnicalInfo(extraDroppedFrames = droppedFrames)
            }

            override fun onBandwidthEstimate(eventTime: AnalyticsListener.EventTime, totalLoadTimeMs: Int, totalBytesLoaded: Long, bitrateEstimate: Long) {
                saveBandwidthEstimate(context.applicationContext, bitrateEstimate)
            }
        })
    }

    private fun updateIptPqc2L1State() {
        runCatching {
            val json = com.fluxa.app.core.rust.FluxaStreamingNative.dvGetCurrentL1Json()
            val obj = com.google.gson.JsonParser.parseString(json).asJsonObject
            if (obj.get("available")?.asBoolean == true) {
                val minPq = obj.get("min_pq")?.asInt ?: return
                val maxPq = obj.get("max_pq")?.asInt ?: return
                IptPqc2L1State.sdrWhiteLinear = pqCodeToLinear(minPq).coerceIn(0.001f, 0.1f)
                IptPqc2L1State.hdrPeakLinear = pqCodeToLinear(maxPq).coerceIn(0.05f, 1.0f)
                IptPqc2L1State.available = true
            }
        }
    }

    private fun pqCodeToLinear(pqCode: Int): Float {
        val e = (pqCode.coerceIn(0, 4095) / 4095.0f)
        val ep = Math.pow(e.toDouble(), 1.0 / 78.84375).toFloat()
        val num = (ep - 0.8359375f).coerceAtLeast(0f)
        val den = (18.8515625f - 18.6875f * ep).coerceAtLeast(1e-6f)
        return Math.pow((num / den).toDouble(), 1.0 / 0.1593017578125).toFloat()
    }

    private fun updateTechnicalInfo(extraDroppedFrames: Int? = null) {
        val runtimeInfo = MediaTechnicalInfoBuilder.build(
            context = context,
            exoPlayer = exoPlayer,
            extraDroppedFrames = extraDroppedFrames,
            url = currentUrl,
            stream = currentStream,
            externalSubtitles = currentExternalSubtitles,
            audioDecoderMode = audioDecoderMode
        )
        val ctx = requestContexts[exoPlayer]
        _technicalInfo.value = listOfNotNull(
            runtimeInfo,
            ctx?.lastDolbyVisionDecision,
            ctx?.dvProxyPlanDebug?.takeIf { it.isNotBlank() }?.let { "\ndv_proxy_data\n$it" }
        ).joinToString("\n")
        LastMediaDebugInfoStore.save(
            context = context,
            url = currentUrl,
            title = currentStream?.effectiveFilename ?: currentStream?.rawDisplayTitle,
            technicalInfo = _technicalInfo.value
        )
    }

    private fun updateTracks(tracks: Tracks) {
        val audios = mutableListOf<MediaTrack>()
        val subtitles = mutableListOf<MediaTrack>()

        tracks.groups.forEachIndexed { groupIndex, group ->
            if (group.type == C.TRACK_TYPE_AUDIO) {
                for (i in 0 until group.length) {
                    val format = group.getTrackFormat(i)
                    audios.add(MediaTrack(
                        id = "audio_$groupIndex-$i",
                        label = format.label ?: format.language ?: "Ses ${audios.size + 1}",
                        language = format.language,
                        type = C.TRACK_TYPE_AUDIO,
                        groupIndex = groupIndex,
                        trackIndex = i,
                        isSelected = group.isTrackSelected(i),
                        isSupported = group.isTrackSupported(i),
                        channelCount = format.channelCount,
                        sampleMimeType = format.sampleMimeType
                    ))
                }
            } else if (group.type == C.TRACK_TYPE_TEXT) {
                for (i in 0 until group.length) {
                    val format = group.getTrackFormat(i)
                    LibassDebugLog.d(
                        "track discovered group=$groupIndex track=$i selected=${group.isTrackSelected(i)} supported=${group.isTrackSupported(i)} ${LibassDebugLog.formatSummary(format)}"
                    )
                    subtitles.add(MediaTrack(
                        id = "sub_$groupIndex-$i",
                        label = format.label ?: format.language ?: "Subtitle ${subtitles.size + 1}",
                        language = format.language,
                        type = C.TRACK_TYPE_TEXT,
                        groupIndex = groupIndex,
                        trackIndex = i,
                        isSelected = group.isTrackSelected(i),
                        isSupported = group.isTrackSupported(i),
                        sampleMimeType = format.sampleMimeType
                    ))
                }
            }
        }
        _availableAudios.value = audios
        _availableSubtitles.value = subtitles
        _currentAudio.value = audios.find { it.isSelected }
        val selectedSub = subtitles.find { it.isSelected }
        _currentSubtitle.value = selectedSub
        LibassDebugLog.d(
            "tracks updated subtitles=${subtitles.size} selected=${
                selectedSub?.let { "${it.id} mime=${it.sampleMimeType} label=${it.label} lang=${it.language}" } ?: "<none>"
            }"
        )
        val relay = libassRelays[exoPlayer]
        if (relay != null) {
            val selectedFormat = if (selectedSub != null) {
                tracks.groups.getOrNull(selectedSub.groupIndex)?.getTrackFormat(selectedSub.trackIndex)
            } else null
            relay.setSelectedTrackId(selectedFormat?.id?.toIntOrNull())
        }
    }

    fun selectTrack(track: MediaTrack) {
        LibassDebugLog.d("select track id=${track.id} type=${track.type} group=${track.groupIndex} track=${track.trackIndex} mime=${track.sampleMimeType} label=${track.label} lang=${track.language}")
        exoPlayer.trackSelectionParameters = exoPlayer.trackSelectionParameters
            .buildUpon()
            .setOverrideForType(TrackSelectionOverride(exoPlayer.currentTracks.groups[track.groupIndex].mediaTrackGroup, track.trackIndex))
            .build()
        if (track.type == C.TRACK_TYPE_AUDIO) _currentAudio.value = track
        else _currentSubtitle.value = track
    }

    fun prepareAndPlay(
        url: String,
        headers: Map<String, String>? = null,
        subtitles: List<ExternalSubtitleTrack> = emptyList(),
        dolbyVisionFallbackMode: DolbyVisionFallbackMode = DolbyVisionFallbackMode.Off,
        dvRpuMode: Int = 2,
        dvZeroLevel5: Boolean = false,
        dvHdr10PlusMode: String = "auto",
        iptPqc2UseHdr: Boolean = false,
        iptPqc2PreDecide: Boolean = false,
        stream: com.fluxa.app.data.remote.Stream? = null
    ) {
        currentUrl = url
        currentStream = stream
        currentExternalSubtitles = subtitles
        LibassDebugLog.d(
            "prepare ExoPlayer url=${LibassDebugLog.urlSummary(url)} externalSubtitles=${subtitles.size} streamTitle=${stream?.effectiveFilename ?: stream?.rawDisplayTitle}"
        )
        val headersForPlayback = headers.orEmpty()
        val streamTitle = stream?.effectiveFilename ?: stream?.rawDisplayTitle
        requestContexts[exoPlayer]?.let { ctx ->
            if (ctx.videoEffectsActive) {
                exoPlayer.setVideoEffects(emptyList())
                ctx.videoEffectsActive = false
            }
            ctx.streamHeaders = headersForPlayback
            libassRelays[exoPlayer]?.resetFonts()
            if (shouldScanEmbeddedAssFonts(url, streamTitle)) {
                LibassDebugLog.d("embedded ASS fonts will be collected from the active Matroska stream")
            }
            ctx.disableDiskCache = stream?.behaviorHints?.let { hints ->
                hints["cs3Type"] != null ||
                    hints["isM3u8"] as? Boolean == true ||
                    hints["isDash"] as? Boolean == true
            } == true
            ctx.dolbyVisionFallbackMode = dolbyVisionFallbackMode
            ctx.lastDolbyVisionDecision = when (dolbyVisionFallbackMode) {
                DolbyVisionFallbackMode.Off -> "dv_fallback=off"
                else -> "dv_fallback=pending_manifest_check"
            }
            ctx.needsIptPqc2ToneMap = iptPqc2PreDecide
            ctx.iptPqc2UseHdr = iptPqc2UseHdr
            ctx.shouldConvertRpuP7 = false
            ctx.cachedCapabilities = null
            ctx.dvRpuMode = dvRpuMode
            ctx.dvZeroLevel5 = dvZeroLevel5
            ctx.dvHdr10PlusMode = dvHdr10PlusMode
        }
        val builder = MediaItem.Builder().setUri(Uri.parse(url))
        val lowerUrl = url.lowercase()
        val hints = stream?.behaviorHints.orEmpty()
        val cs3Type = hints["cs3Type"] as? String
        val isM3u8 = hints["isM3u8"] as? Boolean == true ||
            cs3Type.equals("M3U8", ignoreCase = true) ||
            lowerUrl.contains("m3u8")
        val isDash = hints["isDash"] as? Boolean == true ||
            cs3Type.equals("DASH", ignoreCase = true) ||
            lowerUrl.contains(".mpd")
        when {
            isM3u8 -> builder.setMimeType(MimeTypes.APPLICATION_M3U8)
            isDash -> builder.setMimeType(MimeTypes.APPLICATION_MPD)
        }
        if (subtitles.isNotEmpty()) {
            builder.setSubtitleConfigurations(subtitles.mapNotNull { subtitle ->
                val subtitleUrl = subtitle.url.takeIf { it.isNotBlank() } ?: return@mapNotNull null
                val mimeType = subtitleMimeType(subtitleUrl)
                LibassDebugLog.d(
                    "external subtitle config url=${LibassDebugLog.urlSummary(subtitleUrl)} mime=$mimeType label=${subtitle.label} lang=${subtitle.language}"
                )
                MediaItem.SubtitleConfiguration.Builder(Uri.parse(subtitleUrl))
                    .setMimeType(mimeType)
                    .setLanguage(subtitle.language)
                    .setLabel(subtitle.label ?: subtitle.language)
                    .build()
            })
        }
        exoPlayer.setMediaItem(builder.build())
        exoPlayer.prepare()
        exoPlayer.play()
    }

    fun notifyDvProxyPlan(plan: NativeDvProxyPlan?) {
        val debug = if (plan == null) {
            "plan_error=resolveDvProxyPlan_failed"
        } else {
            buildString {
                appendLine("dv_profile=${plan.profile}")
                appendLine("action=${plan.action}")
                appendLine("reason=${plan.reason}")
                appendLine("compatibility=${plan.compatibility}")
                appendLine("safety=${plan.safety}")
                appendLine("limitations=[${plan.limitations.joinToString(", ")}]")
                appendLine("dv_self_test=${if (dvSelfTestPassed) "ok" else "fail"}")
                appendLine(dvLastStreamStatsLine())
                append("runtime_logs=logcat:[fluxa/dvcc_strip],[fluxa/rpu_convert],[fluxa/rpu_convert_fmp4],[fluxa/rpu_convert_mkv]")
            }
        }
        requestContexts[exoPlayer]?.dvProxyPlanDebug = debug
    }

    private val dvSelfTestPassed: Boolean by lazy {
        runCatching { com.fluxa.app.core.rust.FluxaStreamingNative.dvRpuSelfTest() }.getOrDefault(false)
    }

    private fun dvLastStreamStatsLine(): String = runCatching {
        val json = com.fluxa.app.core.rust.FluxaStreamingNative.dvGetStreamStats()
        val obj = org.json.JSONObject(json)
        val converted = obj.optInt("rpu_converted", 0)
        val failed = obj.optInt("rpu_failed", 0)
        val dropped = obj.optInt("el_dropped", 0)
        val segments = obj.optInt("segments", 0)
        "dv_stats=rpu_converted=$converted rpu_failed=$failed el_dropped=$dropped segments=$segments"
    }.getOrElse { "dv_stats=unavailable" }

    fun addExternalSubtitles(subtitles: List<ExternalSubtitleTrack>) {
        if (subtitles.isEmpty()) {
            LibassDebugLog.d("addExternalSubtitles ignored empty list")
            return
        }
        val currentItem = exoPlayer.currentMediaItem ?: run {
            LibassDebugLog.w("addExternalSubtitles ignored because current media item is null")
            return
        }
        currentExternalSubtitles = (currentExternalSubtitles + subtitles).distinctBy { it.url }
        val subtitleConfigurations = subtitles.mapNotNull { subtitle ->
            val subtitleUrl = subtitle.url.takeIf { it.isNotBlank() } ?: return@mapNotNull null
            val mimeType = subtitleMimeType(subtitleUrl)
            LibassDebugLog.d(
                "adding external subtitle url=${LibassDebugLog.urlSummary(subtitleUrl)} mime=$mimeType label=${subtitle.label} lang=${subtitle.language}"
            )
            MediaItem.SubtitleConfiguration.Builder(Uri.parse(subtitleUrl))
                .setMimeType(mimeType)
                .setLanguage(subtitle.language)
                .setLabel(subtitle.label ?: subtitle.language)
                .build()
        }
        if (subtitleConfigurations.isEmpty()) {
            LibassDebugLog.w("addExternalSubtitles produced no subtitle configurations")
            return
        }
        val currentPosition = exoPlayer.currentPosition.coerceAtLeast(0L)
        val wasPlaying = exoPlayer.playWhenReady
        LibassDebugLog.d("rebuilding media item with added external subtitles count=${subtitleConfigurations.size} positionMs=$currentPosition wasPlaying=$wasPlaying")
        exoPlayer.setMediaItem(currentItem.buildUpon().setSubtitleConfigurations(subtitleConfigurations).build(), currentPosition)
        exoPlayer.prepare()
        exoPlayer.playWhenReady = wasPlaying
    }

    private fun subtitleMimeType(url: String): String {
        return SubtitleFormatSupport.mimeTypeForUrl(url)
    }
    
    /**
     * Disable subtitles by clearing text track override
     */
    fun disableSubtitles() {
        LibassDebugLog.d("disable subtitles")
        exoPlayer.trackSelectionParameters = exoPlayer.trackSelectionParameters
            .buildUpon()
            .clearOverridesOfType(C.TRACK_TYPE_TEXT)
            .build()
        _currentSubtitle.value = null
    }
    
    /**
     * Enable a specific subtitle track
     */
    fun enableSubtitle(track: MediaTrack) {
        if (track.type == C.TRACK_TYPE_TEXT) {
            selectTrack(track)
        }
    }
    
    /**
     * Select an audio track
     */
    fun selectAudio(track: MediaTrack) {
        if (track.type == C.TRACK_TYPE_AUDIO) {
            selectTrack(track)
        }
    }
    
    /**
     * Apply preferred audio language
     */
    fun applyPreferredAudioLanguage(languageCode: String) {
        if (languageCode.isBlank() || languageCode == "none") return
        preferredAudioLanguageCode = languageCode
        exoPlayer.trackSelectionParameters = exoPlayer.trackSelectionParameters
            .buildUpon()
            .setPreferredAudioLanguage(languageCode)
            .build()
    }
}
