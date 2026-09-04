package com.fluxa.app.data.stream

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.models.NativeStreamPlaybackInfo
import com.fluxa.app.data.remote.Stream

internal actual fun resolveStreamPlaybackInfo(stream: Stream): NativeStreamPlaybackInfo =
    FluxaCoreNative.streamPlaybackInfo(stream)
