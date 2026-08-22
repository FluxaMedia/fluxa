package com.fluxa.app.player

import com.fluxa.app.shared.feature.player.MediaTrack
import com.fluxa.app.shared.feature.player.AudioTrackQualityPolicy

import android.content.Context
import androidx.media3.common.*
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.ExoPlaybackException
import androidx.media3.exoplayer.analytics.AnalyticsListener
import androidx.media3.exoplayer.audio.AudioSink
import android.net.Uri
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

import androidx.media3.common.TrackSelectionOverride
import com.fluxa.app.core.rust.models.NativeDvProxyPlan
import java.util.Locale

@UnstableApi
class MediaPlayerController(internal val context: Context, val exoPlayer: ExoPlayer) {
    private var preferredAudioLanguageCode: String = ""
    private var manualAudioSelection = false
    private var automaticAudioFallbackSelection = false
    private var applyingAudioPreference = false
    var audioDecoderMode: String = "hw_prefer"
    private var currentUrl: String? = null
    private var currentStream: com.fluxa.app.data.remote.Stream? = null
    private var currentExternalSubtitles: List<ExternalSubtitleTrack> = emptyList()
    @Volatile private var audioDecoderName: String? = null
    @Volatile private var failedAudioCodecName: String? = null
    @Volatile private var failedAudioMimeType: String? = null
    @Volatile private var audioCodecErrorObserved = false
    @Volatile private var audioFallbackAttempted = false
    @Volatile private var alternateAudioFallbackAttempted = false

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
        fun isAudioFallbackRetrying(player: ExoPlayer): Boolean =
            MediaPlayerControllerFactory.audioFallbackRetrying(player)

        fun getLibassRelay(player: ExoPlayer): LibassEventRelay? =
            MediaPlayerControllerFactory.getLibassRelay(player)

        fun getSubtitleCoordinator(player: ExoPlayer): com.fluxa.app.player.subtitle.SubtitleCoordinator? =
            MediaPlayerControllerFactory.getSubtitleCoordinator(player)

        fun releaseExoPlayer(player: ExoPlayer) =
            MediaPlayerControllerFactory.releaseExoPlayer(player)

