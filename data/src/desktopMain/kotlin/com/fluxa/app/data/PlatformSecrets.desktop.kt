package com.fluxa.app.data

actual object PlatformSecrets {
    actual val traktClientId: String = System.getProperty("fluxa.secret.trakt_client_id").orEmpty()
    actual val simklClientId: String = System.getProperty("fluxa.secret.simkl_client_id").orEmpty()
    actual val anilistClientId: String = System.getProperty("fluxa.secret.anilist_client_id").orEmpty()
}
