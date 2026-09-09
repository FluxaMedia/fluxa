package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreNative
import okhttp3.Request

object HttpRequestSecurity {
    fun preferHttps(url: String): String =
        FluxaCoreNative.preferHttpsAssetUrl(url.trim()) ?: url.trim()

    fun upgradeRemoteHttpRequest(request: Request): Request {
        val url = request.url
        if (url.scheme != "http") return request
        val upgraded = preferHttps(url.toString())
        if (upgraded == url.toString()) return request
        return request.newBuilder()
            .url(upgraded)
            .build()
    }
}
