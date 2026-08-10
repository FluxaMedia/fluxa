package com.fluxa.app.core.rust

import com.fluxa.app.data.repository.HttpEffectExecutor
import com.google.gson.Gson
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.RequestBody.Companion.toRequestBody

fun executeTrailerHttpEffectCommon(
    effect: NativeHeadlessEffect,
    httpEffectExecutor: HttpEffectExecutor,
    httpClient: OkHttpClient,
    gson: Gson,
): HeadlessEffectCompletion {
    val payload = effect.payload
    val headers = payload.objectValue("headers")?.mapValues { it.value.toString() }.orEmpty()
    val method = payload.string("method", "GET")
    val body = payload["body"]?.let {
        gson.toJson(it).toRequestBody("application/json".toMediaType())
    }
    val result = httpEffectExecutor.execute(httpClient, payload.string("url"), method, headers, body)
    val statusCode = result.statusCode
    return if (result.error != null || statusCode == null || statusCode !in 200..299) {
        HeadlessEffectCompletion(
            effectId = effect.id,
            status = "error",
            error = mapOf("code" to "http_${statusCode ?: 0}"),
        )
    } else {
        HeadlessEffectCompletion(
            effectId = effect.id,
            status = "ok",
            value = mapOf("body" to result.body),
        )
    }
}
