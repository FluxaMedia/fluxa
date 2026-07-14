package com.fluxa.app.ui.catalog

import android.content.Context
import android.graphics.PixelFormat
import android.net.Uri
import android.os.SystemClock
import android.util.Log
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.video.VideoFrameMetadataListener
import com.fluxa.app.player.ExternalSubtitleTrack
import com.fluxa.app.player.LibassDebugLog
import com.fluxa.app.player.LibassRenderThread
import com.fluxa.app.player.LibassVideoFrame
import com.fluxa.app.player.MediaPlayerController
import com.fluxa.app.player.MediaTrack
import com.fluxa.app.player.MkvNativeAssExtractor
import com.fluxa.app.player.NativeAssTrack
import com.fluxa.app.player.NativeLibassRenderer
import com.fluxa.app.player.StreamRequestPolicy
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import okhttp3.OkHttpClient
import okhttp3.Request
import java.util.concurrent.TimeUnit

internal fun selectedNativeAssSubtitle(
    currentSubtitle: MediaTrack?,
    externalSubtitles: List<ExternalSubtitleTrack>
): ExternalSubtitleTrack? {
    if (currentSubtitle == null) return null
    val assOnly = externalSubtitles.filter { NativeLibassRenderer.isAssUrl(it.url) }
    return assOnly.firstOrNull { it.label != null && it.label == currentSubtitle.label }
        ?: assOnly.firstOrNull { it.url == currentSubtitle.label }
        ?: assOnly.firstOrNull { it.label != null && it.language != null && it.language == currentSubtitle.language }
}

internal fun selectedEmbeddedNativeAssTrack(
    currentSubtitle: MediaTrack?,
    embeddedTracks: List<NativeAssTrack>
): NativeAssTrack? {
    if (currentSubtitle == null) return null
    return embeddedTracks.firstOrNull { track ->
        track.id == currentSubtitle.id ||
            track.label == currentSubtitle.label ||
            track.language == currentSubtitle.language
    }
}

@Composable
internal fun NativeLibassSubtitleOverlay(
    exoPlayer: ExoPlayer,
    externalSubtitle: ExternalSubtitleTrack?,
    embeddedSubtitle: NativeAssTrack?,
    subtitleDelayMs: Long,
    surfaceView: NativeLibassSubtitleSurfaceView?
) {
    val context = androidx.compose.ui.platform.LocalContext.current
    val renderThread = remember(exoPlayer) { MediaPlayerController.getLibassRelay(exoPlayer)?.renderThread }

    LaunchedEffect(externalSubtitle?.url, embeddedSubtitle?.id) {
        LibassDebugLog.d(
            "overlay selection external=${externalSubtitle?.let { "${LibassDebugLog.urlSummary(it.url)} label=${it.label} lang=${it.language}" } ?: "<none>"} " +
                "embedded=${embeddedSubtitle?.let { "id=${it.id} label=${it.label} lang=${it.language} bytes=${it.assData.size} fonts=${it.fonts.size}" } ?: "<none>"}"
        )
        renderThread?.setLocalRenderer(null)
        val embedded = embeddedSubtitle
        if (embedded != null) {
            val renderer = runCatching {
                NativeLibassRenderer.create(
                    assData = embedded.assData,
                    fonts = embedded.fonts,
                    fontsDir = context.filesDir.resolve("fonts").absolutePath
                )
            }.onFailure { e ->
                Log.w("NativeLibassOverlay", "Embedded renderer failed", e)
                LibassDebugLog.w("overlay embedded renderer failed id=${embedded.id}", e)
            }.getOrNull()
            LibassDebugLog.d("overlay embedded renderer ${if (renderer != null) "ready" else "not ready"} id=${embedded.id}")
            renderThread?.setLocalRenderer(renderer)
            return@LaunchedEffect
        }
        val url = externalSubtitle?.url ?: return@LaunchedEffect
        val renderer = runCatching {
            val data = fetchSubtitleBytes(context, url)
            LibassDebugLog.d("overlay fetched external ASS bytes=${data.size} url=${LibassDebugLog.urlSummary(url)}")
            val fonts = MkvNativeAssExtractor.extractExternalFontHints(url)
            LibassDebugLog.d("overlay external font hints fonts=${fonts.size} url=${LibassDebugLog.urlSummary(url)}")
            NativeLibassRenderer.create(
                assData = data,
                fonts = fonts,
                fontsDir = context.filesDir.resolve("fonts").absolutePath
            )
        }.onFailure { e ->
            Log.w("NativeLibassOverlay", "External renderer failed", e)
            LibassDebugLog.w("overlay external renderer failed url=${LibassDebugLog.urlSummary(url)}", e)
        }.getOrNull()
        LibassDebugLog.d("overlay external renderer ${if (renderer != null) "ready" else "not ready"} url=${LibassDebugLog.urlSummary(url)}")
        renderThread?.setLocalRenderer(renderer)
    }

    DisposableEffect(Unit) {
        onDispose { renderThread?.setLocalRenderer(null) }
    }

    surfaceView?.configure(exoPlayer, renderThread, subtitleDelayMs)
}

