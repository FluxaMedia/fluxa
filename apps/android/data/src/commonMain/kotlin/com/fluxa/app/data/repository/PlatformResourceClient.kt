package com.fluxa.app.data.repository

import com.fluxa.app.data.platform.PlatformHttpClient
import com.fluxa.app.data.platform.PlatformHttpRequest

class PlatformResourceClient(
    private val httpClient: PlatformHttpClient
) {
    suspend fun <T> get(
        url: String,
        headers: Map<String, String> = emptyMap(),
        timeoutMillis: Long? = null,
        parse: (String) -> T?
    ): AddonResourceResult<T> {
        val response = try {
            httpClient.execute(
                PlatformHttpRequest(
                    url = url,
                    headers = headers,
                    timeoutMillis = timeoutMillis
                )
            )
        } catch (cause: Throwable) {
            return AddonResourceResult.NetworkError(url, cause)
        }
        if (!response.isSuccessful) {
            return AddonResourceResult.NetworkError(url, statusCode = response.statusCode)
        }
        if (response.body.isBlank()) return AddonResourceResult.Empty(url)
        val value = try {
            parse(response.body)
        } catch (cause: Throwable) {
            return AddonResourceResult.ParseError(url, cause)
        }
        return if (value == null) AddonResourceResult.Empty(url) else AddonResourceResult.Success(value, url)
    }
}
