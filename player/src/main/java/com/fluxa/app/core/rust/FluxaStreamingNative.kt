package com.fluxa.app.core.rust

import com.google.gson.Gson

object FluxaStreamingNative {
    private val gson = Gson()
    private val loaded: Boolean = run { System.loadLibrary("fluxa_streaming_engine"); true }

    fun startLocalStreamServer(
        targetUrl: String,
        headers: Map<String, String>,
        preferredPort: Int = 0
    ): String = call { startLocalStreamServerNative(targetUrl, gson.toJson(headers), preferredPort) }

    fun startDvRewriteLocalStreamServer(
        targetUrl: String,
        headers: Map<String, String>,
        dvConfigJson: String,
        preferredPort: Int = 0
    ): String = call {
        startDvRewriteLocalStreamServerNative(targetUrl, gson.toJson(headers), dvConfigJson, preferredPort)
    }

    fun stopLocalStreamServer(serverId: String): Boolean = call {
        stopLocalStreamServerNative(serverId)
    }

    fun startTorrentServer(cacheDir: String, preferredPort: Int, accessToken: String = ""): String = call {
        startTorrentServerNative(cacheDir, preferredPort, accessToken)
    }

    fun stopTorrentServer(): Boolean = call { stopTorrentServerNative() }

    fun dvRpuSelfTest(): Boolean = call { dvRpuSelfTestNative() }

    fun dvAutoDetectWasIptPqc2(): Boolean = call { dvAutoDetectWasIptPqc2Native() }

    fun dvRewriteSegmentBytes(
        data: ByteArray,
        rpuMode: Int,
        zeroLevel5: Boolean,
        removeHdr10Plus: Boolean
    ): ByteArray = call { dvRewriteSegmentBytesNative(data, rpuMode, zeroLevel5, removeHdr10Plus) }

    fun dvGetStreamStats(): String = call { dvGetStreamStatsJsonNative() }

    fun parseMkvChapters(data: ByteArray): String = call { parseMkvChaptersNative(data) }

    private inline fun <T> call(block: () -> T): T {
        check(loaded) { "Fluxa streaming engine native library is not loaded." }
        return block()
    }

    private external fun startLocalStreamServerNative(targetUrl: String, headersJson: String, preferredPort: Int): String
    private external fun startDvRewriteLocalStreamServerNative(targetUrl: String, headersJson: String, dvConfigJson: String, preferredPort: Int): String
    private external fun stopLocalStreamServerNative(serverId: String): Boolean
    private external fun startTorrentServerNative(cacheDir: String, preferredPort: Int, accessToken: String): String
    private external fun stopTorrentServerNative(): Boolean
    private external fun dvRpuSelfTestNative(): Boolean
    private external fun dvAutoDetectWasIptPqc2Native(): Boolean
    private external fun dvRewriteSegmentBytesNative(data: ByteArray, rpuMode: Int, zeroLevel5: Boolean, removeHdr10Plus: Boolean): ByteArray
    private external fun dvGetStreamStatsJsonNative(): String
    private external fun parseMkvChaptersNative(data: ByteArray): String
}
