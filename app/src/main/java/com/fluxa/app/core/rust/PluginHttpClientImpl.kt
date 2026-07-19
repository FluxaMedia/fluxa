package com.fluxa.app.core.rust

import android.util.Log
import com.fluxa.app.data.repository.PluginNetGuard
import com.fluxa.core.uniffi.PluginHttpClient
import com.fluxa.core.uniffi.PluginHttpRequest
import com.fluxa.core.uniffi.PluginHttpResponse
import okhttp3.Dns
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.net.InetAddress
import java.net.URI
import java.net.UnknownHostException
import javax.inject.Inject
import javax.inject.Named

private const val MAX_PLUGIN_REDIRECTS = 10

private object PluginGuardedDns : Dns {
    override fun lookup(hostname: String): List<InetAddress> {
        return PluginNetGuard.resolveAllowedAddresses(hostname)
            ?: throw UnknownHostException("blocked address for host $hostname")
    }
}

class PluginHttpClientImpl @Inject constructor(
    @param:Named("PluginScraperClient") baseClient: OkHttpClient
) : PluginHttpClient {

    private val guardedClient = baseClient.newBuilder()
        .dns(PluginGuardedDns)
        .followRedirects(false)
        .followSslRedirects(false)
        .build()

    override fun fetch(request: PluginHttpRequest): PluginHttpResponse {
        var currentUrl = request.url
        var redirectsLeft = if (request.followRedirects) MAX_PLUGIN_REDIRECTS else 0
        try {
            while (true) {
                val uri = try {
                    URI(currentUrl)
                } catch (_: Exception) {
                    return blockedResponse("invalid url")
                }
                if (!PluginNetGuard.isSchemeAllowed(uri.scheme)) {
                    return blockedResponse("unsupported scheme: ${uri.scheme}")
                }
                val httpRequest = Request.Builder().url(currentUrl)
                request.headers.forEach { (key, value) -> httpRequest.header(key, value) }
                val method = request.method.ifBlank { "GET" }.uppercase()
                val body = request.body?.let { raw ->
                    val contentType = request.headers.entries
                        .firstOrNull { it.key.equals("Content-Type", ignoreCase = true) }
                        ?.value?.toMediaTypeOrNull()
                    raw.toRequestBody(contentType)
                }
                httpRequest.method(method, if (method == "GET" || method == "HEAD") null else body ?: "".toRequestBody(null))

                val response = guardedClient.newCall(httpRequest.build()).execute()
                if (response.isRedirect && redirectsLeft > 0) {
                    val location = response.header("Location")
                    response.close()
                    if (location.isNullOrBlank()) return blockedResponse("redirect without location")
                    currentUrl = uri.resolve(location).toString()
                    redirectsLeft--
                    continue
                }
                response.use { httpResponse ->
                    val text = httpResponse.body.string()
                    val headersMap = httpResponse.headers.toMultimap()
                        .mapValues { it.value.joinToString(",") }
                    return PluginHttpResponse(
                        status = httpResponse.code.toUShort(),
                        headers = headersMap,
                        body = text,
                        ok = httpResponse.isSuccessful,
                        error = null
                    )
                }
            }
        } catch (e: Exception) {
            Log.w("PluginHttpClient", "plugin fetch failed: $currentUrl", e)
            return PluginHttpResponse(
                status = 0u,
                headers = emptyMap(),
                body = "",
                ok = false,
                error = e.message ?: "request failed"
            )
        }
    }

    private fun blockedResponse(reason: String): PluginHttpResponse =
        PluginHttpResponse(status = 0u, headers = emptyMap(), body = "", ok = false, error = reason)
}
