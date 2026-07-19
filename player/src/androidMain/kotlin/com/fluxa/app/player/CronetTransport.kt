package com.fluxa.app.player

import android.content.Context
import android.util.Log
import com.google.net.cronet.okhttptransport.CronetInterceptor
import okhttp3.Interceptor
import org.chromium.net.CronetEngine

private var cronetEngine: CronetEngine? = null
private var cronetEngineInitAttempted = false

private fun sharedCronetEngine(context: Context): CronetEngine? {
    if (!cronetEngineInitAttempted) {
        cronetEngineInitAttempted = true
        cronetEngine = runCatching {
            CronetEngine.Builder(context.applicationContext)
                .enableHttp2(true)
                .enableQuic(true)
                .enableBrotli(true)
                .build()
        }.onFailure {
            Log.w("CronetTransport", "Cronet engine unavailable, staying on OkHttp", it)
        }.getOrNull()
    }
    return cronetEngine
}

fun cronetTransportInterceptor(context: Context): Interceptor? =
    sharedCronetEngine(context)?.let { engine ->
        runCatching { CronetInterceptor.newBuilder(engine).build() }.getOrNull()
    }
