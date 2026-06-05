package com.fluxa.app.ui.catalog

import android.util.LruCache
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull

internal class HomeBillboardRuntime(
    private val scope: CoroutineScope,
    private val watchlistManager: WatchlistManager,
    private val pool: () -> List<Meta>,
    private val setPool: (List<Meta>) -> Unit,
    private val index: () -> Int,
    private val setIndex: (Int) -> Unit,
    private val categories: () -> List<HomeCategory>,
    private val language: () -> String,
    private val setMovie: (Meta?) -> Unit,
    private val setLogo: (String?) -> Unit,
    private val setWatchlist: (Boolean) -> Unit,
    private val setTrailerUrl: (String?) -> Unit,
    private val setNextEpisode: (String?) -> Unit,
    private val setSeasonPosterUrl: (String?) -> Unit,
    private val getMetaDetail: suspend (String, String) -> MetaDetail?,
    private val parseSeasonEpisode: (String, String) -> String?,
    private val prefetchDirectPlayback: (Meta, MetaDetail?) -> Unit
) {
    private var rotationJob: Job? = null
    private var prefetchJob: Job? = null
    private val enrichedCache = LruCache<String, Meta>(50)

    fun reset() {
        rotationJob?.cancel()
        prefetchJob?.cancel()
        setPool(emptyList())
        setIndex(0)
        setMovie(null)
        setLogo(null)
        setTrailerUrl(null)
        setNextEpisode(null)
        setSeasonPosterUrl(null)
    }

    fun pauseRotation() {
        rotationJob?.cancel()
    }

    fun next() {
        pauseRotation()
        val items = pool()
        if (items.isNotEmpty()) {
            val nextIndex = (index() + 1) % items.size
            setIndex(nextIndex)
            scope.launch { updateContent(items[nextIndex]) }
            prefetchNeighbors(nextIndex)
        }
        startRotation()
    }

    fun previous() {
        pauseRotation()
        val items = pool()
        if (items.isNotEmpty()) {
            val previousIndex = if (index() <= 0) items.size - 1 else index() - 1
            setIndex(previousIndex)
            scope.launch { updateContent(items[previousIndex]) }
            prefetchNeighbors(previousIndex)
        }
        startRotation()
    }

    fun jumpTo(targetIndex: Int) {
        pauseRotation()
        val items = pool()
        if (items.isNotEmpty() && targetIndex in items.indices) {
            setIndex(targetIndex)
            scope.launch { updateContent(items[targetIndex]) }
            prefetchNeighbors(targetIndex)
        }
        startRotation()
    }

    fun syncIndex(targetIndex: Int) {
        pauseRotation()
        val items = pool()
        if (items.isNotEmpty() && targetIndex in items.indices && index() != targetIndex) {
            setIndex(targetIndex)
            scope.launch { updateContent(items[targetIndex]) }
            prefetchNeighbors(targetIndex)
        }
        startRotation()
    }

    fun toggleWatchlist(movie: Meta?) {
        scope.launch {
            val currentMovie = movie ?: return@launch
            watchlistManager.toggleWatchlist(currentMovie)
            setWatchlist(watchlistManager.isInWatchlist(currentMovie.id))
        }
    }

    fun startRotation() {
        rotationJob?.cancel()
        rotationJob = scope.launch {
            while (isActive) {
                delay(18000)
                val items = pool()
                if (items.isNotEmpty()) {
                    val nextIndex = (index() + 1) % items.size
                    setIndex(nextIndex)
                    updateContent(items[nextIndex])
                    prefetchNeighbors(nextIndex)
                }
            }
        }
    }

    fun prefetchNeighbors(currentIndex: Int) {
        prefetchJob?.cancel()
        prefetchJob = scope.launch(Dispatchers.IO) {
            val currentPool = pool()
            if (currentPool.isEmpty()) return@launch
            val neighborIndices = listOf((currentIndex + 1) % currentPool.size, (currentIndex - 1 + currentPool.size) % currentPool.size)
            neighborIndices.forEach { idx ->
                val item = currentPool[idx]
                if (enrichedCache.get(HomeBillboardRanking.contentIdentityKey(item)) == null) enrich(item)
            }
        }
    }

    suspend fun enrich(item: Meta): Meta? {
        try {
            val detail = withTimeoutOrNull(5000) { getMetaDetail(item.type, item.id) } ?: return null
            val enriched = item.copy(
                poster = detail.poster,
                description = detail.description ?: item.description,
                background = detail.background ?: item.background,
                imdbRating = detail.imdbRating ?: item.imdbRating,
                ageRating = detail.ageRating?.takeIf { it.isNotBlank() } ?: item.ageRating,
                ratings = detail.ratings ?: item.ratings,
                genres = detail.genres ?: item.genres,
                runtime = detail.runtime,
                cast = detail.cast,
                seasonsCount = detail.seasonsCount ?: detail.videos?.mapNotNull { it.season }?.filter { it > 0 }?.distinct()?.size,
                episodesCount = detail.videos?.size,
                releaseInfo = detail.releaseInfo ?: item.releaseInfo,
                logo = detail.logo,
                seasonPosters = detail.seasonPosters ?: item.seasonPosters
            )
            val cacheKey = HomeBillboardRanking.contentIdentityKey(item)
            enrichedCache.put(cacheKey, enriched)
            updatePoolItem(cacheKey, enriched)
            return enriched
        } catch (e: Exception) {
            return null
        }
    }

    suspend fun updateContent(item: Meta) {
        val cacheKey = HomeBillboardRanking.contentIdentityKey(item)
        val cached = enrichedCache.get(cacheKey)
        if (cached != null) {
            setMovie(cached)
            setLogo(cached.logo)
            setWatchlist(watchlistManager.isInWatchlist(item.id))
            setTrailerUrl(null)
            setNextEpisode(null)
            setSeasonPosterUrl(null)
        } else {
            // Show raw catalog data immediately — no mutex blocking here
            setMovie(item)
            setLogo(item.logo)
            setWatchlist(watchlistManager.isInWatchlist(item.id))
            // Enrich in background without blocking the caller
            scope.launch(Dispatchers.IO) { enrichAndPublish(item) }
        }
    }

    private suspend fun enrichAndPublish(item: Meta) {
        val lang = language()
        val cacheKey = HomeBillboardRanking.contentIdentityKey(item)
        try {
            val detail = withTimeoutOrNull(4000) { getMetaDetail(item.type, item.id) } ?: return
            val isSeries = item.type == "series" || item.type == "tv"
            val videos = detail.videos

            // If season posters are available, use the latest season's poster as hero backdrop.
            val seasonBackground: String? = if (isSeries) {
                detail.seasonPosters
                    ?.maxByOrNull { (key, _) -> key.toIntOrNull() ?: 0 }
                    ?.value
                    ?.takeIf { it.isNotBlank() }
            } else null

            val enrichedMeta = item.copy(
                poster = detail.poster,
                description = detail.description ?: item.description,
                background = detail.background ?: item.background,
                logo = detail.logo,
                imdbRating = detail.imdbRating ?: item.imdbRating,
                ageRating = detail.ageRating?.takeIf { it.isNotBlank() } ?: item.ageRating,
                ratings = detail.ratings ?: item.ratings,
                genres = detail.genres ?: item.genres,
                runtime = detail.runtime ?: item.runtime,
                cast = detail.cast,
                seasonsCount = detail.seasonsCount ?: videos?.mapNotNull { it.season }?.filter { it > 0 }?.distinct()?.size,
                episodesCount = videos?.size,
                releaseInfo = detail.releaseInfo ?: item.releaseInfo,
                awards = detail.awards ?: item.awards,
                seasonPosters = detail.seasonPosters ?: item.seasonPosters
            )
            enrichedCache.put(cacheKey, enrichedMeta)
            updatePoolItem(cacheKey, enrichedMeta)
            val currentPool = pool()
            val currentIndex = index()
            val currentItem = currentPool.getOrNull(currentIndex)
            if (currentItem != null && HomeBillboardRanking.contentIdentityKey(currentItem) == cacheKey) {
                setMovie(enrichedMeta)
                setLogo(enrichedMeta.logo)
                if (isSeries && !videos.isNullOrEmpty()) {
                    val libCategory = categories().find { it.id == "library" }
                    val lastWatchedId = libCategory?.items?.find { it.id == item.id }?.lastVideoId
                    val nextVideo = if (lastWatchedId != null) {
                        val videoIndex = videos.indexOfFirst { it.id == lastWatchedId }
                        if (videoIndex != -1 && videoIndex < videos.size - 1) videos[videoIndex + 1] else videos[0]
                    } else {
                        videos[0]
                    }
                    setNextEpisode(parseSeasonEpisode(nextVideo.id, lang))
                }
                setSeasonPosterUrl(seasonBackground)
            }
            prefetchDirectPlayback(enrichedMeta, detail)
        } catch (e: Exception) {
            // non-critical
        }
    }

    private fun updatePoolItem(cacheKey: String, item: Meta) {
        val currentPool = pool().toMutableList()
        val idx = currentPool.indexOfFirst { HomeBillboardRanking.contentIdentityKey(it) == cacheKey }
        if (idx != -1) {
            currentPool[idx] = item
            setPool(normalizePool(currentPool))
        }
    }

    fun normalizePool(items: List<Meta>): List<Meta> {
        return items
            .distinctBy(HomeBillboardRanking::contentIdentityKey)
            .distinctBy { HomeBillboardRanking.normalizeTitle(it.originalName ?: it.name) }
            .take(10)
    }
}
