package com.fluxa.app.data.repository

data class OAuthClientConfig(
    val traktClientId: String,
    val traktClientSecret: String?,
    val simklClientId: String,
    val anilistClientId: String
)
