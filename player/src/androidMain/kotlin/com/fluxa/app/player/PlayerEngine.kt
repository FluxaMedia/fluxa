@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.player

import com.fluxa.app.shared.feature.player.Chapter
import com.fluxa.app.shared.feature.player.MediaTrack

import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.SeekParameters
import androidx.media3.common.C
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.FluxaLocalStreamServer
import com.fluxa.app.core.rust.models.NativeDvProxyPlan
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Stream
import com.google.gson.Gson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import java.net.URI
import java.util.Locale

data class PlayerEngineRequest(
    val url: String,
    val stream: Stream?,
    val subtitles: List<ExternalSubtitleTrack>,
    val startPositionMs: Long,
    val preferredAudioLanguage: String?,
    val preferredSubtitleLanguage: String?,
    val dolbyVisionFallbackMode: DolbyVisionFallbackMode,
    val dvRpuMode: Int = 2,
    val dvZeroLevel5: Boolean = false,
    val dvHdr10PlusMode: String = "auto",
    val audioDecoderMode: String = "hw_prefer"
)

interface PlayerEngine {
    val availableAudios: StateFlow<List<MediaTrack>>
    val availableSubtitles: StateFlow<List<MediaTrack>>
    val currentAudio: StateFlow<MediaTrack?>
    val currentSubtitle: StateFlow<MediaTrack?>
    val technicalInfo: StateFlow<String?>
    val chapters: StateFlow<List<Chapter>>

    fun prepareAndPlay(request: PlayerEngineRequest)
    fun addExternalSubtitles(subtitles: List<ExternalSubtitleTrack>)
    fun pause()
    fun stop()
    fun clear()
    fun seekTo(positionMs: Long, exact: Boolean = true)
    fun setPaused(paused: Boolean)
    fun setSpeed(speed: Float)
    fun setAudioDelayMs(delayMs: Long)
    fun setSubtitleDelayMs(delayMs: Long)
    fun setZoomed(zoomed: Boolean)
    fun selectAudio(track: MediaTrack)
    fun enableSubtitle(track: MediaTrack)
    fun disableSubtitles()
    fun applyPreferredAudioLanguage(languageCode: String)
    fun applySubtitleStyle(profile: UserProfile?)
}

