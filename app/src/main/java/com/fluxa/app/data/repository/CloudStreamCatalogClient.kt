package com.fluxa.app.data.repository

import android.util.Base64
import android.util.Log
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.domain.discovery.cs3CatalogFeedKey
import com.fluxa.app.domain.discovery.cs3PluginFeedKey
import com.fluxa.app.plugins.cloudstream.ExternalExtensionRunner
import com.fluxa.app.plugins.cloudstream.ScraperSearchResult
import com.fluxa.app.ui.catalog.HomeCategory
import com.lagradost.cloudstream3.AnimeSearchResponse
import com.lagradost.cloudstream3.MainAPI
import com.lagradost.cloudstream3.MainPageRequest
import com.lagradost.cloudstream3.MovieSearchResponse
import com.lagradost.cloudstream3.SearchResponse
import com.lagradost.cloudstream3.TvSeriesSearchResponse
import com.lagradost.cloudstream3.TvType
import com.lagradost.cloudstream3.metaproviders.TmdbProvider
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.coroutines.isActive
import javax.inject.Inject
import javax.inject.Singleton

private const val TAG = "CS3CatalogClient"
private const val ROW_TIMEOUT_MS = 20_000L
@Singleton
class CloudStreamCatalogClient @Inject constructor() {

    /**
     * Fetches home catalog rows from all loaded CS3 plugins that support getMainPage().
     */
    suspend fun fetchHomeCatalogCategories(
        apis: List<MainAPI>,
        iconsByApiName: Map<String, String> = emptyMap(),
        enabledFeedKeys: Set<String>? = null
    ): List<HomeCategory> =
        withContext(Dispatchers.IO) {
            val supported = apis.filter { it.hasMainPage }
            if (supported.isEmpty()) return@withContext emptyList()
            coroutineScope {
                supported.map { api ->
                    async {
                        try {
                            fetchApiCatalogRows(api, iconsByApiName[api.name], enabledFeedKeys)
                        } catch (t: Throwable) {
                            Log.w(TAG, "Catalog fetch failed for ${api.name}", t)
                            emptyList()
                        }
                    }
                }.awaitAll().flatten()
            }
        }

    suspend fun fetchFeedItems(
        apis: List<MainAPI>,
        feedKey: String,
        page: Int = 1
    ): List<Meta> =
        withContext(Dispatchers.IO) {
            apis.filter { it.hasMainPage }.forEach { api ->
                val items = fetchApiFeedItems(api, feedKey, page)
                if (items.isNotEmpty()) return@withContext items
            }
            emptyList()
        }

    suspend fun searchRows(
        apis: List<MainAPI>,
        query: String
    ): List<SearchResultRow> =
        withContext(Dispatchers.IO) {
            if (query.isBlank() || apis.isEmpty()) return@withContext emptyList()
            val runner = ExternalExtensionRunner()
            coroutineScope {
                apis.map { api ->
                    async {
                        try {
                            val metas = runner.searchScraper(api, query)
                                .mapNotNull { it.toMeta(api.name) }
                            if (metas.isEmpty()) return@async null
                            SearchResultRow(
                                title = api.name,
                                items = metas,
                                id = "cs3_search_${api.name}",
                                type = metas.first().type
                            )
                        } catch (t: Throwable) {
                            Log.w(TAG, "Search failed for ${api.name}", t)
                            null
                        }
                    }
                }.awaitAll().filterNotNull()
            }
        }

    private suspend fun fetchApiCatalogRows(
        api: MainAPI,
        iconUrl: String? = null,
        enabledFeedKeys: Set<String>? = null
    ): List<HomeCategory> {
        val result = mutableListOf<HomeCategory>()
        api.mainPage.forEachIndexed { pageIdx, pageData ->
            try {
                val catalogName = pageData.name.takeIf { it.isNotBlank() } ?: api.name
                val feedKey = cs3CatalogFeedKey(api.name, catalogName, pageIdx)
                val legacyPluginKey = cs3PluginFeedKey(api.name)
                if (enabledFeedKeys != null && feedKey !in enabledFeedKeys && legacyPluginKey !in enabledFeedKeys) {
                    return@forEachIndexed
                }
                val response = withTimeoutOrNull(ROW_TIMEOUT_MS) {
                    api.getMainPage(
                        1,
                        MainPageRequest(
                            data = pageData.data,
                            name = pageData.name,
                            horizontalImages = pageData.horizontalImages
                        )
                    )
                } ?: return@forEachIndexed

                response.items.forEachIndexed { listIdx, pageList ->
                    val metas = pageList.list.mapNotNull { it.toMeta(api) }
                    if (metas.isEmpty()) return@forEachIndexed
                    val categoryId = if (listIdx == 0) feedKey else "${feedKey}_${listIdx}"
                    result.add(
                        HomeCategory(
                            name = pageList.name.takeIf { it.isNotBlank() }
                                ?: "${api.name}: $catalogName",
                            items = metas,
                            id = categoryId,
                            type = metas.first().type,
                            canLoadMore = false,
                            addonIconUrl = iconUrl
                        )
                    )
                }
            } catch (t: Throwable) {
                Log.w(TAG, "Page '${pageData.name}' from ${api.name} failed: ${t.javaClass.simpleName}: ${t.message}")
            }
        }
        return result
    }

