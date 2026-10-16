package com.fluxa.app.ui.rust

import android.content.Context
import android.media.AudioDeviceInfo
import android.media.AudioFormat
import android.media.AudioManager
import android.os.Build
import android.view.SurfaceHolder
import android.view.SurfaceView
import com.fluxa.app.BuildConfig
import org.json.JSONObject

class MpvVideo(
    context: Context,
    options: String,
    audioFilter: String,
    passthrough: Boolean,
    videoOptions: Map<String, String>,
    thumbnails: JSONObject?,
) {
    private val appContext = context.applicationContext
    private val mpv = Mpv.create().also { require(it != 0L) { "libmpv could not be created" } }
    private var surface = 0L
    private var pendingUrl: String? = null
    private var loaded = false
    private var shaders = emptyList<String>()
    private var held = thumbnails?.optBoolean("hold") == true
    var error: String? = null
        private set

    init {
        val configDir = appContext.filesDir.resolve("mpv").apply { mkdirs() }
        val lowRam = (appContext.getSystemService(Context.ACTIVITY_SERVICE) as? android.app.ActivityManager)?.isLowRamDevice == true
        mapOf(
            "vo" to "mediacodec_embed",
            "hwdec" to "mediacodec",
            "ao" to "audiotrack",
            "android-frame-rate-switch" to if (BuildConfig.IS_TV) "always" else "seamless",
            "config" to "yes",
            "config-dir" to configDir.absolutePath,
            "video-sync" to "audio",
            "cache" to "yes",
            "cache-secs" to if (lowRam) "30" else "60",
            "demuxer-max-bytes" to if (lowRam) "40MiB" else "96MiB",
            "demuxer-lavf-o" to "allowed_extensions=ALL",
            "network-timeout" to "15",
            "keep-open" to "no",
            "sub-auto" to "fuzzy",
            "sub-ass" to "yes",
            "sub-ass-override" to "scale",
            "audio-display" to "no",
            "idle" to "once",
            "thumbnail-width" to "320",
        ).plus(thumbnailOptions(thumbnails)).plus(videoOptions).forEach { (key, value) -> Mpv.setOption(mpv, key, value) }
        options.lines().filter { it.isNotEmpty() }.forEach { line ->
            Mpv.setOption(mpv, line.substringBefore('='), line.substringAfter('='))
        }
        if (passthrough) spdif()?.let { Mpv.setOption(mpv, "audio-spdif", it) }
        if (audioFilter.isNotEmpty()) Mpv.setOption(mpv, "af", audioFilter)
        check(Mpv.initialize(mpv) >= 0) { "libmpv could not be initialized" }
        if (thumbnailOptions(thumbnails)["thumbnail-url"].orEmpty().isNotEmpty()) Mpv.command(mpv, arrayOf("thumbnail-reload"))
    }

    val position get() = double("time-pos")
    val duration get() = double("duration")
    val paused get() = Mpv.getProperty(mpv, "pause") == "yes"
    val buffering get() = Mpv.getProperty(mpv, "paused-for-cache") == "yes" || (loaded && Mpv.getProperty(mpv, "core-idle") == "yes" && !paused)
    val hasFrame get() = double("video-params/w") > 0
    val aspect: Double
        get() {
            val width = double("video-out-params/dw")
            val height = double("video-out-params/dh")
            return if (width > 0 && height > 0) width / height else 0.0
        }
    val displayModeHint: DisplayModeHint?
        get() {
            val width = double("display-mode-hint/width").toInt()
            val height = double("display-mode-hint/height").toInt()
            val rate = double("display-mode-hint/refresh-rate").toFloat()
            return if (width > 0 && height > 0 && rate > 0f) DisplayModeHint(width, height, rate) else null
        }

    fun trackListJson(): String {
        val count = Mpv.getProperty(mpv, "track-list/count")?.toIntOrNull() ?: 0
        val tracks = org.json.JSONArray()
        for (index in 0 until count) {
            val prefix = "track-list/$index"
            val type = Mpv.getProperty(mpv, "$prefix/type")
            if (type != "audio" && type != "sub") continue
            tracks.put(
                org.json.JSONObject()
                    .put("id", Mpv.getProperty(mpv, "$prefix/id"))
                    .put("subtitle", type == "sub")
                    .put("language", Mpv.getProperty(mpv, "$prefix/lang"))
                    .put("title", Mpv.getProperty(mpv, "$prefix/title") ?: Mpv.getProperty(mpv, "$prefix/codec")?.uppercase())
                    .put("selected", Mpv.getProperty(mpv, "$prefix/selected") == "yes")
                    .put("external", Mpv.getProperty(mpv, "$prefix/external") == "yes"),
            )
        }
        return tracks.toString()
    }

    fun chapterListJson(): String {
        val count = Mpv.getProperty(mpv, "chapter-list/count")?.toIntOrNull() ?: 0
        val chapters = org.json.JSONArray()
        for (index in 0 until count) {
            val prefix = "chapter-list/$index"
            val start = Mpv.getProperty(mpv, "$prefix/time")?.toDoubleOrNull() ?: continue
            chapters.put(
                org.json.JSONObject()
                    .put("startMs", (start * 1000).toLong())
                    .put("title", Mpv.getProperty(mpv, "$prefix/title")?.takeIf { it.isNotBlank() } ?: "Chapter ${index + 1}"),
            )
        }
        return chapters.toString()
    }

    val thumbnailProgress: Pair<Int, Int>?
        get() {
            val count = Mpv.getProperty(mpv, "thumbnail-info/count")?.toIntOrNull() ?: return null
            val done = Mpv.getProperty(mpv, "thumbnail-info/done")?.toIntOrNull() ?: return null
            return if (count > 0) done to count else null
        }

    fun retargetThumbnails(plan: JSONObject) {
        thumbnailOptions(plan).forEach { (key, value) -> Mpv.setProperty(mpv, key, value) }
        Mpv.command(mpv, arrayOf("thumbnail-reload"))
    }

    fun releaseStart() {
        if (!held) return
        held = false
        if (loaded) setPaused(false)
    }

    fun thumbnail(seconds: Double): ByteArray? = Mpv.thumbnail(mpv, seconds)

    fun selectTracks(audio: String?, subtitle: String?, subtitlesOff: Boolean) {
        if (audio != null) Mpv.setProperty(mpv, "aid", audio)
        if (subtitle != null) Mpv.setProperty(mpv, "sid", subtitle) else if (subtitlesOff) Mpv.setProperty(mpv, "sid", "no")
    }

    fun mpvCommand(args: Array<String>) {
        Mpv.command(mpv, args)
    }

    private fun double(name: String) = Mpv.getProperty(mpv, name)?.toDoubleOrNull() ?: 0.0

    fun poll(): Boolean {
        val flags = Mpv.drain(mpv)
        if (flags and 1 != 0) {
            error = (Mpv.getProperty(mpv, "last-error/message") ?: Mpv.getProperty(mpv, "last-error/error"))?.trim()?.take(180)
        }
        return flags and 2 != 0
    }

    fun load(url: String, audioLanguage: String?, subtitleLanguage: String?) {
        Mpv.setProperty(mpv, "alang", audioLanguage.orEmpty())
        Mpv.setProperty(mpv, "slang", subtitleLanguage.orEmpty())
        if (url.startsWith("http://127.0.0.1:")) Mpv.setProperty(mpv, "network-timeout", "90")
        if (surface != 0L) start(url) else pendingUrl = url
    }

    private fun start(url: String) {
        loaded = true
        error = null
        Mpv.command(mpv, arrayOf("loadfile", url, "replace"))
        setPaused(held)
    }

    fun setPaused(paused: Boolean) {
        Mpv.setProperty(mpv, "pause", if (paused) "yes" else "no")
    }

    fun seekTo(seconds: Double) {
        Mpv.command(mpv, arrayOf("seek", seconds.coerceAtLeast(0.0).toString(), "absolute"))
    }

    fun toggleMute() {
        Mpv.command(mpv, arrayOf("cycle", "mute"))
    }

    fun setVolume(volume: Double) {
        Mpv.setProperty(mpv, "volume", volume.toString())
    }

    fun setSpeed(rate: Double) {
        Mpv.setProperty(mpv, "speed", rate.toString())
    }

    private fun applyOutput() {
        if (shaders.isEmpty()) {
            Mpv.setProperty(mpv, "glsl-shaders", "")
            Mpv.setProperty(mpv, "hwdec", "mediacodec")
            Mpv.setProperty(mpv, "vo", "mediacodec_embed")
        } else {
            val dir = appContext.filesDir.resolve("mpv/anime4k").apply { mkdirs() }
            val paths = shaders.map { name ->
                dir.resolve(name).also { file ->
                    if (!file.exists()) appContext.assets.open("anime4k/$name").use { input -> file.outputStream().use { input.copyTo(it) } }
                }.absolutePath
            }
            Mpv.setProperty(mpv, "hwdec", "mediacodec")
            Mpv.setProperty(mpv, "gpu-context", "android")
            Mpv.setProperty(mpv, "vo", "gpu-next")
            Mpv.setProperty(mpv, "glsl-shaders", paths.joinToString(":"))
        }
    }

    fun setShaders(files: List<String>) {
        if (files == shaders) return
        shaders = files
        if (surface != 0L) runCatching { applyOutput() }
    }

    fun attach(holder: SurfaceHolder, width: Int, height: Int) {
        if (surface != 0L) return
        surface = Mpv.attach(mpv, holder.surface)
        resize(width, height)
        applyOutput()
        pendingUrl?.let { pendingUrl = null; start(it) }
    }

    fun resize(width: Int, height: Int) {
        if (width > 0 && height > 0) Mpv.setProperty(mpv, "android-surface-size", "${width}x$height")
    }

    fun detach() {
        if (surface == 0L) return
        Mpv.setProperty(mpv, "vo", "null")
        Mpv.detach(mpv, surface)
        surface = 0L
    }

    fun release() {
        detach()
        Mpv.destroy(mpv)
    }

    private fun spdif(): String? {
        val encodings = appContext.getSystemService(AudioManager::class.java)
            ?.getDevices(AudioManager.GET_DEVICES_OUTPUTS)
            ?.filter { it.type == AudioDeviceInfo.TYPE_HDMI || it.type == AudioDeviceInfo.TYPE_HDMI_ARC || (Build.VERSION.SDK_INT >= 31 && it.type == AudioDeviceInfo.TYPE_HDMI_EARC) }
            ?.flatMap { it.encodings.toList() }
            ?.toSet()
            .orEmpty()
        return buildList {
            if (AudioFormat.ENCODING_DOLBY_TRUEHD in encodings) add("truehd")
            if (AudioFormat.ENCODING_E_AC3 in encodings) add("eac3")
            if (AudioFormat.ENCODING_AC3 in encodings) add("ac3")
            if (AudioFormat.ENCODING_DTS_HD in encodings) add("dts-hd")
            if (AudioFormat.ENCODING_DTS in encodings) add("dts")
        }.takeIf { it.isNotEmpty() }?.joinToString(",")
    }
}

