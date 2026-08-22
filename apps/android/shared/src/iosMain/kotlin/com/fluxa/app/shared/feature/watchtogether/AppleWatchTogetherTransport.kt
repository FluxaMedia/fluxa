package com.fluxa.app.shared.feature.watchtogether

import platform.Foundation.NSError
import platform.Foundation.NSURL
import platform.Foundation.NSURLSession
import platform.Foundation.NSURLSessionWebSocketMessage
import platform.Foundation.NSURLSessionWebSocketTask

class AppleWatchTogetherTransport : WatchTogetherTransport {
    private var task: NSURLSessionWebSocketTask? = null
    private var listener: WatchTogetherTransport.Listener? = null
    private var closed = false

    override fun connect(url: String, listener: WatchTogetherTransport.Listener) {
        close()
        closed = false
        this.listener = listener
        val nsUrl = NSURL.URLWithString(url)
        if (nsUrl == null) {
            listener.onFailure("Invalid Watch Together URL")
            return
        }
        val next = NSURLSession.sharedSession.webSocketTaskWithURL(nsUrl)
        task = next
        next.resume()
        listener.onOpen()
        receiveLoop(next)
    }

    override fun send(text: String): Boolean {
        val active = task ?: return false
        active.sendMessage(NSURLSessionWebSocketMessage(text)) { error: NSError? ->
            if (error != null && !closed) listener?.onFailure(error.localizedDescription)
        }
        return true
    }

    override fun close() {
        closed = true
        task?.cancel()
        task = null
        listener = null
    }

    private fun receiveLoop(active: NSURLSessionWebSocketTask) {
        active.receiveMessageWithCompletionHandler { message, error ->
            if (closed || task !== active) return@receiveMessageWithCompletionHandler
            if (error != null) {
                listener?.onFailure(error.localizedDescription)
                return@receiveMessageWithCompletionHandler
            }
            message?.string?.let { listener?.onMessage(it) }
            receiveLoop(active)
        }
    }
}
