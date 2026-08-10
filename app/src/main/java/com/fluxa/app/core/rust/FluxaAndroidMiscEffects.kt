@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.core.rust

import com.fluxa.app.data.local.*

internal fun FluxaAndroidHeadlessEnvironment.writeSettings(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    return ok(
        effect,
        mapOf(
            "key" to effect.payload.string("key"),
            "value" to effect.payload["value"]
        )
    )
}

internal fun FluxaAndroidHeadlessEnvironment.executeTrailerHttpEffect(effect: NativeHeadlessEffect): HeadlessEffectCompletion =
    executeTrailerHttpEffectCommon(
        effect = effect,
        httpEffectExecutor = httpEffectExecutor,
        httpClient = trailerHttpClient,
        gson = gson,
    )
