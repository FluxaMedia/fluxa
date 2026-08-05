package com.fluxa.app.data

actual object PlatformSecrets {
    actual val traktClientId: String = BuildConfig.TRAKT_CLIENT_ID
    actual val simklClientId: String = BuildConfig.SIMKL_CLIENT_ID
    actual val anilistClientId: String = BuildConfig.ANILIST_CLIENT_ID
}
