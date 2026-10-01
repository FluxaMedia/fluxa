package com.fluxa.app.ui.rust

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.os.Handler
import android.os.Looper
import android.view.ViewGroup
import android.widget.FrameLayout
import com.fluxa.app.BuildConfig
import org.json.JSONArray
import org.json.JSONObject
import kotlin.math.abs

class NativeVideoHost(context: Context) : FrameLayout(context) {
    val renderer = FluxaNativeRendererView(context)
    private val videoSurface = MpvSurface(context)
    private val handler = Handler(Looper.getMainLooper())
    private var player: MpvVideo? = null
    private var mpvOptions = ""
    private var audioProcessingMode = "reference"
    private var p7FelGpu = false
    private var audioLanguage: String? = null
    private var subtitleLanguage: String? = null
    private var displayMode: DisplayModeHint? = null
    private val mediaSession = PlaybackMediaSession(context) { command, value ->
        renderer.pushAction(JSONObject(mapOf("type" to "mediaCommand", "command" to command, "value" to value)).toString())
    }
    var onPlayingChanged: ((Boolean) -> Unit)? = null

    private val reportStatus = object : Runnable {
        override fun run() {
            val player = player ?: return
            player.poll()
            if (BuildConfig.IS_TV) applyDisplayMode(player.displayModeHint)
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
                    p7FelGpu = request.optBoolean("p7FelGpu")
                    audioLanguage = request.optString("audioLanguage").takeUnless { it.isBlank() || it == "none" }
                    subtitleLanguage = request.optString("subtitleLanguage").takeUnless { it.isBlank() || it == "none" }
                }
                "load" -> load(request.optString("url"))
                "stop" -> stop()
                "togglePause" -> player?.let { it.setPaused(!it.paused) }
                "seek" -> player?.let { it.seekTo(it.position + request.optDouble("seconds")) }
                "seekTo" -> player?.seekTo(request.optDouble("seconds"))
                "toggleMute" -> player?.toggleMute()
                "setVolume" -> player?.setVolume(request.optDouble("value"))
                "mediaSession" -> request.optJSONObject("plan")?.let(mediaSession::update)
                "setSpeed" -> player?.setSpeed(request.optDouble("value"))
            }
        }
    }

    private fun load(url: String) {
        if (url.isBlank()) return
        stop()
        val player = runCatching { MpvVideo(context, mpvOptions, audioProcessingMode, p7FelGpu) }.getOrNull() ?: return
        this.player = player
        onPlayingChanged?.invoke(true)
        player.load(url, audioLanguage, subtitleLanguage)
        videoSurface.video = player
        handler.post(reportStatus)
    }

    fun stop() {
        if (player != null) onPlayingChanged?.invoke(false)
        handler.removeCallbacks(reportStatus)
        mediaSession.release()
        videoSurface.video = null
        player?.release()
        player = null
        applyDisplayMode(null)
    }

    private fun applyDisplayMode(hint: DisplayModeHint?) {
        if (hint == displayMode) return
        displayMode = hint
        val window = activity()?.window ?: return
        val modes = display?.supportedModes.orEmpty()
        val current = display?.mode
        val mode = hint?.let {
            modes.filter { m -> m.physicalWidth >= it.width && m.physicalHeight >= it.height }
                .filter { m -> rateMatches(m.refreshRate, it.refreshRate) }
                .minWithOrNull(compareBy({ m -> m.physicalWidth != current?.physicalWidth }, { m -> m.physicalWidth }, { m -> m.refreshRate }))
        }
        window.attributes = window.attributes.apply { preferredDisplayModeId = mode?.modeId ?: 0 }
    }

    private fun rateMatches(display: Float, content: Float): Boolean {
        val ratio = display / content
        return ratio >= 0.99f && abs(ratio - Math.round(ratio)) < 0.01f
    }

    private fun activity(): Activity? {
        var ctx = context
        while (ctx is ContextWrapper) {
            if (ctx is Activity) return ctx
            ctx = ctx.baseContext
        }
        return null
    }

    override fun onDetachedFromWindow() {
        stop()
        super.onDetachedFromWindow()
    }
}
