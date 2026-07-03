package com.fluxa.app.ui.catalog

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Canvas
import android.net.Uri
import android.util.Log
import android.view.View
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.fluxa.app.player.ExternalSubtitleTrack
import com.fluxa.app.player.MediaPlayerController
import com.fluxa.app.player.MediaTrack
import com.fluxa.app.player.MkvNativeAssExtractor
import com.fluxa.app.player.NativeAssTrack
import com.fluxa.app.player.NativeLibassRenderer
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.withContext
import java.io.File
import java.net.HttpURLConnection
import java.net.URL

internal fun selectedNativeAssSubtitle(
    currentSubtitle: MediaTrack?,
    externalSubtitles: List<ExternalSubtitleTrack>
): ExternalSubtitleTrack? {
    if (currentSubtitle == null) return null
    return externalSubtitles.firstOrNull { subtitle ->
        NativeLibassRenderer.isAssUrl(subtitle.url) &&
            (
                subtitle.label == currentSubtitle.label ||
                    subtitle.language == currentSubtitle.language ||
                    subtitle.url == currentSubtitle.label
                )
    }
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
    modifier: Modifier = Modifier
) {
    val context = androidx.compose.ui.platform.LocalContext.current

    val relay = remember(exoPlayer) { MediaPlayerController.getLibassRelay(exoPlayer) }
    val relayRenderer by (relay?.activeRenderer ?: MutableStateFlow(null)).collectAsStateWithLifecycle()

    var localRenderer by remember { mutableStateOf<NativeLibassRenderer?>(null) }

    LaunchedEffect(externalSubtitle?.url, embeddedSubtitle?.id) {
        val previous = localRenderer
        localRenderer = null
        previous?.close()
        val embedded = embeddedSubtitle
        if (embedded != null) {
            localRenderer = runCatching {
                NativeLibassRenderer.create(
                    assData = embedded.assData,
                    fonts = embedded.fonts,
                    fontsDir = context.filesDir.resolve("fonts").absolutePath
                )
            }.onFailure { e -> Log.w("NativeLibassOverlay", "Embedded renderer failed", e) }.getOrNull()
            return@LaunchedEffect
        }
        val url = externalSubtitle?.url ?: return@LaunchedEffect
        localRenderer = runCatching {
            val data = fetchSubtitleBytes(context, url)
            val fonts = MkvNativeAssExtractor.extractExternalFontHints(url)
            NativeLibassRenderer.create(
                assData = data,
                fonts = fonts,
                fontsDir = context.filesDir.resolve("fonts").absolutePath
            )
        }.onFailure { e -> Log.w("NativeLibassOverlay", "External renderer failed", e) }.getOrNull()
    }

    DisposableEffect(Unit) {
        onDispose {
            localRenderer?.close()
            localRenderer = null
        }
    }

    val activeRenderer = relayRenderer ?: localRenderer

    AndroidView(
        factory = { NativeLibassSubtitleView(it) },
        update = { view ->
            view.player = exoPlayer
            view.renderer = activeRenderer
            view.subtitleDelayMs = subtitleDelayMs
        },
        modifier = modifier
    )
}

private class NativeLibassSubtitleView(context: Context) : View(context) {
    var player: ExoPlayer? = null
        set(value) {
            field?.removeListener(playerListener)
            field = value
            value?.addListener(playerListener)
            scheduleNextFrame()
        }

    var renderer: NativeLibassRenderer? = null
        set(value) {
            field = value
            scheduleNextFrame()
        }

    var subtitleDelayMs: Long = 0L
        set(value) {
            field = value
            invalidate()
        }

    private var bitmap: Bitmap? = null

    private val playerListener = object : Player.Listener {
        override fun onIsPlayingChanged(isPlaying: Boolean) {
            if (isPlaying) invalidate()
        }

        override fun onPositionDiscontinuity(
            oldPosition: Player.PositionInfo,
            newPosition: Player.PositionInfo,
            reason: Int
        ) {
            invalidate()
        }
    }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        player?.addListener(playerListener)
        scheduleNextFrame()
    }

    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow()
        player?.removeListener(playerListener)
        bitmap?.recycle()
        bitmap = null
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        val activeRenderer = renderer ?: return
        val activePlayer = player ?: return
        if (width <= 0 || height <= 0) return

        val targetBitmap = bitmap?.takeIf { !it.isRecycled && it.width == width && it.height == height }
            ?: Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888).also {
                bitmap?.recycle()
                bitmap = it
            }

        val hasImage = activeRenderer.render(activePlayer.currentPosition + subtitleDelayMs, targetBitmap)
        if (hasImage) {
            canvas.drawBitmap(targetBitmap, 0f, 0f, null)
        }
        scheduleNextFrame()
    }

    private fun scheduleNextFrame() {
        if (renderer == null || player == null || !isAttachedToWindow) return
        if (player?.isPlaying == true) postInvalidateOnAnimation()
    }
}

private suspend fun fetchSubtitleBytes(context: Context, rawUrl: String): ByteArray = withContext(Dispatchers.IO) {
    val uri = runCatching { Uri.parse(rawUrl) }.getOrNull()
    when (uri?.scheme?.lowercase()) {
        "content" -> context.contentResolver.openInputStream(uri)?.use { it.readBytes() } ?: ByteArray(0)
        "file" -> File(requireNotNull(uri.path)).readBytes()
        "http", "https" -> {
            val connection = (URL(rawUrl).openConnection() as HttpURLConnection).apply {
                connectTimeout = 15_000
                readTimeout = 20_000
                instanceFollowRedirects = true
                setRequestProperty("User-Agent", com.fluxa.app.player.StreamRequestPolicy.DEFAULT_USER_AGENT)
            }
            try {
                connection.inputStream.use { it.readBytes() }
            } finally {
                connection.disconnect()
            }
        }
        else -> URL(rawUrl).openStream().use { it.readBytes() }
    }
}
