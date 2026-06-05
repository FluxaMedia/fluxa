package com.fluxa.app.core.rust

import com.google.gson.Gson

object FluxaLocalStreamServer {
    private val gson = Gson()

    data class Session(
        val id: String,
        val url: String,
        val port: Int
    ) {
        fun stop() {
            FluxaStreamingNative.stopLocalStreamServer(id)
        }
    }

    fun start(
        targetUrl: String,
        headers: Map<String, String>,
        preferredPort: Int = 0
    ): Session? {
        val json = FluxaStreamingNative.startLocalStreamServer(targetUrl, headers, preferredPort)
        return runCatching { gson.fromJson(json, Session::class.java) }.getOrNull()
            ?.takeIf { it.id.isNotBlank() && it.url.isNotBlank() }
    }

    fun startWithDvRewrite(
        targetUrl: String,
        headers: Map<String, String>,
        action: String,
        rpuMode: Int = 2,
        deviceHasDvDecoder: Boolean = false,
        deviceHasDvDisplay: Boolean = false,
        zeroLevel5: Boolean = false,
        removeHdr10Plus: Boolean = false,
        fallbackMode: String = "auto",
        preferredPort: Int = 0
    ): Session? {
        val dvConfigJson = gson.toJson(mapOf(
            "action" to action,
            "rpu_mode" to rpuMode,
            "device_has_dv_decoder" to deviceHasDvDecoder,
            "device_has_dv_display" to deviceHasDvDisplay,
            "zero_level5" to zeroLevel5,
            "remove_hdr10plus" to removeHdr10Plus,
            "fallback_mode" to fallbackMode
        ))
        val json = FluxaStreamingNative.startDvRewriteLocalStreamServer(
            targetUrl, headers, dvConfigJson, preferredPort
        )
        return runCatching { gson.fromJson(json, Session::class.java) }.getOrNull()
            ?.takeIf { it.id.isNotBlank() && it.url.isNotBlank() }
    }
}