        fun primeHttpStream(
            context: Context,
            url: String,
            headers: Map<String, String>,
            primeBytes: Long = 2L * 1024L * 1024L,
        ) = MediaPlayerControllerFactory.primeHttpStream(context, url, headers, primeBytes)

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
            enableLibassRelay: Boolean = true,
            audioProcessingMode: String = "reference",
        ): ExoPlayer = MediaPlayerControllerFactory.createExoPlayer(
            context = context,
            audioDecoderMode = audioDecoderMode,
            audioProcessingMode = audioProcessingMode,
            preferredAudioLanguage = preferredAudioLanguage,
            bufferCacheMb = bufferCacheMb,
            forwardBufferSeconds = forwardBufferSeconds,
            backBufferSeconds = backBufferSeconds,
            tunneledPlayback = tunneledPlayback,
            minBufferSeconds = minBufferSeconds,
            playbackBufferMs = playbackBufferMs,
            rebufferBufferMs = rebufferBufferMs,
            enableLibassRelay = enableLibassRelay,
        )
    }

    init {
        MediaPlayerControllerFactory.setAudioRouteListener(exoPlayer) {
            // A fallback track selected by the app is only valid for the
            // route that caused it. A user-selected track remains explicit.
            if (automaticAudioFallbackSelection) {
                automaticAudioFallbackSelection = false
                manualAudioSelection = false
            }
            audioFallbackAttempted = false
            alternateAudioFallbackAttempted = false
            audioCodecErrorObserved = false
            failedAudioCodecName = null
            failedAudioMimeType = null
            updateTracks(exoPlayer.currentTracks)
            updateTechnicalInfo()
        }
        exoPlayer.addListener(object : Player.Listener {
            override fun onTracksChanged(tracks: Tracks) {
                updateTracks(tracks)
                if (!manualAudioSelection && !applyingAudioPreference) {
                    selectBestAudioTrack()
                }
                updateTechnicalInfo()
            }
            override fun onPlaybackStateChanged(state: Int) {
                if (state == Player.STATE_READY) {
                    MediaPlayerControllerFactory.setAudioFallbackRetrying(exoPlayer, false)
                    updateTracks(exoPlayer.currentTracks)
                }
                updateTechnicalInfo()
            }
            override fun onVideoSizeChanged(videoSize: VideoSize) { updateTechnicalInfo() }
            override fun onVolumeChanged(volume: Float) { updateTechnicalInfo() }
            override fun onDeviceVolumeChanged(volume: Int, muted: Boolean) { updateTechnicalInfo() }
            override fun onPlayerError(error: PlaybackException) {
                val retryWasInProgress = MediaPlayerControllerFactory.audioFallbackRetrying(exoPlayer)
                if (retryWasInProgress) {
                    // The FFmpeg retry itself failed. Allow the UI to show the
                    // final error after we try an alternate compatible track.
                    MediaPlayerControllerFactory.setAudioFallbackRetrying(exoPlayer, false)
                }
                val codecName = failedAudioCodecName ?: findAudioCodecName(error)
                val audioSinkFailure = isAudioSinkFailure(error)
                // Media3 can report an audio renderer failure without a
                // codecInfo (for example when no MediaCodec advertises the
                // MIME at all). The renderer format is still authoritative;
                // use it to enter the FFmpeg/alternate-track recovery path.
                val audioRendererFailure = isAudioRendererFailure(error)
                if (audioFallbackAttempted) {
                    if (!alternateAudioFallbackAttempted &&
                        (codecName != null || audioCodecErrorObserved || audioSinkFailure || audioRendererFailure)
                    ) {
                        selectCompatibleAudioFallback()
                    }
                } else {
                    val ffmpegRetryStarted = retryAudioWithFfmpegIfNeeded(
                        error,
                        forceForAudioSinkFailure = audioSinkFailure || audioRendererFailure,
                    )
                    if (!ffmpegRetryStarted &&
                        (codecName != null || audioCodecErrorObserved || audioSinkFailure || audioRendererFailure)
                    ) {
                        selectCompatibleAudioFallback()
                    }
                }
            }
        })
        exoPlayer.addAnalyticsListener(object : AnalyticsListener {
            override fun onVideoInputFormatChanged(
                eventTime: AnalyticsListener.EventTime,
                format: Format,
                decoderReuseEvaluation: androidx.media3.exoplayer.DecoderReuseEvaluation?
            ) {
                updateTechnicalInfo()
                val ctx = MediaPlayerControllerFactory.requestContext(exoPlayer) ?: return
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

            override fun onAudioDecoderInitialized(
                eventTime: AnalyticsListener.EventTime,
                decoderName: String,
                initializedTimestampMs: Long,
                initializationDurationMs: Long,
            ) {
                audioDecoderName = decoderName
                updateTechnicalInfo()
            }

            override fun onAudioCodecError(
                eventTime: AnalyticsListener.EventTime,
                audioCodecError: Exception,
            ) {
                audioCodecErrorObserved = true
                findAudioCodecName(audioCodecError)?.let { codecName ->
                    failedAudioCodecName = codecName
                    MediaPlayerControllerFactory.blacklistAudioCodec(exoPlayer, codecName)
                }
                selectedAudioMimeType()?.let { mimeType ->
                    // Exclude every hardware decoder for this MIME during
                    // the retry; another hardware codec can fail identically.
                    failedAudioMimeType = mimeType
                    MediaPlayerControllerFactory.blacklistAudioMimeType(exoPlayer, mimeType)
                }
            }

            override fun onAudioDecoderReleased(
                eventTime: AnalyticsListener.EventTime,
                decoderName: String,
            ) {
                // A replacement decoder can be initialized before the old
                // one emits its release callback. The active name is cleared
                // by onAudioDisabled instead of trusting callback ordering.
            }

            override fun onAudioDisabled(
                eventTime: AnalyticsListener.EventTime,
                counters: androidx.media3.exoplayer.DecoderCounters,
            ) {
                audioDecoderName = null
                updateTechnicalInfo()
            }

            override fun onDroppedVideoFrames(eventTime: AnalyticsListener.EventTime, droppedFrames: Int, elapsedMs: Long) {
                updateTechnicalInfo(extraDroppedFrames = droppedFrames)
            }

            override fun onBandwidthEstimate(eventTime: AnalyticsListener.EventTime, totalLoadTimeMs: Int, totalBytesLoaded: Long, bitrateEstimate: Long) {
                MediaPlayerControllerFactory.saveBandwidthEstimate(context.applicationContext, bitrateEstimate)
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
            audioDecoderMode = audioDecoderMode,
            audioDecoderName = audioDecoderName,
            audioProcessingMode = MediaPlayerControllerFactory.audioProcessingMode(exoPlayer),
            audioOffloadActive = MediaPlayerControllerFactory.audioOffloadActive(exoPlayer),
            audioSinkStatus = MediaPlayerControllerFactory.audioSinkRuntimeStatus(exoPlayer),
        )
        val ctx = MediaPlayerControllerFactory.requestContext(exoPlayer)
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
                        sampleMimeType = format.sampleMimeType,
                        bitrate = format.bitrate.takeIf { it > 0 }?.toLong(),
                        sampleRate = format.sampleRate.takeIf { it > 0 },
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
                        sampleMimeType = format.sampleMimeType,
                        containerTrackId = format.id
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
        val relay = MediaPlayerControllerFactory.getLibassRelay(exoPlayer)
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
        if (track.type == C.TRACK_TYPE_AUDIO) {
            manualAudioSelection = true
            automaticAudioFallbackSelection = false
            _currentAudio.value = track
        }
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
        MediaPlayerControllerFactory.resetAudioCodecFailures(exoPlayer)
        // A manual choice belongs to the current media item. The next item must
        // be resolved again from language preference and output quality.
        manualAudioSelection = false
        automaticAudioFallbackSelection = false
        audioDecoderName = null
        failedAudioCodecName = null
        failedAudioMimeType = null
        audioCodecErrorObserved = false
        audioFallbackAttempted = false
        alternateAudioFallbackAttempted = false
        MediaPlayerControllerFactory.setAudioFallbackRetrying(exoPlayer, false)
        currentUrl = url
        currentStream = stream
        currentExternalSubtitles = subtitles
        LibassDebugLog.d(
            "prepare ExoPlayer url=${LibassDebugLog.urlSummary(url)} externalSubtitles=${subtitles.size} streamTitle=${stream?.effectiveFilename ?: stream?.rawDisplayTitle}"
        )
        val headersForPlayback = headers.orEmpty()
        val streamTitle = stream?.effectiveFilename ?: stream?.rawDisplayTitle
        MediaPlayerControllerFactory.requestContext(exoPlayer)?.let { ctx ->
            if (ctx.videoEffectsActive) {
                exoPlayer.setVideoEffects(emptyList())
                ctx.videoEffectsActive = false
            }
            ctx.streamHeaders = headersForPlayback
            MediaPlayerControllerFactory.getLibassRelay(exoPlayer)?.resetFonts()
            if (MediaPlayerControllerFactory.shouldScanEmbeddedAssFonts(url, streamTitle)) {
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

    private fun findAudioCodecName(error: Throwable?): String? {
        var current = error
        repeat(16) {
            when (current) {
                is androidx.media3.exoplayer.mediacodec.MediaCodecDecoderException ->
                    current.codecInfo?.name?.let { return it }
                is androidx.media3.exoplayer.mediacodec.MediaCodecRenderer.DecoderInitializationException ->
                    current.codecInfo?.name?.let { return it }
            }
            current = current?.cause
        }
        return null
    }

    private fun isAudioSinkFailure(error: Throwable?): Boolean {
        var current = error
        repeat(16) {
            when (current) {
                is AudioSink.InitializationException -> if (!current.isRecoverable) return true
                is AudioSink.ConfigurationException -> return true
                is AudioSink.WriteException -> if (!current.isRecoverable) return true
            }
            current = current?.cause
        }
        return false
    }

    private fun isAudioRendererFailure(error: Throwable?): Boolean {
        var current = error
        repeat(16) {
            if (current is ExoPlaybackException &&
                current.type == ExoPlaybackException.TYPE_RENDERER &&
                current.rendererFormat?.sampleMimeType?.let(MimeTypes::isAudio) == true
            ) {
                return true
            }
            current = current?.cause
        }
        return false
    }

    private fun rendererAudioMimeType(error: Throwable?): String? {
        var current = error
        repeat(16) {
            if (current is ExoPlaybackException &&
                current.type == ExoPlaybackException.TYPE_RENDERER
            ) {
                return current.rendererFormat?.sampleMimeType?.takeIf(MimeTypes::isAudio)
            }
            current = current?.cause
        }
        return null
    }

    private fun selectedAudioMimeType(): String? = exoPlayer.currentTracks.groups
        .asSequence()
        .filter { it.type == C.TRACK_TYPE_AUDIO }
        .flatMap { group ->
            (0 until group.length).asSequence()
                .filter(group::isTrackSelected)
                .map(group::getTrackFormat)
        }
        .mapNotNull { it.sampleMimeType }
        .firstOrNull()
        ?: exoPlayer.audioFormat?.sampleMimeType

    private fun retryAudioWithFfmpegIfNeeded(
        error: PlaybackException,
        forceForAudioSinkFailure: Boolean = false,
    ): Boolean {
        if (audioFallbackAttempted || audioDecoderMode == "hw_only") return false
        val codecName = failedAudioCodecName ?: findAudioCodecName(error)
        if (
            (codecName.isNullOrBlank() && !audioCodecErrorObserved && !forceForAudioSinkFailure) ||
            !AudioDecoderSupport.isFfmpegAvailable()
        ) return false
        codecName?.takeIf { it.isNotBlank() }?.let {
            MediaPlayerControllerFactory.blacklistAudioCodec(exoPlayer, it)
        }
        (selectedAudioMimeType() ?: rendererAudioMimeType(error))?.let { mimeType ->
            failedAudioMimeType = mimeType
            MediaPlayerControllerFactory.blacklistAudioMimeType(exoPlayer, mimeType)
        } ?: return false
        audioFallbackAttempted = true
        MediaPlayerControllerFactory.setAudioFallbackRetrying(exoPlayer, true)
        val started = MediaPlayerControllerFactory.retryAudioWithFfmpeg(exoPlayer)
        if (started) {
            audioDecoderName = null
            updateTechnicalInfo()
        } else {
            MediaPlayerControllerFactory.setAudioFallbackRetrying(exoPlayer, false)
        }
        return started
    }

    /**
     * Last-resort audio-only recovery. This is intentionally reached only
     * after a decoder-specific error; network/video failures must not change
     * the user's selected audio track.
     */
    private fun selectCompatibleAudioFallback() {
        val selectedAudioIds = exoPlayer.currentTracks.groups
            .mapIndexedNotNull { groupIndex, group ->
                if (group.type != C.TRACK_TYPE_AUDIO) return@mapIndexedNotNull null
                (0 until group.length)
                    .firstOrNull(group::isTrackSelected)
                    ?.let { "audio_$groupIndex-$it" }
            }
            .toSet()
        val failedMime = failedAudioMimeType?.trim()?.lowercase()
        val tracks = _availableAudios.value
            .filter {
                    it.isSupported &&
                    it.id !in selectedAudioIds &&
                    (failedMime == null || it.sampleMimeType?.trim()?.lowercase() != failedMime)
            }
        if (tracks.isEmpty()) return
        val capabilities = AudioCapabilityResolver.resolve(
            context,
            AudioCapabilityResolver.mediaAudioAttributes(),
        )
        val best = AudioTrackQualityPolicy.choose(
            tracks = tracks,
            preferredLanguage = preferredAudioLanguageCode,
            passthroughTrackIds = passthroughTrackIds(tracks, capabilities),
            supportedSampleRates = capabilities.pcmSampleRates,
            maxPcmChannels = capabilities.maxPcmChannels,
        ) ?: return
        val group = exoPlayer.currentTracks.groups.getOrNull(best.groupIndex) ?: return
        val positionMs = exoPlayer.currentPosition.coerceAtLeast(0L)
        val shouldPlay = exoPlayer.playWhenReady
        alternateAudioFallbackAttempted = true
        runCatching {
            exoPlayer.trackSelectionParameters = exoPlayer.trackSelectionParameters
                .buildUpon()
                .setOverrideForType(TrackSelectionOverride(group.mediaTrackGroup, best.trackIndex))
                .build()
            // This is an automatic recovery choice, but it must stay in place
            // for the current item. Otherwise onTracksChanged immediately
            // re-runs the quality policy and can switch back to the codec that
            // just failed (especially when the failed track has a higher
            // lossless codec rank). A new item or an explicit user choice
            // clears manualAudioSelection below.
            manualAudioSelection = true
            automaticAudioFallbackSelection = true
            exoPlayer.stop()
            exoPlayer.seekTo(positionMs)
            exoPlayer.prepare()
            exoPlayer.playWhenReady = shouldPlay
            updateTechnicalInfo()
        }.onFailure {
            LibassDebugLog.d("audio fallback track failed id=${best.id}: ${it.message}")
        }
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
        MediaPlayerControllerFactory.requestContext(exoPlayer)?.dvProxyPlanDebug = debug
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
        currentExternalSubtitles = (currentExternalSubtitles + subtitles).distinctBy { it.url }

        val (assSubtitles, textSubtitles) = subtitles.partition { subtitle ->
            subtitleMimeType(subtitle.url) == SubtitleFormatSupport.SSA
        }

        textSubtitles.firstOrNull()?.let { subtitle ->
            val format = sidecarSubtitleFormat(subtitle.url)
            LibassDebugLog.d(
                "hot-attaching sidecar subtitle via SubtitleCoordinator url=${LibassDebugLog.urlSummary(subtitle.url)} format=$format label=${subtitle.label} lang=${subtitle.language} (no MediaSource rebuild)"
            )
            MediaPlayerControllerFactory.getSubtitleCoordinator(exoPlayer)?.selectSidecar(
                com.fluxa.app.player.subtitle.SubtitleSource.Sidecar(url = subtitle.url, format = format)
            )
        }

        if (assSubtitles.isEmpty()) return
        val currentItem = exoPlayer.currentMediaItem ?: run {
            LibassDebugLog.w("addExternalSubtitles ignored ASS subtitles because current media item is null")
            return
        }
        val subtitleConfigurations = assSubtitles.mapNotNull { subtitle ->
            val subtitleUrl = subtitle.url.takeIf { it.isNotBlank() } ?: return@mapNotNull null
            val mimeType = subtitleMimeType(subtitleUrl)
            LibassDebugLog.d(
                "adding external ASS subtitle url=${LibassDebugLog.urlSummary(subtitleUrl)} mime=$mimeType label=${subtitle.label} lang=${subtitle.language}"
            )
            MediaItem.SubtitleConfiguration.Builder(Uri.parse(subtitleUrl))
                .setMimeType(mimeType)
                .setLanguage(subtitle.language)
                .setLabel(subtitle.label ?: subtitle.language)
                .build()
        }
        if (subtitleConfigurations.isEmpty()) return
        val currentPosition = exoPlayer.currentPosition.coerceAtLeast(0L)
        val wasPlaying = exoPlayer.playWhenReady
        LibassDebugLog.d("rebuilding media item with added ASS subtitles count=${subtitleConfigurations.size} positionMs=$currentPosition wasPlaying=$wasPlaying")
        exoPlayer.setMediaItem(currentItem.buildUpon().setSubtitleConfigurations(subtitleConfigurations).build(), currentPosition)
        exoPlayer.prepare()
        exoPlayer.playWhenReady = wasPlaying
    }

    private fun subtitleMimeType(url: String): String {
        return SubtitleFormatSupport.mimeTypeForUrl(url)
    }

    private fun sidecarSubtitleFormat(url: String): com.fluxa.app.player.subtitle.SubtitleFormat {
        return when (subtitleMimeType(url)) {
            SubtitleFormatSupport.WEB_VTT -> com.fluxa.app.player.subtitle.SubtitleFormat.WEBVTT
            SubtitleFormatSupport.TTML -> com.fluxa.app.player.subtitle.SubtitleFormat.TTML
            else -> com.fluxa.app.player.subtitle.SubtitleFormat.SRT
        }
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
        // A new explicit language preference supersedes an older per-item manual choice.
        manualAudioSelection = false
        automaticAudioFallbackSelection = false
        preferredAudioLanguageCode = languageCode
        exoPlayer.trackSelectionParameters = exoPlayer.trackSelectionParameters
            .buildUpon()
            .setPreferredAudioLanguage(languageCode)
            .build()
        selectBestAudioTrack()
    }

    private fun selectBestAudioTrack() {
        if (manualAudioSelection || applyingAudioPreference) return
        val audioTracks = _availableAudios.value.filter { it.isSupported }
        if (audioTracks.isEmpty()) return
        val outputCapabilities = AudioCapabilityResolver.resolve(
            context,
            AudioCapabilityResolver.mediaAudioAttributes()
        )
        val passthroughTrackIds = passthroughTrackIds(audioTracks, outputCapabilities)
        val best = AudioTrackQualityPolicy.choose(
            tracks = audioTracks,
            preferredLanguage = preferredAudioLanguageCode,
            passthroughTrackIds = passthroughTrackIds,
            supportedSampleRates = outputCapabilities.pcmSampleRates,
            maxPcmChannels = outputCapabilities.maxPcmChannels,
        ) ?: return
        if (best.isSelected) return
        val group = exoPlayer.currentTracks.groups.getOrNull(best.groupIndex) ?: return
        applyingAudioPreference = true
        try {
            exoPlayer.trackSelectionParameters = exoPlayer.trackSelectionParameters
                .buildUpon()
                .setOverrideForType(TrackSelectionOverride(group.mediaTrackGroup, best.trackIndex))
                .build()
        } finally {
            applyingAudioPreference = false
        }
    }

    private fun passthroughTrackIds(
        tracks: List<MediaTrack>,
        capabilities: AudioOutputCapabilities,
    ): Set<String> {
        if (MediaPlayerControllerFactory.audioProcessingMode(exoPlayer) != "reference") {
            return emptySet()
        }
        val attributes = AudioCapabilityResolver.mediaAudioAttributes()
        return tracks.mapNotNull { track ->
            val format = exoPlayer.currentTracks.groups
                .getOrNull(track.groupIndex)
                ?.takeIf { it.type == C.TRACK_TYPE_AUDIO }
                ?.getTrackFormat(track.trackIndex)
                ?: return@mapNotNull null
            track.id.takeIf { capabilities.supportsPassthrough(format, attributes) }
        }.toSet()
    }

}
