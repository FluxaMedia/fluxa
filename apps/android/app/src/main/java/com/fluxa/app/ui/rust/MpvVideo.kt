package com.fluxa.app.ui.rust

import android.content.Context
import android.media.AudioDeviceInfo
import android.media.AudioFormat
import android.media.AudioManager
import android.os.Build
import android.view.SurfaceHolder
import android.view.SurfaceView
import com.fluxa.app.BuildConfig

class MpvVideo(context: Context, options: String, audioProcessingMode: String) {
    private val appContext = context.applicationContext
    private val mpv = Mpv.create().also { require(it != 0L) { "libmpv could not be created" } }
    private var surface = 0L
    private var pendingUrl: String? = null
    private var loaded = false
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
        ).forEach { (key, value) -> Mpv.setOption(mpv, key, value) }
        options.lines().map(String::trim).filter { it.isNotEmpty() && !it.startsWith('#') }.forEach { line ->
            val key = line.substringBefore('=', "").trim()
            if (key.isNotEmpty() && key !in routeOwnedAudioOptions) {
                Mpv.setOption(mpv, key, line.substringAfter('=').trim())
            }
        }
        if (audioProcessingMode == "reference") spdif()?.let { Mpv.setOption(mpv, "audio-spdif", it) }
        filters[audioProcessingMode]?.let { Mpv.setOption(mpv, "af", it) }
        check(Mpv.initialize(mpv) >= 0) { "libmpv could not be initialized" }
    }

    val position get() = double("time-pos")
    val duration get() = double("duration")
    val paused get() = Mpv.getProperty(mpv, "pause") == "yes"
    val buffering get() = Mpv.getProperty(mpv, "paused-for-cache") == "yes" || (loaded && Mpv.getProperty(mpv, "core-idle") == "yes" && !paused)
    val hasFrame get() = double("video-params/w") > 0
    val displayModeHint: DisplayModeHint?
        get() {
            val width = double("display-mode-hint/width").toInt()
            val height = double("display-mode-hint/height").toInt()
            val rate = double("display-mode-hint/refresh-rate").toFloat()
            return if (width > 0 && height > 0 && rate > 0f) DisplayModeHint(width, height, rate) else null
        }

    private fun double(name: String) = Mpv.getProperty(mpv, name)?.toDoubleOrNull() ?: 0.0

    fun poll() {
        if (Mpv.drain(mpv)) {
            error = (Mpv.getProperty(mpv, "last-error/message") ?: Mpv.getProperty(mpv, "last-error/error"))?.trim()?.take(180)
        }
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
        setPaused(false)
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
        Mpv.setProperty(mpv, "volume", volume.coerceIn(0.0, 100.0).toString())
    }

    fun setSpeed(rate: Double) {
        Mpv.setProperty(mpv, "speed", rate.coerceIn(0.25, 4.0).toString())
    }

    fun attach(holder: SurfaceHolder, width: Int, height: Int) {
        if (surface != 0L) return
        surface = Mpv.attach(mpv, holder.surface)
        resize(width, height)
        Mpv.setProperty(mpv, "vo", "mediacodec_embed")
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

private val routeOwnedAudioOptions = setOf("audio-spdif", "audio-channels", "af")

private val filters = mapOf(
    "balanced" to "lavfi=[acompressor=threshold=0.78:ratio=1.5:attack=30:release=300:link=maximum,alimiter=limit=0.98]",
    "night" to "lavfi=[acompressor=threshold=0.55:ratio=3:attack=20:release=250:link=maximum,alimiter=limit=0.98]",
)
