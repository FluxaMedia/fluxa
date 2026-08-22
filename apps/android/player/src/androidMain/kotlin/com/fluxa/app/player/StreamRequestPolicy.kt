package com.fluxa.app.player

import com.fluxa.app.core.rust.FluxaCoreNative

object StreamRequestPolicy {
    const val DEFAULT_USER_AGENT: String =
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"

    fun headersFor(url: String, streamHeaders: Map<String, String> = emptyMap()): Map<String, String> {
        return FluxaCoreNative.streamRequestHeaders(streamHeaders)
    }

    fun refererFor(url: String): String? = FluxaCoreNative.streamRequestReferer(url)
}