class ExoPlayerEngine(
    private val controller: MediaPlayerController,
    private val exoPlayer: ExoPlayer
) : PlayerEngine {
    private var localStreamSession: FluxaLocalStreamServer.Session? = null
    private val engineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private var chapterFetchJob: Job? = null

    override val availableAudios: StateFlow<List<MediaTrack>> = controller.availableAudios
    override val availableSubtitles: StateFlow<List<MediaTrack>> = controller.availableSubtitles
    override val currentAudio: StateFlow<MediaTrack?> = controller.currentAudio
    override val currentSubtitle: StateFlow<MediaTrack?> = controller.currentSubtitle
    override val technicalInfo: StateFlow<String?> = controller.technicalInfo

    private val _chapters = MutableStateFlow<List<Chapter>>(emptyList())
    override val chapters: StateFlow<List<Chapter>> = _chapters

    override fun prepareAndPlay(request: PlayerEngineRequest) {
        val resolved = resolveLocalStream(request.url, request.stream, request.dolbyVisionFallbackMode, request.dvRpuMode, request.dvZeroLevel5, request.dvHdr10PlusMode)
        controller.audioDecoderMode = request.audioDecoderMode
        controller.prepareAndPlay(
            url = resolved.url,
            headers = resolved.headers,
            subtitles = request.subtitles,
            dolbyVisionFallbackMode = request.dolbyVisionFallbackMode,
            dvRpuMode = request.dvRpuMode,
            dvZeroLevel5 = request.dvZeroLevel5,
            dvHdr10PlusMode = request.dvHdr10PlusMode,
            iptPqc2UseHdr = resolved.iptPqc2UseHdr,
            iptPqc2PreDecide = resolved.iptPqc2PreDecide,
            stream = request.stream
        )
        if (request.startPositionMs > 0L) {
            exoPlayer.seekTo(request.startPositionMs)
        }
        fetchChapters(request.url, request.stream)
    }

    private fun fetchChapters(url: String, stream: Stream?) {
        _chapters.value = emptyList()
        chapterFetchJob?.cancel()
        chapterFetchJob = engineScope.launch {
            val result = MkvChapterFetcher.fetch(url, stream?.resolveHeaders().orEmpty())
            if (result.isNotEmpty()) {
                _chapters.value = result
            }
        }
    }

    override fun addExternalSubtitles(subtitles: List<ExternalSubtitleTrack>) {
        controller.addExternalSubtitles(subtitles)
    }

    override fun pause() {
        exoPlayer.pause()
    }

    override fun stop() {
        chapterFetchJob?.cancel()
        stopLocalStreamServer()
        exoPlayer.stop()
    }

    override fun clear() {
        chapterFetchJob?.cancel()
        stopLocalStreamServer()
        exoPlayer.clearMediaItems()
    }

    override fun seekTo(positionMs: Long, exact: Boolean) {
        exoPlayer.setSeekParameters(if (exact) SeekParameters.EXACT else SeekParameters.CLOSEST_SYNC)
        exoPlayer.seekTo(positionMs)
        if (!exact) exoPlayer.setSeekParameters(SeekParameters.EXACT)
    }

    override fun setPaused(paused: Boolean) {
        exoPlayer.playWhenReady = !paused
    }

    override fun setSpeed(speed: Float) {
        exoPlayer.setPlaybackSpeed(speed)
    }

    override fun setAudioDelayMs(delayMs: Long) {
        PlayerDelayController.setAudioDelayMs(delayMs)
    }

    override fun setSubtitleDelayMs(delayMs: Long) = Unit

    override fun setZoomed(zoomed: Boolean) {
        exoPlayer.videoScalingMode = if (zoomed) {
            C.VIDEO_SCALING_MODE_SCALE_TO_FIT_WITH_CROPPING
        } else {
            C.VIDEO_SCALING_MODE_SCALE_TO_FIT
        }
    }

    override fun selectAudio(track: MediaTrack) {
        controller.selectAudio(track)
    }

    override fun enableSubtitle(track: MediaTrack) {
        controller.enableSubtitle(track)
    }

    override fun disableSubtitles() {
        controller.disableSubtitles()
    }

    override fun applyPreferredAudioLanguage(languageCode: String) {
        controller.applyPreferredAudioLanguage(languageCode)
    }

    override fun applySubtitleStyle(profile: UserProfile?) = Unit

    private fun resolveLocalStream(
        url: String,
        stream: Stream?,
        dvMode: DolbyVisionFallbackMode,
        dvRpuMode: Int = 2,
        dvZeroLevel5: Boolean = false,
        dvHdr10PlusMode: String = "auto"
    ): ResolvedPlaybackUrl {
        stopLocalStreamServer()
        val headers = stream?.resolveHeaders().orEmpty()
        val caps = AndroidDolbyVisionCapabilities.detect(controller.context)
        val fallbackModeStr = dvMode.toRustString()

        val dvPlan = resolveDvProxyPlan(url, stream, dvMode, caps)
        controller.notifyDvProxyPlan(dvPlan)

        val isP5 = dvPlan?.profile == "P5"
        val useHdr = caps.displaySupportsHdr10

        if (dvPlan != null && dvPlan.action != "none") {
            val isHlsConvert = dvPlan.action == "hls_rpu_convert"
            val dvSession = FluxaLocalStreamServer.startWithDvRewrite(
                targetUrl = url,
                headers = StreamRequestPolicy.headersFor(url, headers),
                action = dvPlan.action,
                rpuMode = if (dvPlan.action == "rpu_convert" || isHlsConvert) dvRpuMode else dvPlan.rpuMode,
                deviceHasDvDecoder = caps.mediaCodecSupportsDolbyVision,
                deviceHasDvDisplay = caps.displaySupportsDolbyVision,
                zeroLevel5 = (dvPlan.action == "rpu_convert" || isHlsConvert) && dvZeroLevel5,
                removeHdr10Plus = when (dvHdr10PlusMode) {
                    "always" -> true
                    "never" -> false
                    else -> dvPlan.action == "rpu_convert" || isHlsConvert
                },
                fallbackMode = fallbackModeStr
            )
            if (dvSession != null) {
                localStreamSession = dvSession
                return ResolvedPlaybackUrl(dvSession.url, emptyMap(), iptPqc2PreDecide = isP5, iptPqc2UseHdr = useHdr)
            }
        }

        if (dvMode != DolbyVisionFallbackMode.Off && (dvPlan == null || dvPlan.action == "none")) {
            val lowerUrl = url.lowercase()
            val isContainerFile = lowerUrl.endsWith(".mkv") || lowerUrl.contains(".mkv?") ||
                lowerUrl.endsWith(".mp4") || lowerUrl.contains(".mp4?") ||
                lowerUrl.endsWith(".m4v") || lowerUrl.contains(".m4v?")
            if (isContainerFile && shouldUseLocalStreamServer(url, headers)) {
                val autoSession = FluxaLocalStreamServer.startWithDvRewrite(
                    targetUrl = url,
                    headers = StreamRequestPolicy.headersFor(url, headers),
                    action = "auto_detect",
                    rpuMode = dvPlan?.rpuMode ?: 2,
                    deviceHasDvDecoder = caps.mediaCodecSupportsDolbyVision,
                    deviceHasDvDisplay = caps.displaySupportsDolbyVision,
                    removeHdr10Plus = when (dvHdr10PlusMode) {
                        "always" -> true
                        "never" -> false
                        else -> false
                    },
                    fallbackMode = fallbackModeStr
                )
                if (autoSession != null) {
                    localStreamSession = autoSession
                    return ResolvedPlaybackUrl(autoSession.url, emptyMap(), iptPqc2PreDecide = isP5, iptPqc2UseHdr = useHdr)
                }
            }
        }

        if (!shouldUseLocalStreamServer(url, headers)) return ResolvedPlaybackUrl(url, headers, iptPqc2PreDecide = isP5, iptPqc2UseHdr = useHdr)
        val session = FluxaLocalStreamServer.start(
            targetUrl = url,
            headers = StreamRequestPolicy.headersFor(url, headers)
        ) ?: return ResolvedPlaybackUrl(url, headers, iptPqc2PreDecide = isP5, iptPqc2UseHdr = useHdr)
        localStreamSession = session
        return ResolvedPlaybackUrl(session.url, emptyMap(), iptPqc2PreDecide = isP5, iptPqc2UseHdr = useHdr)
    }

    private fun resolveDvProxyPlan(
        url: String,
        stream: Stream?,
        dvMode: DolbyVisionFallbackMode,
        caps: DolbyVisionCapabilities
    ): NativeDvProxyPlan? = runCatching {
        FluxaCoreNative.dvProxyPlan(
            streamJson = Gson().toJson(stream),
            url = url,
            fallbackMode = dvMode.toRustString(),
            deviceHasDvDecoder = caps.mediaCodecSupportsDolbyVision,
            deviceHasDvDisplay = caps.displaySupportsDolbyVision
        )
    }.getOrNull()

    private fun stopLocalStreamServer() {
        localStreamSession?.stop()
        localStreamSession = null
    }
}