internal class NativeLibassSubtitleSurfaceView(context: Context) : SurfaceView(context), SurfaceHolder.Callback {
    var exoPlayer: ExoPlayer? = null
        set(value) {
            field?.let {
                it.clearVideoFrameMetadataListener(ptsListener)
                it.removeListener(playerListener)
            }
            field = value
            if (isAttachedToWindow) {
                value?.setVideoFrameMetadataListener(ptsListener)
                value?.addListener(playerListener)
            }
            value?.videoSize?.let(playerListener::onVideoSizeChanged)
        }

    var renderThread: LibassRenderThread? = null
        set(value) {
            field = value
            value?.setDelay(subtitleDelayMs)
            attachedSurface?.let { surface ->
                value?.setSurface(surface, surfaceWidth, surfaceHeight)
                value?.setVideoFrame(LibassVideoFrame(surfaceWidth, surfaceHeight, 0, 0))
            }
        }

    var subtitleDelayMs: Long = 0L
        set(value) {
            field = value
            renderThread?.setDelay(value)
        }

    private var lastSurfaceLogMs = 0L
    private var attachedSurface: android.view.Surface? = null
    private var surfaceWidth = 0
    private var surfaceHeight = 0

    private val ptsListener = VideoFrameMetadataListener { presentationTimeUs, _, _, _ ->
        renderThread?.onVideoFrame(presentationTimeUs / 1000L)
    }

    init {
        setZOrderMediaOverlay(true)
        holder.setFormat(PixelFormat.TRANSLUCENT)
        holder.addCallback(this)
    }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        LibassDebugLog.d("overlay view attached size=${width}x$height")
        exoPlayer?.setVideoFrameMetadataListener(ptsListener)
        exoPlayer?.addListener(playerListener)
    }

    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow()
        LibassDebugLog.d("overlay view detached")
        exoPlayer?.clearVideoFrameMetadataListener(ptsListener)
        exoPlayer?.removeListener(playerListener)
    }

    override fun surfaceCreated(holder: SurfaceHolder) {}

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        val now = SystemClock.elapsedRealtime()
        if (now - lastSurfaceLogMs > 500L) {
            LibassDebugLog.d("overlay surface changed ${width}x$height")
            lastSurfaceLogMs = now
        }
        attachedSurface = holder.surface
        surfaceWidth = width
        surfaceHeight = height
        renderThread?.setSurface(holder.surface, width, height)
        renderThread?.setVideoFrame(LibassVideoFrame(width, height, 0, 0))
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        LibassDebugLog.d("overlay surface destroyed")
        attachedSurface = null
        surfaceWidth = 0
        surfaceHeight = 0
        renderThread?.setSurface(null, 0, 0)
    }

    private val playerListener = object : androidx.media3.common.Player.Listener {}

    fun configure(player: ExoPlayer, renderThread: LibassRenderThread?, subtitleDelayMs: Long) {
        exoPlayer = player
        this.renderThread = renderThread
        this.subtitleDelayMs = subtitleDelayMs
    }
}

private val subtitleHttpClient: OkHttpClient by lazy {
    OkHttpClient.Builder()
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(20, TimeUnit.SECONDS)
        .build()
}

private suspend fun fetchSubtitleBytes(context: Context, rawUrl: String): ByteArray = withContext(Dispatchers.IO) {
    val uri = runCatching { Uri.parse(rawUrl) }.getOrNull()
    when (uri?.scheme?.lowercase()) {
        "content" -> context.contentResolver.openInputStream(uri)?.use { it.readBytes() } ?: ByteArray(0)
        "file" -> File(requireNotNull(uri.path)).readBytes()
        "http", "https" -> {
            val headers = StreamRequestPolicy.headersFor(rawUrl, emptyMap())
            val requestBuilder = Request.Builder().url(rawUrl)
            headers.forEach { (k, v) -> requestBuilder.header(k, v) }
            subtitleHttpClient.newCall(requestBuilder.build()).execute().use { response ->
                response.body.bytes()
            }
        }
        else -> java.net.URL(rawUrl).openStream().use { it.readBytes() }
    }
}