class MpvSurface(context: Context) : SurfaceView(context), SurfaceHolder.Callback {
    var video: MpvVideo? = null
        set(value) {
            if (field === value) return
            field?.detach()
            field = value
            if (holder.surface?.isValid == true) value?.attach(holder, width, height)
        }

    init {
        holder.addCallback(this)
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        video?.attach(holder, width, height)
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        video?.resize(width, height)
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        video?.detach()
    }
}

data class DisplayModeHint(val width: Int, val height: Int, val refreshRate: Float)

private fun thumbnailOptions(plan: JSONObject?): Map<String, String> {
    if (plan == null) return mapOf("thumbnails" to "no")
    if (!plan.optBoolean("enabled", true)) return mapOf("thumbnails" to "no")
    val headers = plan.optJSONArray("headers")
    val fields = List(headers?.length() ?: 0) { index ->
        val pair = headers!!.optJSONArray(index)
        "${pair?.optString(0)}: ${pair?.optString(1)}".replace(",", "\\,")
    }
    val url = plan.optString("url").takeUnless { plan.isNull("url") }
    return mapOf(
        "thumbnail-url" to url.orEmpty(),
        "thumbnail-http-header-fields" to fields.joinToString(","),
        "thumbnail-interval" to plan.optDouble("interval", 10.0).coerceAtLeast(5.0).toString(),
        "thumbnail-sweep" to if (plan.optBoolean("sheet", true)) "yes" else "no",
        "thumbnail-delay" to "0",
        "thumbnails" to "yes",
    )
}
