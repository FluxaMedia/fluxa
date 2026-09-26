package com.fluxa.app.ui.rust

import android.content.Context
import android.os.Handler
import android.os.Looper
import android.view.SurfaceView
import android.view.ViewGroup
import android.widget.FrameLayout
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.player.MediaPlayerController
import org.json.JSONArray

class NativeVideoHost(context: Context) : FrameLayout(context) {
    val renderer = FluxaNativeRendererView(context)
    private val videoSurface = SurfaceView(context)
    private val handler = Handler(Looper.getMainLooper())
    private var controller: MediaPlayerController? = null
    private var hasFrame = false
    private var error: String? = null
    var createPlayer: (() -> ExoPlayer)? = null

    private val listener = object : Player.Listener {
        override fun onRenderedFirstFrame() {
            hasFrame = true
        }

        override fun onPlayerError(playbackError: PlaybackException) {
            error = playbackError.message ?: playbackError.errorCodeName
        }
    }

    private val reportStatus = object : Runnable {
        override fun run() {
            val player = controller?.exoPlayer ?: return
            val buffering = if (player.playbackState == Player.STATE_BUFFERING) {
                player.bufferedPercentage / 100f
            } else {
                -1f
            }
            renderer.reportVideoStatus(
                player.currentPosition.coerceAtLeast(0L) / 1000.0,
                player.duration.takeIf { it > 0L }?.div(1000.0) ?: 0.0,
                !player.playWhenReady,
                player.volume == 0f,
                hasFrame,
                buffering,
                error,
            )
            handler.postDelayed(this, 250L)
        }
    }

    init {
        val match = LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT)
        addView(videoSurface, match)
        addView(renderer, LayoutParams(match))
        renderer.onVideoRequest = ::onVideoRequest
    }

    private fun onVideoRequest(json: String) {
        val requests = runCatching { JSONArray(json) }.getOrNull() ?: return
        for (index in 0 until requests.length()) {
            val request = requests.optJSONObject(index) ?: continue
            val player = controller?.exoPlayer
            when (request.optString("type")) {
                "load" -> load(request.optString("url"))
                "stop" -> stop()
                "togglePause" -> player?.let { it.playWhenReady = !it.playWhenReady }
                "seek" -> player?.let { it.seekTo((it.currentPosition + request.optDouble("seconds") * 1000).toLong().coerceAtLeast(0L)) }
                "seekTo" -> player?.seekTo((request.optDouble("seconds") * 1000).toLong().coerceAtLeast(0L))
                "toggleMute" -> player?.let { it.volume = if (it.volume == 0f) 1f else 0f }
            }
        }
    }

    private fun load(url: String) {
        if (url.isBlank()) return
        stop()
        val player = createPlayer?.invoke() ?: return
        player.addListener(listener)
        player.setVideoSurfaceView(videoSurface)
        controller = MediaPlayerController(context, player).also { it.prepareAndPlay(url) }
        handler.post(reportStatus)
    }

    fun stop() {
        handler.removeCallbacks(reportStatus)
        controller?.exoPlayer?.let { player ->
            player.removeListener(listener)
            player.clearVideoSurfaceView(videoSurface)
            MediaPlayerController.releaseExoPlayer(player)
        }
        controller = null
        hasFrame = false
        error = null
    }

    override fun onDetachedFromWindow() {
        stop()
        super.onDetachedFromWindow()
    }
}
