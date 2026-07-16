package com.fluxa.app.core.rust

import com.fluxa.app.core.rust.models.NativeActiveProfilePlan
import com.fluxa.app.core.rust.models.NativeAddonFetchResult
import com.fluxa.app.core.rust.models.NativeAddonResourceParseResult
import com.fluxa.app.core.rust.models.NativeAddonResourceRequestPlan
import com.fluxa.app.core.rust.models.NativeAddonStoreSearchPolicy
import com.fluxa.app.core.rust.models.NativeCalendarNotificationContent
import com.fluxa.app.core.rust.models.NativeCacheEntryPolicy
import com.fluxa.app.core.rust.models.NativeCacheTrimPolicy
import com.fluxa.app.core.rust.models.NativeCloudstreamRequest
import com.fluxa.app.core.rust.models.NativeDataFailurePolicy
import com.fluxa.app.core.rust.models.NativeDetailSeasonLoadPlan
import com.fluxa.app.core.rust.models.NativeDetailStreamResultPlan
import com.fluxa.app.core.rust.models.NativeDirectPlaybackPlan
import com.fluxa.app.core.rust.models.NativeDirectPlaybackPolicy
import com.fluxa.app.core.rust.models.NativeDolbyVisionRpuConvertResult
import com.fluxa.app.core.rust.models.NativeDolbyVisionRpuInfo
import com.fluxa.app.core.rust.models.NativeDvProxyPlan
import com.fluxa.app.core.rust.models.NativeEpisodeLocator
import com.fluxa.app.core.rust.models.NativeLibraryCollectionImportValidation
import com.fluxa.app.core.rust.models.NativeLibraryOfflineGrouping
import com.fluxa.app.core.rust.models.NativeManifestFetchDecision
import com.fluxa.app.core.rust.models.NativeManifestFetchPlan
import com.fluxa.app.core.rust.models.NativeOfflineDownloadPlan
import com.fluxa.app.core.rust.models.NativePlayerBackendSelection
import com.fluxa.app.core.rust.models.NativePlayerBufferTargets
import com.fluxa.app.core.rust.models.NativePlayerFlowEffect
import com.fluxa.app.core.rust.models.NativePlayerFlowResult
import com.fluxa.app.core.rust.models.NativePlayerFlowState
import com.fluxa.app.core.rust.models.NativePlayerRetryPolicy
import com.fluxa.app.core.rust.models.NativePlayerTrackState
import com.fluxa.app.core.rust.models.SubtitleTrackRef
import com.fluxa.app.core.rust.models.NativePrefetchDetailStreamsPlan
import com.fluxa.app.core.rust.models.NativeProfileAvatarDefault
import com.fluxa.app.core.rust.models.NativeProfileSafePrefs
import com.fluxa.app.core.rust.models.NativeProfileSettingsMigration
import com.fluxa.app.core.rust.models.NativeProviderAvailabilityPlan
import com.fluxa.app.core.rust.models.NativeRepositoryMetaDetailPlan
import com.fluxa.app.core.rust.models.NativeSearchResultGrouping
import com.fluxa.app.core.rust.models.NativeSimklEpisodeMatch
import com.fluxa.app.core.rust.models.NativeStreamAddonRequest
import com.fluxa.app.core.rust.models.NativeStreamDiscoveryEpisodeContext
import com.fluxa.app.core.rust.models.NativeStreamDiscoveryExecutionPolicy
import com.fluxa.app.core.rust.models.NativeStreamDiscoveryPlan
import com.fluxa.app.core.rust.models.NativeStreamPlaybackInfo
import com.fluxa.app.core.rust.models.NativeTorrentFallbackFilePolicy
import com.fluxa.app.core.rust.models.NativeTraktEpisodeLocator
import com.fluxa.app.core.rust.models.NativeWatchlistTogglePlan
import com.fluxa.app.data.remote.AddonCatalog
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.AddonManifest
import com.fluxa.app.data.remote.LibraryItem
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.TraktHistorySyncRequest
import com.fluxa.app.data.remote.TraktIds
import com.fluxa.app.data.repository.TraktWatchedState
import com.fluxa.app.data.remote.Video
import com.fluxa.app.player.NativeTorrentRuntimeInfo
import com.fluxa.app.player.NativeTorrentStatusInfo
import com.fluxa.app.player.TorrentFileStat
import com.fluxa.app.player.TorrentStatus
import com.google.gson.Gson
import com.google.gson.JsonObject
import com.google.gson.reflect.TypeToken
import com.google.gson.stream.JsonReader
import java.io.Closeable
import java.io.StringReader

private fun JsonObject.stringOrNull(key: String): String? =
    get(key)?.takeUnless { it.isJsonNull }?.asString

private fun JsonObject.intOrNull(key: String): Int? =
    get(key)?.takeUnless { it.isJsonNull }?.asInt

private fun JsonObject.booleanOrDefault(key: String): Boolean =
    get(key)?.takeUnless { it.isJsonNull }?.asBoolean ?: false

private fun JsonObject.toDolbyVisionRpuInfo(): NativeDolbyVisionRpuInfo = NativeDolbyVisionRpuInfo(
    ok = booleanOrDefault("ok"),
    profile = intOrNull("profile"),
    elType = stringOrNull("el_type"),
    error = stringOrNull("error")
)

private fun JsonObject.toDolbyVisionRpuConvertResult(): NativeDolbyVisionRpuConvertResult =
    NativeDolbyVisionRpuConvertResult(
        ok = booleanOrDefault("ok"),
        profileBefore = intOrNull("profile_before"),
        profileAfter = intOrNull("profile_after"),
        elTypeBefore = stringOrNull("el_type_before"),
        elTypeAfter = stringOrNull("el_type_after"),
        rpuHex = stringOrNull("rpu_hex"),
        rpuBase64 = stringOrNull("rpu_base64"),
        error = stringOrNull("error")
    )

data class NativeCacheEntryPolicy(
    val key: String = "",
    val storedAtMillis: Long = 0L,
    val expiresAtMillis: Long = 0L,
    val isExpired: Boolean = false
)

data class NativeCacheTrimPolicy(
    val expiredKeys: List<String> = emptyList(),
    val evictedKeys: List<String> = emptyList()
)

