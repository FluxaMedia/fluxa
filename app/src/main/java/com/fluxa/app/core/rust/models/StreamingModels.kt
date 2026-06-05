package com.fluxa.app.core.rust.models

import com.fluxa.app.data.remote.Stream

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

data class NativeDetailStreamResultPlan(
    val streams: List<Stream> = emptyList(),
    val availableAddons: List<String> = emptyList(),
    val resolvedRequestId: String? = null,
    val hasStreamProviders: Boolean = false
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
