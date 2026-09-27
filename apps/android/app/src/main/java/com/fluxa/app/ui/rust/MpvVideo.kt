package com.fluxa.app.ui.rust

import android.content.Context
import android.media.AudioDeviceInfo
import android.media.AudioFormat
import android.media.AudioManager
import android.os.Build
import android.view.SurfaceHolder
import android.view.SurfaceView
import dev.jdtech.mpv.MPVLib

class MpvVideo(context: Context, options: String, audioProcessingMode: String) {
    private val appContext = context.applicationContext
    private val mpv = requireNotNull(MPVLib.create(appContext)) { "libmpv could not be created" }
    private var surfaceAttached = false
    private var pendingUrl: String? = null
    private var loaded = false
    var error: String? = null
        private set

    init {
        val configDir = appContext.filesDir.resolve("mpv").apply { mkdirs() }
        val cacheDir = appContext.cacheDir.resolve("mpv").apply { mkdirs() }
        val lowRam = (appContext.getSystemService(Context.ACTIVITY_SERVICE) as? android.app.ActivityManager)?.isLowRamDevice == true
        mapOf(
            "vo" to "gpu",
            "ao" to "audiotrack",
            "gpu-api" to "opengl",
            "gpu-context" to "android",
            "hwdec" to "auto-safe",
            "config" to "yes",
            "config-dir" to configDir.absolutePath,
            "gpu-shader-cache-dir" to cacheDir.absolutePath,
            "icc-cache-dir" to cacheDir.absolutePath,
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
        ).forEach { (key, value) -> runCatching { mpv.setOptionString(key, value) } }
        options.lines().map(String::trim).filter { it.isNotEmpty() && !it.startsWith('#') }.forEach { line ->
            val key = line.substringBefore('=', "").trim()
            if (key.isNotEmpty() && key !in routeOwnedAudioOptions) {
                runCatching { mpv.setOptionString(key, line.substringAfter('=').trim()) }
            }
        }
        if (audioProcessingMode == "reference") spdif()?.let { runCatching { mpv.setOptionString("audio-spdif", it) } }
        filters[audioProcessingMode]?.let { runCatching { mpv.setOptionString("af", it) } }
        mpv.init()
        runCatching { mpv.setOptionString("force-window", "no") }
        runCatching { mpv.setOptionString("idle", "once") }
        mpv.addLogObserver(object : MPVLib.LogObserver {
            override fun logMessage(prefix: String, level: Int, text: String) {
                if (level <= 20 && prefix == "cplayer") error = text.trim().take(180)
            }
        })
    }

    val position get() = mpv.getPropertyDouble("time-pos") ?: 0.0
    val duration get() = mpv.getPropertyDouble("duration") ?: 0.0
    val paused get() = mpv.getPropertyBoolean("pause") ?: false
    val buffering get() = mpv.getPropertyBoolean("paused-for-cache") == true || (loaded && mpv.getPropertyBoolean("core-idle") == true && !paused)
    val hasFrame get() = (mpv.getPropertyInt("video-params/w") ?: 0) > 0

    fun load(url: String, audioLanguage: String?, subtitleLanguage: String?) {
        runCatching { mpv.setOptionString("alang", audioLanguage.orEmpty()) }
        runCatching { mpv.setOptionString("slang", subtitleLanguage.orEmpty()) }
        if (url.startsWith("http://127.0.0.1:")) runCatching { mpv.setOptionString("network-timeout", "90") }
        if (surfaceAttached) start(url) else pendingUrl = url
    }

    private fun start(url: String) {
        loaded = true
        error = null
        mpv.command(arrayOf("loadfile", url, "replace"))
        setPaused(false)
    }

    fun setPaused(paused: Boolean) {
        runCatching { mpv.setPropertyBoolean("pause", paused) }
    }

    fun seekTo(seconds: Double) {
        runCatching { mpv.command(arrayOf("seek", seconds.coerceAtLeast(0.0).toString(), "absolute")) }
    }

    fun toggleMute() {
        runCatching { mpv.command(arrayOf("cycle", "mute")) }
    }

    fun attach(holder: SurfaceHolder, width: Int, height: Int) {
        runCatching { mpv.attachSurface(holder.surface) }
        surfaceAttached = true
        runCatching { mpv.setOptionString("force-window", "yes") }
        runCatching { mpv.setOptionString("vo", "gpu") }
        resize(width, height)
        pendingUrl?.let { pendingUrl = null; start(it) }
    }

    fun resize(width: Int, height: Int) {
        if (width > 0 && height > 0) runCatching { mpv.setPropertyString("android-surface-size", "${width}x$height") }
    }

    fun detach() {
        if (!surfaceAttached) return
        surfaceAttached = false
        runCatching { mpv.setOptionString("vo", "null") }
        runCatching { mpv.setOptionString("force-window", "no") }
        runCatching { mpv.detachSurface() }
    }

    fun release() {
        detach()
        runCatching { mpv.destroy() }
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

private val routeOwnedAudioOptions = setOf("audio-spdif", "audio-channels", "af")

private val filters = mapOf(
    "balanced" to "lavfi=[acompressor=threshold=0.78:ratio=1.5:attack=30:release=300:link=maximum,alimiter=limit=0.98]",
    "night" to "lavfi=[acompressor=threshold=0.55:ratio=3:attack=20:release=250:link=maximum,alimiter=limit=0.98]",
)
