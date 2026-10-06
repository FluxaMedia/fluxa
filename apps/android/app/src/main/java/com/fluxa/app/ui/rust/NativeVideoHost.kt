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
    private var shaders = emptyList<String>()
    private var audioProcessingMode = "reference"
    private var videoOptions = emptyMap<String, String>()
    private var audioLanguage: String? = null
    private var subtitleLanguage: String? = null
    private var displayMode: DisplayModeHint? = null
    private val mediaSession = PlaybackMediaSession(context) { command, value ->
        renderer.pushAction(JSONObject(mapOf("type" to "mediaCommand", "command" to command, "value" to value)).toString())
    }
    var onPlayingChanged: ((Boolean) -> Unit)? = null

    private var lastTracks = ""
    private var tick = 0

    private val reportStatus = object : Runnable {
        override fun run() {
            val player = player ?: return
            if (player.poll()) renderer.reportVideoChapters(player.chapterListJson())
            if (tick++ % 4 == 0) {
                val tracks = player.trackListJson()
                if (tracks != lastTracks) {
                    lastTracks = tracks
                    renderer.reportVideoTracks(tracks)
                }
            }
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
                    videoOptions = mapOf(
                        "hwdec-dolby-vision" to request.optString("dolbyVisionMode").replace('_', '-').ifBlank { "auto" },
                        "android-dovi-tonemap" to request.optString("dolbyVisionTonemap").ifBlank { "auto" },
                        "android-display-peak" to request.optString("displayPeakNits").ifBlank { "0" },
                    )
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
                "shaders" -> {
                    val files = request.optJSONArray("files")
                    shaders = List(files?.length() ?: 0) { files!!.optString(it) }
                    player?.setShaders(shaders)
                }
                "mediaSession" -> request.optJSONObject("plan")?.let(mediaSession::update)
                "setSpeed" -> player?.setSpeed(request.optDouble("value"))
                "selectTracks" -> player?.selectTracks(
                    request.optString("audio").takeUnless { request.isNull("audio") || it.isBlank() },
                    request.optString("subtitle").takeUnless { request.isNull("subtitle") || it.isBlank() },
                    request.optBoolean("subtitlesOff"),
                )
                "thumbnail" -> player?.thumbnail(request.optDouble("time"))?.let(renderer::reportVideoThumbnail)
                "mpv" -> request.optJSONArray("args")?.let { args ->
                    player?.mpvCommand(Array(args.length()) { args.optString(it) })
                }
            }
        }
    }

    private fun load(url: String) {
        if (url.isBlank()) return
        stop()
        val player = runCatching { MpvVideo(context, mpvOptions, audioProcessingMode, videoOptions) }.getOrNull() ?: return
        this.player = player
        player.setShaders(shaders)
        lastTracks = ""
        tick = 0
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
