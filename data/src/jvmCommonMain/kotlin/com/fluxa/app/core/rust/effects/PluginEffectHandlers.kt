package com.fluxa.app.core.rust.effects

import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.core.rust.HeadlessEffectCompletion
import com.fluxa.app.core.rust.NativeHeadlessEffect
import com.fluxa.app.core.rust.string
import com.fluxa.app.data.repository.HttpEffectExecutor
import okhttp3.OkHttpClient

fun fetchPluginManifest(
    effect: NativeHeadlessEffect,
    httpEffectExecutor: HttpEffectExecutor,
    httpClient: OkHttpClient
): HeadlessEffectCompletion {
    val manifestUrl = effect.payload.string("manifestUrl")
    val result = httpEffectExecutor.execute(httpClient, manifestUrl)
    val body = result.body
    val statusCode = result.statusCode
    if (result.error != null || statusCode == null || statusCode !in 200..299 || body == null) {
        return HeadlessEffectCompletion(
            effectId = effect.id,
            status = "error",
            error = mapOf("code" to (result.error ?: "http_${statusCode ?: 0}"))
        )
    }
    val manifest = FluxaCoreUniFfi.coreInvokeValue("pluginManifestParse", body)
    return HeadlessEffectCompletion(
        effectId = effect.id,
        status = "ok",
        value = mapOf("manifestUrl" to manifestUrl, "manifest" to manifest)
    )
}
