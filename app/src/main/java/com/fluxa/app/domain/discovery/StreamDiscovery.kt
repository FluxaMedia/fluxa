package com.fluxa.app.domain.discovery

import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.models.NativeStreamDiscoveryExecutionPolicy

import com.fluxa.app.BuildConfig
import com.fluxa.app.common.Constants
import android.util.Log
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.joinAll
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.sync.withPermit
import kotlinx.coroutines.launch
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.withTimeoutOrNull
import javax.inject.Inject
import javax.inject.Singleton

private inline fun logDebug(tag: String, message: () -> String) {
    if (BuildConfig.DEBUG) Log.d(tag, message())
}

data class StreamDiscoveryRequest(
    val addons: List<AddonDescriptor>,
    val type: String,
    val id: String,
    val language: String,
    val preferFastStart: Boolean = false,
    val expectedEpisodeTitles: List<String> = emptyList(),
    val seasonEpisodeTitles: Map<Int, List<String>> = emptyMap(),
    val seasonEpisodeIds: Map<Int, String> = emptyMap(),
    val cs3PluginApis: List<com.lagradost.cloudstream3.MainAPI> = emptyList(),
    val cs3SearchQuery: String? = null,
    val cs3OriginalName: String? = null,
    val cs3Year: Int? = null
)

object StreamSourceOrderPolicy {
    fun keepSourceOrder(streams: List<Stream>, request: StreamDiscoveryRequest): List<Stream> {
        return streams
    }
}

