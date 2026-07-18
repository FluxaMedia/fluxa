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

private fun urlArgs(url: String): String = JsonObject().apply { addProperty("url", url) }.toString()

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

    fun coreCapabilities(portable: Boolean = false): NativeCoreCapabilitySet {
        val args = JsonObject().apply { addProperty("portable", portable) }
        val value = FluxaCoreUniFfi.coreInvokeValue("coreCapabilities", args.toString())
        return gson.fromJson(value, NativeCoreCapabilitySet::class.java) ?: NativeCoreCapabilitySet()
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

    fun normalizeManifestUrl(rawUrl: String): String =
        FluxaCoreUniFfi.coreInvokeValue("normalizeManifestUrl", urlArgs(rawUrl)).asString

    fun identity(rawUrl: String): String =
        FluxaCoreUniFfi.coreInvokeValue("identity", urlArgs(rawUrl)).asString

    fun manifestCandidates(rawUrl: String): List<String> {
        val value = FluxaCoreUniFfi.coreInvokeValue("manifestCandidates", urlArgs(rawUrl))
        return gson.fromJson(value, stringListType) ?: emptyList()
    }

    fun manifestFetchPlan(rawUrl: String): NativeManifestFetchPlan? {
        val value = FluxaCoreUniFfi.coreInvokeValue("manifestFetchPlan", urlArgs(rawUrl))
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, NativeManifestFetchPlan::class.java) }
    }

    fun baseUrl(rawUrl: String): String =
        FluxaCoreUniFfi.coreInvokeValue("baseUrl", urlArgs(rawUrl)).asString

    fun preferHttpsAssetUrl(rawUrl: String): String? {
        val value = FluxaCoreUniFfi.coreInvokeValue("preferHttpsAssetUrl", urlArgs(rawUrl))
        return value.takeUnless { it.isJsonNull }?.asString?.takeIf { it.isNotBlank() }
    }

    fun addonStoreInputType(text: String): String =
        FluxaCoreUniFfi.coreInvokeValue(
            "addonStoreInputType",
            JsonObject().apply { addProperty("input", text) }.toString()
        ).asString

    fun normalizeCloudstreamRepoUrl(rawUrl: String): String =
        FluxaCoreUniFfi.coreInvokeValue("normalizeCloudstreamRepoUrl", urlArgs(rawUrl)).asString

    fun normalizePluginRepositoryUrl(rawUrl: String): String =
        FluxaCoreUniFfi.coreInvokeValue("normalizePluginRepositoryUrl", urlArgs(rawUrl)).asString

    fun pluginIsSecureRemoteUrl(url: String): Boolean =
        FluxaCoreUniFfi.coreInvokeValue("isSecureRemoteUrl", urlArgs(url)).asBoolean

    fun pluginSameRepositoryUrl(left: String, right: String): Boolean =
        FluxaCoreUniFfi.coreInvokeValue(
            "samePluginRepositoryUrl",
            JsonObject().apply {
                addProperty("left", left)
                addProperty("right", right)
            }.toString()
        ).asBoolean

    fun extractAddonManifestUrl(detailText: String): String? {
        val value = FluxaCoreUniFfi.coreInvokeValue(
            "extractAddonManifestUrl",
            JsonObject().apply { addProperty("text", detailText) }.toString()
        )
        return value.takeUnless { it.isJsonNull }?.asString?.takeIf { it.isNotBlank() }
    }

    fun addonStoreSearchPolicy(
        query: String,
        nowMillis: Long,
        cachedAtMillis: Long?,
        ttlMillis: Long
    ): NativeAddonStoreSearchPolicy {
        val request = NativeAddonStoreSearchPolicyRequest(
            query = query,
            nowMillis = nowMillis,
            cachedAtMillis = cachedAtMillis,
            ttlMillis = ttlMillis
        )
        val value = FluxaCoreUniFfi.coreInvokeValue("addonStoreSearchPolicy", gson.toJson(request))
        return gson.fromJson(value, NativeAddonStoreSearchPolicy::class.java) ?: NativeAddonStoreSearchPolicy()
    }

    fun repositoryMetaDetailPlan(
        useConfiguredAddons: Boolean,
        authKey: String?,
        localAddons: List<String>?
    ): NativeRepositoryMetaDetailPlan {
        val request = NativeRepositoryMetaDetailPlanRequest(
            useConfiguredAddons = useConfiguredAddons,
            authKey = authKey.orEmpty(),
            localAddons = localAddons.orEmpty()
        )
        val value = FluxaCoreUniFfi.coreInvokeValue("repositoryMetaDetailPlan", gson.toJson(request))
        return gson.fromJson(value, NativeRepositoryMetaDetailPlan::class.java) ?: NativeRepositoryMetaDetailPlan()
    }

    fun repositorySeasonVideos(metaDetail: MetaDetail?, seasonNumber: Int): List<Video> {
        val args = JsonObject().apply {
            addProperty("metaDetailJson", gson.toJson(metaDetail))
            addProperty("seasonNumber", seasonNumber)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("repositorySeasonVideos", args.toString())
        return gson.fromJson(value, object : TypeToken<List<Video>>() {}.type) ?: emptyList()
    }

    fun manifestFetchDecision(
        forceRefresh: Boolean,
        memoryHit: Boolean,
        persistentHit: Boolean
    ): NativeManifestFetchDecision {
        val request = NativeManifestFetchDecisionRequest(
            forceRefresh = forceRefresh,
            memoryHit = memoryHit,
            persistentHit = persistentHit
        )
        val value = FluxaCoreUniFfi.coreInvokeValue("manifestFetchDecision", gson.toJson(request))
        return gson.fromJson(value, NativeManifestFetchDecision::class.java) ?: NativeManifestFetchDecision()
    }

    fun addonResourceRequestPlan(
        transportUrl: String,
        resource: String,
        type: String,
        id: String,
        extraArgs: Map<String, String?> = emptyMap(),
        extraRaw: String = ""
    ): NativeAddonResourceRequestPlan {
        val request = NativeAddonResourceRequestPlanRequest(
            transportUrl = transportUrl,
            resource = resource,
            contentType = type,
            id = id,
            extraArgs = extraArgs,
            extraRaw = extraRaw
        )
        val value = FluxaCoreUniFfi.coreInvokeValue("addonResourceRequestPlan", gson.toJson(request))
        return gson.fromJson(value, NativeAddonResourceRequestPlan::class.java) ?: NativeAddonResourceRequestPlan()
    }

    fun addonStreamsWithProvider(streamsJson: String, addonName: String): String {
        val args = JsonObject().apply {
            addProperty("streamsJson", streamsJson)
            addProperty("addonName", addonName)
        }
        return FluxaCoreUniFfi.coreInvokeValue("addonStreamsWithProvider", args.toString()).toString()
    }

    fun buildResourceUrl(
        transportUrl: String,
        resource: String,
        type: String,
        id: String,
        extraArgs: Map<String, String?>
    ): String {
        val args = JsonObject().apply {
            addProperty("transportUrl", transportUrl)
            addProperty("resource", resource)
            addProperty("contentType", type)
            addProperty("id", id)
            addProperty("extraJson", gson.toJson(extraArgs.filterValues { !it.isNullOrBlank() }))
        }
        return FluxaCoreUniFfi.coreInvokeValue("buildResourceUrl", args.toString()).asString
    }

    fun parseManifestJson(
        body: String,
        transportUrl: String,
        unknownName: String
    ): AddonDescriptor? {
        val args = JsonObject().apply {
            addProperty("body", body)
            addProperty("transportUrl", transportUrl)
            addProperty("unknownName", unknownName)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("parseManifest", args.toString())
        if (value.isJsonNull) return null
        return gson.fromJson(value, AddonDescriptor::class.java)
    }

    fun resolveManifestAssets(descriptor: AddonDescriptor): AddonDescriptor? {
        val value = FluxaCoreUniFfi.coreInvokeValue("resolveManifestAssets", gson.toJson(descriptor))
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, AddonDescriptor::class.java) }
    }

    fun mergeLiveManifest(
        descriptor: AddonDescriptor,
        live: AddonDescriptor?,
        unknownName: String
    ): AddonDescriptor? {
        val args = JsonObject().apply {
            addProperty("descriptor", gson.toJson(descriptor))
            live?.let { addProperty("live", gson.toJson(it)) }
            addProperty("unknownName", unknownName)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("mergeLiveManifest", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, AddonDescriptor::class.java) }
    }

    fun supportsResource(
        manifest: AddonManifest,
        resourceName: String,
        type: String?,
        id: String?
    ): Boolean {
        val args = JsonObject().apply {
            addProperty("manifest", gson.toJson(manifest))
            addProperty("resource", resourceName)
            type?.takeIf { it.isNotEmpty() }?.let { addProperty("contentType", it) }
            id?.takeIf { it.isNotEmpty() }?.let { addProperty("id", it) }
        }
        return FluxaCoreUniFfi.coreInvokeValue("supportsResource", args.toString()).asBoolean
    }

    fun catalogSupportsExtra(catalog: AddonCatalog, extraName: String): Boolean {
        val args = JsonObject().apply {
            addProperty("catalog", gson.toJson(catalog))
            addProperty("extraName", extraName)
        }
        return FluxaCoreUniFfi.coreInvokeValue("catalogSupportsExtra", args.toString()).asBoolean
    }

    fun catalogRequiresExtra(catalog: AddonCatalog, extraName: String): Boolean {
        val args = JsonObject().apply {
            addProperty("catalog", gson.toJson(catalog))
            addProperty("extraName", extraName)
        }
        return FluxaCoreUniFfi.coreInvokeValue("catalogRequiresExtra", args.toString()).asBoolean
    }

    fun catalogHasRequiredExtraExcept(catalog: AddonCatalog, allowedNames: Set<String>): Boolean {
        val args = JsonObject().apply {
            addProperty("catalog", gson.toJson(catalog))
            addProperty("allowedNames", gson.toJson(allowedNames))
        }
        return FluxaCoreUniFfi.coreInvokeValue("catalogHasRequiredExtraExcept", args.toString()).asBoolean
    }

    fun parseEpisodeLocator(raw: String?): NativeEpisodeLocator? {
        val args = JsonObject().apply { addProperty("input", raw.orEmpty()) }
        val value = FluxaCoreUniFfi.coreInvokeValue("parseEpisodeLocator", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, NativeEpisodeLocator::class.java) }
    }

    fun streamRequestIds(
        type: String,
        id: String,
        detailId: String?,
        currentSeriesLookupId: String?,
        canonicalBaseId: String?
    ): List<String> {
        val args = JsonObject().apply {
            addProperty("contentType", type)
            addProperty("id", id)
            detailId?.takeIf { it.isNotEmpty() }?.let { addProperty("detailId", it) }
            currentSeriesLookupId?.takeIf { it.isNotEmpty() }?.let { addProperty("currentSeriesLookupId", it) }
            canonicalBaseId?.takeIf { it.isNotEmpty() }?.let { addProperty("canonicalBaseId", it) }
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("streamRequestIds", args.toString())
        return gson.fromJson(value, stringListType) ?: emptyList()
    }

    fun playbackStreamRequestIds(type: String, id: String, detailId: String?): List<String> {
        val args = JsonObject().apply {
            addProperty("contentType", type)
            addProperty("id", id)
            detailId?.takeIf { it.isNotEmpty() }?.let { addProperty("detailId", it) }
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("playbackStreamRequestIds", args.toString())
        return gson.fromJson(value, stringListType) ?: emptyList()
    }

    fun playbackIntroLookupContentId(id: String): String {
        val args = JsonObject().apply { addProperty("id", id) }
        return FluxaCoreUniFfi.coreInvokeValue("playbackIntroLookupContentId", args.toString()).asString
    }

    fun directPlaybackPlan(meta: Meta, detail: MetaDetail?, todayIso: String): NativeDirectPlaybackPlan {
        val args = JsonObject().apply {
            addProperty("metaJson", gson.toJson(meta))
            detail?.let { addProperty("detailJson", gson.toJson(it)) }
            addProperty("todayIso", todayIso)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("directPlaybackPlan", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, NativeDirectPlaybackPlan::class.java) }
            ?: NativeDirectPlaybackPlan(meta = meta, lookupId = meta.id)
    }

    fun headlessProviderAvailability(
        addons: List<AddonDescriptor>,
        pluginNames: List<String>
    ): NativeProviderAvailabilityPlan {
        val request = NativeProviderAvailabilityPlanRequest(addons, pluginNames)
        val value = FluxaCoreUniFfi.coreInvokeValue("providerAvailabilityPlan", gson.toJson(request))
        return gson.fromJson(value, NativeProviderAvailabilityPlan::class.java) ?: NativeProviderAvailabilityPlan()
    }

    fun headlessDetailStreamResult(
        attempts: List<Pair<String, List<Stream>>>,
        hasStreamProviders: Boolean
    ): NativeDetailStreamResultPlan {
        val request = NativeDetailStreamResultPlanRequest(
            attempts = attempts.map { (requestId, streams) ->
                NativeDetailStreamAttemptRequest(requestId, streams)
            },
            hasStreamProviders = hasStreamProviders
        )
        val value = FluxaCoreUniFfi.coreInvokeValue("detailStreamResultPlan", gson.toJson(request))
        return gson.fromJson(value, NativeDetailStreamResultPlan::class.java)
            ?: NativeDetailStreamResultPlan(hasStreamProviders = hasStreamProviders)
    }

    fun headlessPrefetchDetailStreams(streams: List<Stream>): NativePrefetchDetailStreamsPlan {
        val request = NativePrefetchDetailStreamsPlanRequest(streams)
        val value = FluxaCoreUniFfi.coreInvokeValue("prefetchDetailStreamsPlan", gson.toJson(request))
        return gson.fromJson(value, NativePrefetchDetailStreamsPlan::class.java)
            ?: NativePrefetchDetailStreamsPlan(count = streams.size)
    }

    fun headlessDirectPlaybackPolicy(): NativeDirectPlaybackPolicy {
        val value = FluxaCoreUniFfi.coreInvokeValue("directPlaybackPolicy", "{}")
        return gson.fromJson(value, NativeDirectPlaybackPolicy::class.java) ?: NativeDirectPlaybackPolicy()
    }

    fun streamDiscoveryEpisodeContext(
        type: String,
        requestId: String,
        detail: MetaDetail?,
        seasonEpisodes: List<Video>
    ): NativeStreamDiscoveryEpisodeContext {
        val args = JsonObject().apply {
            addProperty("contentType", type)
            addProperty("requestId", requestId)
            detail?.let { addProperty("detailJson", gson.toJson(it)) }
            addProperty("seasonEpisodesJson", gson.toJson(seasonEpisodes))
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("streamDiscoveryEpisodeContext", args.toString())
        return gson.fromJson(value, NativeStreamDiscoveryEpisodeContext::class.java)
            ?: NativeStreamDiscoveryEpisodeContext()
    }

    fun streamPlaybackInfo(stream: Stream): NativeStreamPlaybackInfo {
        val value = FluxaCoreUniFfi.coreInvokeValue("streamPlaybackInfo", gson.toJson(stream))
        return gson.fromJson(value, NativeStreamPlaybackInfo::class.java) ?: NativeStreamPlaybackInfo()
    }

    fun dvProxyPlan(
        streamJson: String,
        url: String,
        fallbackMode: String,
        deviceHasDvDecoder: Boolean,
        deviceHasDvDisplay: Boolean
    ): NativeDvProxyPlan {
        val requestJson = gson.toJson(mapOf(
            "stream" to gson.fromJson(streamJson, Any::class.java),
            "url" to url,
            "fallbackMode" to fallbackMode,
            "deviceHasDvDecoder" to deviceHasDvDecoder,
            "deviceHasDvDisplay" to deviceHasDvDisplay
        ))
        val value = FluxaCoreUniFfi.coreInvokeValue("dvProxyPlan", requestJson)
        return gson.fromJson(value, NativeDvProxyPlan::class.java) ?: NativeDvProxyPlan()
    }

    fun dolbyVisionRpuInfo(rpuBase64: String): NativeDolbyVisionRpuInfo {
        val value = FluxaCoreUniFfi.coreInvokeValue("dolbyVisionRpuInfo", gson.toJson(mapOf("rpu_base64" to rpuBase64)))
        return value.takeUnless { it.isJsonNull }?.asJsonObject?.toDolbyVisionRpuInfo() ?: NativeDolbyVisionRpuInfo()
    }

    fun dolbyVisionConvertRpu(rpuBase64: String, mode: Int = 2): NativeDolbyVisionRpuConvertResult {
        val value = FluxaCoreUniFfi.coreInvokeValue(
            "dolbyVisionConvertRpu",
            gson.toJson(mapOf("rpu_base64" to rpuBase64, "mode" to mode))
        )
        return value.takeUnless { it.isJsonNull }?.asJsonObject?.toDolbyVisionRpuConvertResult()
            ?: NativeDolbyVisionRpuConvertResult()
    }

    fun streamRequestHeaders(streamHeaders: Map<String, String>): Map<String, String> {
        val args = JsonObject().apply { addProperty("headersJson", gson.toJson(streamHeaders)) }
        val value = FluxaCoreUniFfi.coreInvokeValue("streamRequestHeaders", args.toString())
        return gson.fromJson(value, object : TypeToken<Map<String, String>>() {}.type) ?: emptyMap()
    }

    fun streamRequestReferer(url: String): String? {
        val value = FluxaCoreUniFfi.coreInvokeValue("streamRequestReferer", urlArgs(url))
        return value.takeUnless { it.isJsonNull }?.asString?.takeIf { it.isNotBlank() }
    }

    fun isTorrentPlaybackUrl(url: String?): Boolean {
        val stream = Stream(name = null, title = null, url = url)
        return streamPlaybackInfo(stream).isTorrentPlaybackUrl
    }

    fun episodeTextMatches(text: String?, season: Int, episode: Int): Boolean {
        val args = JsonObject().apply {
            addProperty("text", text.orEmpty())
            addProperty("season", season)
            addProperty("episode", episode)
        }
        return FluxaCoreUniFfi.coreInvokeValue("episodeTextMatches", args.toString()).asBoolean
    }

    fun streamMatchesEpisode(
        videoId: String?,
        title: String?,
        name: String?,
        description: String?,
        filename: String?,
        effectiveFilename: String?
    ): Boolean {
        val args = JsonObject().apply {
            addProperty("videoId", videoId.orEmpty())
            addProperty("title", title.orEmpty())
            addProperty("name", name.orEmpty())
            addProperty("description", description.orEmpty())
            addProperty("filename", filename.orEmpty())
            addProperty("effectiveFilename", effectiveFilename.orEmpty())
        }
        return FluxaCoreUniFfi.coreInvokeValue("streamMatchesEpisode", args.toString()).asBoolean
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
    ): Int {
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
        val args = JsonObject().apply {
            addProperty("streamsJson", gson.toJson(nativeStreams))
            addProperty("currentVideoId", currentVideoId.orEmpty())
            addProperty("initialStreamIndex", initialStreamIndex)
            savedUrl?.takeIf { it.isNotEmpty() }?.let { addProperty("savedUrl", it) }
            savedTitle?.takeIf { it.isNotEmpty() }?.let { addProperty("savedTitle", it) }
            addProperty("sourceSelectionMode", sourceSelectionMode)
            regexPattern?.takeIf { it.isNotEmpty() }?.let { addProperty("regexPattern", it) }
            preferredBingeGroup?.takeIf { it.isNotEmpty() }?.let { addProperty("preferredBingeGroup", it) }
        }
        return FluxaCoreUniFfi.coreInvokeValue("selectStreamIndex", args.toString()).asInt
    }

    fun mergeContinueWatchingDuplicates(items: List<Meta>): List<Meta> {
        val args = JsonObject().apply { addProperty("itemsJson", gson.toJson(items)) }
        val value = FluxaCoreUniFfi.coreInvokeValue("mergeContinueWatchingDuplicates", args.toString())
        return gson.fromJson(value, metaListType) ?: emptyList()
    }

    fun filterDiscoverResults(
        items: List<Meta>,
        year: String?,
        rating: Float?,
        region: String?
    ): List<Meta> {
        val args = JsonObject().apply {
            addProperty("itemsJson", gson.toJson(items))
            year?.takeIf { it.isNotBlank() }?.let { addProperty("year", it) }
            rating?.let { addProperty("rating", it) }
            region?.takeIf { it.isNotBlank() }?.let { addProperty("region", it) }
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("filterDiscoverResults", args.toString())
        return gson.fromJson(value, metaListType) ?: emptyList()
    }

    fun resolvePreferredAudioLanguage(
        lastAudioLanguage: String?,
        preferredAudioLanguage: String?,
        originalLanguage: String?
    ): String {
        val args = JsonObject().apply {
            lastAudioLanguage?.let { addProperty("lastAudioLanguage", it) }
            preferredAudioLanguage?.let { addProperty("preferredAudioLanguage", it) }
            originalLanguage?.let { addProperty("originalLanguage", it) }
        }
        return FluxaCoreUniFfi.coreInvokeValue("resolvePreferredAudioLanguage", args.toString()).asString
    }

    fun subtitleLanguageMatches(label: String, language: String?, preferredLanguage: String): Boolean {
        val args = JsonObject().apply {
            addProperty("label", label)
            language?.let { addProperty("language", it) }
            addProperty("preferredLanguage", preferredLanguage)
        }
        return FluxaCoreUniFfi.coreInvokeValue("subtitleLanguageMatches", args.toString()).asBoolean
    }

    fun playerTrackState(
        availableSubtitles: List<SubtitleTrackRef>,
        lastAudioLanguage: String?,
        preferredAudioLanguage: String?,
        originalLanguage: String?,
        lastSubtitleLanguage: String?,
        preferredSubtitleLanguage: String?,
        secondarySubtitleLanguage: String?
    ): NativePlayerTrackState {
        val request = NativePlayerTrackStateRequest(
            availableSubtitles = availableSubtitles.map { NativeSubtitleTrackEntry(it.id, it.label, it.language) },
            lastAudioLanguage = lastAudioLanguage,
            preferredAudioLanguage = preferredAudioLanguage,
            originalLanguage = originalLanguage,
            lastSubtitleLanguage = lastSubtitleLanguage,
            preferredSubtitleLanguage = preferredSubtitleLanguage,
            secondarySubtitleLanguage = secondarySubtitleLanguage
        )
        val value = FluxaCoreUniFfi.coreInvokeValue("playerTrackState", gson.toJson(request))
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, NativePlayerTrackState::class.java) }
            ?: NativePlayerTrackState()
    }

    fun parseAddonResourceResult(
        resource: String,
        url: String,
        statusCode: Int,
        body: String?
    ): NativeAddonResourceParseResult {
        val args = JsonObject().apply {
            addProperty("resource", resource)
            addProperty("url", url)
            addProperty("statusCode", statusCode)
            body?.let { addProperty("body", it) }
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("parseAddonResourceResult", args.toString())
        return gson.fromJson(value, NativeAddonResourceParseResult::class.java) ?: NativeAddonResourceParseResult(
            kind = "parse_error",
            url = url,
            statusCode = statusCode,
            error = "empty native response"
        )
    }

    fun parseAddonStreamResult(
        url: String,
        statusCode: Int,
        body: String?,
        addonName: String
    ): NativeAddonResourceParseResult {
        val args = JsonObject().apply {
            addProperty("url", url)
            addProperty("statusCode", statusCode)
            body?.let { addProperty("body", it) }
            addProperty("addonName", addonName)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("parseAddonStreamResult", args.toString())
        return gson.fromJson(value, NativeAddonResourceParseResult::class.java) ?: NativeAddonResourceParseResult(
            kind = "parse_error",
            url = url,
            statusCode = statusCode,
            error = "empty native response"
        )
    }

    fun normalizeAddonSubtitles(subtitlesJson: String, resourceUrl: String): String {
        val args = JsonObject().apply {
            addProperty("subtitles", subtitlesJson)
            addProperty("resourceUrl", resourceUrl)
        }
        return FluxaCoreUniFfi.coreInvokeValue("normalizeAddonSubtitles", args.toString()).toString()
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
    ): NativeTorrentRuntimeInfo {
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
        val value = FluxaCoreUniFfi.coreInvokeValue("torrentRuntimeInfo", gson.toJson(request))
        return gson.fromJson(value, NativeTorrentRuntimeInfo::class.java) ?: NativeTorrentRuntimeInfo()
    }

    fun torrentStatusInfo(status: TorrentStatus): NativeTorrentStatusInfo {
        val value = FluxaCoreUniFfi.coreInvokeValue("torrentStatusInfo", gson.toJson(status))
        return gson.fromJson(value, NativeTorrentStatusInfo::class.java) ?: NativeTorrentStatusInfo()
    }

    fun stableFeedPart(value: String): String {
        val args = JsonObject().apply { addProperty("value", value) }
        return FluxaCoreUniFfi.coreInvokeValue("stableFeedPart", args.toString()).asString
    }

    fun normalizeContentType(value: String): String? {
        val args = JsonObject().apply { addProperty("value", value) }
        val result = FluxaCoreUniFfi.coreInvokeValue("normalizeContentType", args.toString())
        return result.takeUnless { it.isJsonNull }?.asString?.takeIf { it.isNotBlank() }
    }

    fun parseExtraArgs(extra: String): Map<String, String> {
        val args = JsonObject().apply { addProperty("extra", extra) }
        val value = FluxaCoreUniFfi.coreInvokeValue("parseExtraArgs", args.toString())
        return gson.fromJson(value, object : TypeToken<Map<String, String>>() {}.type) ?: emptyMap()
    }

    fun providerSearchTerms(provider: String): List<String> {
        val args = JsonObject().apply { addProperty("provider", provider) }
        val value = FluxaCoreUniFfi.coreInvokeValue("providerSearchTerms", args.toString())
        return gson.fromJson(value, stringListType) ?: emptyList()
    }

    fun effectiveMetadataFeedSelection(selectedKeys: List<String>?, availableKeys: List<String>): List<String>? {
        val args = JsonObject().apply {
            addProperty("selectedKeys", gson.toJson(selectedKeys))
            addProperty("availableKeys", gson.toJson(availableKeys))
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("effectiveMetadataFeedSelection", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson<List<String>>(it, stringListType) }
    }

    fun toggleMetadataFeed(selectedKeys: List<String>?, availableKeys: List<String>, key: String): List<String> {
        val args = JsonObject().apply {
            addProperty("selectedKeys", gson.toJson(selectedKeys))
            addProperty("availableKeys", gson.toJson(availableKeys))
            addProperty("key", key)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("toggleMetadataFeed", args.toString())
        return gson.fromJson(value, stringListType) ?: emptyList()
    }

    fun toggleMetadataFeed(
        selectedKeys: List<String>?,
        availableKeys: List<String>,
        key: String,
        maxEnabled: Int
    ): List<String> {
        val args = JsonObject().apply {
            addProperty("selectedKeys", gson.toJson(selectedKeys))
            addProperty("availableKeys", gson.toJson(availableKeys))
            addProperty("key", key)
            addProperty("maxEnabled", maxEnabled)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("toggleMetadataFeedLimited", args.toString())
        return gson.fromJson(value, stringListType) ?: emptyList()
    }

    fun setMetadataFeedGroupEnabled(
        selectedKeys: List<String>?,
        availableKeys: List<String>,
        groupKeys: List<String>,
        enabled: Boolean
    ): List<String> {
        val args = JsonObject().apply {
            addProperty("selectedKeys", gson.toJson(selectedKeys))
            addProperty("availableKeys", gson.toJson(availableKeys))
            addProperty("groupKeys", gson.toJson(groupKeys))
            addProperty("enabled", enabled)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("setMetadataFeedGroupEnabled", args.toString())
        return gson.fromJson(value, stringListType) ?: emptyList()
    }

    fun orderedMetadataFeedKeys(optionKeys: List<String>, order: List<String>?): List<String> {
        val args = JsonObject().apply {
            addProperty("optionKeys", gson.toJson(optionKeys))
            addProperty("order", gson.toJson(order))
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("orderedMetadataFeedKeys", args.toString())
        return gson.fromJson(value, stringListType) ?: optionKeys
    }

    fun moveMetadataFeedOrder(optionKeys: List<String>, currentOrder: List<String>?, key: String, delta: Int): List<String> {
        val args = JsonObject().apply {
            addProperty("optionKeys", gson.toJson(optionKeys))
            addProperty("currentOrder", gson.toJson(currentOrder))
            addProperty("key", key)
            addProperty("delta", delta)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("moveMetadataFeedOrder", args.toString())
        return gson.fromJson(value, stringListType) ?: optionKeys
    }

    fun contentTraktKey(meta: Meta): String = contentTraktKeysBatch(listOf(meta)).first()

    fun contentTraktKeysBatch(metas: List<Meta>): List<String> {
        if (metas.isEmpty()) return emptyList()
        val args = JsonObject().apply { addProperty("metasJson", gson.toJson(metas)) }
        val value = FluxaCoreUniFfi.coreInvokeValue("contentTraktKeysBatch", args.toString())
        return gson.fromJson(value, stringListType) ?: emptyList()
    }

    fun contentMergeKeys(meta: Meta): Set<String> {
        val args = JsonObject().apply { addProperty("metaJson", gson.toJson(meta)) }
        val value = FluxaCoreUniFfi.coreInvokeValue("contentMergeKeys", args.toString())
        return (gson.fromJson<List<String>>(value, stringListType) ?: emptyList()).toSet()
    }

    fun contentWatchedKeysBatch(metas: List<Meta>): List<Set<String>> {
        if (metas.isEmpty()) return emptyList()
        val args = JsonObject().apply { addProperty("metasJson", gson.toJson(metas)) }
        val value = FluxaCoreUniFfi.coreInvokeValue("contentWatchedKeysBatch", args.toString())
        return (gson.fromJson<List<List<String>>>(value, stringListListType) ?: emptyList()).map { it.toSet() }
    }

    fun episodeFilenameCandidate(stream: Stream, videoId: String?): String? {
        val args = JsonObject().apply {
            addProperty("streamJson", gson.toJson(stream))
            addProperty("videoId", videoId.orEmpty())
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("episodeFilenameCandidate", args.toString())
        return value.takeUnless { it.isJsonNull }?.asString?.takeIf { it.isNotBlank() }
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
    ): String {
        val request = NativeStreamDiscoveryCacheKeyRequest(
            type = type,
            id = id,
            language = language,
            cs3SearchQuery = cs3SearchQuery,
            cs3Year = cs3Year,
            cs3OriginalName = cs3OriginalName,
            addonSignatures = addonSignatures,
            cs3PluginNames = cs3PluginNames
        )
        return FluxaCoreUniFfi.coreInvokeValue("streamDiscoveryCacheKey", gson.toJson(request)).asString
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
    ): NativeStreamDiscoveryPlan {
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
        val value = FluxaCoreUniFfi.coreInvokeValue("streamDiscoveryPlan", gson.toJson(request))
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, NativeStreamDiscoveryPlan::class.java) }
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
    ): NativeStreamDiscoveryExecutionPolicy {
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
        val value = FluxaCoreUniFfi.coreInvokeValue("streamDiscoveryExecutionPolicy", gson.toJson(request))
        return value.takeUnless { it.isJsonNull }
            ?.let { gson.fromJson(it, NativeStreamDiscoveryExecutionPolicy::class.java) }
            ?: NativeStreamDiscoveryExecutionPolicy()
    }

    fun streamDiscoveryCachePrefix(type: String, id: String, language: String): String {
        val args = JsonObject().apply {
            addProperty("contentType", type)
            addProperty("id", id)
            addProperty("language", language)
        }
        return FluxaCoreUniFfi.coreInvokeValue("streamDiscoveryCachePrefix", args.toString()).asString
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
    ): String {
        val request = NativeDiscoverCatalogCacheKeyRequest(
            type = type,
            catalogKey = catalogKey,
            genre = genre,
            year = year,
            rating = rating,
            provider = provider,
            region = region,
            catalogSignatures = catalogSignatures
        )
        return FluxaCoreUniFfi.coreInvokeValue("discoverCatalogCacheKey", gson.toJson(request)).asString
    }

    fun curateHomeItemsJson(categoryJson: String): String {
        val args = JsonObject().apply { addProperty("categoryJson", categoryJson) }
        return FluxaCoreUniFfi.coreInvokeValue("curateHomeItems", args.toString()).toString()
    }

    fun homeOverlapRatioJson(firstCategoryJson: String, secondCategoryJson: String): Float {
        val args = JsonObject().apply {
            addProperty("firstJson", firstCategoryJson)
            addProperty("secondJson", secondCategoryJson)
        }
        return FluxaCoreUniFfi.coreInvokeValue("homeOverlapRatio", args.toString()).asFloat
    }

    fun homePersonalizationScoreJson(
        categoryJson: String,
        preferredGenresJson: String,
        preferredTypesJson: String,
        priorityLabelsJson: String
    ): Int {
        val args = JsonObject().apply {
            addProperty("categoryJson", categoryJson)
            addProperty("preferredGenresJson", preferredGenresJson)
            addProperty("preferredTypesJson", preferredTypesJson)
            addProperty("priorityLabelsJson", priorityLabelsJson)
        }
        return FluxaCoreUniFfi.coreInvokeValue("homePersonalizationScore", args.toString()).asInt
    }

    fun prioritizeHomeRowsJson(
        categoriesJson: String,
        preferredOrderLabelsJson: String,
        preferredGenresJson: String,
        preferredTypesJson: String,
        priorityLabelsJson: String
    ): String {
        val args = JsonObject().apply {
            addProperty("categoriesJson", categoriesJson)
            addProperty("preferredOrderLabelsJson", preferredOrderLabelsJson)
            addProperty("preferredGenresJson", preferredGenresJson)
            addProperty("preferredTypesJson", preferredTypesJson)
            addProperty("priorityLabelsJson", priorityLabelsJson)
        }
        return FluxaCoreUniFfi.coreInvokeValue("prioritizeHomeRows", args.toString()).toString()
    }

    fun optimizeHomeRowsJson(requestJson: String): String =
        FluxaCoreUniFfi.coreInvokeValue("optimizeHomeRows", requestJson).toString()

    fun buildBillboardPool(enriched: List<Meta>, candidates: List<Meta>): List<Meta> {
        val args = JsonObject().apply {
            addProperty("enrichedJson", gson.toJson(enriched))
            addProperty("candidatesJson", gson.toJson(candidates))
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("buildBillboardPool", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson<List<Meta>>(it, metaListType) } ?: emptyList()
    }

    fun normalizeHomeCatalogItems(items: List<Meta>, catalogId: String, genre: String?): List<Meta> {
        val todayIso = java.time.LocalDate.now(java.time.ZoneId.systemDefault()).toString()
        val args = JsonObject().apply {
            addProperty("itemsJson", gson.toJson(items))
            addProperty("catalogId", catalogId)
            genre?.takeIf { it.isNotEmpty() }?.let { addProperty("genre", it) }
            addProperty("todayIso", todayIso)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("normalizeHomeCatalogItems", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson<List<Meta>>(it, metaListType) } ?: emptyList()
    }

    fun playerProgressPercent(positionMs: Long, durationMs: Long): Float {
        val args = JsonObject().apply {
            addProperty("positionMs", positionMs)
            addProperty("durationMs", durationMs)
        }
        return FluxaCoreUniFfi.coreInvokeValue("playerProgressPercent", args.toString()).asFloat
    }

    fun playerShouldSendScrobbleStart(
        token: String?,
        isPlaying: Boolean,
        hasScrobbledStart: Boolean,
        progress: Float
    ): Boolean {
        val args = JsonObject().apply {
            token?.let { addProperty("token", it) }
            addProperty("isPlaying", isPlaying)
            addProperty("hasScrobbledStart", hasScrobbledStart)
            addProperty("progress", progress)
        }
        return FluxaCoreUniFfi.coreInvokeValue("playerShouldSendScrobbleStart", args.toString()).asBoolean
    }

    fun playerShouldMarkScrobbleStopped(hasScrobbledStop: Boolean, progress: Float): Boolean {
        val args = JsonObject().apply {
            addProperty("hasScrobbledStop", hasScrobbledStop)
            addProperty("progress", progress)
        }
        return FluxaCoreUniFfi.coreInvokeValue("playerShouldMarkScrobbleStopped", args.toString()).asBoolean
    }

    fun playerShouldQueueScrobblePause(
        token: String?,
        wasPlayWhenReady: Boolean,
        hasScrobbledStart: Boolean,
        hasScrobbledStop: Boolean
    ): Boolean {
        val args = JsonObject().apply {
            token?.let { addProperty("token", it) }
            addProperty("wasPlayWhenReady", wasPlayWhenReady)
            addProperty("hasScrobbledStart", hasScrobbledStart)
            addProperty("hasScrobbledStop", hasScrobbledStop)
        }
        return FluxaCoreUniFfi.coreInvokeValue("playerShouldQueueScrobblePause", args.toString()).asBoolean
    }

    fun playerShouldEnqueueDurableScrobble(action: String, token: String?, progress: Float): Boolean {
        val args = JsonObject().apply {
            addProperty("action", action)
            token?.let { addProperty("token", it) }
            addProperty("progress", progress)
        }
        return FluxaCoreUniFfi.coreInvokeValue("playerShouldEnqueueDurableScrobble", args.toString()).asBoolean
    }

    fun playerShouldSavePeriodicProgress(isPlaying: Boolean, nowMs: Long, lastSavedAtMs: Long): Boolean {
        val args = JsonObject().apply {
            addProperty("isPlaying", isPlaying)
            addProperty("nowMs", nowMs)
            addProperty("lastSavedAtMs", lastSavedAtMs)
        }
        return FluxaCoreUniFfi.coreInvokeValue("playerShouldSavePeriodicProgress", args.toString()).asBoolean
    }

    fun playerShouldSaveOnDispose(positionMs: Long): Boolean {
        val args = JsonObject().apply { addProperty("positionMs", positionMs) }
        return FluxaCoreUniFfi.coreInvokeValue("playerShouldSaveOnDispose", args.toString()).asBoolean
    }

    fun safePlayerBufferCacheMb(value: Int?): Int {
        val args = JsonObject().apply { value?.let { addProperty("value", it) } }
        return FluxaCoreUniFfi.coreInvokeValue("safePlayerBufferCacheMb", args.toString()).asInt
    }

    fun safeStreamSourceSelectionMode(mode: String?): String {
        val args = JsonObject().apply { mode?.takeIf { it.isNotEmpty() }?.let { addProperty("mode", it) } }
        return FluxaCoreUniFfi.coreInvokeValue("safeStreamSourceSelectionMode", args.toString()).asString
    }

    fun playerFlowDispatch(state: Any, action: Any): NativePlayerFlowResult {
        val args = JsonObject().apply {
            addProperty("state", gson.toJson(state))
            addProperty("action", gson.toJson(action))
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("playerFlowDispatch", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, NativePlayerFlowResult::class.java) }
            ?: NativePlayerFlowResult()
    }

    fun traktHasClient(apiKey: String): Boolean {
        val args = JsonObject().apply { addProperty("apiKey", apiKey) }
        return FluxaCoreUniFfi.coreInvokeValue("traktHasClient", args.toString()).asBoolean
    }

    fun traktBearer(token: String): String {
        val args = JsonObject().apply { addProperty("token", token) }
        return FluxaCoreUniFfi.coreInvokeValue("traktBearer", args.toString()).asString
    }

    fun traktScrobbleUrl(action: String): String {
        val args = JsonObject().apply { addProperty("action", action) }
        return FluxaCoreUniFfi.coreInvokeValue("traktScrobbleUrl", args.toString()).asString
    }

    fun traktPlaybackUrl(type: String?): String {
        val args = JsonObject().apply { type?.takeIf { it.isNotBlank() }?.let { addProperty("contentType", it) } }
        return FluxaCoreUniFfi.coreInvokeValue("traktPlaybackUrl", args.toString()).asString
    }

    fun traktTokenExpiresAt(createdAtSeconds: Long, expiresInSeconds: Long): Long {
        val args = JsonObject().apply {
            addProperty("createdAtSeconds", createdAtSeconds)
            addProperty("expiresInSeconds", expiresInSeconds)
        }
        return FluxaCoreUniFfi.coreInvokeValue("traktTokenExpiresAt", args.toString()).asLong
    }

    fun traktContentIdFrom(ids: TraktIds): String? {
        val args = JsonObject().apply { addProperty("idsJson", gson.toJson(ids)) }
        val value = FluxaCoreUniFfi.coreInvokeValue("traktContentIdFromIds", args.toString())
        return value.takeUnless { it.isJsonNull }?.asString?.takeIf { it.isNotBlank() }
    }

    fun traktIdsFromContentId(rawId: String): TraktIds? {
        val args = JsonObject().apply { addProperty("rawId", rawId) }
        val value = FluxaCoreUniFfi.coreInvokeValue("traktIdsFromContentId", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, TraktIds::class.java) }
    }

    fun traktEpisodeLocator(videoId: String): NativeTraktEpisodeLocator? {
        val args = JsonObject().apply { addProperty("videoId", videoId) }
        val value = FluxaCoreUniFfi.coreInvokeValue("traktEpisodeLocator", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, NativeTraktEpisodeLocator::class.java) }
    }

    fun traktShowIdFromEpisodeId(videoId: String): String {
        val args = JsonObject().apply { addProperty("videoId", videoId) }
        return FluxaCoreUniFfi.coreInvokeValue("traktShowIdFromEpisodeId", args.toString()).asString
    }

    fun traktScrobbleMediaId(parentId: String, videoId: String?, mediaType: String): String {
        val args = JsonObject().apply {
            addProperty("parentId", parentId)
            videoId?.takeIf { it.isNotEmpty() }?.let { addProperty("videoId", it) }
            addProperty("mediaType", mediaType)
        }
        return FluxaCoreUniFfi.coreInvokeValue("traktScrobbleMediaId", args.toString()).asString
    }

    fun traktOAuthErrorCode(body: String): String? {
        val args = JsonObject().apply { addProperty("body", body) }
        val value = FluxaCoreUniFfi.coreInvokeValue("traktOAuthErrorCode", args.toString())
        return value.takeUnless { it.isJsonNull }?.asString?.takeIf { it.isNotBlank() }
    }

    fun traktHistoryRequest(meta: Meta, episodes: List<Video>): TraktHistorySyncRequest? {
        val args = JsonObject().apply {
            addProperty("metaJson", gson.toJson(meta))
            addProperty("episodesJson", gson.toJson(episodes))
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("traktHistoryRequest", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, TraktHistorySyncRequest::class.java) }
    }

    fun simklScrobbleBody(
        idsJson: String,
        isEpisode: Boolean,
        season: Long,
        epNumber: Long,
        timePosSec: Double,
        durationSec: Double
    ): String? {
        val args = JsonObject().apply {
            addProperty("idsJson", idsJson)
            addProperty("isEpisode", isEpisode)
            addProperty("season", season)
            addProperty("epNumber", epNumber)
            addProperty("timePosSec", timePosSec)
            addProperty("durationSec", durationSec)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("simklScrobbleBody", args.toString())
        return value.takeUnless { it.isJsonNull }?.toString()?.takeIf { it.isNotBlank() }
    }

    fun simklMatchEpisode(episodesJson: String, targetJson: String): NativeSimklEpisodeMatch? {
        val args = JsonObject().apply {
            addProperty("episodesJson", episodesJson)
            addProperty("targetJson", targetJson)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("simklMatchEpisode", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, NativeSimklEpisodeMatch::class.java) }
    }

    fun simklWatchingToItems(showsJson: String, moviesJson: String): String? {
        val args = JsonObject().apply {
            addProperty("showsJson", showsJson)
            addProperty("moviesJson", moviesJson)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("simklWatchingToItems", args.toString())
        return value.takeUnless { it.isJsonNull }?.toString()?.takeIf { it.isNotBlank() }
    }

    fun simklWatchlistToItems(showsJson: String, moviesJson: String): String? {
        val args = JsonObject().apply {
            addProperty("showsJson", showsJson)
            addProperty("moviesJson", moviesJson)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("simklWatchlistToItems", args.toString())
        return value.takeUnless { it.isJsonNull }?.toString()?.takeIf { it.isNotBlank() }
    }

    fun simklWatchedToIds(showsJson: String, moviesJson: String): String? {
        val args = JsonObject().apply {
            addProperty("showsJson", showsJson)
            addProperty("moviesJson", moviesJson)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("simklWatchedToIds", args.toString())
        return value.takeUnless { it.isJsonNull }?.toString()?.takeIf { it.isNotBlank() }
    }

    fun playbackProgressItem(meta: Meta, timeOffset: Long, duration: Long, nowUtc: String): LibraryItem? {
        val args = JsonObject().apply {
            addProperty("metaJson", gson.toJson(meta))
            addProperty("timeOffset", timeOffset)
            addProperty("duration", duration)
            addProperty("nowUtc", nowUtc)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("playbackProgressItem", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, LibraryItem::class.java) }
    }

    fun clearPlaybackProgressItem(meta: Meta): LibraryItem? {
        val args = JsonObject().apply { addProperty("metaJson", gson.toJson(meta)) }
        val value = FluxaCoreUniFfi.coreInvokeValue("clearPlaybackProgressItem", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, LibraryItem::class.java) }
    }

    fun watchedStateItems(meta: Meta, episodes: List<Video>, watched: Boolean, watchedAt: String?): List<LibraryItem> {
        val args = JsonObject().apply {
            addProperty("metaJson", gson.toJson(meta))
            addProperty("episodesJson", gson.toJson(episodes))
            addProperty("watched", watched)
            watchedAt?.takeIf { it.isNotEmpty() }?.let { addProperty("watchedAt", it) }
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("watchedStateItems", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson<List<LibraryItem>>(it, libraryItemListType) } ?: emptyList()
    }

    fun libraryContinueWatchingItems(items: List<LibraryItem>): List<Meta> {
        val value = FluxaCoreUniFfi.coreInvokeValue("libraryContinueWatchingItems", gson.toJson(items))
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson<List<Meta>>(it, metaListType) } ?: emptyList()
    }

    fun libraryWatchlistItems(items: List<LibraryItem>): com.google.gson.JsonElement {
        return FluxaCoreUniFfi.coreInvokeValue("libraryWatchlistItems", gson.toJson(items))
    }

    fun filterHomeContinueWatching(items: List<Meta>, traktWatchedState: TraktWatchedState): List<Meta> {
        val args = JsonObject().apply {
            addProperty("itemsJson", gson.toJson(items))
            addProperty("traktWatchedJson", gson.toJson(traktWatchedState))
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("filterHomeContinueWatching", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson<List<Meta>>(it, metaListType) } ?: emptyList()
    }

    fun watchedVideoIds(items: List<LibraryItem>, imdbId: String): List<String> {
        val args = JsonObject().apply {
            addProperty("itemsJson", gson.toJson(items))
            addProperty("imdbId", imdbId)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("watchedVideoIds", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson<List<String>>(it, stringListType) } ?: emptyList()
    }

    fun offlineDownloadPlan(
        meta: Meta,
        video: Video?,
        videoId: String?,
        stream: Stream,
        subtitleUrl: String?,
        downloadId: String
    ): NativeOfflineDownloadPlan {
        val request = NativeOfflineDownloadPlanRequest(
            meta = meta,
            video = video,
            videoId = videoId,
            stream = stream,
            subtitleUrl = subtitleUrl,
            downloadId = downloadId
        )
        val value = FluxaCoreUniFfi.coreInvokeValue("offlineDownloadPlan", gson.toJson(request))
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, NativeOfflineDownloadPlan::class.java) }
            ?: NativeOfflineDownloadPlan(reason = "unsupported_source")
    }

    fun profileSafePrefs(profile: Any): NativeProfileSafePrefs {
        val value = FluxaCoreUniFfi.coreInvokeValue("profileSafePrefs", gson.toJson(profile))
        return gson.fromJson(value, NativeProfileSafePrefs::class.java) ?: NativeProfileSafePrefs()
    }

    fun <T> sanitizeProfile(
        profile: T,
        mirroredAddons: Collection<String>,
        mergeMirroredAddons: Boolean,
        type: Class<T>
    ): T? {
        val args = JsonObject().apply {
            addProperty("profile", gson.toJson(profile))
            addProperty("mirroredAddons", gson.toJson(mirroredAddons))
            addProperty("mergeMirroredAddons", mergeMirroredAddons)
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("sanitizeProfile", args.toString())
        return value.takeUnless { it.isJsonNull }?.let { gson.fromJson(it, type) }
    }

    fun profileLocalAddonsKey(profile: Any): String =
        FluxaCoreUniFfi.coreInvokeValue("profileLocalAddonsKey", gson.toJson(profile)).asString

    fun cacheEntryPolicy(
        key: String,
        storedAtMillis: Long,
        ttlMillis: Long,
        nowMillis: Long
    ): NativeCacheEntryPolicy {
        val request = NativeCacheEntryPolicyRequest(
            key = key,
            storedAtMillis = storedAtMillis,
            ttlMillis = ttlMillis,
            nowMillis = nowMillis
        )
        val value = FluxaCoreUniFfi.coreInvokeValue("cacheEntryPolicy", gson.toJson(request))
        return gson.fromJson(value, NativeCacheEntryPolicy::class.java)
            ?: NativeCacheEntryPolicy(key = key, storedAtMillis = storedAtMillis)
    }

    fun cacheTrimPolicy(
        entries: List<Triple<String, Long, Long>>,
        maxEntries: Int,
        nowMillis: Long
    ): NativeCacheTrimPolicy {
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
        val value = FluxaCoreUniFfi.coreInvokeValue("cacheTrimPolicy", gson.toJson(request))
        return gson.fromJson(value, NativeCacheTrimPolicy::class.java) ?: NativeCacheTrimPolicy()
    }

    fun dataFailurePolicy(
        operation: String,
        kind: String,
        message: String? = null,
        throwableClass: String? = null,
        reason: String? = null,
        statusCode: Long? = null
    ): NativeDataFailurePolicy {
        val request = NativeDataFailurePolicyRequest(
            operation = operation,
            kind = kind,
            message = message,
            throwableClass = throwableClass,
            reason = reason,
            statusCode = statusCode
        )
        val value = FluxaCoreUniFfi.coreInvokeValue("dataFailurePolicy", gson.toJson(request))
        return gson.fromJson(value, NativeDataFailurePolicy::class.java)
            ?: NativeDataFailurePolicy(operation = operation, kind = kind, message = message ?: reason.orEmpty())
    }

    // ── profile_contract ──────────────────────────────────────────────────────
    fun activeProfilePlanJson(requestJson: String): NativeActiveProfilePlan {
        val value = FluxaCoreUniFfi.coreInvokeValue("activeProfilePlan", requestJson)
        return gson.fromJson(value, NativeActiveProfilePlan::class.java) ?: NativeActiveProfilePlan()
    }

    fun tokenMergePlanJson(requestJson: String): String =
        FluxaCoreUniFfi.coreInvokeValue("tokenMergePlan", requestJson).toString()

    fun profileDefaultSeedJson(requestJson: String): String =
        FluxaCoreUniFfi.coreInvokeValue("profileDefaultSeed", requestJson).toString()

    fun profileSettingsMigrationPlan(requestJson: String): NativeProfileSettingsMigration {
        val value = FluxaCoreUniFfi.coreInvokeValue("profileSettingsMigrationPlan", requestJson)
        return gson.fromJson(value, NativeProfileSettingsMigration::class.java) ?: NativeProfileSettingsMigration()
    }

    fun profileAvatarDefault(requestJson: String): NativeProfileAvatarDefault {
        val value = FluxaCoreUniFfi.coreInvokeValue("profileAvatarDefault", requestJson)
        return gson.fromJson(value, NativeProfileAvatarDefault::class.java) ?: NativeProfileAvatarDefault()
    }

    // ── watchlist_plan ────────────────────────────────────────────────────────
    fun watchlistTogglePlan(requestJson: String): NativeWatchlistTogglePlan {
        val value = FluxaCoreUniFfi.coreInvokeValue("watchlistTogglePlan", requestJson)
        return gson.fromJson(value, NativeWatchlistTogglePlan::class.java) ?: NativeWatchlistTogglePlan()
    }

    fun libraryExternalMergePlanJson(requestJson: String): String =
        FluxaCoreUniFfi.coreInvokeValue("libraryExternalMergePlan", requestJson).toString()

    fun libraryCollectionImportValidation(requestJson: String): NativeLibraryCollectionImportValidation {
        val value = FluxaCoreUniFfi.coreInvokeValue("libraryCollectionImportValidation", requestJson)
        return gson.fromJson(value, NativeLibraryCollectionImportValidation::class.java)
            ?: NativeLibraryCollectionImportValidation()
    }

    fun libraryOfflineGrouping(requestJson: String): NativeLibraryOfflineGrouping {
        val value = FluxaCoreUniFfi.coreInvokeValue("libraryOfflineGrouping", requestJson)
        return gson.fromJson(value, NativeLibraryOfflineGrouping::class.java)
            ?: NativeLibraryOfflineGrouping()
    }

    fun playbackProgressMergePlanJson(requestJson: String): String =
        FluxaCoreUniFfi.coreInvokeValue("playbackProgressMergePlan", requestJson).toString()

    // ── player_policy ─────────────────────────────────────────────────────────
    fun playerBackendSelection(requestJson: String): NativePlayerBackendSelection {
        val value = FluxaCoreUniFfi.coreInvokeValue("playerBackendSelection", requestJson)
        return gson.fromJson(value, NativePlayerBackendSelection::class.java) ?: NativePlayerBackendSelection()
    }

    fun torrentFallbackFilePolicy(requestJson: String): NativeTorrentFallbackFilePolicy {
        val value = FluxaCoreUniFfi.coreInvokeValue("torrentFallbackFilePolicy", requestJson)
        return gson.fromJson(value, NativeTorrentFallbackFilePolicy::class.java)
            ?: NativeTorrentFallbackFilePolicy()
    }

    fun playerBufferTargets(requestJson: String): NativePlayerBufferTargets {
        val value = FluxaCoreUniFfi.coreInvokeValue("playerBufferTargets", requestJson)
        return gson.fromJson(value, NativePlayerBufferTargets::class.java) ?: NativePlayerBufferTargets()
    }

    fun playerRetryPolicy(requestJson: String): NativePlayerRetryPolicy {
        val value = FluxaCoreUniFfi.coreInvokeValue("playerRetryPolicy", requestJson)
        return gson.fromJson(value, NativePlayerRetryPolicy::class.java) ?: NativePlayerRetryPolicy()
    }

    fun playerSourceSidebarPlanJson(requestJson: String): String =
        FluxaCoreUniFfi.coreInvokeValue("playerSourceSidebarPlan", requestJson).toString()

    // ── search_plan ───────────────────────────────────────────────────────────
    fun searchResultGrouping(requestJson: String): NativeSearchResultGrouping {
        val value = FluxaCoreUniFfi.coreInvokeValue("searchResultGrouping", requestJson)
        return gson.fromJson(value, NativeSearchResultGrouping::class.java) ?: NativeSearchResultGrouping()
    }

    fun discoverSortPlanJson(requestJson: String): String =
        FluxaCoreUniFfi.coreInvokeValue("discoverSortPlan", requestJson).toString()

    fun librarySortPlanJson(requestJson: String): String =
        FluxaCoreUniFfi.coreInvokeValue("librarySortPlan", requestJson).toString()

    fun detailSeriesLookupId(rawId: String): String {
        val args = JsonObject().apply { addProperty("id", rawId) }
        return FluxaCoreUniFfi.coreInvokeValue("detailSeriesLookupId", args.toString()).asString
    }

    fun detailSeasonLoadPlan(requestJson: String): NativeDetailSeasonLoadPlan {
        val value = FluxaCoreUniFfi.coreInvokeValue("detailSeasonLoadPlan", requestJson)
        return gson.fromJson(value, NativeDetailSeasonLoadPlan::class.java) ?: NativeDetailSeasonLoadPlan()
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
    private external fun createAppCoreStateNative(initialJson: String): Long
    private external fun destroyAppCoreStateNative(handle: Long): Boolean
    private external fun appCoreStateJsonNative(handle: Long): String?
    private external fun appCoreDispatchJsonNative(handle: Long, actionJson: String): String?
    private external fun createHeadlessEngineNative(initialJson: String): Long
    private external fun destroyHeadlessEngineNative(handle: Long): Boolean
    private external fun headlessEngineSnapshotJsonNative(handle: Long): String?
    private external fun headlessEngineDispatchJsonNative(handle: Long, actionJson: String): String?
    private external fun headlessEngineCompleteEffectJsonNative(handle: Long, resultJson: String): String?
}
