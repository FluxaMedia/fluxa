package com.fluxa.app.core.rust.models

import com.fluxa.app.data.remote.Meta

data class NativeEpisodeLocator(
    val baseId: String = "",
    val season: Int = 0,
    val episode: Int = 0
)

data class NativeAddonFetchResult(
    val url: String = "",
    val statusCode: Int? = null,
    val body: String? = null,
    val error: String? = null
)

data class NativeAddonResourceParseResult(
    val kind: String = "",
    val url: String = "",
    val statusCode: Int? = null,
    val valueJson: String? = null,
    val error: String? = null
)

data class NativeManifestFetchPlan(
    val normalizedTransportUrl: String = "",
    val cacheKey: String = "",
    val candidateUrls: List<String> = emptyList()
)

data class NativeTraktEpisodeLocator(
    val season: Int = 0,
    val episode: Int = 0
)

data class NativeStreamPlaybackInfo(
    val playableUrl: String? = null,
    val effectiveVideoHash: String? = null,
    val effectiveVideoSize: Long? = null,
    val effectiveFilename: String? = null,
    val subtitleExtraArgs: String = "",
    val isTorrentPlaybackUrl: Boolean = false,
    val isLikelyPlayerCompatible: Boolean = false
)

data class NativeRepositoryMetaDetailPlan(
    val preferAddonMetaDetail: Boolean = false,
    val fallbackToStremioMetaDetail: Boolean = true
)

data class NativeManifestFetchDecision(
    val phase: String = "fetch",
    val allowStaleFallback: Boolean = true
)

data class NativeAddonResourceRequestPlan(
    val urls: List<String> = emptyList()
)

data class NativeDirectPlaybackPlan(
    val meta: Meta? = null,
    val targetVideoId: String? = null,
    val lookupId: String = ""
)

data class NativeDetailSeasonLoadPlan(
    val firstSeasonToLoad: Int = 1,
    val savedSeason: Int? = null
)