@Singleton
class StreamDiscoveryUseCase @Inject constructor(
    private val repository: StremioRepository,
    private val cache: StreamDiscoveryMemoryCache
) {
    suspend fun discover(request: StreamDiscoveryRequest): List<Stream> = supervisorScope {
        val policy = executionPolicy(request)
        cache.get(policy.cacheKey)?.let { return@supervisorScope it }
        val addonRequestSemaphore = Semaphore(policy.maxConcurrentAddonRequests.toInt().coerceAtLeast(1))

        val remoteDeferred = policy.addonRequests.map { addonRequest ->
            async {
                addonRequestSemaphore.withPermit {
                    try {
                        val streams = withTimeoutOrNull(addonRequest.timeoutMs) {
                            repository.getStreamsFromAddon(
                                addonRequest.transportUrl,
                                addonRequest.addonName,
                                addonRequest.type,
                                addonRequest.id
                            )
                        } ?: emptyList()
                        logDebug("StreamDiscovery") {
                            "addon id=${addonRequest.id} addon=${addonRequest.addonName} streams=${streams.size} timeout=${addonRequest.timeoutMs}"
                        }
                        streams
                    } catch (e: Exception) {
                        Log.w(
                            "StreamDiscovery",
                            "addon id=${addonRequest.id} addon=${addonRequest.addonName} failed=${e::class.java.simpleName}"
                        )
                        emptyList()
                    }
                }
            }
        }

        val cs3Deferred = async {
            val cloudstream = policy.cloudstreamRequest
            if (cloudstream != null) {
                try {
                    withTimeoutOrNull(cloudstream.timeoutMs) {
                        repository.getStreamsFromCloudStreamPlugins(
                            pluginApis = request.cs3PluginApis,
                            id = cloudstream.id,
                            title = cloudstream.title,
                            year = cloudstream.year?.toInt(),
                            type = cloudstream.type,
                            season = cloudstream.season,
                            episode = cloudstream.episode,
                            originalName = cloudstream.originalName
                        )
                    } ?: emptyList()
                } catch (e: Exception) {
                    Log.e("StreamDiscovery", "CS3 Plugin discovery failed", e)
                    emptyList()
                }
            } else {
                emptyList()
            }
        }

        val allStreams = mutableListOf<Stream>()

        val remoteResults = remoteDeferred.awaitAll().flatten()
        allStreams.addAll(remoteResults)

        val cs3Results = cs3Deferred.await()
        allStreams.addAll(cs3Results)

        logDebug("StreamDiscovery") {
            "discover id=${request.id} remote=${remoteResults.size} cs3=${cs3Results.size} addons=${policy.addonRequests.size}"
        }

        val ranked = finalizeStreams(allStreams, request)

        if (shouldCache(policy, ranked)) {
            cache.put(policy.cacheKey, ranked)
        }
        ranked
    }

    suspend fun discoverProgressive(
        request: StreamDiscoveryRequest,
        onProgress: (streams: List<Stream>, completedAddonNames: List<String>, loadingAddonNames: List<String>) -> Unit
    ): List<Stream> = supervisorScope {
        val policy = executionPolicy(request)
        cache.get(policy.cacheKey)?.let { cached ->
            if (policy.emitCachedResult) {
                val names = cached.mapNotNull { it.addonName?.takeIf(String::isNotBlank) }.distinct()
                onProgress(cached, names, emptyList())
            }
            return@supervisorScope cached
        }
        val addonRequestSemaphore = Semaphore(policy.maxConcurrentAddonRequests.toInt().coerceAtLeast(1))

        val aggregateMutex = Mutex()
        val rawStreams = mutableListOf<Stream>()
        val completedAddons = mutableListOf<String>()

        // Count pending requests per unique addon name so we only mark it done when all its ID variants finish.
        val pendingPerAddon = policy.addonRequests
            .filter { it.addonName.isNotBlank() }
            .groupBy { it.addonName }
            .mapValues { it.value.size }
            .toMutableMap()
        val hasCs3 = policy.cloudstreamRequest != null
        if (hasCs3) pendingPerAddon["CloudStream"] = 1

        val initialLoading = pendingPerAddon.keys.toList()
        if (initialLoading.isNotEmpty()) {
            onProgress(emptyList(), emptyList(), initialLoading)
        }

        suspend fun onRequestDone(addonName: String, streams: List<Stream>) {
            aggregateMutex.withLock {
                rawStreams.addAll(streams)
                val remaining = (pendingPerAddon[addonName] ?: 1) - 1
                pendingPerAddon[addonName] = remaining
                if (remaining == 0 && addonName.isNotBlank()) completedAddons.add(addonName)
                val loadingNow = pendingPerAddon.filter { it.value > 0 }.keys.toList()
                if (policy.emitPartialNonEmptyResults) {
                    val ranked = finalizeStreams(rawStreams, request)
                    if (ranked.isNotEmpty() || loadingNow.isEmpty()) {
                        onProgress(ranked, completedAddons.toList(), loadingNow)
                    }
                }
            }
        }

        val remoteDeferred = policy.addonRequests.map { addonRequest ->
            addonRequest to async {
                addonRequestSemaphore.withPermit {
                    try {
                        val streams = withTimeoutOrNull(addonRequest.timeoutMs) {
                            repository.getStreamsFromAddon(
                                addonRequest.transportUrl,
                                addonRequest.addonName,
                                addonRequest.type,
                                addonRequest.id
                            )
                        } ?: emptyList()
                        logDebug("StreamDiscovery") {
                            "addon id=${addonRequest.id} addon=${addonRequest.addonName} streams=${streams.size} timeout=${addonRequest.timeoutMs}"
                        }
                        streams
                    } catch (e: Exception) {
                        Log.w(
                            "StreamDiscovery",
                            "addon id=${addonRequest.id} addon=${addonRequest.addonName} failed=${e::class.java.simpleName}"
                        )
                        emptyList()
                    }
                }
            }
        }

        val cs3Deferred = async {
            val cloudstream = policy.cloudstreamRequest
            if (cloudstream != null) {
                try {
                    withTimeoutOrNull(cloudstream.timeoutMs) {
                        repository.getStreamsFromCloudStreamPlugins(
                            pluginApis = request.cs3PluginApis,
                            id = cloudstream.id,
                            title = cloudstream.title,
                            year = cloudstream.year?.toInt(),
                            type = cloudstream.type,
                            season = cloudstream.season,
                            episode = cloudstream.episode,
                            originalName = cloudstream.originalName
                        )
                    } ?: emptyList()
                } catch (e: Exception) {
                    Log.e("StreamDiscovery", "CS3 Plugin discovery failed", e)
                    emptyList()
                }
            } else {
                emptyList()
            }
        }

        val progressJobs = buildList {
            remoteDeferred.forEach { (addonRequest, deferred) ->
                add(launch { onRequestDone(addonRequest.addonName, deferred.await()) })
            }
            if (hasCs3) add(launch { onRequestDone("CloudStream", cs3Deferred.await()) })
        }
        progressJobs.joinAll()

        val finalRanked = aggregateMutex.withLock { finalizeStreams(rawStreams, request) }
        if (shouldCache(policy, finalRanked)) {
            cache.put(policy.cacheKey, finalRanked)
        }
        finalRanked
    }

    suspend fun prefetch(request: StreamDiscoveryRequest): List<Stream> = discover(request)

    fun peek(type: String, id: String, language: String): List<Stream>? {
        val prefix = FluxaCoreNative.streamDiscoveryCachePrefix(type, id, language)
        return cache.firstWithPrefix(prefix)
    }

    fun invalidate(type: String, id: String, language: String) {
        val prefix = FluxaCoreNative.streamDiscoveryCachePrefix(type, id, language)
        cache.invalidatePrefix(prefix)
    }

    fun cacheSize(): Int = cache.size()

    private fun executionPolicy(request: StreamDiscoveryRequest): NativeStreamDiscoveryExecutionPolicy {
        return FluxaCoreNative.streamDiscoveryExecutionPolicy(
            type = request.type,
            id = request.id,
            language = request.language,
            preferFastStart = request.preferFastStart,
            addonRequestTimeoutMs = Constants.Timeouts.PLUGIN_SEARCH,
            fastAddonRequestTimeoutMs = Constants.Timeouts.ADDON_REQUEST * 3,
            cloudstreamTimeoutMs = Constants.Timeouts.PLUGIN_LOAD_LINKS,
            addons = request.addons,
            cs3PluginNames = request.cs3PluginApis.map { it.name },
            cs3SearchQuery = request.cs3SearchQuery,
            cs3Year = request.cs3Year,
            cs3OriginalName = request.cs3OriginalName
        )
    }

    private fun shouldCache(policy: NativeStreamDiscoveryExecutionPolicy, streams: List<Stream>): Boolean {
        return streams.size >= policy.cacheWriteMinimumResultCount
    }

    private suspend fun finalizeStreams(
        streams: List<Stream>,
        request: StreamDiscoveryRequest
    ): List<Stream> {
        return StreamSourceOrderPolicy.keepSourceOrder(
            streams = streams,
            request = request
        )
    }

}
