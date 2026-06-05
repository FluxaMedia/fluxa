package com.fluxa.app.data.repository

import okhttp3.HttpUrl.Companion.toHttpUrlOrNull
import okhttp3.Request

internal object HttpRequestSecurity {
    private val ipv4Regex = Regex("""^\d{1,3}(\.\d{1,3}){3}$""")

    fun isLocalHost(host: String): Boolean {
        val normalized = host.trim().lowercase()
        return normalized == "localhost" ||
            normalized == "127.0.0.1" ||
            normalized == "10.0.2.2" ||
            normalized.startsWith("192.168.") ||
            normalized.startsWith("10.") ||
            normalized.matches(ipv4Regex)
    }

    fun preferHttps(url: String): String {
        val trimmed = url.trim()
        return when {
            trimmed.startsWith("http://", ignoreCase = true) -> {
                val host = trimmed.toHttpUrlOrNull()?.host
                if (host != null && isLocalHost(host)) trimmed else trimmed.replace(Regex("^http://", RegexOption.IGNORE_CASE), "https://")
            }
            trimmed.startsWith("//") -> "https:$trimmed"
            else -> trimmed
        }
    }

    fun upgradeRemoteHttpRequest(request: Request): Request {
        val url = request.url
        if (url.scheme != "http" || isLocalHost(url.host)) return request
        return request.newBuilder()
            .url(url.newBuilder().scheme("https").build())
            .build()
    }
}
