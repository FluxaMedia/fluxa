package com.fluxa.app.shared.feature.watchtogether

interface WatchTogetherTransport {
    fun connect(url: String, listener: Listener)
    fun send(text: String): Boolean
    fun close()

    interface Listener {
        fun onOpen()
        fun onMessage(text: String)
        fun onClosed()
        fun onFailure(message: String)
    }
}

fun interface WatchTogetherTransportFactory {
    fun create(): WatchTogetherTransport
}
