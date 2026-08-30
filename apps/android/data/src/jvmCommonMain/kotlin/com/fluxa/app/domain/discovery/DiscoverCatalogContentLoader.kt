package com.fluxa.app.domain.discovery

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.StremioRepository
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import javax.inject.Inject
import javax.inject.Singleton

data class DiscoverRequest(
    val type: String,
    val catalogKey: String?,
    val genre: String?,
    val year: String?,
    val rating: Float?,
    val provider: String?,
    val region: String?
)

@Singleton
class DiscoverCatalogContentLoader @Inject constructor(
    private val repository: StremioRepository
) {
    private val memoryCache = LinkedHashMap<String, CachedDiscoverResult>()

    suspend fun discover(
        request: DiscoverRequest,
        catalogOptions: List<DiscoverCatalogOption>,
        normalizeCatalogItems: suspend (
            items: List<Meta>,
            type: String,
            catalogId: String,
            genre: String?
        ) -> List<Meta>
    ): List<Meta> = coroutineScope {
        val candidateKeys = FluxaCoreNative.discoverCatalogCandidates(
            request = mapOf(
                "contentType" to request.type,
                "catalogKey" to request.catalogKey,
                "genre" to request.genre,
                "provider" to request.provider,
                "limit" to DEFAULT_DISCOVER_CATALOG_LIMIT,
            ),
            catalogs = catalogOptions,
        )
        val candidateCatalogs = candidateKeys.mapNotNull { key -> catalogOptions.firstOrNull { it.key == key } }

        if (candidateCatalogs.any { it.requiresGenre } && request.genre.isNullOrBlank()) {
            val optionalOnly = candidateCatalogs.filterNot { it.requiresGenre }
            if (optionalOnly.isEmpty()) return@coroutineScope emptyList()
            return@coroutineScope fetchCatalogs(optionalOnly, request, normalizeCatalogItems)
        }

        val cacheKey = request.cacheKey(candidateCatalogs)
        memoryCache[cacheKey]
            ?.takeIf { System.currentTimeMillis() - it.loadedAtMs < DISCOVER_CACHE_TTL_MS }
            ?.let { return@coroutineScope it.items }

        fetchCatalogs(candidateCatalogs, request, normalizeCatalogItems).also { items ->
            memoryCache[cacheKey] = CachedDiscoverResult(System.currentTimeMillis(), items)
            trimCache()
        }
    }

    private suspend fun fetchCatalogs(
        catalogs: List<DiscoverCatalogOption>,
        request: DiscoverRequest,
        normalizeCatalogItems: suspend (
            items: List<Meta>,
            type: String,
            catalogId: String,
            genre: String?
        ) -> List<Meta>
    ): List<Meta> = coroutineScope {
        val requestSemaphore = Semaphore(MAX_CONCURRENT_DISCOVER_CATALOG_REQUESTS)
        catalogs
            .map { catalog ->
                val selectedTypes = if (catalog.type == "all") listOf("movie", "series") else listOf(catalog.type)
                async {
                    selectedTypes
                        .map { type ->
                            async {
                                requestSemaphore.withPermit {
                                    val raw = repository.getAddonCatalog(
                                        transportUrl = catalog.transportUrl,
                                        type = type,
                                        id = catalog.id,
                                        genre = request.genre
                                    )
                                    normalizeCatalogItems(raw, type, catalog.id, request.genre)
                                }
                            }
                        }
                        .awaitAll()
                        .fold(emptyList<Meta>()) { acc, items -> interleave(acc, items) }
                }
            }
            .awaitAll()
            .fold(emptyList<Meta>()) { acc, items -> interleave(acc, items) }
            .distinctBy { it.id }
            .let { items -> DiscoverResultFilters.apply(items, request) }
    }

    private fun interleave(primary: List<Meta>, secondary: List<Meta>): List<Meta> {
        val result = mutableListOf<Meta>()
        val maxSize = maxOf(primary.size, secondary.size)
        for (index in 0 until maxSize) {
            primary.getOrNull(index)?.let(result::add)
            secondary.getOrNull(index)?.let(result::add)
        }
        return result.distinctBy { it.id }
    }

    private fun trimCache() {
        while (memoryCache.size > MAX_DISCOVER_CACHE_ENTRIES) {
            val firstKey = memoryCache.keys.firstOrNull() ?: return
            memoryCache.remove(firstKey)
        }
    }

    private companion object {
        const val DEFAULT_DISCOVER_CATALOG_LIMIT = 5
        const val MAX_CONCURRENT_DISCOVER_CATALOG_REQUESTS = 3
        const val DISCOVER_CACHE_TTL_MS = 5 * 60 * 1000L
        const val MAX_DISCOVER_CACHE_ENTRIES = 24
    }
}

private data class CachedDiscoverResult(
    val loadedAtMs: Long,
    val items: List<Meta>
)

private fun DiscoverRequest.cacheKey(catalogs: List<DiscoverCatalogOption>): String {
    return FluxaCoreNative.discoverCatalogCacheKey(
        type = type,
        catalogKey = catalogKey,
        genre = genre,
        year = year,
        rating = rating,
        provider = provider,
        region = region,
        catalogSignatures = catalogs.map { "${it.transportUrl}:${it.type}:${it.id}:${it.genres.joinToString(",")}" }
    )
}

object DiscoverResultFilters {
    fun apply(items: List<Meta>, request: DiscoverRequest): List<Meta> {
        return FluxaCoreNative.filterDiscoverResults(
            items = items,
            year = request.year,
            rating = request.rating,
            region = request.region
        )
    }
}
