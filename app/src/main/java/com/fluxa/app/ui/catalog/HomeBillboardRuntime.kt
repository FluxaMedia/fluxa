package com.fluxa.app.ui.catalog

import com.fluxa.app.shared.feature.player.TrailerCue
import com.fluxa.app.shared.feature.player.TrailerResolveResult
import com.fluxa.app.shared.feature.player.TrailerResult
import com.fluxa.app.shared.feature.player.TrailerSubtitle

import android.util.LruCache
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.NativeHeadlessEngineResult
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.remote.DetailTrailer
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import okhttp3.OkHttpClient
import okhttp3.Request

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
    private val watchlistValue: () -> Boolean,
    private val setWatchlist: (Boolean) -> Unit,
    private val setTrailerUrl: (String?) -> Unit,
    private val setTrailerSubtitleCues: (List<TrailerCue>) -> Unit,
    private val setNextEpisode: (String?) -> Unit,
    private val setSeasonPosterUrl: (String?) -> Unit,
    private val getMetaDetail: suspend (String, String) -> MetaDetail?,
    private val parseSeasonEpisode: (String, String) -> String?,
    private val prefetchDirectPlayback: (Meta, MetaDetail?) -> Unit,
    private val activeProfile: () -> UserProfile? = { null },
    private val getTrailers: suspend (String, String, String) -> List<DetailTrailer> = { _, _, _ -> emptyList() },
    private val dispatchHeadless: suspend (Any) -> NativeHeadlessEngineResult
) {
    private var rotationJob: Job? = null
    private var prefetchJob: Job? = null
    private var trailerJob: Job? = null
    private val enrichedCache = LruCache<String, Meta>(50)

    fun reset() {
        rotationJob?.cancel()
        prefetchJob?.cancel()
        trailerJob?.cancel()
        setPool(emptyList())
        setIndex(0)
        setMovie(null)
        setLogo(null)
        setTrailerUrl(null)
        setTrailerSubtitleCues(emptyList())
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
        val currentMovie = movie ?: return
        val previous = watchlistValue()
        setWatchlist(!previous)
        scope.launch {
            try {
                watchlistManager.toggleWatchlist(currentMovie)
                setWatchlist(watchlistManager.isInWatchlist(currentMovie.id))
            } catch (e: Exception) {
                setWatchlist(previous)
            }
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
        trailerJob?.cancel()
        setTrailerUrl(null)
        setTrailerSubtitleCues(emptyList())
        val cacheKey = HomeBillboardRanking.contentIdentityKey(item)
        val cached = enrichedCache.get(cacheKey)
        if (cached != null) {
            setMovie(cached)
            setLogo(cached.logo)
            setWatchlist(watchlistManager.isInWatchlist(item.id))
            setNextEpisode(null)
            setSeasonPosterUrl(null)
        } else {
            setMovie(item)
            setLogo(item.logo)
            setWatchlist(watchlistManager.isInWatchlist(item.id))
            scope.launch(Dispatchers.IO) { enrichAndPublish(item) }
        }
        maybeAutoPlayTrailer(item)
    }

    private fun maybeAutoPlayTrailer(item: Meta) {
        if (activeProfile()?.safeTrailerOnHomeHeroEnabled != true) return
        val delaySeconds = activeProfile()?.safeTrailerOnHomeHeroDelaySeconds ?: 4
        val cacheKey = HomeBillboardRanking.contentIdentityKey(item)
        trailerJob = scope.launch {
            if (delaySeconds > 0) delay(delaySeconds * 1000L)
            val isStillActive = HomeBillboardRanking.contentIdentityKey(pool().getOrNull(index()) ?: return@launch) == cacheKey
            if (!isStillActive) return@launch
            val videoId = getTrailers(item.type, item.id, language())
                .firstOrNull { it.url.extractYoutubeVideoId() != null }
                ?.url?.extractYoutubeVideoId() ?: return@launch
            val resolution = resolveYoutubeTrailerViaCore(videoId, dispatchHeadless) as? TrailerResolveResult.Ok ?: return@launch
            val stillActive = HomeBillboardRanking.contentIdentityKey(pool().getOrNull(index()) ?: return@launch) == cacheKey
            if (!stillActive) return@launch
            setTrailerUrl(resolution.data.streamUrl)
            setTrailerSubtitleCues(resolveTrailerSubtitleCues(resolution.data.subtitles, activeProfile()?.safeLanguage))
        }
    }

    private suspend fun enrichAndPublish(item: Meta) {
        val lang = language()
        val cacheKey = HomeBillboardRanking.contentIdentityKey(item)
        try {
            val detail = withTimeoutOrNull(4000) { getMetaDetail(item.type, item.id) } ?: return
            val isSeries = item.type == "series" || item.type == "tv"
            val videos = detail.videos

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

internal suspend fun resolvePlayableTrailerUrl(
    trailers: List<DetailTrailer>,
    dispatchHeadless: suspend (Any) -> NativeHeadlessEngineResult
): String? {
    val videoId = trailers.firstOrNull { it.url.extractYoutubeVideoId() != null }?.url?.extractYoutubeVideoId()
    if (videoId != null) {
        return (resolveYoutubeTrailerViaCore(videoId, dispatchHeadless) as? TrailerResolveResult.Ok)?.data?.streamUrl
    }
    return trailers.firstOrNull { it.url.isDirectVideoPreviewUrl() }?.url
}

private suspend fun resolveYoutubeTrailerViaCore(
    videoId: String,
    dispatchHeadless: suspend (Any) -> NativeHeadlessEngineResult
): TrailerResolveResult {
    val requestId = java.util.UUID.randomUUID().toString()
    val result = dispatchHeadless(mapOf("type" to "trailerResolveRequested", "requestId" to requestId, "videoId" to videoId))
    val resolution = (result.state["trailer"] as? Map<*, *>)
        ?.get("resolutions") as? Map<*, *>
        ?: return TrailerResolveResult.Failed
    val entry = resolution[requestId] as? Map<*, *> ?: return TrailerResolveResult.Failed
    if (entry["status"] != "ok") return TrailerResolveResult.Failed
    val streamUrl = entry["streamUrl"] as? String ?: return TrailerResolveResult.Failed
    val subtitles = (entry["subtitles"] as? List<*>).orEmpty().mapNotNull { raw ->
        val track = raw as? Map<*, *> ?: return@mapNotNull null
        TrailerSubtitle(
            languageTag = track["languageTag"] as? String ?: "und",
            label = track["label"] as? String ?: "",
            url = track["url"] as? String ?: return@mapNotNull null,
            mimeType = track["mimeType"] as? String ?: "text/vtt",
            isAuto = track["isAuto"] as? Boolean ?: false
        )
    }
    return TrailerResolveResult.Ok(
        TrailerResult(
            streamUrl = streamUrl,
            audioUrl = entry["audioUrl"] as? String,
            subtitles = subtitles,
            streamMimeType = null
        )
    )
}

private val subtitleHttpClient = OkHttpClient()
private val subtitleGson = Gson()

internal suspend fun resolveTrailerSubtitleCues(
    subtitles: List<TrailerSubtitle>,
    preferredLanguage: String?
): List<TrailerCue> {
    if (subtitles.isEmpty()) return emptyList()
    return withContext(Dispatchers.IO) {
        runCatching {
            val selectedJson = FluxaCoreNative.trailerSubtitleSelectionPlan(
                tracksJson = subtitleGson.toJson(subtitles),
                preferred = preferredLanguage,
                secondary = null,
                systemLanguage = java.util.Locale.getDefault().toLanguageTag()
            ) ?: return@withContext emptyList()
            val selected = subtitleGson.fromJson(selectedJson, TrailerSubtitle::class.java) ?: return@withContext emptyList()
            val normalizedUrl = FluxaCoreNative.normalizeTrailerSubtitleUrl(selected.url)
            val body = subtitleHttpClient.newCall(Request.Builder().url(normalizedUrl).build()).execute().use { response ->
                if (!response.isSuccessful) return@withContext emptyList()
                response.body.string()
            }
            val cuesJson = FluxaCoreNative.parseTrailerSubtitleCues(body)
            subtitleGson.fromJson<List<TrailerCue>>(cuesJson, object : TypeToken<List<TrailerCue>>() {}.type).orEmpty()
        }.getOrDefault(emptyList())
    }
}
