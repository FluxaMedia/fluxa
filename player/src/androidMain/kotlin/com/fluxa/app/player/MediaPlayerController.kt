package com.fluxa.app.player

import com.fluxa.app.shared.feature.player.MediaTrack

import android.content.Context
import androidx.media3.common.*
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.analytics.AnalyticsListener
import android.net.Uri
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

import androidx.media3.common.TrackSelectionOverride
import com.fluxa.app.core.rust.models.NativeDvProxyPlan

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
        ): ExoPlayer = MediaPlayerControllerFactory.createExoPlayer(
            context = context,
            audioDecoderMode = audioDecoderMode,
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
            audioDecoderMode = audioDecoderMode
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
        preferredAudioLanguageCode = languageCode
        exoPlayer.trackSelectionParameters = exoPlayer.trackSelectionParameters
            .buildUpon()
            .setPreferredAudioLanguage(languageCode)
            .build()
    }
}
