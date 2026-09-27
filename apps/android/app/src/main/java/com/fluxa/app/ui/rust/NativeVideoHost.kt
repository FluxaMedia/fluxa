package com.fluxa.app.ui.rust

import android.content.Context
import android.os.Handler
import android.os.Looper
import android.view.ViewGroup
import android.widget.FrameLayout
import com.fluxa.app.player.MpvAndroidSurfaceView
import com.fluxa.app.player.MpvEmbeddedPlayer
import org.json.JSONArray

class NativeVideoHost(context: Context) : FrameLayout(context) {
    val renderer = FluxaNativeRendererView(context)
    private val videoSurface = MpvAndroidSurfaceView(context)
    private val handler = Handler(Looper.getMainLooper())
    private var player: MpvEmbeddedPlayer? = null
    private var mpvOptions = ""
    private var audioProcessingMode = "reference"
    private var audioLanguage: String? = null
    private var subtitleLanguage: String? = null
    var onPlayingChanged: ((Boolean) -> Unit)? = null

    private val reportStatus = object : Runnable {
        override fun run() {
            val state = player?.state?.value ?: return
            renderer.reportVideoStatus(
                state.positionMs.coerceAtLeast(0L) / 1000.0,
                state.durationMs.coerceAtLeast(0L) / 1000.0,
                !state.isPlaying,
                false,
                state.isVideoReady,
                if (state.isBuffering) 0f else -1f,
                state.error,
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
            val player = player
            when (request.optString("type")) {
                "configure" -> {
                    mpvOptions = request.optString("mpvOptions")
                    audioProcessingMode = request.optString("audioProcessingMode").ifBlank { "reference" }
                    audioLanguage = request.optString("audioLanguage").takeUnless { it.isBlank() || it == "none" }
                    subtitleLanguage = request.optString("subtitleLanguage").takeUnless { it.isBlank() || it == "none" }
                }
                "load" -> load(request.optString("url"))
                "stop" -> stop()
                "togglePause" -> player?.let { it.setPaused(it.state.value.isPlaying) }
                "seek" -> player?.let {
                    it.seekTo((it.state.value.positionMs + request.optDouble("seconds") * 1000).toLong().coerceAtLeast(0L))
                }
                "seekTo" -> player?.seekTo((request.optDouble("seconds") * 1000).toLong().coerceAtLeast(0L))
            }
        }
    }

    private fun load(url: String) {
        if (url.isBlank()) return
        stop()
        val player = runCatching { MpvEmbeddedPlayer(context, mpvOptions, audioProcessingMode) }.getOrNull() ?: return
        this.player = player
        onPlayingChanged?.invoke(true)
        videoSurface.bind(player)
        player.prepareAndPlay(url, null, emptyList(), 0L, audioLanguage, subtitleLanguage)
        handler.post(reportStatus)
    }

    fun stop() {
        if (player != null) onPlayingChanged?.invoke(false)
        handler.removeCallbacks(reportStatus)
        videoSurface.bind(null)
        player?.release()
        player = null
    }

    override fun onDetachedFromWindow() {
        stop()
        super.onDetachedFromWindow()
    }
}
