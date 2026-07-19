package com.fluxa.app.ui.catalog

import android.util.LruCache
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.TraktWatchedState
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull

internal class HomeContinueWatchingCoordinator(
    private val repository: StremioRepository,
    private val watchlistManager: WatchlistManager,
    private val scope: CoroutineScope,
    private val activeProfile: () -> UserProfile?,
    private val localItems: () -> List<Meta>,
    private val externalItems: () -> List<Meta>,
    private val watchedState: () -> TraktWatchedState,
    private val setLocalItems: (List<Meta>) -> Unit,
    private val setExternalItems: (List<Meta>) -> Unit,
    private val setWatchlistState: (List<Meta>) -> Unit,
    private val setTraktUpdatedAt: (Long) -> Unit,
    private val refreshDynamicRows: () -> Unit,
    private val getConfiguredMetaDetail: suspend (String, String, String) -> MetaDetail?,
    private val getSeasonEpisodes: suspend (String, Int, String) -> List<Video>
) {
    private val artworkCache = LruCache<String, Pair<String?, String?>>(80)
    private val upcomingCache = mutableMapOf<String, Boolean>()

    fun isUpcoming(meta: Meta): Boolean = upcomingCache[upcomingCacheKey(meta)] == true

    fun classifyUpcoming(items: List<Meta>) {
        if (activeProfile()?.safeUpcomingRowEnabled != true) return
        val lang = activeProfile()?.safeLanguage ?: "en"
        val candidates = items.filter { meta ->
            val isSeries = meta.type == "series" || meta.type == "tv" || meta.type == "anime"
            val isUpNext = isSeries && !meta.lastVideoId.isNullOrBlank() &&
                (meta.timeOffset ?: 0L) <= 0L && (meta.duration ?: 0L) <= 0L
            isUpNext && upcomingCacheKey(meta) !in upcomingCache
        }
        if (candidates.isEmpty()) return

        scope.launch(Dispatchers.IO) {
            var changed = false
            candidates.forEach { meta ->
                val locator = meta.lastVideoId?.let(::parseEpisodeLocator) ?: return@forEach
                val video = runCatching {
                    getSeasonEpisodes(meta.id, locator.first, lang).firstOrNull { it.number == locator.second }
                }.getOrNull() ?: return@forEach
                val released = FluxaCoreNative.isEpisodeReleased(video, System.currentTimeMillis())
                upcomingCache[upcomingCacheKey(meta)] = !released
                changed = true
            }
            if (changed) refreshDynamicRows()
        }
    }

    private fun upcomingCacheKey(meta: Meta): String = "${meta.id}:${meta.lastVideoId}"

    fun buildItems(lang: String, playbackController: HomePlaybackController): List<Meta> {
        val source = activeProfile()?.safeContinueWatchingSource ?: "fluxa"
        if (source != "fluxa") {
            val providerItems = when (source) {
                "stremio" -> localItems()
                "trakt" -> externalItems().filter { it.reason.equals("Trakt.tv", ignoreCase = true) }
                "simkl" -> externalItems().filter { it.reason.equals("Simkl", ignoreCase = true) }
                "nuvio" -> externalItems().filter { it.reason.equals("Nuvio", ignoreCase = true) }
                "anilist" -> externalItems().filter { it.reason.equals("AniList", ignoreCase = true) }
                else -> externalItems() + localItems()
            }
            val filteredProviderItems = providerItems.filterNot(playbackController::isForgotten)
            val ranked = FluxaCoreNative.filterHomeContinueWatching(filteredProviderItems, watchedState())
            classifyUpcoming(ranked)
            return ranked.map { assignHomeBadge(it, lang) }
        }
        val sourceItems = externalItems() + localItems()
        val merged = ContinueWatchingListMerger.mergeDuplicates(sourceItems)
            .filterNot(playbackController::isForgotten)
        val filtered = FluxaCoreNative.filterHomeContinueWatching(merged, watchedState())
        classifyUpcoming(filtered)
        return filtered.map { assignHomeBadge(it, lang) }
    }

    suspend fun fetchExternal(profile: UserProfile?): List<Meta> {
        if (profile == null) return emptyList()
        val items = withTimeoutOrNull(8_000L) {
            repository.getExternalContinueWatching(profile, profile.safeLanguage)
        }
        if (!profile.traktAccessToken.isNullOrBlank() && items != null) {
            setTraktUpdatedAt(System.currentTimeMillis())
        }
        if (items != null) {
            watchlistManager.replaceExternalContinueWatching(setOf("trakt", "mal", "simkl", "anilist"), items)
        }
        return items ?: externalItems()
    }

    fun prefetchArtwork(items: List<Meta>) {
        val lang = activeProfile()?.safeLanguage ?: "en"
        val targets = items.filter {
            val isSeries = it.type == "series" || it.type == "tv" || it.type == "anime"
            val hasEpisode = isSeries && !it.lastVideoId.isNullOrBlank()
            val hasOnlyTitleArtwork = it.continueWatchingPoster.isNullOrBlank() ||
                it.continueWatchingPoster == it.poster ||
                it.continueWatchingPoster == it.background ||
                it.continueWatchingBackground == it.background
            (it.timeOffset ?: 0L) > 0L &&
                (
                    (hasEpisode && hasOnlyTitleArtwork) ||
                        (it.type == "movie" && it.continueWatchingPoster.isNullOrBlank())
                )
        }
        if (targets.isEmpty()) return

        scope.launch(Dispatchers.IO) {
            val updated = targets.map { meta ->
                val cacheKey = "episode_still_v2:${meta.id}:${meta.lastVideoId}"
                val cached = artworkCache.get(cacheKey)
                if (cached != null) {
                    meta.copy(
                        continueWatchingPoster = cached.first,
                        continueWatchingBackground = cached.second
                    )
                } else {
                    val detail = runCatching {
                        getConfiguredMetaDetail(meta.type, meta.id, lang)
                    }.getOrNull()
                    val episodeLocator = meta.lastVideoId?.let(::parseEpisodeLocator)
                    val seasonEpisodeArtwork = if (episodeLocator != null && (meta.type == "series" || meta.type == "tv" || meta.type == "anime")) {
                        runCatching {
                            getSeasonEpisodes(meta.id, episodeLocator.first, lang)
                                .firstOrNull { it.number == episodeLocator.second }
                                ?.thumbnail
                        }.getOrNull()
                    } else {
                        null
                    }
                    val episode = meta.lastVideoId?.let { videoId ->
                        detail?.videos?.firstOrNull { it.id == videoId }
                            ?: episodeLocator?.let { locator ->
                                detail?.videos?.firstOrNull { it.season == locator.first && it.number == locator.second }
                            }
                    }
                    val artwork = if (meta.type == "movie") {
                        detail?.poster
                    } else {
                        seasonEpisodeArtwork ?: episode?.thumbnail
                    }
                    val background = if (meta.type == "movie") detail?.background else artwork
                    artworkCache.put(cacheKey, artwork to background)
                    if (!artwork.isNullOrBlank() || !background.isNullOrBlank()) {
                        runCatching {
                            watchlistManager.updateContinueWatchingArtwork(meta.id, artwork, background)
                        }
                    }
                    meta.copy(
                        continueWatchingPoster = artwork,
                        continueWatchingBackground = background
                    )
                }
            }
            if (updated.isEmpty()) return@launch

            val merged = localItems().map { existing ->
                updated.firstOrNull { it.id == existing.id } ?: existing
            }
            val externalMerged = externalItems().map { existing ->
                updated.firstOrNull { it.id == existing.id } ?: existing
            }
            if (merged != localItems() || externalMerged != externalItems()) {
                setLocalItems(merged)
                setExternalItems(externalMerged)
                setWatchlistState(merged)
                refreshDynamicRows()
            }
        }
    }

}
