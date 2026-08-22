package com.fluxa.app.player

import com.fluxa.app.shared.feature.player.Chapter
import com.fluxa.app.shared.feature.player.MediaTrack
import com.fluxa.app.shared.feature.player.AudioChannelLayoutPolicy
import com.fluxa.app.shared.feature.player.AudioPassthroughPolicy
import com.fluxa.app.shared.feature.player.AudioTrackQualityPolicy

import android.content.Context
import android.graphics.Color
import android.media.AudioFormat
import android.media.AudioManager
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.view.Surface
import androidx.media3.common.C
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeSubtitleSize
import com.fluxa.app.data.local.safeSubtitleOutlineOpacity
import com.fluxa.app.data.local.safeSubtitleTextOpacity
import com.fluxa.app.data.local.safeSubtitleColor
import com.fluxa.app.data.local.safeSubtitleOutlineColor
import com.fluxa.app.data.local.safeSubtitleBackgroundOpacity
import com.fluxa.app.data.local.safeSubtitleBackgroundColor
import com.fluxa.app.data.local.safeSubtitleShadow
import com.fluxa.app.data.remote.Stream
import dev.jdtech.mpv.MPVLib
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import java.util.Locale

data class MpvPlaybackState(
    val isPlaying: Boolean = false,
    val isBuffering: Boolean = true,
    val isVideoReady: Boolean = false,
    val positionMs: Long = 0L,
    val durationMs: Long = 0L,
    val technicalInfo: String? = null,
    val error: String? = null
)

private data class PendingMpvLoad(
    val url: String,
    val stream: Stream?,
    val subtitles: List<ExternalSubtitleTrack>,
    val startPositionMs: Long,
    val preferredAudioLanguage: String?,
    val preferredSubtitleLanguage: String?,
    val allowHardwareDecode: Boolean = true,
    val shouldPlay: Boolean = true
)

internal fun isRouteOwnedAudioOption(key: String): Boolean = when (key.trim().lowercase(Locale.ROOT)) {
    "audio-spdif", "af", "audio-channels" -> true
    else -> false
}

/**
 * MPV's Android audio backend has emitted several different diagnostics for
 * the same sink/bitstream failure across Android and libmpv versions. Keep
 * the classification independent and tested so a wording change cannot leave
 * passthrough stuck until the user manually changes tracks.
 */
internal object MpvAudioErrorPolicy {
    fun isPassthroughFailure(
        lowercaseLogLine: String,
        passthroughConfigured: Boolean,
        fallbackUsed: Boolean,
    ): Boolean {
        if (!passthroughConfigured || fallbackUsed) return false
        val line = lowercaseLogLine.lowercase(Locale.ROOT)
        val outputMentioned = line.contains("audio output") ||
            line.contains("audio driver") ||
            line.contains("audiotrack") ||
            line.contains("[audiotrack]") ||
            line.contains("ao/") ||
            line.startsWith("ao:") ||
            line.contains("spdif")
        val failureMentioned = line.contains("failed") ||
            line.contains("error") ||
            line.contains("could not") ||
            line.contains("cannot") ||
            line.contains("unavailable") ||
            line.contains("unsupported") ||
            line.contains("not supported") ||
            line.contains("refused") ||
            line.contains("no such")
        return outputMentioned && failureMentioned
    }
}

private data class MpvMemoryPolicy(
    val cacheSeconds: Int,
    val demuxerMaxBytes: String,
    val readaheadSeconds: Int
)

