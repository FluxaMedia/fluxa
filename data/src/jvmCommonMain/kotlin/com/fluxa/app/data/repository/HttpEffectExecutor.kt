package com.fluxa.app.data.repository

import android.util.Log
import com.fluxa.app.core.rust.models.NativeAddonFetchResult
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class HttpEffectExecutor @Inject constructor() {

    fun execute(
        client: OkHttpClient,
        url: String,
        method: String = "GET",
        headers: Map<String, String> = emptyMap(),
        body: RequestBody? = null
    ): NativeAddonFetchResult {
        return try {
            val requestBuilder = Request.Builder().url(url)
            headers.forEach { (key, value) -> requestBuilder.header(key, value) }
            requestBuilder.method(method, body)
            client.newCall(requestBuilder.build()).execute().use { response ->
                NativeAddonFetchResult(url = url, statusCode = response.code, body = response.body.string())
            }
        } catch (e: Exception) {
            Log.w("HttpEffectExecutor", "HTTP request failed: $url", e)
            NativeAddonFetchResult(url = url, error = e.message)
        }
    }
}