    private suspend fun fetchApiFeedItems(
        api: MainAPI,
        feedKey: String,
        page: Int
    ): List<Meta> {
        val legacyPluginKey = cs3PluginFeedKey(api.name)
        api.mainPage.forEachIndexed { pageIdx, pageData ->
            val catalogName = pageData.name.takeIf { it.isNotBlank() } ?: api.name
            val catalogFeedKey = cs3CatalogFeedKey(api.name, catalogName, pageIdx)
            if (feedKey != catalogFeedKey && feedKey != legacyPluginKey) return@forEachIndexed
            return try {
                val response = withTimeoutOrNull(ROW_TIMEOUT_MS) {
                    api.getMainPage(
                        page,
                        MainPageRequest(
                            data = pageData.data,
                            name = pageData.name,
                            horizontalImages = pageData.horizontalImages
                        )
                    )
                } ?: return emptyList()
                response.items.flatMap { pageList -> pageList.list.mapNotNull { it.toMeta(api) } }
            } catch (t: Throwable) {
                Log.w(TAG, "Feed '$catalogName' from ${api.name} failed: ${t.javaClass.simpleName}: ${t.message}")
                emptyList()
            }
        }
        return emptyList()
    }

    private fun SearchResponse.toMeta(api: MainAPI): Meta? {
        val title = name.takeIf { it.isNotBlank() } ?: return null
        val id = resolveId(this, api)
        val type = this.type?.toStremioType() ?: "movie"
        val year = when (this) {
            is MovieSearchResponse -> this.year
            is TvSeriesSearchResponse -> this.year
            is AnimeSearchResponse -> this.year
            else -> null
        }
        return Meta(
            id = id,
            name = title,
            type = type,
            poster = posterUrl?.trim()?.takeIf { it.isNotBlank() }
                ?.let { if (it.startsWith("http://")) it.replaceFirst("http://", "https://") else it },
            releaseInfo = year?.toString()
        )
    }

    private fun ScraperSearchResult.toMeta(apiName: String): Meta? {
        val title = this.title.takeIf { it.isNotBlank() } ?: return null
        val id = encodeCsId(apiName, url)
        return Meta(
            id = id,
            name = title,
            type = type?.toStremioType() ?: "movie",
            poster = posterUrl?.trim()?.takeIf { it.isNotBlank() }
                ?.let { if (it.startsWith("http://")) it.replaceFirst("http://", "https://") else it },
            releaseInfo = year?.toString()
        )
    }

    private fun resolveId(result: SearchResponse, api: MainAPI): String {
        if (api is TmdbProvider) {
            result.id?.let { return "tmdb:$it" }
            extractTmdbId(result.url)?.let { return "tmdb:$it" }
        }
        return encodeCsId(api.name, result.url)
    }

    private fun extractTmdbId(url: String): Int? =
        Regex("""themoviedb\.org/(?:movie|tv)/(\d+)""")
            .find(url)?.groupValues?.get(1)?.toIntOrNull()

    companion object {
        fun encodeCsId(apiName: String, data: String): String {
            val n = Base64.encodeToString(apiName.toByteArray(), Base64.NO_WRAP or Base64.NO_PADDING)
            val d = Base64.encodeToString(data.toByteArray(), Base64.NO_WRAP or Base64.NO_PADDING)
            return "cs3:$n:$d"
        }

        fun decodeCsId(id: String): Pair<String, String>? {
            if (!id.startsWith("cs3:")) return null
            val body = id.removePrefix("cs3:")
            val sep = body.indexOf(':')
            if (sep < 0) return null
            return try {
                val apiName = String(Base64.decode(body.substring(0, sep), Base64.NO_WRAP or Base64.NO_PADDING))
                val data = String(Base64.decode(body.substring(sep + 1), Base64.NO_WRAP or Base64.NO_PADDING))
                apiName to data
            } catch (e: Exception) {
                Log.w(TAG, "decodeCsId failed for: $id", e)
                null
            }
        }
    }
}

internal fun TvType.toStremioType(): String = when (this) {
    TvType.Movie, TvType.AnimeMovie, TvType.Documentary -> "movie"
    TvType.TvSeries, TvType.Anime, TvType.OVA, TvType.Cartoon, TvType.AsianDrama -> "series"
    else -> "movie"
}
