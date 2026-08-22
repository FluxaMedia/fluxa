package com.fluxa.app.common

/**
 * Centralized constants for the entire app.
 * No more magic numbers scattered around the codebase.
 */
object Constants {
    
    // Network timeouts in milliseconds
    object Timeouts {
        const val HTTP_CONNECT: Long = 30_000L
        const val HTTP_READ: Long = 30_000L
        const val HTTP_WRITE: Long = 30_000L
        const val CACHE_PING: Int = 1_200
        const val TMDB_REQUEST: Long = 10_000L
        const val TRAKT_REQUEST: Long = 2_000L
        const val ADDON_REQUEST: Long = 2_500L
        const val PLUGIN_SEARCH: Long = 15_000L
        const val PLUGIN_LOAD_LINKS: Long = 30_000L
    }
    
    // Cache durations (milliseconds)
    object Cache {
        const val DEFAULT_DURATION_MS: Long = 15 * 60 * 1000L // 15 minutes
        const val META_DURATION_MS: Long = 60 * 60 * 1000L    // 1 hour
        const val SEASON_DURATION_MS: Long = 30 * 60 * 1000L  // 30 minutes
        const val STREAM_DURATION_MS: Long = 5 * 60 * 1000L    // 5 minutes
    }
    
    // Image sizes
    object Images {
        const val TMDB_BASE_URL = "https://image.tmdb.org/t/p/"
        const val POSTER_SIZE = "w500"
        const val BACKDROP_SIZE = "original"
        const val PROFILE_SIZE = "w185"
        const val STILL_SIZE = "original"
    }
    
    // Pagination
    object Pagination {
        const val CATALOG_PAGE_SIZE = 100
        const val SEARCH_MAX_RESULTS = 50
        const val SIMILAR_MAX_RESULTS = 12
        const val CAST_MAX_RESULTS = 12
    }
    
    // TMDB
    object Tmdb {
        const val API_VERSION = "3"
        const val DEFAULT_LANGUAGE = "en"
        const val FALLBACK_LANGUAGE = "en"
        const val INCLUDE_IMAGE_LANG = "en,null"
        const val APPEND_TO_RESPONSE = "credits,external_ids,videos,images,content_ratings,release_dates"
    }
    
    // Trakt
    object Trakt {
        const val API_URL = "https://api.trakt.tv"
        const val STAGING_URL = "https://api-staging.trakt.tv"
    }
    
    // Local server
    object LocalServer {
        const val HOST = "127.0.0.1"
        const val PORT = 11471
        const val PROXY_PORT = 11470
        const val BASE_URL = "http://$HOST:$PORT"
        const val TORRENT_SERVER_PORT = 8090
        const val TORRENT_SERVER_BASE_URL = "http://$HOST:$TORRENT_SERVER_PORT"
    }
    
    // Plugin/Extension
    object Extensions {
        const val CS3_EXTENSION_DIR = "cs_extensions"
        const val MAX_EXTENSION_SIZE_MB = 10
        const val MAX_CONCURRENT_SCRAPERS = 5
        const val SEMAPHORE_PERMITS = 5
    }
    
    // UI
    object UI {
        const val TV_HORIZONTAL_PADDING_DP = 58
        const val MOBILE_HORIZONTAL_PADDING_DP = 16
        const val SCROLL_ANIMATION_DURATION_MS = 300
        const val DEBOUNCE_DELAY_MS = 300L
        const val TOAST_DURATION_MS = 3_000L
    }

    object Player {
        const val SCROBBLE_START_PROGRESS_PERCENT = 0.2f
        const val SCROBBLE_STOP_PROGRESS_PERCENT = 80f
        const val DURABLE_SCROBBLE_MIN_PROGRESS_PERCENT = 1f
        const val PERIODIC_PROGRESS_SAVE_MS = 30_000L
        const val DISPOSAL_PROGRESS_SAVE_MIN_MS = 5_000L
        const val PAUSE_SCROBBLE_DELAY_MS = 2_000L
    }
    
    // Regex patterns
    object Patterns {
        val IMDB_ID = Regex("""tt\d+""")
        val SEASON_EPISODE = Regex("""S?\d{1,2}\s*[-_. ]?[EXB]?\d{1,3}""", RegexOption.IGNORE_CASE)
        val CAM_QUALITY = Regex("""\b(cam|telesync|hdts|hd-ts|scr|dvdscr)\b""", RegexOption.IGNORE_CASE)
        val PACK_INDICATOR = Regex("""s\d{2}[-]s\d{2}""", RegexOption.IGNORE_CASE)
        val VIDEO_EXTENSION = Regex("""\.(mp4|mkv|avi|mov|webm)$""", RegexOption.IGNORE_CASE)
    }
    
    // Video hosts
    object VideoHosts {
        val SUPPORTED_EMBEDS = listOf(
            "oload", "openload", "streamtape", "vidmoly", "mixdrop",
            "voe", "dood", "streamsb", "fembed", "fastload",
            "upstream", "videobin", "vidlox", "vidoza", "vshare"
        )
    }
    
    // Languages
    object Languages {
        const val DEFAULT = "en"
        const val ENGLISH = "en"
        val SUPPORTED = listOf("tr", "en", "de", "fr", "es", "it", "ru", "ja", "ko")
    }
}
