package com.fluxa.app.ui.rust

import java.util.concurrent.TimeUnit
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject

object PlatformHttp {
    private val client = OkHttpClient.Builder()
        .followRedirects(false)
        .followSslRedirects(false)
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(25, TimeUnit.SECONDS)
        .build()

    @JvmStatic
    fun fetch(request: String): String? = runCatching {
        val spec = JSONObject(request)
        val builder = Request.Builder().url(spec.getString("url"))
        var contentType = "application/x-www-form-urlencoded"
        spec.optJSONObject("headers")?.let { headers ->
            headers.keys().forEach { name ->
                val value = headers.getString(name)
                if (name.equals("Content-Type", true)) contentType = value
                if (!name.equals("Accept-Encoding", true)) builder.header(name, value)
            }
        }
        val method = spec.optString("method", "GET").uppercase()
        val body = if (spec.isNull("body")) null else spec.getString("body")
        builder.method(method, if (method == "GET" || method == "HEAD") null else (body ?: "").toRequestBody(contentType.toMediaType()))
        client.newCall(builder.build()).execute().use { response ->
            val headers = JSONObject()
            response.headers.names().forEach { headers.put(it, response.headers.values(it).last()) }
            JSONObject()
                .put("status", response.code)
                .put("headers", headers)
                .put("body", response.body?.string().orEmpty())
                .toString()
        }
    }.getOrNull()
}
