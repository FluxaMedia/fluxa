package com.fluxa.app.ui.rust

import android.annotation.SuppressLint
import android.content.Context
import android.os.Handler
import android.os.Looper
import android.webkit.CookieManager
import android.webkit.WebView
import android.webkit.WebViewClient

internal object ChallengeNative {
    init {
        System.loadLibrary("fluxa_android_renderer")
    }

    @JvmStatic external fun setAvailableNative(available: Boolean)
    @JvmStatic external fun nextRequestNative(timeoutMs: Long): String?
    @JvmStatic external fun completeNative(id: Long, cookies: String, userAgent: String)
}

object CloudflareSolver {
    private const val SOLVE_MS = 15_000L
    private const val POLL_MS = 400L

    private val main = Handler(Looper.getMainLooper())
    private var started = false

    @Synchronized
    fun start(context: Context) {
        if (started) return
        started = true
        val app = context.applicationContext
        ChallengeNative.setAvailableNative(true)
        Thread({
            while (true) {
                val request = ChallengeNative.nextRequestNative(1_000) ?: continue
                val id = request.substringBefore('\t').toLongOrNull() ?: continue
                solve(app, id, request.substringAfter('\t'))
            }
        }, "fluxa-cf-solver").apply { isDaemon = true }.start()
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun solve(context: Context, id: Long, url: String) {
        val lock = Object()
        var finished = false
        main.post {
            val webView = runCatching { WebView(context) }.getOrNull()
            if (webView == null) {
                ChallengeNative.completeNative(id, "", "")
                synchronized(lock) { finished = true; lock.notifyAll() }
                return@post
            }
            webView.settings.javaScriptEnabled = true
            webView.settings.domStorageEnabled = true
            webView.webViewClient = WebViewClient()
            val agent = webView.settings.userAgentString
            val deadline = System.currentTimeMillis() + SOLVE_MS
            val finish = { cookies: String ->
                ChallengeNative.completeNative(id, cookies, agent)
                webView.stopLoading()
                webView.destroy()
                synchronized(lock) { finished = true; lock.notifyAll() }
            }
            val poll = object : Runnable {
                override fun run() {
                    val cookies = CookieManager.getInstance().getCookie(url).orEmpty()
                    when {
                        cookies.contains("cf_clearance=") -> finish(cookies)
                        System.currentTimeMillis() >= deadline -> finish("")
                        else -> main.postDelayed(this, POLL_MS)
                    }
                }
            }
            webView.loadUrl(url)
            main.postDelayed(poll, POLL_MS)
        }
        synchronized(lock) {
            while (!finished) lock.wait(500)
        }
    }
}
