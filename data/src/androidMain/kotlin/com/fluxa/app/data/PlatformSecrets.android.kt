package com.fluxa.app.data

actual object PlatformSecrets {
    actual val traktClientId: String = BuildConfig.TRAKT_CLIENT_ID
    actual val traktClientSecret: String = BuildConfig.TRAKT_CLIENT_SECRET
    actual val simklClientId: String = BuildConfig.SIMKL_CLIENT_ID
    actual val anilistClientId: String = BuildConfig.ANILIST_CLIENT_ID
    actual val nuvioSupabaseUrl: String = BuildConfig.NUVIO_SUPABASE_URL
    actual val nuvioSupabaseKey: String = BuildConfig.NUVIO_SUPABASE_KEY
}