data class NativeAddonStoreSearchPolicy(
    val normalizedQuery: String = "",
    val url: String = "",
    val useCache: Boolean = false,
    val shouldFetch: Boolean = false
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

data class NativeDataFailurePolicy(
    val operation: String = "",
    val kind: String = "",
    val message: String = "",
    val retryable: Boolean = false,
    val staleFallbackAllowed: Boolean = false
)

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

data class NativeDirectPlaybackPlan(
    val meta: Meta? = null,
    val targetVideoId: String? = null,
    val lookupId: String = ""
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

// ── calendar_plan ─────────────────────────────────────────────────────────────

data class NativeCalendarNotificationContent(
    val items: List<Map<String, Any?>> = emptyList(),
    val keys: List<String> = emptyList()
)

data class NativeDetailSeasonLoadPlan(
    val firstSeasonToLoad: Int = 1,
    val savedSeason: Int? = null
)

// ── profile_contract ──────────────────────────────────────────────────────────

data class NativeActiveProfilePlan(
    val activeId: String = "guest",
    val shouldCreateDefault: Boolean = false,
    val activeProfile: Map<String, Any?> = emptyMap()
)

data class NativeProfileSettingsMigration(
    val migratedProfile: Map<String, Any?> = emptyMap(),
    val appliedMigrations: List<String> = emptyList(),
    val schemaVersion: Int = 2
)

data class NativeProfileAvatarDefault(
    val avatarUrl: String? = null,
    val fromCatalog: Boolean = false
)

// ── watchlist_plan ────────────────────────────────────────────────────────────

data class NativeWatchlistTogglePlan(
    val command: String = "add",
    val itemId: String = "",
    val optimisticIsInWatchlist: Boolean = false,
    val profileId: String? = null
)

data class NativeLibraryCollectionImportValidation(
    val isValid: Boolean = false,
    val validCollections: List<Map<String, Any?>> = emptyList(),
    val issues: List<String> = emptyList()
)

data class NativeLibraryOfflineGrouping(
    val ready: List<Map<String, Any?>> = emptyList(),
    val downloading: List<Map<String, Any?>> = emptyList(),
    val queued: List<Map<String, Any?>> = emptyList(),
    val failed: List<Map<String, Any?>> = emptyList()
)

// ── player_policy ─────────────────────────────────────────────────────────────

data class NativePlayerBackendSelection(
    val backend: String = "exoplayer",
    val reason: String = "default"
)

data class NativeTorrentFallbackFilePolicy(
    val fallbackFileIndexes: List<Int> = emptyList(),
    val rejectedIndex: Int? = null
)

data class NativePlayerBufferTargets(
    val forwardBufferMs: Long = 120_000L,
    val backBufferMs: Long = 30_000L,
    val cacheSizeBytes: Long = 100_000_000L
)

data class NativePlayerRetryPolicy(
    val shouldRetry: Boolean = false,
    val fallbackAction: String = "show_error",
    val delayMs: Long = 0L,
    val retryCount: Int = 0
)

// ── search_plan ───────────────────────────────────────────────────────────────

data class NativeSearchResultGrouping(
    val groups: List<Map<String, Any?>> = emptyList(),
    val totalCount: Int = 0,
    val query: String = ""
)

data class NativeOfflineDownloadPlan(
    val supported: Boolean = false,
    val reason: String? = null,
    val playbackUrl: String = "",
    val baseName: String = "",
    val videoFileName: String = "",
    val subtitleFileName: String? = null,
    val posterFileName: String = "",
    val backgroundFileName: String = "",
    val logoFileName: String = "",
    val videoId: String? = null,
    val streamTitle: String? = null
)

class FluxaHeadlessEngineHandle internal constructor(
    private val handle: Long,
    private val gson: Gson
) : Closeable, FluxaHeadlessEngine {
    init {
        check(handle != 0L) { "Fluxa headless engine could not be created." }
    }

    fun snapshotJson(): String = FluxaCoreNative.headlessEngineSnapshotJson(handle)

    fun dispatchJson(actionJson: String): String = FluxaCoreNative.headlessEngineDispatchJson(handle, actionJson)

    override fun dispatch(action: Any): NativeHeadlessEngineResult {
        return FluxaCoreNative.parseHeadlessResult(dispatchJson(gson.toJson(action)))
    }

    fun completeEffectJson(resultJson: String): String = FluxaCoreNative.headlessEngineCompleteEffectJson(handle, resultJson)

    override fun completeEffect(result: Any): NativeHeadlessEngineResult {
        return FluxaCoreNative.parseHeadlessResult(completeEffectJson(gson.toJson(result)))
    }

    override fun close() {
        FluxaCoreNative.destroyHeadlessEngine(handle)
    }
}

class FluxaCoreStateHandle internal constructor(
    private val handle: Long,
    private val gson: Gson
) : Closeable {
    init {
        check(handle != 0L) { "Fluxa core state could not be created." }
    }

    fun snapshotJson(): String = FluxaCoreNative.appCoreStateJson(handle)

    fun dispatchJson(actionJson: String): String = FluxaCoreNative.appCoreDispatchJson(handle, actionJson)

    fun dispatch(action: Any): String = dispatchJson(gson.toJson(action))

    override fun close() {
        FluxaCoreNative.destroyAppCoreState(handle)
    }
}

private data class NativeStreamSelectionItem(
    val name: String?,
    val title: String?,
    val description: String?,
    val addonName: String?,
    val playableUrl: String?,
    val bingeGroup: String?,
    val filename: String?,
    val effectiveFilename: String?
)

private data class NativeSubtitleSelectionTrack(
    val id: String?,
    val label: String,
    val language: String?
)

private data class NativeTorrentRuntimeRequest(
    val link: String,
    val title: String,
    val requestedFileIdx: Int?,
    val preferredFilename: String?,
    val sources: List<String>,
    val fileStats: List<TorrentFileStat>,
    val rejectedIndex: Int?,
    val baseUrl: String,
    val play: Boolean,
    val stat: Boolean
)

private data class NativeSubtitleTrackEntry(val id: String?, val label: String, val language: String?)

private data class NativePlayerTrackStateRequest(
    val availableSubtitles: List<NativeSubtitleTrackEntry>,
    val lastAudioLanguage: String?,
    val preferredAudioLanguage: String?,
    val originalLanguage: String?,
    val lastSubtitleLanguage: String?,
    val preferredSubtitleLanguage: String?,
    val secondarySubtitleLanguage: String?
)

private data class NativeStreamDiscoveryCacheKeyRequest(
    val type: String,
    val id: String,
    val language: String,
    val cs3SearchQuery: String?,
    val cs3Year: Int?,
    val cs3OriginalName: String?,
    val addonSignatures: List<String>,
    val cs3PluginNames: List<String>
)

private data class NativeDiscoverCatalogCacheKeyRequest(
    val type: String,
    val catalogKey: String?,
    val genre: String?,
    val year: String?,
    val rating: Float?,
    val provider: String?,
    val region: String?,
    val catalogSignatures: List<String>
)

private data class NativeStreamDiscoveryPlanRequest(
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

private data class NativeProviderAvailabilityPlanRequest(
    val addons: List<AddonDescriptor>,
    val pluginNames: List<String>
)

private data class NativeDetailStreamAttemptRequest(
    val requestId: String,
    val streams: List<Stream>
)

private data class NativeDetailStreamResultPlanRequest(
    val attempts: List<NativeDetailStreamAttemptRequest>,
    val hasStreamProviders: Boolean
)

private data class NativePrefetchDetailStreamsPlanRequest(
    val streams: List<Stream>
)

private data class NativeOfflineDownloadPlanRequest(
    val meta: Meta,
    val video: Video?,
    val videoId: String?,
    val stream: Stream,
    val subtitleUrl: String?,
    val downloadId: String
)

private data class NativeCacheEntryPolicyRequest(
    val key: String,
    val storedAtMillis: Long,
    val ttlMillis: Long,
    val nowMillis: Long
)

private data class NativeCacheTrimPolicyEntry(
    val key: String,
    val expiresAtMillis: Long,
    val storedAtMillis: Long
)

private data class NativeCacheTrimPolicyRequest(
    val entries: List<NativeCacheTrimPolicyEntry>,
    val maxEntries: Int,
    val nowMillis: Long
)

private data class NativeAddonStoreSearchPolicyRequest(
    val query: String,
    val nowMillis: Long,
    val cachedAtMillis: Long?,
    val ttlMillis: Long
)

private data class NativeRepositoryMetaDetailPlanRequest(
    val useConfiguredAddons: Boolean,
    val authKey: String,
    val localAddons: List<String>
)

private data class NativeManifestFetchDecisionRequest(
    val forceRefresh: Boolean,
    val memoryHit: Boolean,
    val persistentHit: Boolean
)

private data class NativeAddonResourceRequestPlanRequest(
    val transportUrl: String,
    val resource: String,
    val contentType: String,
    val id: String,
    val extraArgs: Map<String, String?> = emptyMap(),
    val extraRaw: String = ""
)

private data class NativeDataFailurePolicyRequest(
    val operation: String,
    val kind: String,
    val message: String?,
    val throwableClass: String?,
    val reason: String?,
    val statusCode: Long?
)

object FluxaCoreNative {
    private val gson = Gson()
    private val stringListType = object : TypeToken<List<String>>() {}.type
    private val stringListListType = object : TypeToken<List<List<String>>>() {}.type
    private val metaListType = object : TypeToken<List<Meta>>() {}.type
    private val libraryItemListType = object : TypeToken<List<LibraryItem>>() {}.type
    private val headlessEffectListType = object : TypeToken<List<NativeHeadlessEffect>>() {}.type
    private val headlessStateMapType = object : TypeToken<Map<String, Any?>>() {}.type
    private val loaded: Boolean = loadCore()

    fun createAppCoreState(initialState: Any = emptyMap<String, Any?>()): FluxaCoreStateHandle = call {
        FluxaCoreStateHandle(createAppCoreStateNative(gson.toJson(initialState)), gson)
    }

    fun createHeadlessEngine(initialState: Any = emptyMap<String, Any?>()): FluxaHeadlessEngineHandle = call {
        FluxaHeadlessEngineHandle(createHeadlessEngineNative(gson.toJson(initialState)), gson)
    }

    internal fun headlessEngineSnapshotJson(handle: Long): String = call {
        headlessEngineSnapshotJsonNative(handle).orEmpty()
    }

    internal fun headlessEngineDispatchJson(handle: Long, actionJson: String): String = call {
        headlessEngineDispatchJsonNative(handle, actionJson).orEmpty()
    }

    internal fun headlessEngineCompleteEffectJson(handle: Long, resultJson: String): String = call {
        headlessEngineCompleteEffectJsonNative(handle, resultJson).orEmpty()
    }

    internal fun destroyHeadlessEngine(handle: Long): Boolean = call {
        destroyHeadlessEngineNative(handle)
    }

    fun parseHeadlessResult(json: String): NativeHeadlessEngineResult {
        if (json.isBlank()) return NativeHeadlessEngineResult()
        // Parse the (small) effects array eagerly; defer the (potentially large) state map so
        // intermediate drain steps that only read `effects` never pay the full-state parse cost.
        val effects = parseHeadlessEffects(json)
        return NativeHeadlessEngineResult(effects = effects, stateProvider = { parseHeadlessState(json) })
    }

    private fun parseHeadlessEffects(json: String): List<NativeHeadlessEffect> {
        return runCatching {
            JsonReader(StringReader(json)).use { reader ->
                var effects: List<NativeHeadlessEffect> = emptyList()
                reader.beginObject()
                while (reader.hasNext()) {
                    if (reader.nextName() == "effects") {
                        effects = gson.fromJson(reader, headlessEffectListType) ?: emptyList()
                    } else {
                        reader.skipValue()
                    }
                }
                reader.endObject()
                effects
            }
        }.getOrDefault(emptyList())
    }

    private fun parseHeadlessState(json: String): Map<String, Any?> {
        return runCatching {
            JsonReader(StringReader(json)).use { reader ->
                var state: Map<String, Any?> = emptyMap()
                reader.beginObject()
                while (reader.hasNext()) {
                    if (reader.nextName() == "state") {
                        state = gson.fromJson(reader, headlessStateMapType) ?: emptyMap()
                    } else {
                        reader.skipValue()
                    }
                }
                reader.endObject()
                state
            }
        }.getOrDefault(emptyMap())
    }

    fun coreCapabilities(portable: Boolean = false): NativeCoreCapabilitySet = call {
        val json = coreCapabilitiesJsonNative(portable)
        json.takeIf { it.isNotBlank() }?.let { gson.fromJson(it, NativeCoreCapabilitySet::class.java) }
            ?: NativeCoreCapabilitySet()
    }

    internal fun appCoreStateJson(handle: Long): String = call {
        appCoreStateJsonNative(handle).orEmpty()
    }

    internal fun appCoreDispatchJson(handle: Long, actionJson: String): String = call {
        appCoreDispatchJsonNative(handle, actionJson).orEmpty()
    }

    internal fun destroyAppCoreState(handle: Long): Boolean = call {
        destroyAppCoreStateNative(handle)
    }

    fun coreInvoke(method: String, argsJson: String): String = call {
        coreInvokeNative(method, argsJson).orEmpty()
    }

    fun normalizeManifestUrl(rawUrl: String): String = call { normalizeManifestUrlNative(rawUrl) }

    fun identity(rawUrl: String): String = call { identityNative(rawUrl) }

    fun manifestCandidates(rawUrl: String): List<String> = call {
        manifestCandidatesJsonNative(rawUrl)
    }.let { json ->
        gson.fromJson<List<String>>(json, stringListType)
    }

    fun manifestFetchPlan(rawUrl: String): NativeManifestFetchPlan? = call {
        manifestFetchPlanJsonNative(rawUrl)
            ?.takeIf { it.isNotBlank() }
            ?.let { gson.fromJson(it, NativeManifestFetchPlan::class.java) }
    }

    fun baseUrl(rawUrl: String): String = call { baseUrlNative(rawUrl) }

    fun preferHttpsAssetUrl(rawUrl: String): String? {
        val resolved = call {
            preferHttpsAssetUrlNative(rawUrl).takeIf { it.isNotBlank() }
        }
        return resolved
    }

    fun addonStoreInputType(text: String): String = call {
        addonStoreInputTypeNative(text)
    }

    fun normalizeCloudstreamRepoUrl(rawUrl: String): String = call {
        normalizeCloudstreamRepoUrlNative(rawUrl)
    }

    fun normalizePluginRepositoryUrl(rawUrl: String): String = call {
        normalizePluginRepositoryUrlNative(rawUrl)
    }

    fun pluginIsSecureRemoteUrl(url: String): Boolean = call {
        pluginIsSecureRemoteUrlNative(url)
    }

    fun pluginSameRepositoryUrl(left: String, right: String): Boolean = call {
        pluginSameRepositoryUrlNative(left, right)
    }

    fun extractAddonManifestUrl(detailText: String): String? = call {
        extractAddonManifestUrlNative(detailText).takeIf { it.isNotBlank() }
    }

    fun addonStoreSearchPolicy(
        query: String,
        nowMillis: Long,
        cachedAtMillis: Long?,
        ttlMillis: Long
    ): NativeAddonStoreSearchPolicy = call {
        val request = NativeAddonStoreSearchPolicyRequest(
            query = query,
            nowMillis = nowMillis,
            cachedAtMillis = cachedAtMillis,
            ttlMillis = ttlMillis
        )
        gson.fromJson(
            addonStoreSearchPolicyJsonNative(gson.toJson(request)),
            NativeAddonStoreSearchPolicy::class.java
        ) ?: NativeAddonStoreSearchPolicy()
    }

    fun repositoryMetaDetailPlan(
        useConfiguredAddons: Boolean,
        authKey: String?,
        localAddons: List<String>?
    ): NativeRepositoryMetaDetailPlan = call {
        val request = NativeRepositoryMetaDetailPlanRequest(
            useConfiguredAddons = useConfiguredAddons,
            authKey = authKey.orEmpty(),
            localAddons = localAddons.orEmpty()
        )
        gson.fromJson(
            repositoryMetaDetailPlanJsonNative(gson.toJson(request)),
            NativeRepositoryMetaDetailPlan::class.java
        ) ?: NativeRepositoryMetaDetailPlan()
    }

    fun repositorySeasonVideos(metaDetail: MetaDetail?, seasonNumber: Int): List<Video> = call {
        val json = repositorySeasonVideosJsonNative(gson.toJson(metaDetail), seasonNumber)
        gson.fromJson<List<Video>>(json, object : TypeToken<List<Video>>() {}.type) ?: emptyList()
    }

    fun manifestFetchDecision(
        forceRefresh: Boolean,
        memoryHit: Boolean,
        persistentHit: Boolean
    ): NativeManifestFetchDecision = call {
        val request = NativeManifestFetchDecisionRequest(
            forceRefresh = forceRefresh,
            memoryHit = memoryHit,
            persistentHit = persistentHit
        )
        gson.fromJson(
            manifestFetchDecisionJsonNative(gson.toJson(request)),
            NativeManifestFetchDecision::class.java
        ) ?: NativeManifestFetchDecision()
    }

    fun addonResourceRequestPlan(
        transportUrl: String,
        resource: String,
        type: String,
        id: String,
        extraArgs: Map<String, String?> = emptyMap(),
        extraRaw: String = ""
    ): NativeAddonResourceRequestPlan = call {
        val request = NativeAddonResourceRequestPlanRequest(
            transportUrl = transportUrl,
            resource = resource,
            contentType = type,
            id = id,
            extraArgs = extraArgs,
            extraRaw = extraRaw
        )
        gson.fromJson(
            addonResourceRequestPlanJsonNative(gson.toJson(request)),
            NativeAddonResourceRequestPlan::class.java
        ) ?: NativeAddonResourceRequestPlan()
    }

    fun addonStreamsWithProvider(streamsJson: String, addonName: String): String = call {
        addonStreamsWithProviderJsonNative(streamsJson, addonName)
    }

    fun buildResourceUrl(
        transportUrl: String,
        resource: String,
        type: String,
        id: String,
        extraArgs: Map<String, String?>
    ): String = call {
        buildResourceUrlNative(
            transportUrl,
            resource,
            type,
            id,
            gson.toJson(extraArgs.filterValues { !it.isNullOrBlank() })
        )
    }

    fun parseManifestJson(
        body: String,
        transportUrl: String,
        unknownName: String
    ): AddonDescriptor? {
        val json = call {
            parseManifestJsonNative(body, transportUrl, unknownName)
        }
        return json.takeIf { it.isNotBlank() }?.let { gson.fromJson(it, AddonDescriptor::class.java) }
    }

    fun resolveManifestAssets(descriptor: AddonDescriptor): AddonDescriptor? = call {
        resolveManifestAssetsJsonNative(gson.toJson(descriptor))
            .takeIf { it.isNotBlank() }
            ?.let { gson.fromJson(it, AddonDescriptor::class.java) }
    }

    fun mergeLiveManifest(
        descriptor: AddonDescriptor,
        live: AddonDescriptor?,
        unknownName: String
    ): AddonDescriptor? = call {
        mergeLiveManifestJsonNative(
            gson.toJson(descriptor),
            live?.let(gson::toJson).orEmpty(),
            unknownName
        )
            .takeIf { it.isNotBlank() }
            ?.let { gson.fromJson(it, AddonDescriptor::class.java) }
    }

    fun supportsResource(
        manifest: AddonManifest,
        resourceName: String,
        type: String?,
        id: String?
    ): Boolean = call {
        supportsResourceNative(
            gson.toJson(manifest),
            resourceName,
            type.orEmpty(),
            id.orEmpty()
        )
    }

    fun catalogSupportsExtra(catalog: AddonCatalog, extraName: String): Boolean = call {
        catalogSupportsExtraNative(gson.toJson(catalog), extraName)
    }

    fun catalogRequiresExtra(catalog: AddonCatalog, extraName: String): Boolean = call {
        catalogRequiresExtraNative(gson.toJson(catalog), extraName)
    }

    fun catalogHasRequiredExtraExcept(catalog: AddonCatalog, allowedNames: Set<String>): Boolean = call {
        catalogHasRequiredExtraExceptNative(gson.toJson(catalog), gson.toJson(allowedNames))
    }

    fun parseEpisodeLocator(raw: String?): NativeEpisodeLocator? = call {
        val json = parseEpisodeLocatorJsonNative(raw.orEmpty()).orEmpty()
        json.takeIf { it.isNotBlank() }?.let { gson.fromJson(it, NativeEpisodeLocator::class.java) }
    }

    fun streamRequestIds(
        type: String,
        id: String,
        detailId: String?,
        currentSeriesLookupId: String?,
        canonicalBaseId: String?
    ): List<String> = call {
        val json = streamRequestIdsJsonNative(
            type,
            id,
            detailId.orEmpty(),
            currentSeriesLookupId.orEmpty(),
            canonicalBaseId.orEmpty()
        )
        gson.fromJson<List<String>>(json, stringListType) ?: emptyList()
    }

    fun playbackStreamRequestIds(type: String, id: String, detailId: String?): List<String> = call {
        val json = playbackStreamRequestIdsJsonNative(type, id, detailId.orEmpty())
        gson.fromJson<List<String>>(json, stringListType) ?: emptyList()
    }

    fun playbackIntroLookupContentId(id: String): String = call {
        playbackIntroLookupContentIdNative(id)
    }

    fun directPlaybackPlan(meta: Meta, detail: MetaDetail?, todayIso: String): NativeDirectPlaybackPlan = call {
        val json = directPlaybackPlanJsonNative(
            gson.toJson(meta),
            detail?.let(gson::toJson).orEmpty(),
            todayIso
        )
        gson.fromJson(json, NativeDirectPlaybackPlan::class.java)
            ?: NativeDirectPlaybackPlan(meta = meta, lookupId = meta.id)
    }

    fun headlessProviderAvailability(
        addons: List<AddonDescriptor>,
        pluginNames: List<String>
    ): NativeProviderAvailabilityPlan = call {
        val request = NativeProviderAvailabilityPlanRequest(addons, pluginNames)
        gson.fromJson(
            headlessProviderAvailabilityPlanJsonNative(gson.toJson(request)),
            NativeProviderAvailabilityPlan::class.java
        ) ?: NativeProviderAvailabilityPlan()
    }

    fun headlessDetailStreamResult(
        attempts: List<Pair<String, List<Stream>>>,
        hasStreamProviders: Boolean
    ): NativeDetailStreamResultPlan = call {
        val request = NativeDetailStreamResultPlanRequest(
            attempts = attempts.map { (requestId, streams) ->
                NativeDetailStreamAttemptRequest(requestId, streams)
            },
            hasStreamProviders = hasStreamProviders
        )
        gson.fromJson(
            headlessDetailStreamResultPlanJsonNative(gson.toJson(request)),
            NativeDetailStreamResultPlan::class.java
        ) ?: NativeDetailStreamResultPlan(hasStreamProviders = hasStreamProviders)
    }

    fun headlessPrefetchDetailStreams(streams: List<Stream>): NativePrefetchDetailStreamsPlan = call {
        val request = NativePrefetchDetailStreamsPlanRequest(streams)
        gson.fromJson(
            headlessPrefetchDetailStreamsPlanJsonNative(gson.toJson(request)),
            NativePrefetchDetailStreamsPlan::class.java
        ) ?: NativePrefetchDetailStreamsPlan(count = streams.size)
    }

    fun headlessDirectPlaybackPolicy(): NativeDirectPlaybackPolicy = call {
        gson.fromJson(headlessDirectPlaybackPolicyJsonNative(), NativeDirectPlaybackPolicy::class.java)
            ?: NativeDirectPlaybackPolicy()
    }

    fun streamDiscoveryEpisodeContext(
        type: String,
        requestId: String,
        detail: MetaDetail?,
        seasonEpisodes: List<Video>
    ): NativeStreamDiscoveryEpisodeContext = call {
        val json = streamDiscoveryEpisodeContextJsonNative(
            type,
            requestId,
            detail?.let(gson::toJson).orEmpty(),
            gson.toJson(seasonEpisodes)
        )
        gson.fromJson(json, NativeStreamDiscoveryEpisodeContext::class.java)
            ?: NativeStreamDiscoveryEpisodeContext()
    }

    fun streamPlaybackInfo(stream: Stream): NativeStreamPlaybackInfo = call {
        val json = streamPlaybackInfoJsonNative(gson.toJson(stream))
        gson.fromJson(json, NativeStreamPlaybackInfo::class.java) ?: NativeStreamPlaybackInfo()
    }

    fun dvProxyPlan(
        streamJson: String,
        url: String,
        fallbackMode: String,
        deviceHasDvDecoder: Boolean,
        deviceHasDvDisplay: Boolean
    ): NativeDvProxyPlan = call {
        val requestJson = gson.toJson(mapOf(
            "stream" to gson.fromJson(streamJson, Any::class.java),
            "url" to url,
            "fallbackMode" to fallbackMode,
            "deviceHasDvDecoder" to deviceHasDvDecoder,
            "deviceHasDvDisplay" to deviceHasDvDisplay
        ))
        gson.fromJson(dvProxyPlanJsonNative(requestJson), NativeDvProxyPlan::class.java)
            ?: NativeDvProxyPlan()
    }

    fun dolbyVisionRpuInfo(rpuBase64: String): NativeDolbyVisionRpuInfo = call {
        val json = dolbyVisionRpuInfoJsonNative(gson.toJson(mapOf("rpu_base64" to rpuBase64)))
        gson.fromJson(json, JsonObject::class.java)?.toDolbyVisionRpuInfo() ?: NativeDolbyVisionRpuInfo()
    }

    fun dolbyVisionConvertRpu(rpuBase64: String, mode: Int = 2): NativeDolbyVisionRpuConvertResult = call {
        val json = dolbyVisionConvertRpuJsonNative(gson.toJson(mapOf("rpu_base64" to rpuBase64, "mode" to mode)))
        gson.fromJson(json, JsonObject::class.java)?.toDolbyVisionRpuConvertResult()
            ?: NativeDolbyVisionRpuConvertResult()
    }

    fun streamRequestHeaders(streamHeaders: Map<String, String>): Map<String, String> = call {
        val json = streamRequestHeadersJsonNative(gson.toJson(streamHeaders))
        gson.fromJson<Map<String, String>>(json, object : TypeToken<Map<String, String>>() {}.type) ?: emptyMap()
    }

    fun streamRequestReferer(url: String): String? = call {
        streamRequestRefererNative(url).takeIf { it.isNotBlank() }
    }

    fun isTorrentPlaybackUrl(url: String?): Boolean = call {
        val stream = Stream(name = null, title = null, url = url)
        streamPlaybackInfo(stream).isTorrentPlaybackUrl
    }

    fun episodeTextMatches(text: String?, season: Int, episode: Int): Boolean = call {
        episodeTextMatchesNative(text.orEmpty(), season, episode)
    }

    fun streamMatchesEpisode(
        videoId: String?,
        title: String?,
        name: String?,
        description: String?,
        filename: String?,
        effectiveFilename: String?
    ): Boolean = call {
        streamMatchesEpisodeNative(
            videoId.orEmpty(),
            title.orEmpty(),
            name.orEmpty(),
            description.orEmpty(),
            filename.orEmpty(),
            effectiveFilename.orEmpty()
        )
    }

    fun selectStreamIndex(
        streams: List<Stream>,
        currentVideoId: String?,
        initialStreamIndex: Int,
        savedUrl: String?,
        savedTitle: String?,
        sourceSelectionMode: String,
        regexPattern: String?,
        preferredBingeGroup: String?
    ): Int = call {
        val nativeStreams = streams.map { stream ->
            NativeStreamSelectionItem(
                name = stream.name,
                title = stream.title,
                description = stream.description,
                addonName = stream.addonName,
                playableUrl = stream.playableUrl,
                bingeGroup = stream.bingeGroup,
                filename = stream.filename,
                effectiveFilename = stream.effectiveFilename
            )
        }
        selectStreamIndexNative(
            gson.toJson(nativeStreams),
            currentVideoId.orEmpty(),
            initialStreamIndex,
            savedUrl.orEmpty(),
            savedTitle.orEmpty(),
            sourceSelectionMode,
            regexPattern.orEmpty(),
            preferredBingeGroup.orEmpty()
        )
    }

    fun mergeContinueWatchingDuplicates(items: List<Meta>): List<Meta> = call {
        val json = mergeContinueWatchingDuplicatesJsonNative(gson.toJson(items))
        gson.fromJson<List<Meta>>(json, metaListType) ?: emptyList()
    }

    fun filterDiscoverResults(
        items: List<Meta>,
        year: String?,
        rating: Float?,
        region: String?
    ): List<Meta> = call {
        val json = filterDiscoverResultsJsonNative(
            gson.toJson(items),
            year.orEmpty(),
            rating ?: 0f,
            rating != null,
            region.orEmpty()
        )
        gson.fromJson<List<Meta>>(json, metaListType) ?: emptyList()
    }

    fun resolvePreferredAudioLanguage(
        lastAudioLanguage: String?,
        preferredAudioLanguage: String?,
        originalLanguage: String?
    ): String = call {
        resolvePreferredAudioLanguageNative(
            lastAudioLanguage.orEmpty(),
            preferredAudioLanguage.orEmpty(),
            originalLanguage.orEmpty()
        )
    }

    fun subtitleLanguageMatches(label: String, language: String?, preferredLanguage: String): Boolean = call {
        subtitleLanguageMatchesNative(label, language.orEmpty(), preferredLanguage)
    }

    fun playerTrackState(
        availableSubtitles: List<SubtitleTrackRef>,
        lastAudioLanguage: String?,
        preferredAudioLanguage: String?,
        originalLanguage: String?,
        lastSubtitleLanguage: String?,
        preferredSubtitleLanguage: String?,
        secondarySubtitleLanguage: String?
    ): NativePlayerTrackState = call {
        val request = NativePlayerTrackStateRequest(
            availableSubtitles = availableSubtitles.map { NativeSubtitleTrackEntry(it.id, it.label, it.language) },
            lastAudioLanguage = lastAudioLanguage,
            preferredAudioLanguage = preferredAudioLanguage,
            originalLanguage = originalLanguage,
            lastSubtitleLanguage = lastSubtitleLanguage,
            preferredSubtitleLanguage = preferredSubtitleLanguage,
            secondarySubtitleLanguage = secondarySubtitleLanguage
        )
        gson.fromJson(playerTrackStateJsonNative(gson.toJson(request)), NativePlayerTrackState::class.java)
            ?: NativePlayerTrackState()
    }

    fun parseAddonResourceResult(
        resource: String,
        url: String,
        statusCode: Int,
        body: String?
    ): NativeAddonResourceParseResult = call {
        val json = parseAddonResourceResultJsonNative(resource, url, statusCode, body.orEmpty())
        gson.fromJson(json, NativeAddonResourceParseResult::class.java) ?: NativeAddonResourceParseResult(
            kind = "parse_error",
            url = url,
            statusCode = statusCode,
            error = "empty native response"
        )
    }

    fun normalizeAddonSubtitles(subtitlesJson: String, resourceUrl: String): String = call {
        normalizeAddonSubtitlesJsonNative(subtitlesJson, resourceUrl)
    }

    fun torrentRuntimeInfo(
        link: String,
        title: String,
        requestedFileIdx: Int?,
        preferredFilename: String?,
        sources: List<String>,
        fileStats: List<TorrentFileStat>,
        rejectedIndex: Int?,
        baseUrl: String,
        play: Boolean,
        stat: Boolean
    ): NativeTorrentRuntimeInfo = call {
        val request = NativeTorrentRuntimeRequest(
            link = link,
            title = title,
            requestedFileIdx = requestedFileIdx,
            preferredFilename = preferredFilename,
            sources = sources,
            fileStats = fileStats,
            rejectedIndex = rejectedIndex,
            baseUrl = baseUrl,
            play = play,
            stat = stat
        )
        val json = torrentRuntimeInfoJsonNative(gson.toJson(request))
        gson.fromJson(json, NativeTorrentRuntimeInfo::class.java) ?: NativeTorrentRuntimeInfo()
    }

    fun torrentStatusInfo(status: TorrentStatus): NativeTorrentStatusInfo = call {
        val json = torrentStatusInfoJsonNative(gson.toJson(status))
        gson.fromJson(json, NativeTorrentStatusInfo::class.java) ?: NativeTorrentStatusInfo()
    }

    fun stableFeedPart(value: String): String = call {
        stableFeedPartNative(value)
    }

    fun normalizeContentType(value: String): String? = call {
        normalizeContentTypeNative(value).takeIf { it.isNotBlank() }
    }

    fun parseExtraArgs(extra: String): Map<String, String> = call {
        val json = parseExtraArgsJsonNative(extra)
        gson.fromJson<Map<String, String>>(json, object : TypeToken<Map<String, String>>() {}.type) ?: emptyMap()
    }

    fun providerSearchTerms(provider: String): List<String> = call {
        val json = providerSearchTermsJsonNative(provider)
        gson.fromJson<List<String>>(json, stringListType) ?: emptyList()
    }

    fun effectiveMetadataFeedSelection(selectedKeys: List<String>?, availableKeys: List<String>): List<String>? = call {
        val json = effectiveMetadataFeedSelectionJsonNative(gson.toJson(selectedKeys), gson.toJson(availableKeys))
        json?.takeIf { it.isNotBlank() }?.let { gson.fromJson<List<String>>(it, stringListType) }
    }

    fun toggleMetadataFeed(selectedKeys: List<String>?, availableKeys: List<String>, key: String): List<String> = call {
        val json = toggleMetadataFeedJsonNative(gson.toJson(selectedKeys), gson.toJson(availableKeys), key)
        gson.fromJson<List<String>>(json, stringListType) ?: emptyList()
    }

    fun toggleMetadataFeed(
        selectedKeys: List<String>?,
        availableKeys: List<String>,
        key: String,
        maxEnabled: Int
    ): List<String> = call {
        val json = toggleMetadataFeedLimitedJsonNative(
            gson.toJson(selectedKeys),
            gson.toJson(availableKeys),
            key,
            maxEnabled
        )
        gson.fromJson<List<String>>(json, stringListType) ?: emptyList()
    }

    fun setMetadataFeedGroupEnabled(
        selectedKeys: List<String>?,
        availableKeys: List<String>,
        groupKeys: List<String>,
        enabled: Boolean
    ): List<String> = call {
        val json = setMetadataFeedGroupEnabledJsonNative(
            gson.toJson(selectedKeys),
            gson.toJson(availableKeys),
            gson.toJson(groupKeys),
            enabled
        )
        gson.fromJson<List<String>>(json, stringListType) ?: emptyList()
    }

    fun orderedMetadataFeedKeys(optionKeys: List<String>, order: List<String>?): List<String> = call {
        val json = orderedMetadataFeedKeysJsonNative(gson.toJson(optionKeys), gson.toJson(order))
        gson.fromJson<List<String>>(json, stringListType) ?: optionKeys
    }

    fun moveMetadataFeedOrder(optionKeys: List<String>, currentOrder: List<String>?, key: String, delta: Int): List<String> = call {
        val json = moveMetadataFeedOrderJsonNative(gson.toJson(optionKeys), gson.toJson(currentOrder), key, delta)
        gson.fromJson<List<String>>(json, stringListType) ?: optionKeys
    }

    fun contentTraktKey(meta: Meta): String = call {
        contentTraktKeysBatchJsonNative(gson.toJson(listOf(meta))).let { json ->
            (gson.fromJson<List<String>>(json, stringListType) ?: emptyList()).first()
        }
    }

    fun contentTraktKeysBatch(metas: List<Meta>): List<String> = call {
        if (metas.isEmpty()) return@call emptyList()
        val json = contentTraktKeysBatchJsonNative(gson.toJson(metas))
        gson.fromJson<List<String>>(json, stringListType) ?: emptyList()
    }

    fun contentMergeKeys(meta: Meta): Set<String> = call {
        val json = contentMergeKeysJsonNative(gson.toJson(meta))
        (gson.fromJson<List<String>>(json, stringListType) ?: emptyList()).toSet()
    }

    fun contentWatchedKeysBatch(metas: List<Meta>): List<Set<String>> = call {
        if (metas.isEmpty()) return@call emptyList()
        val json = contentWatchedKeysBatchJsonNative(gson.toJson(metas))
        (gson.fromJson<List<List<String>>>(json, stringListListType) ?: emptyList()).map { it.toSet() }
    }

    fun episodeFilenameCandidate(stream: Stream, videoId: String?): String? = call {
        episodeFilenameCandidateNative(gson.toJson(stream), videoId.orEmpty()).takeIf { it.isNotBlank() }
    }

    fun streamDiscoveryCacheKey(
        type: String,
        id: String,
        language: String,
        cs3SearchQuery: String?,
        cs3Year: Int?,
        cs3OriginalName: String?,
        addonSignatures: List<String>,
        cs3PluginNames: List<String>
    ): String = call {
        streamDiscoveryCacheKeyNative(
            gson.toJson(
                NativeStreamDiscoveryCacheKeyRequest(
                    type = type,
                    id = id,
                    language = language,
                    cs3SearchQuery = cs3SearchQuery,
                    cs3Year = cs3Year,
                    cs3OriginalName = cs3OriginalName,
                    addonSignatures = addonSignatures,
                    cs3PluginNames = cs3PluginNames
                )
            )
        )
    }

    fun streamDiscoveryPlan(
        type: String,
        id: String,
        language: String,
        preferFastStart: Boolean,
        addonRequestTimeoutMs: Long,
        fastAddonRequestTimeoutMs: Long,
        cloudstreamTimeoutMs: Long,
        addons: List<AddonDescriptor>,
        cs3PluginNames: List<String>,
        cs3SearchQuery: String?,
        cs3OriginalName: String?,
        cs3Year: Int?
    ): NativeStreamDiscoveryPlan = call {
        val request = NativeStreamDiscoveryPlanRequest(
            type = type,
            id = id,
            language = language,
            preferFastStart = preferFastStart,
            addonRequestTimeoutMs = addonRequestTimeoutMs,
            fastAddonRequestTimeoutMs = fastAddonRequestTimeoutMs,
            cloudstreamTimeoutMs = cloudstreamTimeoutMs,
            addons = addons,
            cs3PluginNames = cs3PluginNames,
            cs3SearchQuery = cs3SearchQuery,
            cs3OriginalName = cs3OriginalName,
            cs3Year = cs3Year
        )
        streamDiscoveryPlanJsonNative(gson.toJson(request))
            ?.takeIf { it.isNotBlank() }
            ?.let { gson.fromJson(it, NativeStreamDiscoveryPlan::class.java) }
            ?: NativeStreamDiscoveryPlan()
    }

    fun streamDiscoveryExecutionPolicy(
        type: String,
        id: String,
        language: String,
        preferFastStart: Boolean,
        addonRequestTimeoutMs: Long,
        fastAddonRequestTimeoutMs: Long,
        cloudstreamTimeoutMs: Long,
        addons: List<AddonDescriptor>,
        cs3PluginNames: List<String>,
        cs3SearchQuery: String?,
        cs3OriginalName: String?,
        cs3Year: Int?
    ): NativeStreamDiscoveryExecutionPolicy = call {
        val request = NativeStreamDiscoveryPlanRequest(
            type = type,
            id = id,
            language = language,
            preferFastStart = preferFastStart,
            addonRequestTimeoutMs = addonRequestTimeoutMs,
            fastAddonRequestTimeoutMs = fastAddonRequestTimeoutMs,
            cloudstreamTimeoutMs = cloudstreamTimeoutMs,
            addons = addons,
            cs3PluginNames = cs3PluginNames,
            cs3SearchQuery = cs3SearchQuery,
            cs3OriginalName = cs3OriginalName,
            cs3Year = cs3Year
        )
        streamDiscoveryExecutionPolicyJsonNative(gson.toJson(request))
            ?.takeIf { it.isNotBlank() }
            ?.let { gson.fromJson(it, NativeStreamDiscoveryExecutionPolicy::class.java) }
            ?: NativeStreamDiscoveryExecutionPolicy()
    }

    fun streamDiscoveryCachePrefix(type: String, id: String, language: String): String = call {
        streamDiscoveryCachePrefixNative(type, id, language)
    }

    fun discoverCatalogCacheKey(
        type: String,
        catalogKey: String?,
        genre: String?,
        year: String?,
        rating: Float?,
        provider: String?,
        region: String?,
        catalogSignatures: List<String>
    ): String = call {
        discoverCatalogCacheKeyNative(
            gson.toJson(
                NativeDiscoverCatalogCacheKeyRequest(
                    type = type,
                    catalogKey = catalogKey,
                    genre = genre,
                    year = year,
                    rating = rating,
                    provider = provider,
                    region = region,
                    catalogSignatures = catalogSignatures
                )
            )
        )
    }

    fun curateHomeItemsJson(categoryJson: String): String = call {
        curateHomeItemsJsonNative(categoryJson)
    }

    fun homeOverlapRatioJson(firstCategoryJson: String, secondCategoryJson: String): Float = call {
        homeOverlapRatioNative(firstCategoryJson, secondCategoryJson)
    }

    fun homePersonalizationScoreJson(
        categoryJson: String,
        preferredGenresJson: String,
        preferredTypesJson: String,
        priorityLabelsJson: String
    ): Int = call {
        homePersonalizationScoreNative(
            categoryJson,
            preferredGenresJson,
            preferredTypesJson,
            priorityLabelsJson
        )
    }

    fun prioritizeHomeRowsJson(
        categoriesJson: String,
        preferredOrderLabelsJson: String,
        preferredGenresJson: String,
        preferredTypesJson: String,
        priorityLabelsJson: String
    ): String = call {
        prioritizeHomeRowsJsonNative(
            categoriesJson,
            preferredOrderLabelsJson,
            preferredGenresJson,
            preferredTypesJson,
            priorityLabelsJson
        )
    }

    fun optimizeHomeRowsJson(requestJson: String): String = call {
        optimizeHomeRowsJsonNative(requestJson)
    }

    fun buildBillboardPool(enriched: List<Meta>, candidates: List<Meta>): List<Meta> = call {
        val json = buildBillboardPoolJsonNative(gson.toJson(enriched), gson.toJson(candidates))
        if (json != null) gson.fromJson<List<Meta>>(json, metaListType) ?: emptyList() else emptyList()
    }

    fun normalizeHomeCatalogItems(items: List<Meta>, catalogId: String, genre: String?): List<Meta> = call {
        val todayIso = java.time.LocalDate.now(java.time.ZoneId.systemDefault()).toString()
        val json = normalizeHomeCatalogItemsJsonNative(
            gson.toJson(items),
            catalogId,
            genre.orEmpty(),
            todayIso
        )
        if (json != null) gson.fromJson<List<Meta>>(json, metaListType) ?: emptyList() else emptyList()
    }

    fun playerProgressPercent(positionMs: Long, durationMs: Long): Float = call {
        playerProgressPercentNative(positionMs, durationMs)
    }

    fun playerShouldSendScrobbleStart(
        token: String?,
        isPlaying: Boolean,
        hasScrobbledStart: Boolean,
        progress: Float
    ): Boolean = call {
        playerShouldSendScrobbleStartNative(token.orEmpty(), isPlaying, hasScrobbledStart, progress)
    }

    fun playerShouldMarkScrobbleStopped(hasScrobbledStop: Boolean, progress: Float): Boolean = call {
        playerShouldMarkScrobbleStoppedNative(hasScrobbledStop, progress)
    }

    fun playerShouldQueueScrobblePause(
        token: String?,
        wasPlayWhenReady: Boolean,
        hasScrobbledStart: Boolean,
        hasScrobbledStop: Boolean
    ): Boolean = call {
        playerShouldQueueScrobblePauseNative(
            token.orEmpty(),
            wasPlayWhenReady,
            hasScrobbledStart,
            hasScrobbledStop
        )
    }

    fun playerShouldEnqueueDurableScrobble(action: String, token: String?, progress: Float): Boolean = call {
        playerShouldEnqueueDurableScrobbleNative(action, token.orEmpty(), progress)
    }

    fun playerShouldSavePeriodicProgress(isPlaying: Boolean, nowMs: Long, lastSavedAtMs: Long): Boolean = call {
        playerShouldSavePeriodicProgressNative(isPlaying, nowMs, lastSavedAtMs)
    }

    fun playerShouldSaveOnDispose(positionMs: Long): Boolean = call {
        playerShouldSaveOnDisposeNative(positionMs)
    }

    fun safePlayerBufferCacheMb(value: Int?): Int = call {
        safePlayerBufferCacheMbNative(value ?: 0, value != null)
    }

    fun safeStreamSourceSelectionMode(mode: String?): String = call {
        safeStreamSourceSelectionModeNative(mode.orEmpty())
    }

    fun playerFlowDispatch(state: Any, action: Any): NativePlayerFlowResult = call {
        val json = playerFlowDispatchJsonNative(gson.toJson(state), gson.toJson(action))
        json?.takeIf { it.isNotBlank() }?.let { gson.fromJson(it, NativePlayerFlowResult::class.java) }
            ?: NativePlayerFlowResult()
    }

    fun traktHasClient(apiKey: String): Boolean = call {
        traktHasClientNative(apiKey)
    }

    fun traktBearer(token: String): String = call {
        traktBearerNative(token)
    }

    fun traktScrobbleUrl(action: String): String = call {
        traktScrobbleUrlNative(action)
    }

    fun traktPlaybackUrl(type: String?): String = call {
        traktPlaybackUrlNative(type.orEmpty())
    }

    fun traktTokenExpiresAt(createdAtSeconds: Long, expiresInSeconds: Long): Long = call {
        traktTokenExpiresAtNative(createdAtSeconds, expiresInSeconds)
    }

    fun traktContentIdFrom(ids: TraktIds): String? = call {
        traktContentIdFromIdsNative(gson.toJson(ids)).takeIf { it.isNotBlank() }
    }

    fun traktIdsFromContentId(rawId: String): TraktIds? = call {
        traktIdsFromContentIdJsonNative(rawId)
            ?.takeIf { it.isNotBlank() }
            ?.let { gson.fromJson(it, TraktIds::class.java) }
    }

    fun traktEpisodeLocator(videoId: String): NativeTraktEpisodeLocator? = call {
        traktEpisodeLocatorJsonNative(videoId)
            ?.takeIf { it.isNotBlank() }
            ?.let { gson.fromJson(it, NativeTraktEpisodeLocator::class.java) }
    }

    fun traktShowIdFromEpisodeId(videoId: String): String = call {
        traktShowIdFromEpisodeIdNative(videoId)
    }

    fun traktScrobbleMediaId(parentId: String, videoId: String?, mediaType: String): String = call {
        traktScrobbleMediaIdNative(parentId, videoId.orEmpty(), mediaType)
    }

    fun traktOAuthErrorCode(body: String): String? = call {
        traktOAuthErrorCodeNative(body).takeIf { it.isNotBlank() }
    }

    fun traktHistoryRequest(meta: Meta, episodes: List<Video>): TraktHistorySyncRequest? = call {
        traktHistoryRequestJsonNative(gson.toJson(meta), gson.toJson(episodes))
            ?.takeIf { it.isNotBlank() }
            ?.let { gson.fromJson(it, TraktHistorySyncRequest::class.java) }
    }

    fun simklScrobbleBody(
        idsJson: String,
        isEpisode: Boolean,
        season: Long,
        epNumber: Long,
        timePosSec: Double,
        durationSec: Double
    ): String? = call {
        simklScrobbleBodyJsonNative(idsJson, isEpisode, season, epNumber, timePosSec, durationSec)
            ?.takeIf { it.isNotBlank() }
    }

    fun simklMatchEpisode(episodesJson: String, targetJson: String): NativeSimklEpisodeMatch? = call {
        simklMatchEpisodeJsonNative(episodesJson, targetJson)
            ?.takeIf { it.isNotBlank() }
            ?.let { gson.fromJson(it, NativeSimklEpisodeMatch::class.java) }
    }

    fun simklWatchingToItems(showsJson: String, moviesJson: String): String? = call {
        simklWatchingToItemsJsonNative(showsJson, moviesJson)?.takeIf { it.isNotBlank() }
    }

    fun simklWatchlistToItems(showsJson: String, moviesJson: String): String? = call {
        simklWatchlistToItemsJsonNative(showsJson, moviesJson)?.takeIf { it.isNotBlank() }
    }

    fun simklWatchedToIds(showsJson: String, moviesJson: String): String? = call {
        simklWatchedToIdsJsonNative(showsJson, moviesJson)?.takeIf { it.isNotBlank() }
    }

    fun playbackProgressItem(meta: Meta, timeOffset: Long, duration: Long, nowUtc: String): LibraryItem? = call {
        playbackProgressItemJsonNative(gson.toJson(meta), timeOffset, duration, nowUtc)
            ?.takeIf { it.isNotBlank() }
            ?.let { gson.fromJson(it, LibraryItem::class.java) }
    }

    fun clearPlaybackProgressItem(meta: Meta): LibraryItem? = call {
        clearPlaybackProgressItemJsonNative(gson.toJson(meta))
            ?.takeIf { it.isNotBlank() }
            ?.let { gson.fromJson(it, LibraryItem::class.java) }
    }

    fun watchedStateItems(meta: Meta, episodes: List<Video>, watched: Boolean, watchedAt: String?): List<LibraryItem> = call {
        val json = watchedStateItemsJsonNative(gson.toJson(meta), gson.toJson(episodes), watched, watchedAt.orEmpty())
        gson.fromJson<List<LibraryItem>>(json, libraryItemListType) ?: emptyList()
    }

    fun libraryContinueWatchingItems(items: List<LibraryItem>): List<Meta> = call {
        val json = libraryContinueWatchingItemsJsonNative(gson.toJson(items))
        gson.fromJson<List<Meta>>(json, metaListType) ?: emptyList()
    }

    fun filterHomeContinueWatching(items: List<Meta>, traktWatchedState: TraktWatchedState): List<Meta> = call {
        val traktJson = gson.toJson(traktWatchedState)
        val json = filterHomeContinueWatchingJsonNative(gson.toJson(items), traktJson)
        gson.fromJson<List<Meta>>(json, metaListType) ?: emptyList()
    }

    fun watchedVideoIds(items: List<LibraryItem>, imdbId: String): List<String> = call {
        val json = watchedVideoIdsJsonNative(gson.toJson(items), imdbId)
        gson.fromJson<List<String>>(json, stringListType) ?: emptyList()
    }

    fun offlineDownloadPlan(
        meta: Meta,
        video: Video?,
        videoId: String?,
        stream: Stream,
        subtitleUrl: String?,
        downloadId: String
    ): NativeOfflineDownloadPlan = call {
        val request = NativeOfflineDownloadPlanRequest(
            meta = meta,
            video = video,
            videoId = videoId,
            stream = stream,
            subtitleUrl = subtitleUrl,
            downloadId = downloadId
        )
        gson.fromJson(offlineDownloadPlanJsonNative(gson.toJson(request)), NativeOfflineDownloadPlan::class.java)
            ?: NativeOfflineDownloadPlan(reason = "unsupported_source")
    }

    fun profileSafePrefs(profile: Any): NativeProfileSafePrefs = call {
        gson.fromJson(profileSafePrefsJsonNative(gson.toJson(profile)), NativeProfileSafePrefs::class.java)
            ?: NativeProfileSafePrefs()
    }

    fun <T> sanitizeProfile(
        profile: T,
        mirroredAddons: Collection<String>,
        mergeMirroredAddons: Boolean,
        type: Class<T>
    ): T? = call {
        sanitizeProfileJsonNative(gson.toJson(profile), gson.toJson(mirroredAddons), mergeMirroredAddons)
            ?.takeIf { it.isNotBlank() }
            ?.let { gson.fromJson(it, type) }
    }

    fun profileLocalAddonsKey(profile: Any): String = call {
        profileLocalAddonsKeyNative(gson.toJson(profile))
    }

    fun cacheEntryPolicy(
        key: String,
        storedAtMillis: Long,
        ttlMillis: Long,
        nowMillis: Long
    ): NativeCacheEntryPolicy = call {
        val request = NativeCacheEntryPolicyRequest(
            key = key,
            storedAtMillis = storedAtMillis,
            ttlMillis = ttlMillis,
            nowMillis = nowMillis
        )
        gson.fromJson(cacheEntryPolicyJsonNative(gson.toJson(request)), NativeCacheEntryPolicy::class.java)
            ?: NativeCacheEntryPolicy(key = key, storedAtMillis = storedAtMillis)
    }

    fun cacheTrimPolicy(
        entries: List<Triple<String, Long, Long>>,
        maxEntries: Int,
        nowMillis: Long
    ): NativeCacheTrimPolicy = call {
        val request = NativeCacheTrimPolicyRequest(
            entries = entries.map { (key, expiresAtMillis, storedAtMillis) ->
                NativeCacheTrimPolicyEntry(
                    key = key,
                    expiresAtMillis = expiresAtMillis,
                    storedAtMillis = storedAtMillis
                )
            },
            maxEntries = maxEntries,
            nowMillis = nowMillis
        )
        gson.fromJson(cacheTrimPolicyJsonNative(gson.toJson(request)), NativeCacheTrimPolicy::class.java)
            ?: NativeCacheTrimPolicy()
    }

    fun dataFailurePolicy(
        operation: String,
        kind: String,
        message: String? = null,
        throwableClass: String? = null,
        reason: String? = null,
        statusCode: Long? = null
    ): NativeDataFailurePolicy = call {
        val request = NativeDataFailurePolicyRequest(
            operation = operation,
            kind = kind,
            message = message,
            throwableClass = throwableClass,
            reason = reason,
            statusCode = statusCode
        )
        gson.fromJson(dataFailurePolicyJsonNative(gson.toJson(request)), NativeDataFailurePolicy::class.java)
            ?: NativeDataFailurePolicy(operation = operation, kind = kind, message = message ?: reason.orEmpty())
    }

    // ── calendar_plan ─────────────────────────────────────────────────────────
    fun calendarContentPlanJson(requestJson: String): String = call {
        calendarContentPlanJsonNative(requestJson)
    }

    fun calendarSeasonCandidatesJson(requestJson: String): List<Int> = call {
        val json = calendarSeasonCandidatesJsonNative(requestJson)
        gson.fromJson<List<Int>>(json, object : TypeToken<List<Int>>() {}.type) ?: emptyList()
    }

    fun calendarWidgetRowsJson(requestJson: String): String = call {
        calendarWidgetRowsJsonNative(requestJson)
    }

    fun calendarNotificationContent(requestJson: String): NativeCalendarNotificationContent = call {
        gson.fromJson(calendarNotificationContentJsonNative(requestJson), NativeCalendarNotificationContent::class.java)
            ?: NativeCalendarNotificationContent()
    }

    fun calendarReleaseDetectionJson(requestJson: String): String = call {
        calendarReleaseDetectionJsonNative(requestJson)
    }

    // ── profile_contract ──────────────────────────────────────────────────────
    fun activeProfilePlanJson(requestJson: String): NativeActiveProfilePlan = call {
        gson.fromJson(activeProfilePlanJsonNative(requestJson), NativeActiveProfilePlan::class.java)
            ?: NativeActiveProfilePlan()
    }

    fun tokenMergePlanJson(requestJson: String): String = call {
        tokenMergePlanJsonNative(requestJson)
    }

    fun profileDefaultSeedJson(requestJson: String): String = call {
        profileDefaultSeedJsonNative(requestJson)
    }

    fun profileSettingsMigrationPlan(requestJson: String): NativeProfileSettingsMigration = call {
        gson.fromJson(profileSettingsMigrationPlanJsonNative(requestJson), NativeProfileSettingsMigration::class.java)
            ?: NativeProfileSettingsMigration()
    }

    fun profileAvatarDefault(requestJson: String): NativeProfileAvatarDefault = call {
        gson.fromJson(profileAvatarDefaultJsonNative(requestJson), NativeProfileAvatarDefault::class.java)
            ?: NativeProfileAvatarDefault()
    }

    // ── watchlist_plan ────────────────────────────────────────────────────────
    fun watchlistTogglePlan(requestJson: String): NativeWatchlistTogglePlan = call {
        gson.fromJson(watchlistTogglePlanJsonNative(requestJson), NativeWatchlistTogglePlan::class.java)
            ?: NativeWatchlistTogglePlan()
    }

    fun libraryExternalMergePlanJson(requestJson: String): String = call {
        libraryExternalMergePlanJsonNative(requestJson)
    }

    fun libraryCollectionImportValidation(requestJson: String): NativeLibraryCollectionImportValidation = call {
        gson.fromJson(libraryCollectionImportValidationJsonNative(requestJson), NativeLibraryCollectionImportValidation::class.java)
            ?: NativeLibraryCollectionImportValidation()
    }

    fun libraryOfflineGrouping(requestJson: String): NativeLibraryOfflineGrouping = call {
        gson.fromJson(libraryOfflineGroupingJsonNative(requestJson), NativeLibraryOfflineGrouping::class.java)
            ?: NativeLibraryOfflineGrouping()
    }

    fun playbackProgressMergePlanJson(requestJson: String): String = call {
        playbackProgressMergePlanJsonNative(requestJson)
    }

    // ── player_policy ─────────────────────────────────────────────────────────
    fun playerBackendSelection(requestJson: String): NativePlayerBackendSelection = call {
        gson.fromJson(playerBackendSelectionJsonNative(requestJson), NativePlayerBackendSelection::class.java)
            ?: NativePlayerBackendSelection()
    }

    fun torrentFallbackFilePolicy(requestJson: String): NativeTorrentFallbackFilePolicy = call {
        gson.fromJson(torrentFallbackFilePolicyJsonNative(requestJson), NativeTorrentFallbackFilePolicy::class.java)
            ?: NativeTorrentFallbackFilePolicy()
    }

    fun playerBufferTargets(requestJson: String): NativePlayerBufferTargets = call {
        gson.fromJson(playerBufferTargetsJsonNative(requestJson), NativePlayerBufferTargets::class.java)
            ?: NativePlayerBufferTargets()
    }

    fun playerRetryPolicy(requestJson: String): NativePlayerRetryPolicy = call {
        gson.fromJson(playerRetryPolicyJsonNative(requestJson), NativePlayerRetryPolicy::class.java)
            ?: NativePlayerRetryPolicy()
    }

    fun playerSourceSidebarPlanJson(requestJson: String): String = call {
        playerSourceSidebarPlanJsonNative(requestJson)
    }

    // ── search_plan ───────────────────────────────────────────────────────────
    fun searchResultGrouping(requestJson: String): NativeSearchResultGrouping = call {
        gson.fromJson(searchResultGroupingJsonNative(requestJson), NativeSearchResultGrouping::class.java)
            ?: NativeSearchResultGrouping()
    }

    fun discoverSortPlanJson(requestJson: String): String = call {
        discoverSortPlanJsonNative(requestJson)
    }

    fun librarySortPlanJson(requestJson: String): String = call {
        librarySortPlanJsonNative(requestJson)
    }

    fun detailSeriesLookupId(rawId: String): String = call {
        detailSeriesLookupIdNative(rawId)
    }

    fun detailSeasonLoadPlan(requestJson: String): NativeDetailSeasonLoadPlan = call {
        gson.fromJson(detailSeasonLoadPlanJsonNative(requestJson), NativeDetailSeasonLoadPlan::class.java)
            ?: NativeDetailSeasonLoadPlan()
    }

    private inline fun <T> call(block: () -> T): T {
        check(loaded) { "Fluxa core native library is not loaded." }
        return block()
    }

    private fun loadCore(): Boolean {
        val explicitPath = System.getProperty("fluxa.core.library.path")?.takeIf { it.isNotBlank() }
        if (explicitPath != null) {
            System.load(explicitPath)
            return true
        }
        System.loadLibrary("fluxa_core")
        return true
    }

    private external fun coreInvokeNative(method: String, argsJson: String): String?
    private external fun normalizeManifestUrlNative(rawUrl: String): String
    private external fun createAppCoreStateNative(initialJson: String): Long
    private external fun destroyAppCoreStateNative(handle: Long): Boolean
    private external fun appCoreStateJsonNative(handle: Long): String?
    private external fun appCoreDispatchJsonNative(handle: Long, actionJson: String): String?
    private external fun createHeadlessEngineNative(initialJson: String): Long
    private external fun destroyHeadlessEngineNative(handle: Long): Boolean
    private external fun headlessEngineSnapshotJsonNative(handle: Long): String?
    private external fun headlessEngineDispatchJsonNative(handle: Long, actionJson: String): String?
    private external fun headlessEngineCompleteEffectJsonNative(handle: Long, resultJson: String): String?
    private external fun coreCapabilitiesJsonNative(portable: Boolean): String
    private external fun identityNative(rawUrl: String): String
    private external fun manifestCandidatesJsonNative(rawUrl: String): String
    private external fun manifestFetchPlanJsonNative(rawUrl: String): String?
    private external fun baseUrlNative(rawUrl: String): String
    private external fun preferHttpsAssetUrlNative(rawUrl: String): String
    private external fun addonStoreInputTypeNative(text: String): String
    private external fun normalizeCloudstreamRepoUrlNative(rawUrl: String): String
    private external fun normalizePluginRepositoryUrlNative(rawUrl: String): String
    private external fun pluginIsSecureRemoteUrlNative(url: String): Boolean
    private external fun pluginSameRepositoryUrlNative(left: String, right: String): Boolean
    private external fun extractAddonManifestUrlNative(detailText: String): String
    private external fun addonStoreSearchPolicyJsonNative(requestJson: String): String
    private external fun repositoryMetaDetailPlanJsonNative(requestJson: String): String
    private external fun repositorySeasonVideosJsonNative(metaDetailJson: String, seasonNumber: Int): String
    private external fun manifestFetchDecisionJsonNative(requestJson: String): String
    private external fun addonResourceRequestPlanJsonNative(requestJson: String): String
    private external fun addonStreamsWithProviderJsonNative(streamsJson: String, addonName: String): String
    private external fun headlessProviderAvailabilityPlanJsonNative(requestJson: String): String
    private external fun headlessDetailStreamResultPlanJsonNative(requestJson: String): String
    private external fun headlessPrefetchDetailStreamsPlanJsonNative(requestJson: String): String
    private external fun headlessDirectPlaybackPolicyJsonNative(): String
    private external fun offlineDownloadPlanJsonNative(requestJson: String): String
    private external fun profileSafePrefsJsonNative(profileJson: String): String
    private external fun sanitizeProfileJsonNative(
        profileJson: String,
        mirroredAddonsJson: String,
        mergeMirroredAddons: Boolean
    ): String?
    private external fun profileLocalAddonsKeyNative(profileJson: String): String
    private external fun cacheEntryPolicyJsonNative(requestJson: String): String
    private external fun cacheTrimPolicyJsonNative(requestJson: String): String
    private external fun dataFailurePolicyJsonNative(requestJson: String): String
    private external fun buildResourceUrlNative(
        transportUrl: String,
        resource: String,
        type: String,
        id: String,
        extraJson: String
    ): String
    private external fun parseManifestJsonNative(body: String, transportUrl: String, unknownName: String): String
    private external fun resolveManifestAssetsJsonNative(descriptorJson: String): String
    private external fun mergeLiveManifestJsonNative(
        descriptorJson: String,
        liveJson: String,
        unknownName: String
    ): String
    private external fun supportsResourceNative(manifestJson: String, resourceName: String, type: String, id: String): Boolean
    private external fun catalogSupportsExtraNative(catalogJson: String, extraName: String): Boolean
    private external fun catalogRequiresExtraNative(catalogJson: String, extraName: String): Boolean
    private external fun catalogHasRequiredExtraExceptNative(catalogJson: String, allowedNamesJson: String): Boolean
    private external fun parseEpisodeLocatorJsonNative(raw: String): String?
    private external fun streamRequestIdsJsonNative(
        type: String,
        id: String,
        detailId: String,
        currentSeriesLookupId: String,
        canonicalBaseId: String
    ): String
    private external fun playbackStreamRequestIdsJsonNative(type: String, id: String, detailId: String): String
    private external fun streamDiscoveryEpisodeContextJsonNative(
        type: String,
        requestId: String,
        detailJson: String,
        seasonEpisodesJson: String
    ): String
    private external fun playbackIntroLookupContentIdNative(id: String): String
    private external fun directPlaybackPlanJsonNative(metaJson: String, detailJson: String, todayIso: String): String
    private external fun streamPlaybackInfoJsonNative(streamJson: String): String
    private external fun dvProxyPlanJsonNative(requestJson: String): String
    private external fun dolbyVisionRpuInfoJsonNative(requestJson: String): String
    private external fun dolbyVisionConvertRpuJsonNative(requestJson: String): String
    private external fun streamRequestHeadersJsonNative(headersJson: String): String
    private external fun streamRequestRefererNative(url: String): String
    private external fun episodeTextMatchesNative(text: String, season: Int, episode: Int): Boolean
    private external fun streamMatchesEpisodeNative(
        videoId: String,
        title: String,
        name: String,
        description: String,
        filename: String,
        effectiveFilename: String
    ): Boolean
    private external fun selectStreamIndexNative(
        streamsJson: String,
        currentVideoId: String,
        initialStreamIndex: Int,
        savedUrl: String,
        savedTitle: String,
        sourceSelectionMode: String,
        regexPattern: String,
        preferredBingeGroup: String
    ): Int
    private external fun mergeContinueWatchingDuplicatesJsonNative(itemsJson: String): String
    private external fun filterDiscoverResultsJsonNative(
        itemsJson: String,
        year: String,
        rating: Float,
        hasRating: Boolean,
        region: String
    ): String
    private external fun resolvePreferredAudioLanguageNative(
        lastAudioLanguage: String,
        preferredAudioLanguage: String,
        originalLanguage: String
    ): String
    private external fun subtitleLanguageMatchesNative(
        label: String,
        language: String,
        preferredLanguage: String
    ): Boolean
    private external fun findPreferredSubtitleIndexNative(
        tracksJson: String,
        lastSubtitleLanguage: String,
        preferredSubtitleLanguage: String,
        secondarySubtitleLanguage: String
    ): Int
    private external fun playerTrackStateJsonNative(requestJson: String): String
    private external fun parseAddonResourceResultJsonNative(
        resource: String,
        url: String,
        statusCode: Int,
        body: String
    ): String
    private external fun normalizeAddonSubtitlesJsonNative(subtitlesJson: String, resourceUrl: String): String
    private external fun torrentRuntimeInfoJsonNative(requestJson: String): String
    private external fun torrentStatusInfoJsonNative(statusJson: String): String
    private external fun stableFeedPartNative(value: String): String
    private external fun normalizeContentTypeNative(value: String): String
    private external fun parseExtraArgsJsonNative(extra: String): String
    private external fun providerSearchTermsJsonNative(provider: String): String
    private external fun effectiveMetadataFeedSelectionJsonNative(selectedKeysJson: String, availableKeysJson: String): String?
    private external fun toggleMetadataFeedJsonNative(selectedKeysJson: String, availableKeysJson: String, key: String): String
    private external fun toggleMetadataFeedLimitedJsonNative(
        selectedKeysJson: String,
        availableKeysJson: String,
        key: String,
        maxEnabled: Int
    ): String
    private external fun setMetadataFeedGroupEnabledJsonNative(
        selectedKeysJson: String,
        availableKeysJson: String,
        groupKeysJson: String,
        enabled: Boolean
    ): String
    private external fun orderedMetadataFeedKeysJsonNative(optionKeysJson: String, orderJson: String): String
    private external fun moveMetadataFeedOrderJsonNative(
        optionKeysJson: String,
        currentOrderJson: String,
        key: String,
        delta: Int
    ): String
    private external fun contentTraktKeysBatchJsonNative(metasJson: String): String
    private external fun contentMergeKeysJsonNative(metaJson: String): String
    private external fun contentWatchedKeysBatchJsonNative(metasJson: String): String
    private external fun episodeFilenameCandidateNative(streamJson: String, videoId: String): String
    private external fun streamDiscoveryCacheKeyNative(requestJson: String): String
    private external fun discoverCatalogCacheKeyNative(requestJson: String): String
    private external fun streamDiscoveryPlanJsonNative(requestJson: String): String?
    private external fun streamDiscoveryExecutionPolicyJsonNative(requestJson: String): String?
    private external fun streamDiscoveryCachePrefixNative(type: String, id: String, language: String): String
    private external fun buildBillboardPoolJsonNative(enrichedJson: String, candidatesJson: String): String?
    private external fun normalizeHomeCatalogItemsJsonNative(
        itemsJson: String,
        catalogId: String,
        genre: String,
        todayIso: String
    ): String?
    private external fun curateHomeItemsJsonNative(categoryJson: String): String
    private external fun homeOverlapRatioNative(firstCategoryJson: String, secondCategoryJson: String): Float
    private external fun homePersonalizationScoreNative(
        categoryJson: String,
        preferredGenresJson: String,
        preferredTypesJson: String,
        priorityLabelsJson: String
    ): Int
    private external fun prioritizeHomeRowsJsonNative(
        categoriesJson: String,
        preferredOrderLabelsJson: String,
        preferredGenresJson: String,
        preferredTypesJson: String,
        priorityLabelsJson: String
    ): String
    private external fun optimizeHomeRowsJsonNative(requestJson: String): String
    private external fun playerProgressPercentNative(positionMs: Long, durationMs: Long): Float
    private external fun playerShouldSendScrobbleStartNative(
        token: String,
        isPlaying: Boolean,
        hasScrobbledStart: Boolean,
        progress: Float
    ): Boolean
    private external fun playerShouldMarkScrobbleStoppedNative(hasScrobbledStop: Boolean, progress: Float): Boolean
    private external fun playerShouldQueueScrobblePauseNative(
        token: String,
        wasPlayWhenReady: Boolean,
        hasScrobbledStart: Boolean,
        hasScrobbledStop: Boolean
    ): Boolean
    private external fun playerShouldEnqueueDurableScrobbleNative(
        action: String,
        token: String,
        progress: Float
    ): Boolean
    private external fun playerShouldSavePeriodicProgressNative(
        isPlaying: Boolean,
        nowMs: Long,
        lastSavedAtMs: Long
    ): Boolean
    private external fun playerShouldSaveOnDisposeNative(positionMs: Long): Boolean
    private external fun safePlayerBufferCacheMbNative(value: Int, hasValue: Boolean): Int
    private external fun safeStreamSourceSelectionModeNative(mode: String): String
    private external fun playerFlowDispatchJsonNative(stateJson: String, actionJson: String): String?
    private external fun traktHasClientNative(apiKey: String): Boolean
    private external fun traktBearerNative(token: String): String
    private external fun traktScrobbleUrlNative(action: String): String
    private external fun traktPlaybackUrlNative(type: String): String
    private external fun traktTokenExpiresAtNative(createdAtSeconds: Long, expiresInSeconds: Long): Long
    private external fun traktContentIdFromIdsNative(idsJson: String): String
    private external fun traktIdsFromContentIdJsonNative(rawId: String): String?
    private external fun traktEpisodeLocatorJsonNative(videoId: String): String?
    private external fun traktShowIdFromEpisodeIdNative(videoId: String): String
    private external fun traktScrobbleMediaIdNative(parentId: String, videoId: String, mediaType: String): String
    private external fun traktOAuthErrorCodeNative(body: String): String
    private external fun traktHistoryRequestJsonNative(metaJson: String, episodesJson: String): String?
    private external fun simklScrobbleBodyJsonNative(
        idsJson: String,
        isEpisode: Boolean,
        season: Long,
        epNumber: Long,
        timePosSec: Double,
        durationSec: Double
    ): String?
    private external fun simklMatchEpisodeJsonNative(episodesJson: String, targetJson: String): String?
    private external fun simklWatchingToItemsJsonNative(showsJson: String, moviesJson: String): String?
    private external fun simklWatchlistToItemsJsonNative(showsJson: String, moviesJson: String): String?
    private external fun simklWatchedToIdsJsonNative(showsJson: String, moviesJson: String): String?
    private external fun playbackProgressItemJsonNative(metaJson: String, timeOffset: Long, duration: Long, nowUtc: String): String?
    private external fun clearPlaybackProgressItemJsonNative(metaJson: String): String?
    private external fun watchedStateItemsJsonNative(
        metaJson: String,
        episodesJson: String,
        watched: Boolean,
        watchedAt: String
    ): String
    private external fun libraryContinueWatchingItemsJsonNative(itemsJson: String): String
    private external fun filterHomeContinueWatchingJsonNative(itemsJson: String, traktWatchedJson: String): String
    private external fun watchedVideoIdsJsonNative(itemsJson: String, imdbId: String): String

    // ── calendar_plan ─────────────────────────────────────────────────────────
    private external fun calendarContentPlanJsonNative(requestJson: String): String
    private external fun calendarSeasonCandidatesJsonNative(requestJson: String): String
    private external fun calendarWidgetRowsJsonNative(requestJson: String): String
    private external fun calendarNotificationContentJsonNative(requestJson: String): String
    private external fun calendarReleaseDetectionJsonNative(requestJson: String): String

    // ── profile_contract ──────────────────────────────────────────────────────
    private external fun activeProfilePlanJsonNative(requestJson: String): String
    private external fun tokenMergePlanJsonNative(requestJson: String): String
    private external fun profileDefaultSeedJsonNative(requestJson: String): String
    private external fun profileSettingsMigrationPlanJsonNative(requestJson: String): String
    private external fun profileAvatarDefaultJsonNative(requestJson: String): String

    // ── watchlist_plan ────────────────────────────────────────────────────────
    private external fun watchlistTogglePlanJsonNative(requestJson: String): String
    private external fun libraryExternalMergePlanJsonNative(requestJson: String): String
    private external fun libraryCollectionImportValidationJsonNative(requestJson: String): String
    private external fun libraryOfflineGroupingJsonNative(requestJson: String): String
    private external fun playbackProgressMergePlanJsonNative(requestJson: String): String

    // ── player_policy ─────────────────────────────────────────────────────────
    private external fun playerBackendSelectionJsonNative(requestJson: String): String
    private external fun torrentFallbackFilePolicyJsonNative(requestJson: String): String
    private external fun playerBufferTargetsJsonNative(requestJson: String): String
    private external fun playerRetryPolicyJsonNative(requestJson: String): String
    private external fun playerSourceSidebarPlanJsonNative(requestJson: String): String

    // ── search_plan ───────────────────────────────────────────────────────────
    private external fun searchResultGroupingJsonNative(requestJson: String): String
    private external fun discoverSortPlanJsonNative(requestJson: String): String
    private external fun librarySortPlanJsonNative(requestJson: String): String
    private external fun detailSeriesLookupIdNative(rawId: String): String
    private external fun detailSeasonLoadPlanJsonNative(requestJson: String): String
}
