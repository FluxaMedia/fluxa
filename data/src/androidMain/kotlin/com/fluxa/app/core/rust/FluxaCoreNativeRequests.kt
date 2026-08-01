package com.fluxa.app.core.rust

import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video
import com.fluxa.app.player.TorrentFileStat

internal data class NativeStreamSelectionItem(
    val name: String?,
    val title: String?,
    val description: String?,
    val addonName: String?,
    val playableUrl: String?,
    val bingeGroup: String?,
    val filename: String?,
    val effectiveFilename: String?
)

internal data class NativeSubtitleSelectionTrack(
    val id: String?,
    val label: String,
    val language: String?
)

internal data class NativeTorrentRuntimeRequest(
    val link: String,
    val title: String,
    val requestedFileIdx: Int?,
    val preferredFilename: String?,
    val sources: List<String>,
    val fileStats: List<TorrentFileStat>,
    val rejectedIndex: Int?,
    val baseUrl: String,
    val play: Boolean,
    val stat: Boolean,
    val durationMs: Long? = null
)

internal data class NativeSubtitleTrackEntry(val id: String?, val label: String, val language: String?)

internal data class NativePlayerTrackStateRequest(
    val availableSubtitles: List<NativeSubtitleTrackEntry>,
    val lastAudioLanguage: String?,
    val preferredAudioLanguage: String?,
    val originalLanguage: String?,
    val contentGenres: List<String>,
    val profileAudioLanguage: String?,
    val animePreferJapaneseAudio: Boolean,
    val deviceLanguage: String?,
    val lastSubtitleLanguage: String?,
    val preferredSubtitleLanguage: String?,
    val secondarySubtitleLanguage: String?
)

internal data class NativeStreamDiscoveryCacheKeyRequest(
    val type: String,
    val id: String,
    val language: String,
    val cs3SearchQuery: String?,
    val cs3Year: Int?,
    val cs3OriginalName: String?,
    val addonSignatures: List<String>,
    val cs3PluginNames: List<String>
)

internal data class NativeDiscoverCatalogCacheKeyRequest(
    val type: String,
    val catalogKey: String?,
    val genre: String?,
    val year: String?,
    val rating: Float?,
    val provider: String?,
    val region: String?,
    val catalogSignatures: List<String>
)

internal data class NativeStreamDiscoveryPlanRequest(
    val type: String,
    val id: String,
    val language: String,
    val preferFastStart: Boolean,
    val addonRequestTimeoutMs: Long,
    val fastAddonRequestTimeoutMs: Long,
    val cloudstreamTimeoutMs: Long,
    val maxConcurrentAddonRequests: Long = 0L,
    val addons: List<AddonDescriptor>,
    val cs3PluginNames: List<String>,
    val cs3SearchQuery: String?,
    val cs3OriginalName: String?,
    val cs3Year: Int?
)

internal data class NativeProviderAvailabilityPlanRequest(
    val addons: List<AddonDescriptor>,
    val pluginNames: List<String>
)

internal data class NativeDetailStreamAttemptRequest(
    val requestId: String,
    val streams: List<Stream>
)

internal data class NativeDetailStreamResultPlanRequest(
    val attempts: List<NativeDetailStreamAttemptRequest>,
    val hasStreamProviders: Boolean
)

internal data class NativePrefetchDetailStreamsPlanRequest(
    val streams: List<Stream>
)

internal data class NativeOfflineDownloadPlanRequest(
    val meta: Meta,
    val video: Video?,
    val videoId: String?,
    val stream: Stream,
    val subtitleUrl: String?,
    val downloadId: String
)

internal data class NativeCacheEntryPolicyRequest(
    val key: String,
    val storedAtMillis: Long,
    val ttlMillis: Long,
    val nowMillis: Long
)

internal data class NativeCacheTrimPolicyEntry(
    val key: String,
    val expiresAtMillis: Long,
    val storedAtMillis: Long
)

internal data class NativeCacheTrimPolicyRequest(
    val entries: List<NativeCacheTrimPolicyEntry>,
    val maxEntries: Int,
    val nowMillis: Long
)

internal data class NativeAddonStoreSearchPolicyRequest(
    val query: String,
    val nowMillis: Long,
    val cachedAtMillis: Long?,
    val ttlMillis: Long
)

internal data class NativeRepositoryMetaDetailPlanRequest(
    val useConfiguredAddons: Boolean,
    val authKey: String,
    val localAddons: List<String>
)

internal data class NativeManifestFetchDecisionRequest(
    val forceRefresh: Boolean,
    val memoryHit: Boolean,
    val persistentHit: Boolean
)

internal data class NativeAddonResourceRequestPlanRequest(
    val transportUrl: String,
    val resource: String,
    val contentType: String,
    val id: String,
    val extraArgs: Map<String, String?> = emptyMap(),
    val extraRaw: String = ""
)

internal data class NativeDataFailurePolicyRequest(
    val operation: String,
    val kind: String,
    val message: String?,
    val throwableClass: String?,
    val reason: String?,
    val statusCode: Long?
)