class MpvEmbeddedPlayer(
    context: Context,
    private val customOptions: String = "",
    private val audioProcessingMode: String = "reference",
) {
    private val appContext = context.applicationContext
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private val mpv: MPVLib = requireNotNull(MPVLib.create(appContext)) { "libmpv could not be created" }
    private var pollJob: Job? = null
    private var initialized = false
    private var currentSubtitles: List<ExternalSubtitleTrack> = emptyList()
    @Volatile private var hasLoadedCurrentFile = false
    @Volatile private var lastErrorLog: String? = null
    @Volatile private var surfaceAttached = false
    @Volatile private var pendingLoad: PendingMpvLoad? = null
    @Volatile private var activeLoad: PendingMpvLoad? = null
    @Volatile private var externalSubtitlesAdded = false
    private var hardwareWatchdogJob: Job? = null
    private val voInUse = "gpu"
    @Volatile private var hardwareDecodeEnabled = true
    @Volatile private var hardwareFallbackUsed = false
    @Volatile private var audioPassthroughConfigured = false
    @Volatile private var audioPassthroughFallbackUsed = false
    @Volatile private var audioRouteSignature: String? = null
    @Volatile private var lastTrackListKey = ""
    @Volatile private var pendingPlaybackStateUpdate = false
    private var preferredAudioLanguageCode = ""
    private var manualAudioSelection = false
    private var applyingAudioPreference = false
    private var audioRouteCallback: android.media.AudioDeviceCallback? = null
    private var audioSpatializer: android.media.Spatializer? = null
    private var audioSpatializerListener: android.media.Spatializer.OnSpatializerStateChangedListener? = null
    private var audioRouteRefreshJob: Job? = null

    private val _state = MutableStateFlow(MpvPlaybackState())
    val state: StateFlow<MpvPlaybackState> = _state

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

    private val _videoAspectRatio = MutableStateFlow<Float?>(null)
    val videoAspectRatio: StateFlow<Float?> = _videoAspectRatio

    private val memoryPolicy = mpvMemoryPolicy()

    private val _chapters = MutableStateFlow<List<Chapter>>(emptyList())
    val chapters: StateFlow<List<Chapter>> = _chapters

    fun initialize() {
        if (initialized) return
        applyPerformanceOptions()
        mpv.init()
        runCatching { mpv.setOptionString("force-window", "no") }
        runCatching { mpv.setOptionString("idle", "once") }
        mpv.addObserver(object : MPVLib.EventObserver {
            override fun eventProperty(property: String) = schedulePlaybackStateUpdate()
            override fun eventProperty(property: String, value: Long) = schedulePlaybackStateUpdate()
            override fun eventProperty(property: String, value: Double) = schedulePlaybackStateUpdate()
            override fun eventProperty(property: String, value: Boolean) = schedulePlaybackStateUpdate()
            override fun eventProperty(property: String, value: String) = schedulePlaybackStateUpdate()
            override fun event(eventId: Int) {
                when (eventId) {
                    MPVLib.MpvEvent.MPV_EVENT_START_FILE -> {
                        hasLoadedCurrentFile = false
                        lastErrorLog = null
                        _chapters.value = emptyList()
                        _state.value = _state.value.copy(isBuffering = true, isVideoReady = false, error = null)
                    }
                    MPVLib.MpvEvent.MPV_EVENT_VIDEO_RECONFIG,
                    MPVLib.MpvEvent.MPV_EVENT_PLAYBACK_RESTART -> {
                        markPlaybackReady()
                        updateVideoAspectRatio()
                        updateChapters()
                    }
                    MPVLib.MpvEvent.MPV_EVENT_FILE_LOADED -> {
                        hasLoadedCurrentFile = true
                        addExternalSubtitlesOnce()
                        _state.value = _state.value.copy(isBuffering = true, error = null)
                    }
                    MPVLib.MpvEvent.MPV_EVENT_END_FILE -> {
                        _state.value = _state.value.copy(
                            isPlaying = false,
                            isBuffering = false,
                            error = if (hasLoadedCurrentFile) null else lastErrorLog
                        )
                    }
                    MPVLib.MpvEvent.MPV_EVENT_SHUTDOWN -> {
                        _state.value = _state.value.copy(isPlaying = false, isBuffering = false, isVideoReady = false)
                    }
                }
            }
        })
        mpv.addLogObserver(object : MPVLib.LogObserver {
            override fun logMessage(prefix: String, level: Int, text: String) {
                val line = text.trim()
                if (line.isBlank()) return
                Log.d("MpvEmbeddedPlayer", "[$prefix/$level] $line")
                val lower = line.lowercase(Locale.ROOT)
                if (
                    lower.contains("event: playback-restart") ||
                    lower.contains("event: video-reconfig")
                ) {
                    markPlaybackReady()
                }
                if (hardwareDecodeEnabled && isHardwareDecodeFailure(lower)) {
                    retryWithSoftwareDecode(line)
                    return
                }
                if (isAudioPassthroughFailure(lower)) {
                    retryWithPcmAudio(line)
                    return
                }
                if (
                    lower.contains("error") ||
                    (lower.contains("failed") && !lower.contains("failed sending hook")) ||
                    lower.contains("could not") ||
                    lower.contains("refused") ||
                    lower.contains("403") ||
                    lower.contains("404") ||
                    lower.contains("http error")
                ) {
                    lastErrorLog = line.take(180)
                }
            }
        })
        initialized = true
        registerAudioRouteCallback()
        startPolling()
    }

    fun attachSurface(surface: Surface) {
        onSurfaceCreatedByView(surface, 0, 0)
    }

    fun detachSurface() {
        onSurfaceDestroyedByView()
    }

    fun onSurfaceCreatedByView(surface: Surface, width: Int, height: Int) {
        initialize()
        runCatching { mpv.attachSurface(surface) }
        surfaceAttached = true
        runCatching { mpv.setOptionString("force-window", "yes") }
        runCatching { mpv.setOptionString("vo", voInUse) }
        if (width > 0 && height > 0) {
            runCatching { mpv.setPropertyString("android-surface-size", "${width}x$height") }
        }
        pendingLoad?.let { load ->
            pendingLoad = null
            loadFile(load)
        }
    }

    fun onSurfaceChangedByView(surface: Surface, width: Int, height: Int) {
        if (!surfaceAttached) {
            onSurfaceCreatedByView(surface, width, height)
            return
        }
        if (width > 0 && height > 0) {
            runCatching { mpv.setPropertyString("android-surface-size", "${width}x$height") }
        }
    }

    fun onSurfaceDestroyedByView() {
        surfaceAttached = false
        if (!initialized) return
        runCatching { mpv.setOptionString("vo", "null") }
        runCatching { mpv.setOptionString("force-window", "no") }
        runCatching { mpv.detachSurface() }
    }

    fun prepareAndPlay(
        url: String,
        stream: Stream?,
        subtitles: List<ExternalSubtitleTrack>,
        startPositionMs: Long,
        preferredAudioLanguage: String?,
        preferredSubtitleLanguage: String?,
        shouldPlay: Boolean = true
    ) {
        initialize()
        // Do not carry a manual track override into a different media item.
        manualAudioSelection = false
        lastTrackListKey = ""
        hardwareFallbackUsed = false
        audioPassthroughFallbackUsed = false
        _state.value = MpvPlaybackState(isBuffering = true)
        val load = PendingMpvLoad(
            url = url,
            stream = stream,
            subtitles = subtitles,
            startPositionMs = startPositionMs,
            preferredAudioLanguage = preferredAudioLanguage,
            preferredSubtitleLanguage = preferredSubtitleLanguage,
            shouldPlay = shouldPlay
        )
        if (!surfaceAttached) {
            pendingLoad = load
            return
        }
        loadFile(load)
    }

    fun addExternalSubtitles(subtitles: List<ExternalSubtitleTrack>) {
        if (subtitles.isEmpty()) return
        val existingUrls = currentSubtitles.map { it.url }.toSet()
        val newSubtitles = subtitles.filter { it.url !in existingUrls }
        if (newSubtitles.isEmpty()) return
        currentSubtitles = currentSubtitles + newSubtitles
        if (hasLoadedCurrentFile) {
            newSubtitles.forEach { subtitle ->
                runCatching {
                    mpv.command(
                        arrayOf(
                            "sub-add",
                            subtitle.url,
                            "auto",
                            subtitle.label ?: subtitle.language ?: "Subtitle",
                            subtitle.language.orEmpty()
                        )
                    )
                }
            }
            updateTracksFromProperties()
        }
    }

    private fun loadFile(load: PendingMpvLoad) {
        activeLoad = load
        hardwareDecodeEnabled = load.allowHardwareDecode
        currentSubtitles = load.subtitles
        applyStreamOptions(
            url = load.url,
            stream = load.stream,
            preferredAudioLanguage = load.preferredAudioLanguage,
            preferredSubtitleLanguage = load.preferredSubtitleLanguage
        )
        hasLoadedCurrentFile = false
        externalSubtitlesAdded = false
        lastErrorLog = null
        lastTrackListKey = ""
        _state.value = MpvPlaybackState(isBuffering = true)
        if (surfaceAttached) {
            runCatching { mpv.setOptionString("force-window", "yes") }
            runCatching { mpv.setOptionString("vo", voInUse) }
        }
        runCatching { mpv.setOptionString("hwdec", if (load.allowHardwareDecode) "auto-safe" else "no") }
        startHardwareWatchdog(load)
        val options = buildList {
            add("force-media-title=${load.stream?.effectiveFilename ?: load.url.substringAfterLast('/').ifBlank { "Fluxa" }}")
            if (load.startPositionMs > 0L) add("start=${String.format(Locale.US, "%.3f", load.startPositionMs / 1000.0)}")
        }.joinToString(",")
        if (options.isBlank()) {
            mpv.command(arrayOf("loadfile", load.url, "replace"))
        } else {
            mpv.command(arrayOf("loadfile", load.url, "replace", "-1", options))
        }
        setPaused(load.shouldPlay.not())
    }

    fun setPaused(paused: Boolean) {
        if (initialized) mpv.setPropertyBoolean("pause", paused)
        _state.value = _state.value.copy(isPlaying = !paused)
    }

    fun seekTo(positionMs: Long, exact: Boolean = true) {
        if (!initialized) return
        val mode = if (exact) "exact" else "keyframe"
        mpv.command(arrayOf("seek", String.format(Locale.US, "%.3f", positionMs / 1000.0), "absolute", mode))
        _state.value = _state.value.copy(positionMs = positionMs.coerceAtLeast(0L))
    }

    fun setSpeed(speed: Float) {
        if (initialized) mpv.setPropertyDouble("speed", speed.toDouble().coerceIn(0.25, 4.0))
    }

    fun setZoomed(zoomed: Boolean) {
        if (initialized) mpv.setPropertyDouble("panscan", if (zoomed) 1.0 else 0.0)
    }

    fun setAudioDelayMs(delayMs: Long) {
        if (initialized) mpv.setPropertyDouble("audio-delay", delayMs.coerceIn(-5_000L, 5_000L) / 1000.0)
    }

    fun setSubtitleDelayMs(delayMs: Long) {
        if (initialized) mpv.setPropertyDouble("sub-delay", delayMs.coerceIn(-5_000L, 5_000L) / 1000.0)
    }

    fun selectAudio(track: MediaTrack) {
        if (track.type != C.TRACK_TYPE_AUDIO || !initialized) return
        manualAudioSelection = true
        mpv.setPropertyString("aid", track.mpvTrackId())
        updateTracksFromProperties()
    }

    fun enableSubtitle(track: MediaTrack) {
        if (track.type != C.TRACK_TYPE_TEXT || !initialized) return
        mpv.setPropertyString("sid", track.mpvTrackId())
        updateTracksFromProperties()
    }

    fun disableSubtitles() {
        if (!initialized) return
        mpv.setPropertyString("sid", "no")
        _currentSubtitle.value = null
        updateTracksFromProperties()
    }

    fun applyPreferredAudioLanguage(languageCode: String) {
        if (languageCode.isBlank() || languageCode == "none" || !initialized) return
        // A new explicit language preference supersedes an older per-item manual choice.
        manualAudioSelection = false
        preferredAudioLanguageCode = languageCode
        mpv.setOptionString("alang", languageCode)
        selectBestAudioTrack()
    }

    fun applySubtitleStyle(profile: UserProfile?) {
        if (!initialized || profile == null) return
        val fontSize = profile.safeSubtitleSize.coerceIn(10f, 80f).toInt().toString()
        val outlineOpacity = profile.safeSubtitleOutlineOpacity
        runCatching { mpv.setOptionString("sub-font-size", fontSize) }
        runCatching { mpv.setOptionString("sub-color", profile.safeSubtitleColor.toMpvColor(profile.safeSubtitleTextOpacity)) }
        runCatching { mpv.setOptionString("sub-border-color", profile.safeSubtitleOutlineColor.toMpvColor(outlineOpacity)) }
        runCatching { mpv.setOptionString("sub-border-size", if (outlineOpacity > 0f) "3" else "0") }
        runCatching { mpv.setOptionString("sub-back-color", profile.safeSubtitleBackgroundColor.toMpvColor(profile.safeSubtitleBackgroundOpacity)) }
        runCatching { mpv.setOptionString("sub-shadow-offset", if (profile.safeSubtitleShadow) "1" else "0") }
    }

    fun stop() {
        pendingLoad = null
        activeLoad = null
        hardwareWatchdogJob?.cancel()
        hardwareWatchdogJob = null
        if (initialized) {
            runCatching { mpv.command(arrayOf("stop")) }
        }
        _state.value = MpvPlaybackState(isBuffering = false)
    }

    fun release() {
        pendingLoad = null
        activeLoad = null
        hardwareWatchdogJob?.cancel()
        hardwareWatchdogJob = null
        audioRouteRefreshJob?.cancel()
        audioRouteRefreshJob = null
        audioRouteCallback?.let { callback ->
            runCatching {
                appContext.getSystemService(AudioManager::class.java)
                    ?.unregisterAudioDeviceCallback(callback)
            }
        }
        audioRouteCallback = null
        if (Build.VERSION.SDK_INT >= 31) {
            val spatializer = audioSpatializer
            val listener = audioSpatializerListener
            if (spatializer != null && listener != null) {
                runCatching { spatializer.removeOnSpatializerStateChangedListener(listener) }
            }
        }
        audioSpatializer = null
        audioSpatializerListener = null
        pollJob?.cancel()
        if (initialized) {
            onSurfaceDestroyedByView()
            runCatching { mpv.destroy() }
            initialized = false
        }
    }

    private fun registerAudioRouteCallback() {
        if (Build.VERSION.SDK_INT < 23 || audioRouteCallback != null) return
        val manager = appContext.getSystemService(AudioManager::class.java) ?: return
        val callback = object : android.media.AudioDeviceCallback() {
            override fun onAudioDevicesAdded(addedDevices: Array<out android.media.AudioDeviceInfo>) {
                scheduleAudioRouteRefresh()
            }

            override fun onAudioDevicesRemoved(removedDevices: Array<out android.media.AudioDeviceInfo>) {
                scheduleAudioRouteRefresh()
            }
        }
        manager.registerAudioDeviceCallback(callback, Handler(Looper.getMainLooper()))
        audioRouteCallback = callback
        if (Build.VERSION.SDK_INT >= 31) {
            val spatializer = manager.spatializer
            val listener = object : android.media.Spatializer.OnSpatializerStateChangedListener {
                override fun onSpatializerEnabledChanged(
                    spatializer: android.media.Spatializer,
                    enabled: Boolean,
                ) = scheduleAudioRouteRefresh()

                override fun onSpatializerAvailableChanged(
                    spatializer: android.media.Spatializer,
                    available: Boolean,
                ) = scheduleAudioRouteRefresh()
            }
            spatializer.addOnSpatializerStateChangedListener(
                java.util.concurrent.Executor { command ->
                    Handler(Looper.getMainLooper()).post(command)
                },
                listener,
            )
            audioSpatializer = spatializer
            audioSpatializerListener = listener
        }
    }

    private fun scheduleAudioRouteRefresh() {
        audioRouteRefreshJob?.cancel()
        audioRouteRefreshJob = scope.launch {
            delay(150L)
            if (!initialized) return@launch
            val currentCapabilities = AudioCapabilityResolver.resolve(
                appContext,
                AudioCapabilityResolver.mediaAudioAttributes(),
            )
            if (currentCapabilities.routeSignature() == audioRouteSignature) return@launch
            val active = activeLoad ?: return@launch
            val positionMs = (mpv.getPropertyDouble("time-pos") ?: 0.0)
                .times(1000.0)
                .toLong()
                .coerceAtLeast(0L)
            val wasPlaying = !(mpv.getPropertyBoolean("pause") ?: true)
            audioPassthroughFallbackUsed = false
            applyPerformanceOptions()
            loadFile(active.copy(startPositionMs = positionMs))
            setPaused(!wasPlaying)
        }
    }

    private fun applyPerformanceOptions() {
        val configDir = appContext.filesDir.resolve("mpv").apply { mkdirs() }
        val cacheDir = appContext.cacheDir.resolve("mpv").apply { mkdirs() }
        val audioCapabilities = AudioCapabilityResolver.resolve(
            appContext,
            AudioCapabilityResolver.mediaAudioAttributes()
        )
        audioRouteSignature = audioCapabilities.routeSignature()
        val audioChannelPreference = AudioChannelLayoutPolicy.preferredLayouts(
            audioCapabilities.maxPcmChannels,
        )
        mapOf(
            "vo" to voInUse,
            "ao" to "audiotrack",
            "gpu-api" to "opengl",
            "hwdec" to "auto-safe",
            "gpu-context" to "android",
            "config" to "yes",
            "config-dir" to configDir.absolutePath,
            "gpu-shader-cache-dir" to cacheDir.absolutePath,
            "icc-cache-dir" to cacheDir.absolutePath,
            "video-sync" to "audio",
            "interpolation" to "no",
            "deband" to "no",
            "cache" to "yes",
            "cache-secs" to memoryPolicy.cacheSeconds.toString(),
            "demuxer-max-bytes" to memoryPolicy.demuxerMaxBytes,
            "demuxer-readahead-secs" to memoryPolicy.readaheadSeconds.toString(),
            "demuxer-lavf-o" to "allowed_extensions=ALL",
            "vd-lavc-threads" to "0",
            "ad-lavc-threads" to "0",
            "network-timeout" to "15",
            "keep-open" to "no",
            "audio-channels" to audioChannelPreference,
            "sub-auto" to "fuzzy",
            "sub-ass" to "yes",
            "sub-ass-override" to "scale",
            "sub-fix-timing" to "yes",
            "deinterlace" to "auto",
            "audio-display" to "no"
        ).forEach { (key, value) ->
            runCatching { mpv.setOptionString(key, value) }
        }
        // Custom options may still configure video, cache, subtitle, or other
        // engine details. Audio routing is different: these three options are
        // owned by the live sink capability resolver and must be applied last.
        // Otherwise a stale custom audio-spdif value could re-enable a
        // bitstream after HDMI/eARC was replaced by speakers or Bluetooth.
        applyCustomOptionLines(customOptions)
        // These options are sticky in libmpv. Clear them before resolving the
        // newly active route so an old HDMI passthrough profile cannot survive
        // a switch to speakers/Bluetooth (or vice versa).
        runCatching { mpv.setOptionString("audio-spdif", "") }
        val spdif = buildSpdifOption()
        audioPassthroughConfigured = !spdif.isNullOrBlank()
        spdif?.let {
            runCatching { mpv.setOptionString("audio-spdif", it) }
        }
        runCatching { mpv.setOptionString("af", "") }
        val audioFilter = when (audioProcessingMode) {
            "balanced" -> "lavfi=[acompressor=threshold=0.78:ratio=1.5:attack=30:release=300:link=maximum,alimiter=limit=0.98]"
            "night" -> "lavfi=[acompressor=threshold=0.55:ratio=3:attack=20:release=250:link=maximum,alimiter=limit=0.98]"
            else -> null
        }
        if (audioFilter != null) {
            runCatching {
                mpv.setOptionString("af", audioFilter)
            }
        }
    }

    private fun buildSpdifOption(): String? {
        // A passthrough failure is a runtime fact for this media item/route.
        // loadFile() is also used by the PCM retry, so do not immediately
        // reconstruct the same audio-spdif option from static capabilities.
        // Route refresh or a new media item explicitly clears this latch.
        if (audioProcessingMode != "reference" || audioPassthroughFallbackUsed) return null
        val encodings = AudioCapabilityResolver.resolve(
            appContext,
            AudioCapabilityResolver.mediaAudioAttributes()
        ).passthroughEncodings
        val formats = buildList {
            if (AudioFormat.ENCODING_DOLBY_TRUEHD in encodings) add("truehd")
            if (AudioFormat.ENCODING_E_AC3 in encodings ||
                (Build.VERSION.SDK_INT >= 28 && AudioFormat.ENCODING_E_AC3_JOC in encodings)
            ) add("eac3")
            if (AudioFormat.ENCODING_AC3 in encodings) add("ac3")
            if (Build.VERSION.SDK_INT >= 29 && AudioFormat.ENCODING_AC4 in encodings) add("ac4")
            if (AudioFormat.ENCODING_DTS_HD in encodings ||
                (Build.VERSION.SDK_INT >= 34 && AudioFormat.ENCODING_DTS_HD_MA in encodings)
            ) add("dts-hd")
            if (AudioFormat.ENCODING_DTS in encodings) add("dts")
        }
        return formats.takeIf { it.isNotEmpty() }?.joinToString(",")
    }

    private fun AudioOutputCapabilities.routeSignature(): String =
        "$routeDescription|$maxPcmChannels|${pcmSampleRates.sorted()}|${passthroughEncodings.sorted()}|$speakerLayoutMasks|$spatialLayoutMasks"

    private fun applyCustomOptionLines(raw: String) {
        if (raw.isBlank()) return
        raw.lines().forEach { line ->
            val trimmed = line.trim()
            if (trimmed.isBlank() || trimmed.startsWith('#')) return@forEach
            val eq = trimmed.indexOf('=')
            if (eq > 0) {
                val key = trimmed.substring(0, eq).trim()
                val value = trimmed.substring(eq + 1).trim()
                if (key.isNotBlank() && !isRouteOwnedAudioOption(key)) {
                    runCatching { mpv.setOptionString(key, value) }
                }
            }
        }
    }

    private fun applyStreamOptions(url: String, stream: Stream?, preferredAudioLanguage: String?, preferredSubtitleLanguage: String?) {
        val headers = StreamRequestPolicy.headersFor(url, stream?.resolveHeaders().orEmpty())
        runCatching { mpv.setOptionString("user-agent", StreamRequestPolicy.DEFAULT_USER_AGENT) }
        runCatching { mpv.setOptionString("referrer", StreamRequestPolicy.refererFor(url).orEmpty()) }
        runCatching {
            mpv.setOptionString(
                "http-header-fields",
                headers.entries.joinToString(",") { "${it.key}: ${it.value}" }
            )
        }
        runCatching { mpv.setOptionString("alang", preferredAudioLanguage?.takeUnless { it == "none" }.orEmpty()) }
        runCatching { mpv.setOptionString("slang", preferredSubtitleLanguage?.takeUnless { it == "none" }.orEmpty()) }
    }

    private fun addExternalSubtitles() {
        currentSubtitles.forEach { subtitle ->
            runCatching {
                mpv.command(
                    arrayOf(
                        "sub-add",
                        subtitle.url,
                        "auto",
                        subtitle.label ?: subtitle.language ?: "Subtitle",
                        subtitle.language.orEmpty()
                    )
                )
            }
        }
    }

    private fun markPlaybackReady() {
        hardwareWatchdogJob?.cancel()
        hardwareWatchdogJob = null
        if (!hasLoadedCurrentFile) hasLoadedCurrentFile = true
        addExternalSubtitlesOnce()
        _state.value = _state.value.copy(isBuffering = false, isVideoReady = true, error = null)
        updatePositionOnly()
        updateTechnicalInfo()
    }

    private fun addExternalSubtitlesOnce() {
        if (externalSubtitlesAdded) return
        externalSubtitlesAdded = true
        addExternalSubtitles()
    }

    private fun updateVideoAspectRatio() {
        val width = mpv.getPropertyInt("dwidth") ?: return
        val height = mpv.getPropertyInt("dheight") ?: return
        if (width > 0 && height > 0) {
            _videoAspectRatio.value = width.toFloat() / height.toFloat()
        }
    }

    private fun updateChapters() {
        if (_chapters.value.isNotEmpty()) return
        val count = mpv.getPropertyInt("chapters") ?: 0
        if (count <= 0) return
        val chapters = (0 until count).mapNotNull { index ->
            val startSec = mpv.getPropertyDouble("chapter-list/$index/time") ?: return@mapNotNull null
            val title = mpv.getPropertyString("chapter-list/$index/title").orEmpty()
            Chapter(title = title, startMs = (startSec * 1000.0).toLong())
        }
        if (chapters.isNotEmpty()) {
            _chapters.value = chapters
        }
    }

    private fun isHardwareDecodeFailure(lowercaseLogLine: String): Boolean {
        return lowercaseLogLine.contains("mediacodec") && (
            lowercaseLogLine.contains("failed") ||
                lowercaseLogLine.contains("both surface and native_window are null") ||
                lowercaseLogLine.contains("codec failed to configure") ||
                lowercaseLogLine.contains("error")
            )
    }

    private fun retryWithSoftwareDecode(reason: String) {
        if (hardwareFallbackUsed) return
        val load = activeLoad ?: pendingLoad ?: return
        hardwareFallbackUsed = true
        hardwareDecodeEnabled = false
        hardwareWatchdogJob?.cancel()
        hardwareWatchdogJob = null
        lastErrorLog = null
        Log.w("MpvEmbeddedPlayer", "Hardware decode failed, retrying with software decode: $reason")
        scope.launch {
            runCatching { mpv.command(arrayOf("stop")) }
            loadFile(load.copy(allowHardwareDecode = false))
        }
    }

    private fun isAudioPassthroughFailure(lowercaseLogLine: String): Boolean {
        return MpvAudioErrorPolicy.isPassthroughFailure(
            lowercaseLogLine = lowercaseLogLine,
            passthroughConfigured = audioPassthroughConfigured,
            fallbackUsed = audioPassthroughFallbackUsed,
        )
    }

    private fun retryWithPcmAudio(reason: String) {
        if (!audioPassthroughConfigured || audioPassthroughFallbackUsed) return
        val load = activeLoad ?: pendingLoad ?: return
        audioPassthroughFallbackUsed = true
        audioPassthroughConfigured = false
        val positionMs = (mpv.getPropertyDouble("time-pos") ?: load.startPositionMs / 1000.0)
            .times(1000.0)
            .toLong()
            .coerceAtLeast(0L)
        val wasPlaying = !(mpv.getPropertyBoolean("pause") ?: !load.shouldPlay)
        Log.w("MpvEmbeddedPlayer", "Audio passthrough failed, retrying with PCM: $reason")
        scope.launch {
            runCatching { mpv.command(arrayOf("stop")) }
            runCatching { mpv.setOptionString("audio-spdif", "") }
            loadFile(load.copy(startPositionMs = positionMs, shouldPlay = wasPlaying))
        }
    }

    private fun startHardwareWatchdog(load: PendingMpvLoad) {
        hardwareWatchdogJob?.cancel()
        hardwareWatchdogJob = null
        if (!load.allowHardwareDecode) return
        hardwareWatchdogJob = scope.launch {
            delay(HARDWARE_VIDEO_READY_TIMEOUT_MS)
            if (
                activeLoad?.url == load.url &&
                hardwareDecodeEnabled &&
                !_state.value.isVideoReady &&
                surfaceAttached
            ) {
                retryWithSoftwareDecode("video output was not ready after ${HARDWARE_VIDEO_READY_TIMEOUT_MS}ms")
            }
        }
    }

    private fun startPolling() {
        pollJob?.cancel()
        pollJob = scope.launch {
            while (isActive) {
                updatePositionOnly()
                delay(
                    when {
                        !_state.value.isPlaying -> 1_500L
                        !surfaceAttached -> 1_000L
                        else -> 500L
                    }
                )
            }
        }
    }

    private fun mpvMemoryPolicy(): MpvMemoryPolicy {
        val activityManager = appContext.getSystemService(Context.ACTIVITY_SERVICE) as? android.app.ActivityManager
        val memoryInfo = android.app.ActivityManager.MemoryInfo()
        activityManager?.getMemoryInfo(memoryInfo)
        val totalRamMb = memoryInfo.totalMem / (1024L * 1024L)
        val isTelevision = appContext.packageManager.hasSystemFeature(android.content.pm.PackageManager.FEATURE_LEANBACK)
        return when {
            activityManager?.isLowRamDevice == true || (isTelevision && totalRamMb <= 2_048L) ->
                MpvMemoryPolicy(30, "40MiB", 30)
            isTelevision && totalRamMb <= 3_072L ->
                MpvMemoryPolicy(45, "64MiB", 45)
            else ->
                MpvMemoryPolicy(60, "96MiB", 60)
        }
    }

    private fun schedulePlaybackStateUpdate() {
        if (pendingPlaybackStateUpdate) return
        pendingPlaybackStateUpdate = true
        scope.launch {
            pendingPlaybackStateUpdate = false
            updatePlaybackState()
        }
    }

    private fun updatePlaybackState() {
        if (!initialized) return
        val paused = mpv.getPropertyBoolean("pause") ?: !_state.value.isPlaying
        val idle = mpv.getPropertyBoolean("idle-active") ?: !hasLoadedCurrentFile
        if (!hasLoadedCurrentFile) {
            val next = _state.value.copy(isPlaying = !paused && !idle)
            if (next != _state.value) _state.value = next
            return
        }
        val position = mpv.getPropertyDouble("time-pos")?.secondsToMs() ?: _state.value.positionMs
        val duration = mpv.getPropertyDouble("duration")?.secondsToMs() ?: _state.value.durationMs
        val next = _state.value.copy(
            isPlaying = !paused && !idle,
            isBuffering = idle && !_state.value.isVideoReady,
            positionMs = position,
            durationMs = duration
        )
        if (next != _state.value) _state.value = next
        updateTracksFromProperties()
    }

    private fun updatePositionOnly() {
        if (!initialized) return
        if (!hasLoadedCurrentFile) {
            val paused = mpv.getPropertyBoolean("pause") ?: !_state.value.isPlaying
            val idle = mpv.getPropertyBoolean("idle-active") ?: true
            val next = _state.value.copy(isPlaying = !paused && !idle)
            if (next != _state.value) _state.value = next
            return
        }
        val position = mpv.getPropertyDouble("time-pos")?.secondsToMs() ?: _state.value.positionMs
        val duration = mpv.getPropertyDouble("duration")?.secondsToMs() ?: _state.value.durationMs
        val paused = mpv.getPropertyBoolean("pause") ?: !_state.value.isPlaying
        val idle = mpv.getPropertyBoolean("idle-active") ?: false
        val next = _state.value.copy(
            isPlaying = !paused && !idle,
            isBuffering = idle && !_state.value.isVideoReady,
            positionMs = position,
            durationMs = duration
        )
        if (next != _state.value) _state.value = next
        updateTracksFromProperties()
    }

    private fun updateTechnicalInfo() {
        if (!initialized) return
        val width = mpv.getPropertyInt("width")
        val height = mpv.getPropertyInt("height")
        val fps = mpv.getPropertyDouble("container-fps")
        val codec = mpv.getPropertyString("video-codec")
        val audioCodec = mpv.getPropertyString("audio-codec")
        val audioOutputFormat = mpv.getPropertyString("audio-out-params/format")
        val audioOutputChannels = mpv.getPropertyString("audio-out-params/channels")
        val hwdec = mpv.getPropertyString("hwdec-current")
        val report = buildMpvProbeReport()
        val info = listOfNotNull(
            MediaSourceInfoBuilder.build(
                url = (activeLoad ?: pendingLoad)?.url,
                stream = (activeLoad ?: pendingLoad)?.stream,
                externalSubtitles = currentSubtitles
            ),
            "player_data",
            report.format(),
            if (width != null && height != null) "Active Quality: ${width}x$height" else null,
            fps?.takeIf { it > 0.0 }?.let { "Active FPS: ${String.format(Locale.US, "%.2f", it)}" },
            codec?.let { "Active Codec: $it" },
            audioCodec?.let { "Active Audio: $it" },
            "Audio Path: ${mpvAudioPath(audioOutputFormat)}",
            audioOutputFormat?.let { format -> "Audio Output Format: $format${audioOutputChannels?.let { " / $it" } ?: ""}" },
            "Audio Output: ${AudioCapabilityResolver.describePassthroughSupport(appContext, AudioCapabilityResolver.mediaAudioAttributes())}",
            hwdec?.let { "HW Decode: $it" }
        ).joinToString("\n").ifBlank { null }
        _technicalInfo.value = info
        LastMediaDebugInfoStore.save(
            context = appContext,
            url = (activeLoad ?: pendingLoad)?.url,
            title = (activeLoad ?: pendingLoad)?.stream?.effectiveFilename ?: (activeLoad ?: pendingLoad)?.stream?.rawDisplayTitle,
            technicalInfo = info
        )
        _state.value = _state.value.copy(technicalInfo = info)
    }

    private fun mpvAudioPath(outputFormat: String?): String = when {
        audioProcessingMode == "night" -> "PCM DSP (Night)"
        audioProcessingMode == "balanced" -> "PCM gentle DSP"
        isMpvBitstreamFormat(outputFormat) -> "passthrough"
        !outputFormat.isNullOrBlank() -> "PCM decoded"
        else -> "passthrough-preferred (pending)"
    }

    private fun isMpvBitstreamFormat(outputFormat: String?): Boolean {
        val value = outputFormat?.lowercase()?.replace('-', '_') ?: return false
        return value.startsWith("spdif") || value in setOf("ac3", "eac3", "dts", "dts_hd", "truehd", "ac4")
    }

    private fun buildMpvProbeReport(): MediaProbeReport {
        val load = activeLoad ?: pendingLoad
        val fileFormat = mpv.getPropertyString("file-format")?.takeIf { it.isNotBlank() }
        val duration = mpv.getPropertyDouble("duration")?.secondsToMs()
        val bitrate = mpv.getPropertyInt("demuxer-cache-state/raw-input-rate")
            ?.takeIf { it > 0 }
            ?.toLong()
            ?.bytesPerSecondToBitrateLabel()
            ?.let(ProbeValue::verified)
            ?: ProbeValue.unknown()
        val seekable = mpv.getPropertyBoolean("seekable")?.toString()?.let(ProbeValue::verified) ?: ProbeValue.unknown()
        val count = mpv.getPropertyInt("track-list/count") ?: 0
        val videoTracks = mutableListOf<VideoProbeTrack>()
        val audioTracks = mutableListOf<AudioProbeTrack>()
        val subtitleTracks = mutableListOf<SubtitleProbeTrack>()

        for (index in 0 until count) {
            val type = mpv.getPropertyString("track-list/$index/type").orEmpty()
            val codec = mpv.getPropertyString("track-list/$index/codec")?.takeIf { it.isNotBlank() }
            val selected = mpv.getPropertyBoolean("track-list/$index/selected") ?: false
            val language = mpv.getPropertyString("track-list/$index/lang")?.takeIf { it.isNotBlank() }
            val title = mpv.getPropertyString("track-list/$index/title")?.takeIf { it.isNotBlank() }
            val external = mpv.getPropertyBoolean("track-list/$index/external") ?: false
            when (type) {
                "video" -> {
                    val width = mpv.getPropertyInt("track-list/$index/demux-w") ?: mpv.getPropertyInt("width")
                    val height = mpv.getPropertyInt("track-list/$index/demux-h") ?: mpv.getPropertyInt("height")
                    val fps = mpv.getPropertyDouble("track-list/$index/demux-fps") ?: mpv.getPropertyDouble("container-fps")
                    val bitrateValue = mpv.getPropertyInt("track-list/$index/demux-bitrate")
                    val pixelFormat = mpv.getPropertyString("video-params/pixelformat")
                        ?: mpv.getPropertyString("video-out-params/pixelformat")
                    videoTracks += VideoProbeTrack(
                        codec = ProbeValue.verified(codec),
                        profile = MediaProbeClassifier.profileFromCodecs(codec),
                        level = MediaProbeClassifier.levelFromCodecs(codec),
                        pixelFormat = pixelFormat?.let(ProbeValue::verified) ?: ProbeValue.unknown(),
                        resolution = if (width != null && height != null) ProbeValue.verified("${width}x$height") else ProbeValue.unknown(),
                        fps = fps?.takeIf { it > 0.0 }?.let { ProbeValue.verified(String.format(Locale.US, "%.2f", it)) } ?: ProbeValue.unknown(),
                        bitrate = bitrateValue?.takeIf { it > 0 }?.let { ProbeValue.verified(it.bitrateLabel()) } ?: ProbeValue.unknown(),
                        language = language,
                        selected = selected
                    )
                }
                "audio" -> {
                    val channels = mpv.getPropertyInt("track-list/$index/demux-channel-count")
                        ?: mpv.getPropertyInt("audio-params/channel-count")
                    val sampleRate = mpv.getPropertyInt("track-list/$index/demux-samplerate")
                        ?: mpv.getPropertyInt("audio-params/samplerate")
                    val bitrateValue = mpv.getPropertyInt("track-list/$index/demux-bitrate")
                    audioTracks += AudioProbeTrack(
                        codec = ProbeValue.verified(codec),
                        channels = channels?.takeIf { it > 0 }?.let { ProbeValue.verified("${it}ch") } ?: ProbeValue.unknown(),
                        sampleRate = sampleRate?.takeIf { it > 0 }?.let { ProbeValue.verified("${it}Hz") } ?: ProbeValue.unknown(),
                        bitrate = bitrateValue?.takeIf { it > 0 }?.let { ProbeValue.verified(it.bitrateLabel()) } ?: ProbeValue.unknown(),
                        language = language,
                        title = title,
                        selected = selected
                    )
                }
                "sub" -> {
                    subtitleTracks += SubtitleProbeTrack(
                        codec = ProbeValue.verified(codec),
                        language = language,
                        title = title,
                        external = external,
                        selected = selected
                    )
                }
            }
        }

        currentSubtitles.forEach { subtitle ->
            if (subtitleTracks.none { it.external && it.title == subtitle.label && it.language == subtitle.language }) {
                subtitleTracks += SubtitleProbeTrack(
                    codec = MediaProbeClassifier.subtitleCodecFromUrl(subtitle.url),
                    language = subtitle.language,
                    title = subtitle.label,
                    external = true,
                    selected = false
                )
            }
        }

        val activeCodec = mpv.getPropertyString("video-codec")
            ?: videoTracks.firstOrNull { it.selected }?.codec?.value
            ?: videoTracks.firstOrNull()?.codec?.value
        val hdr = hdrFromMpv()
        val quirks = buildList {
            if (seekable.value == "false") add(ProbeValue.verified("non_seekable"))
            val editionCount = mpv.getPropertyInt("edition-list/count") ?: 0
            if (editionCount > 1) add(ProbeValue.verified("multiple_editions=$editionCount"))
            val chapterCount = mpv.getPropertyInt("chapter-list/count") ?: 0
            if (chapterCount > 0) add(ProbeValue.verified("chapters=$chapterCount"))
            val containerFps = mpv.getPropertyDouble("container-fps")
            val estimatedFps = mpv.getPropertyDouble("estimated-vf-fps")
            if (containerFps != null && estimatedFps != null && kotlin.math.abs(containerFps - estimatedFps) > 0.25) {
                add(ProbeValue.inferred("possible_vfr"))
            }
        }

        return MediaProbeReport(
            engine = "MPV",
            source = MediaProbeClassifier.sourceFor(load?.url, load?.stream),
            container = fileFormat?.let(ProbeValue::verified)
                ?: MediaProbeClassifier.containerFromUrl(load?.url, load?.stream),
            durationMs = duration,
            bitrate = bitrate,
            seekable = seekable,
            video = videoTracks,
            audio = audioTracks,
            subtitles = subtitleTracks,
            hdr = hdr,
            dolbyVision = MediaProbeClassifier.dolbyVisionFromCodecs(activeCodec),
            quirks = quirks
        )
    }

    private fun hdrFromMpv(): ProbeValue {
        val gamma = mpv.getPropertyString("video-params/gamma")
            ?: mpv.getPropertyString("video-out-params/gamma")
        val primaries = mpv.getPropertyString("video-params/primaries")
            ?: mpv.getPropertyString("video-out-params/primaries")
        val codec = mpv.getPropertyString("video-codec")
        val lower = listOfNotNull(gamma, primaries, codec).joinToString(" ").lowercase(Locale.ROOT)
        val hdr = when {
            lower.contains("dvh") || lower.contains("dvhe") || lower.contains("dva1") || lower.contains("dvav") -> "Dolby Vision"
            lower.contains("hlg") -> "HLG"
            lower.contains("pq") || lower.contains("smpte2084") || lower.contains("bt.2020") || lower.contains("bt2020") -> "HDR10/PQ"
            gamma != null || primaries != null -> "SDR"
            else -> null
        }
        return if (gamma != null || primaries != null) ProbeValue.verified(hdr) else ProbeValue.inferred(hdr)
    }

    private fun updateTracksFromProperties() {
        val count = mpv.getPropertyInt("track-list/count") ?: 0
        val aid = mpv.getPropertyString("aid").orEmpty()
        val sid = mpv.getPropertyString("sid").orEmpty()
        val key = "$count:$aid:$sid"
        if (key == lastTrackListKey) return
        lastTrackListKey = key
        val audios = mutableListOf<MediaTrack>()
        val subtitles = mutableListOf<MediaTrack>()
        for (index in 0 until count) {
            val type = mpv.getPropertyString("track-list/$index/type").orEmpty()
            val mpvId = mpv.getPropertyInt("track-list/$index/id") ?: continue
            val selected = mpv.getPropertyBoolean("track-list/$index/selected") ?: false
            val language = mpv.getPropertyString("track-list/$index/lang")?.takeIf { it.isNotBlank() }
            val title = mpv.getPropertyString("track-list/$index/title")?.takeIf { it.isNotBlank() }
            val codec = mpv.getPropertyString("track-list/$index/codec")?.takeIf { it.isNotBlank() }
            val channelCount = mpv.getPropertyInt("track-list/$index/demux-channel-count")
                ?: mpv.getPropertyInt("audio-params/channel-count")
            val bitrate = mpv.getPropertyInt("track-list/$index/demux-bitrate")
                ?.takeIf { it > 0 }
                ?.toLong()
            val sampleRate = mpv.getPropertyInt("track-list/$index/demux-samplerate")
                ?.takeIf { it > 0 }
            when (type) {
                "audio" -> audios.add(
                    MediaTrack(
                        id = "mpv_audio_$mpvId",
                        label = title ?: language ?: "Audio ${audios.size + 1}",
                        language = language,
                        type = C.TRACK_TYPE_AUDIO,
                        groupIndex = mpvId,
                        trackIndex = index,
                        isSelected = selected,
                        channelCount = channelCount,
                        sampleMimeType = codec,
                        bitrate = bitrate,
                        sampleRate = sampleRate,
                    )
                )
                "sub" -> subtitles.add(
                    MediaTrack(
                        id = "mpv_sub_$mpvId",
                        label = title ?: language ?: "Subtitle ${subtitles.size + 1}",
                        language = language,
                        type = C.TRACK_TYPE_TEXT,
                        groupIndex = mpvId,
                        trackIndex = index,
                        isSelected = selected,
                        sampleMimeType = codec
                    )
                )
            }
        }
        _availableAudios.value = audios
        _availableSubtitles.value = subtitles
        _currentAudio.value = audios.firstOrNull { it.isSelected }
        _currentSubtitle.value = subtitles.firstOrNull { it.isSelected }
        if (!manualAudioSelection && !applyingAudioPreference) {
            selectBestAudioTrack()
        }
    }

    private fun selectBestAudioTrack() {
        if (!initialized || manualAudioSelection || applyingAudioPreference) return
        val tracks = _availableAudios.value
        val capabilities = AudioCapabilityResolver.resolve(
            appContext,
            AudioCapabilityResolver.mediaAudioAttributes()
        )
        val passthroughTrackIds = if (audioProcessingMode == "reference") {
            tracks.filter {
                // The route resolver describes Android's capabilities, but
                // MPV's Android audiotrack backend only emits the formats
                // represented by its audio-spdif option. Keep engine support
                // in the decision so Dolby MAT/MPEG-H/DTS-UHD are decoded
                // instead of being falsely marked as passthrough.
                mpvCanPassthroughMime(it.sampleMimeType) &&
                capabilities.supportsPassthroughMime(
                    sampleMimeType = it.sampleMimeType,
                    channelCount = it.channelCount ?: 2,
                    sampleRate = it.sampleRate ?: 48_000,
                )
            }
                .mapTo(mutableSetOf()) { it.id }
        } else {
            emptySet()
        }
        val best = AudioTrackQualityPolicy.choose(
            tracks = tracks,
            preferredLanguage = preferredAudioLanguageCode,
            passthroughTrackIds = passthroughTrackIds,
            supportedSampleRates = capabilities.pcmSampleRates,
            maxPcmChannels = capabilities.maxPcmChannels,
        ) ?: return
        if (best.isSelected) return
        applyingAudioPreference = true
        try {
            mpv.setPropertyString("aid", best.mpvTrackId())
        } finally {
            applyingAudioPreference = false
        }
    }

    private fun MediaTrack.mpvTrackId(): String {
        return id.substringAfterLast('_').takeIf { it.toIntOrNull() != null } ?: groupIndex.toString()
    }

    private fun mpvCanPassthroughMime(sampleMimeType: String?): Boolean {
        return AudioPassthroughPolicy.isMpvCandidate(sampleMimeType)
    }

    private fun Int.toMpvColor(opacity: Float = 1f): String {
        val a = (opacity.coerceIn(0f, 1f) * 255).toInt()
        return "#%02X%02X%02X%02X".format(Locale.US, a, Color.red(this), Color.green(this), Color.blue(this))
    }

    private fun Double.secondsToMs(): Long = (this * 1000.0).toLong().coerceAtLeast(0L)

    private companion object {
        const val HARDWARE_VIDEO_READY_TIMEOUT_MS = 5_000L
    }
}
