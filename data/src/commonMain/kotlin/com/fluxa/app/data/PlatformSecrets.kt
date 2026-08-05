package com.fluxa.app.data

expect object PlatformSecrets {
    val traktClientId: String
    val traktClientSecret: String
    val simklClientId: String
    val simklClientSecret: String
    val anilistClientId: String
    val anilistClientSecret: String
    val nuvioSupabaseUrl: String
    val nuvioSupabaseKey: String
}
