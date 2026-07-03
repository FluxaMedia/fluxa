package com.fluxa.app.data.repository

import android.util.Base64
import android.util.Log
import com.fluxa.app.data.remote.Meta
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
private const val MAX_ITEMS_PER_ROW = 25

@Singleton
class CloudStreamCatalogClient @Inject constructor() {

    /**
     * Fetches home catalog rows from all loaded CS3 plugins that support getMainPage().
     */
    suspend fun fetchHomeCatalogCategories(
        apis: List<MainAPI>,
        iconsByApiName: Map<String, String> = emptyMap()
    ): List<HomeCategory> =
        withContext(Dispatchers.IO) {
            val supported = apis.filter { it.hasMainPage }
            if (supported.isEmpty()) return@withContext emptyList()
            coroutineScope {
                supported.map { api ->
                    async {
                        try {
                            fetchApiCatalogRows(api, iconsByApiName[api.name])
                        } catch (t: Throwable) {
                            Log.w(TAG, "Catalog fetch failed for ${api.name}", t)
                            emptyList()
                        }
                    }
                }.awaitAll().flatten()
            }
        }

    private suspend fun fetchApiCatalogRows(api: MainAPI, iconUrl: String? = null): List<HomeCategory> {
        val result = mutableListOf<HomeCategory>()
        api.mainPage.forEachIndexed { pageIdx, pageData ->
            try {
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
                    val metas = pageList.list.take(MAX_ITEMS_PER_ROW).mapNotNull { it.toMeta(api) }
                    if (metas.isEmpty()) return@forEachIndexed
                    val categoryId = "cs3_${api.name.sanitize()}_${pageIdx}_$listIdx"
                    result.add(
                        HomeCategory(
                            name = pageList.name.takeIf { it.isNotBlank() }
                                ?: "${api.name}: ${pageData.name}",
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

    private fun String.sanitize() = replace(Regex("[^a-zA-Z0-9]"), "_").lowercase()

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
