package com.fluxa.app.data.remote

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.models.NativeStreamPlaybackInfo

internal actual fun resolveStreamPlaybackInfo(stream: Stream): NativeStreamPlaybackInfo =
    FluxaCoreNative.streamPlaybackInfo(stream)
