package com.fluxa.app.ui.rust

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.os.Handler
import android.os.Looper
import android.view.Gravity
import android.view.MotionEvent
import android.view.ScaleGestureDetector
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
    private var thumbnailPlan: org.json.JSONObject? = null
    private var shaders = emptyList<String>()
    private var fit = "fit"
    private var videoAspect = 0.0
    private var videoSize = 0 to 0
    private var audioFilter = ""
    private var passthrough = false
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
            player.aspect.takeIf { it != videoAspect }?.let {
                videoAspect = it
                videoSize = player.size
                layoutVideo()
            }
            player.thumbnailProgress?.let { (done, count) -> renderer.reportVideoThumbnailProgress(done, count) }
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
        if (!BuildConfig.IS_TV) {
            renderer.nativeTextFields = true
            addView(NativeTextField(context, renderer), LayoutParams(0, 0))
        }
        renderer.onVideoRequest = ::onVideoRequest
    }

    private val pinch = ScaleGestureDetector(context, object : ScaleGestureDetector.SimpleOnScaleGestureListener() {
        private var factor = 1f
        private var stepped = false

        override fun onScaleBegin(detector: ScaleGestureDetector): Boolean {
            factor = 1f
            stepped = false
            return player != null
        }

        override fun onScale(detector: ScaleGestureDetector): Boolean {
            factor *= detector.scaleFactor
            if (!stepped && (factor > PINCH_STEP || factor < 1f / PINCH_STEP)) {
                stepped = true
                renderer.pushAction(JSONObject(mapOf("type" to "mediaCommand", "command" to "fitStep", "value" to if (factor > 1f) 1.0 else -1.0)).toString())
            }
            return true
        }
    }).apply { isQuickScaleEnabled = false }

    private var pinching = false

    override fun dispatchTouchEvent(event: MotionEvent): Boolean {
        if (player == null) return super.dispatchTouchEvent(event)
        pinch.onTouchEvent(event)
        if (pinch.isInProgress && !pinching) {
            pinching = true
            val cancel = MotionEvent.obtain(event).apply { action = MotionEvent.ACTION_CANCEL }
            super.dispatchTouchEvent(cancel)
            cancel.recycle()
        }
        if (pinching) {
            if (event.actionMasked == MotionEvent.ACTION_UP || event.actionMasked == MotionEvent.ACTION_CANCEL) pinching = false
            return true
        }
        return super.dispatchTouchEvent(event)
    }

    override fun onSizeChanged(width: Int, height: Int, oldWidth: Int, oldHeight: Int) {
        super.onSizeChanged(width, height, oldWidth, oldHeight)
        layoutVideo()
    }

    private fun layoutVideo() {
        if (width <= 0 || height <= 0) return
        val aspect = videoAspect.takeIf { it > 0 && fit != "stretch" }
        val (w, h) = when {
            aspect == null -> width to height
            fit == "original" && videoSize.first > 0 -> videoSize.first to videoSize.second
            fit == "fill" -> {
                val w = maxOf(width.toDouble(), height * aspect)
                w.toInt() to (w / aspect).toInt()
            }
            else -> {
                val w = minOf(width.toDouble(), height * aspect)
                w.toInt() to (w / aspect).toInt()
            }
        }
        val params = videoSurface.layoutParams as LayoutParams
        if (params.width == w && params.height == h) return
        videoSurface.layoutParams = LayoutParams(w, h, Gravity.CENTER)
    }

    private fun onVideoRequest(json: String) {
        val requests = runCatching { JSONArray(json) }.getOrNull() ?: return
        for (index in 0 until requests.length()) {
            val request = requests.optJSONObject(index) ?: continue
            val player = player
            when (request.optString("type")) {
                "configure" -> {
                    mpvOptions = request.optString("mpvOptions")
                    audioFilter = request.optString("audioFilter")
                    passthrough = request.optBoolean("passthrough")
                    videoOptions = mapOf(
                        "hwdec-dolby-vision" to request.optString("dolbyVisionMode"),
                        "android-dovi-tonemap" to request.optString("dolbyVisionTonemap"),
                        "android-display-peak" to request.optString("displayPeakNits"),
                    )
                    audioLanguage = request.optString("audioLanguage").ifEmpty { null }
                    subtitleLanguage = request.optString("subtitleLanguage").ifEmpty { null }
                }
                "thumbnails" -> {
                    thumbnailPlan = request
                    player?.retargetThumbnails(request)
                }
                "releaseStart" -> player?.releaseStart()
                "load" -> load(request.optString("url"), requestHeaders(request))
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
                "fit" -> {
                    fit = request.optString("mode").ifBlank { "fit" }
                    layoutVideo()
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

    private fun requestHeaders(request: JSONObject): List<Pair<String, String>> {
        val pairs = request.optJSONArray("headers") ?: return emptyList()
        return (0 until pairs.length()).mapNotNull { index ->
            val pair = pairs.optJSONArray(index) ?: return@mapNotNull null
            pair.optString(0) to pair.optString(1)
        }
    }

    private fun load(url: String, headers: List<Pair<String, String>>) {
        if (url.isBlank()) return
        stop()
        val player = runCatching { MpvVideo(context, mpvOptions, audioFilter, passthrough, videoOptions, thumbnailPlan) }.getOrNull() ?: return
        this.player = player
        player.setShaders(shaders)
        lastTracks = ""
        tick = 0
        videoAspect = 0.0
        videoSize = 0 to 0
        onPlayingChanged?.invoke(true)
        player.load(url, audioLanguage, subtitleLanguage, headers)
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

private const val PINCH_STEP = 1.15f
