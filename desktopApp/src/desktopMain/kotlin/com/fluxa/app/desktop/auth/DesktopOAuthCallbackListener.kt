package com.fluxa.app.desktop.auth

import java.net.InetAddress
import java.net.ServerSocket
import java.net.URI
import kotlin.concurrent.thread

private const val CALLBACK_PORT = 17864

class DesktopOAuthCallbackListener(
    private val onCallback: (service: String, code: String, state: String?) -> Unit
) {
    fun start() {
        thread(isDaemon = true, name = "fluxa-oauth-callback") {
            runCatching {
                ServerSocket(CALLBACK_PORT, 4, InetAddress.getLoopbackAddress()).use { server ->
                    while (true) {
                        val socket = server.accept()
                        socket.use {
                            val line = it.getInputStream().bufferedReader().readLine() ?: return@use
                            handleUrl(line.trim())
                        }
                    }
                }
            }
        }
    }

    private fun handleUrl(raw: String) {
        if (!raw.startsWith("fluxa://oauth/")) return
        val uri = runCatching { URI(raw) }.getOrNull() ?: return
        val service = uri.path?.removePrefix("/")?.takeIf { it.isNotBlank() } ?: return
        val params = uri.query.orEmpty().split('&').mapNotNull { part ->
            val idx = part.indexOf('=')
            if (idx < 0) null else part.substring(0, idx) to part.substring(idx + 1)
        }.toMap()
        val code = params["code"] ?: return
        onCallback(service, code, params["state"])
    }
}