class MpvPlayerEngine(
    private val player: MpvEmbeddedPlayer
) : PlayerEngine {
    private var localStreamSession: FluxaLocalStreamServer.Session? = null

    override val availableAudios: StateFlow<List<MediaTrack>> = player.availableAudios
    override val availableSubtitles: StateFlow<List<MediaTrack>> = player.availableSubtitles
    override val currentAudio: StateFlow<MediaTrack?> = player.currentAudio
    override val currentSubtitle: StateFlow<MediaTrack?> = player.currentSubtitle
    override val technicalInfo: StateFlow<String?> = player.technicalInfo
    override val chapters: StateFlow<List<Chapter>> = player.chapters

    override fun prepareAndPlay(request: PlayerEngineRequest) {
        val resolved = resolveLocalStream(request.url, request.stream)
        player.prepareAndPlay(
            url = resolved.url,
            stream = request.stream,
            subtitles = request.subtitles,
            startPositionMs = request.startPositionMs,
            preferredAudioLanguage = request.preferredAudioLanguage,
            preferredSubtitleLanguage = request.preferredSubtitleLanguage
        )
    }

    override fun addExternalSubtitles(subtitles: List<ExternalSubtitleTrack>) {
        player.addExternalSubtitles(subtitles)
    }

    override fun pause() {
        player.setPaused(true)
    }

    override fun stop() {
        stopLocalStreamServer()
        player.stop()
    }

    override fun clear() {
        stopLocalStreamServer()
        player.stop()
    }

    override fun seekTo(positionMs: Long, exact: Boolean) {
        player.seekTo(positionMs, exact)
    }

    override fun setPaused(paused: Boolean) {
        player.setPaused(paused)
    }

    override fun setSpeed(speed: Float) {
        player.setSpeed(speed)
    }

    override fun setAudioDelayMs(delayMs: Long) {
        player.setAudioDelayMs(delayMs)
    }

    override fun setSubtitleDelayMs(delayMs: Long) {
        player.setSubtitleDelayMs(delayMs)
    }

    override fun setZoomed(zoomed: Boolean) {
        player.setZoomed(zoomed)
    }

    override fun selectAudio(track: MediaTrack) {
        player.selectAudio(track)
    }

    override fun enableSubtitle(track: MediaTrack) {
        player.enableSubtitle(track)
    }

    override fun disableSubtitles() {
        player.disableSubtitles()
    }

    override fun applyPreferredAudioLanguage(languageCode: String) {
        player.applyPreferredAudioLanguage(languageCode)
    }

    override fun applySubtitleStyle(profile: UserProfile?) {
        player.applySubtitleStyle(profile)
    }

    private fun resolveLocalStream(url: String, stream: Stream?): ResolvedPlaybackUrl {
        stopLocalStreamServer()
        val headers = stream?.resolveHeaders().orEmpty()
        if (!shouldUseLocalStreamServer(url, headers)) return ResolvedPlaybackUrl(url, headers)
        val session = FluxaLocalStreamServer.start(
            targetUrl = url,
            headers = StreamRequestPolicy.headersFor(url, headers)
        ) ?: return ResolvedPlaybackUrl(url, headers)
        localStreamSession = session
        return ResolvedPlaybackUrl(session.url, emptyMap())
    }

    private fun stopLocalStreamServer() {
        localStreamSession?.stop()
        localStreamSession = null
    }
}

private data class ResolvedPlaybackUrl(
    val url: String,
    val headers: Map<String, String>,
    val iptPqc2PreDecide: Boolean = false,
    val iptPqc2UseHdr: Boolean = false
)

private fun shouldUseLocalStreamServer(url: String, headers: Map<String, String>): Boolean {
    if (headers.isEmpty()) return false
    val uri = runCatching { URI(url) }.getOrNull() ?: return false
    val scheme = uri.scheme?.lowercase(Locale.ROOT) ?: return false
    if (scheme != "http" && scheme != "https") return false
    val host = uri.host?.lowercase(Locale.ROOT).orEmpty()
    if (host == "127.0.0.1" || host == "localhost" || host == "::1") return false
    val path = uri.path.orEmpty().lowercase(Locale.ROOT)
    return !path.endsWith(".m3u8") && !path.endsWith(".mpd")
}
