package com.fluxa.app.shared.feature.watchtogether

import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener

class JvmWatchTogetherTransport(
    private val client: OkHttpClient = sharedClient,
) : WatchTogetherTransport {
    private var socket: WebSocket? = null

    override fun connect(url: String, listener: WatchTogetherTransport.Listener) {
        close()
        socket = client.newWebSocket(
            Request.Builder().url(url).build(),
            object : WebSocketListener() {
                override fun onOpen(webSocket: WebSocket, response: Response) = listener.onOpen()
                override fun onMessage(webSocket: WebSocket, text: String) = listener.onMessage(text)
                override fun onClosing(webSocket: WebSocket, code: Int, reason: String) {
                    webSocket.close(code, reason)
                }
                override fun onClosed(webSocket: WebSocket, code: Int, reason: String) = listener.onClosed()
                override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                    listener.onFailure(t.message ?: "Watch Together connection failed")
                }
            }
        )
    }

    override fun send(text: String): Boolean = socket?.send(text) == true

    override fun close() {
        socket?.close(1000, "Fluxa Watch Together closed")
        socket = null
    }

    private companion object {
        val sharedClient = OkHttpClient.Builder()
            .retryOnConnectionFailure(true)
            .build()
    }
}
