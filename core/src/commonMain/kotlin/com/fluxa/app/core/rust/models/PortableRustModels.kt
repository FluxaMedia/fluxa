package com.fluxa.app.core.rust.models

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

data class NativeSimklEpisodeMatch(
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

data class NativeDetailSeasonLoadPlan(
    val firstSeasonToLoad: Int = 1,
    val savedSeason: Int? = null
)

data class NativePlayerFlowEffect(
    val type: String = "",
    val contentType: String = "",
    val id: String = "",
    val useInitialStreams: Boolean = false
)

data class NativePlayerTrackState(
    val preferredAudioLanguage: String = "",
    val preferredSubtitleIndex: Int = -1,
    val preferredSubtitleId: String? = null,
    val subtitlesDisabled: Boolean = true
)

data class SubtitleTrackRef(val id: String?, val label: String, val language: String?)

data class NativeStreamDiscoveryPlan(
    val cacheKey: String = "",
    val addonRequests: List<NativeStreamAddonRequest> = emptyList(),
    val cloudstreamRequest: NativeCloudstreamRequest? = null
)

data class NativeStreamDiscoveryExecutionPolicy(
    val cacheKey: String = "",
    val cacheLookupPrefix: String = "",
    val maxConcurrentAddonRequests: Long = 8L,
    val cacheWriteMinimumResultCount: Long = 1L,
    val emitCachedResult: Boolean = true,
    val emitPartialNonEmptyResults: Boolean = true,
    val addonRequests: List<NativeStreamAddonRequest> = emptyList(),
    val cloudstreamRequest: NativeCloudstreamRequest? = null
)

data class NativeStreamDiscoveryEpisodeContext(
    val expectedEpisodeTitles: List<String> = emptyList(),
    val seasonEpisodeTitles: Map<String, List<String>> = emptyMap(),
    val seasonEpisodeIds: Map<String, String> = emptyMap()
)

data class NativeProviderAvailabilityPlan(
    val hasStremioStreamAddons: Boolean = false,
    val hasPluginStreamProviders: Boolean = false,
    val hasStreamProviders: Boolean = false,
    val pluginNames: List<String> = emptyList()
)

data class NativePrefetchDetailStreamsPlan(
    val count: Int = 0,
    val prewarmUrl: String? = null,
    val shouldPrewarmTorrent: Boolean = false
)

data class NativeDirectPlaybackPolicy(
    val metaDetailTimeoutMs: Long = 3500L,
    val streamDetailTimeoutMs: Long = 2500L
)

data class NativeStreamAddonRequest(
    val transportUrl: String = "",
    val addonName: String = "",
    val type: String = "",
    val id: String = "",
    val timeoutMs: Long = 0L
)

data class NativeCloudstreamRequest(
    val id: String = "",
    val title: String = "",
    val year: Long? = null,
    val type: String = "",
    val season: Int? = null,
    val episode: Int? = null,
    val originalName: String? = null,
    val timeoutMs: Long = 0L
)
