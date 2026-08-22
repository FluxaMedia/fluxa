package com.fluxa.app.shared

import com.fluxa.app.core.rust.FluxaCoreNative

actual fun shortenHeroSynopsis(text: String): String = FluxaCoreNative.shortenSynopsis(text)
