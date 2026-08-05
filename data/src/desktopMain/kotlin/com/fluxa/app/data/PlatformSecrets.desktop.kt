package com.fluxa.app.data

actual object PlatformSecrets {
    actual val traktClientId: String = System.getProperty("fluxa.secret.trakt_client_id").orEmpty()
    actual val traktClientSecret: String = System.getProperty("fluxa.secret.trakt_client_secret").orEmpty()
    actual val simklClientId: String = System.getProperty("fluxa.secret.simkl_client_id").orEmpty()
    actual val simklClientSecret: String = System.getProperty("fluxa.secret.simkl_client_secret").orEmpty()
    actual val anilistClientId: String = System.getProperty("fluxa.secret.anilist_client_id").orEmpty()
    actual val anilistClientSecret: String = System.getProperty("fluxa.secret.anilist_client_secret").orEmpty()
    actual val nuvioSupabaseUrl: String = System.getProperty("fluxa.secret.nuvio_supabase_url").orEmpty()
    actual val nuvioSupabaseKey: String = System.getProperty("fluxa.secret.nuvio_supabase_key").orEmpty()
}
