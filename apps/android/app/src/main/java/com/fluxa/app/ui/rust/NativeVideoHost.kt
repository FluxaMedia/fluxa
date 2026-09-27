package com.fluxa.app.ui.rust

import android.content.Context
import android.os.Handler
import android.os.Looper
import android.view.ViewGroup
import android.widget.FrameLayout
import org.json.JSONArray

class NativeVideoHost(context: Context) : FrameLayout(context) {
    val renderer = FluxaNativeRendererView(context)
    private val videoSurface = MpvSurface(context)
    private val handler = Handler(Looper.getMainLooper())
    private var player: MpvVideo? = null
    private var mpvOptions = ""
    private var audioProcessingMode = "reference"
    private var audioLanguage: String? = null
    private var subtitleLanguage: String? = null
    var onPlayingChanged: ((Boolean) -> Unit)? = null

    private val reportStatus = object : Runnable {
        override fun run() {
            val player = player ?: return
            renderer.reportVideoStatus(
                player.position,
                player.duration,
                player.paused,
                false,
                player.hasFrame,
                if (player.buffering) 0f else -1f,
                player.error,
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
                "togglePause" -> player?.let { it.setPaused(!it.paused) }
                "seek" -> player?.let { it.seekTo(it.position + request.optDouble("seconds")) }
                "seekTo" -> player?.seekTo(request.optDouble("seconds"))
                "toggleMute" -> player?.toggleMute()
            }
        }
    }

    private fun load(url: String) {
        if (url.isBlank()) return
        stop()
        val player = runCatching { MpvVideo(context, mpvOptions, audioProcessingMode) }.getOrNull() ?: return
        this.player = player
        onPlayingChanged?.invoke(true)
        player.load(url, audioLanguage, subtitleLanguage)
        videoSurface.video = player
        handler.post(reportStatus)
    }

    fun stop() {
        if (player != null) onPlayingChanged?.invoke(false)
        handler.removeCallbacks(reportStatus)
        videoSurface.video = null
        player?.release()
        player = null
    }

    override fun onDetachedFromWindow() {
        stop()
        super.onDetachedFromWindow()
    }
}
