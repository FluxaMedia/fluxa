package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.fluxa.app.core.fromState
import com.fluxa.app.core.fromStateList
import com.fluxa.app.core.rust.FluxaAndroidHeadlessEnvironment
import com.fluxa.app.core.rust.StreamProgressUpdate
import com.fluxa.app.core.StremioId
import com.fluxa.app.core.rust.FluxaHeadlessRuntimeFactory
import com.fluxa.app.domain.discovery.supportsStremioResource
import com.google.gson.Gson
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.withContext

import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject

@HiltViewModel
class DetailViewModel @Inject constructor(
    private val headlessEnvironment: FluxaAndroidHeadlessEnvironment,
    private val gson: Gson
) : ViewModel() {

    private val _uiState = MutableStateFlow(DetailUiState())
    val uiState: StateFlow<DetailUiState> = _uiState.asStateFlow()

    private var currentProfile: UserProfile? = null
    private var currentSeriesLookupId: String? = null

    private val headlessRuntime = FluxaHeadlessRuntimeFactory.createUniFfi(headlessEnvironment)

    init {
        viewModelScope.launch {
            headlessEnvironment.streamProgressFlow.collect { progress ->
                if (_uiState.value.isLoadingStreams) {
                    val sel = _uiState.value.selectedAddon
                    _uiState.update {
                        it.copy(
                            streams = progress.streams,
                            filteredStreams = if (sel == null) progress.streams
                                             else progress.streams.filter { s -> s.addonName == sel },
                            availableAddons = progress.completedAddonNames,
                            loadingAddonNames = progress.loadingAddonNames
                        )
                    }
                }
            }
        }
    }

    fun loadDetail(type: String, id: String, profile: UserProfile? = null, sourceAddonTransportUrl: String? = null, sourceAddonCatalogType: String? = null, initialMeta: Meta? = null) {
        currentProfile = profile
        currentSeriesLookupId = normalizeSeriesLookupId(id)
        val lang = profile?.language ?: "en"
        val initialDetail = initialMeta?.toInitialDetailFallback()
        viewModelScope.launch {
            _uiState.update {
                it.copy(
                    detail = initialDetail,
                    seasonEpisodes = emptyList(),
                    localWatchedVideoIds = emptySet(),
                    savedPlayback = null,
                    similarItems = emptyList(),
                    trailers = emptyList(),
                    hasStreamProviders = true,
                    isLoading = true
                )
            }

            try {
                launch { refreshStreamProviderState(profile) }
                val headlessResult = headlessRuntime.dispatch(
                    mapOf(
                        "type" to "detailLoadRequested",
                        "contentType" to type,
                        "id" to id,
                        "language" to lang,
                        "sourceAddonTransportUrl" to sourceAddonTransportUrl,
                        "sourceAddonCatalogType" to sourceAddonCatalogType,
                        "profile" to profile
                    )
                )
                val detailState = headlessResult.state["detail"] as? Map<*, *>
                val result = gson.fromState<MetaDetail>(detailState?.get("meta"))
                _uiState.update {
                    it.copy(
                        detail = result ?: initialDetail,
                        trailers = gson.fromStateList(detailState?.get("trailers")),
                        isLoading = false
                    )
                }

                val effectiveType = if (id.startsWith("cs3:") && result?.type != null) result.type else type
                if (effectiveType == "series") {
                    currentSeriesLookupId = normalizeSeriesLookupId(result?.id ?: id)
                }

                val resolvedId = result?.id ?: id
                android.util.Log.d("CS3Detail", "loadDetail: type=$type effectiveType=$effectiveType resolvedId=${resolvedId.take(30)} resultVideos=${result?.videos?.size ?: "null"}")
                val localState = headlessRuntime.dispatch(
                    mapOf(
                        "type" to "detailLocalStateRequested",
                        "primaryId" to resolvedId,
                        "fallbackId" to id,
                        "contentType" to type,
                        "profile" to profile
                    )
                ).state["detail"] as? Map<*, *>

                _uiState.update {
                    it.copy(
                        savedPlayback = gson.fromState(localState?.get("savedPlayback")),
                        localWatchedVideoIds = gson.fromStateList<String>(localState?.get("localWatchedVideoIds")).toSet(),
                        hasStreamProviders = localState?.get("hasStreamProviders") as? Boolean ?: false,
                        userAddons = gson.fromStateList(localState?.get("userAddons")),
                        isInWatchlist = localState?.get("isInWatchlist") as? Boolean ?: false,
                        feedback = localState?.get("feedback") as? Boolean
                    )
                }

                launch {
                    if (effectiveType == "series") loadSeason(resolvedId, 1)
                    val secondary = headlessRuntime.dispatch(
                        mapOf(
                            "type" to "detailSecondaryRequested",
                            "contentType" to effectiveType,
                            "id" to resolvedId,
                            "language" to lang,
                            "profile" to profile
                        )
                    ).state["detail"] as? Map<*, *>
                    _uiState.update {
                        it.copy(
                            watchedVideoIds = gson.fromStateList(secondary?.get("watchedVideoIds")),
                            similarItems = gson.fromStateList(secondary?.get("similarItems")),
                            trailers = if (it.trailers.isEmpty()) gson.fromStateList(secondary?.get("trailers")) else it.trailers
                        )
                    }
                }

                launch {
                    val streamLookupId = if (effectiveType == "series") {
                        currentSeriesLookupId ?: normalizeSeriesLookupId(resolvedId)
                    } else {
                        resolvedId
                    }
                    headlessRuntime.dispatch(
                        mapOf(
                            "type" to "detailPrefetchRequested",
                            "contentType" to effectiveType,
                            "id" to id,
                            "streamLookupId" to streamLookupId,
                            "title" to result?.name,
                            "originalName" to result?.originalName,
                            "year" to result?.releaseInfo?.toIntOrNull(),
                            "language" to lang,
                            "profile" to profile
                        )
                    )
                }

            } catch (e: Exception) {
                e.printStackTrace()
                _uiState.update { it.copy(isLoading = false) }
            }
        }
    }

    private fun Meta.toInitialDetailFallback(): MetaDetail {
        return MetaDetail(
            id = id,
            type = type,
            name = name,
            genres = genres,
            poster = poster,
            background = background,
            logo = logo,
            description = description,
            releaseInfo = releaseInfo,
            released = released,
            runtime = runtime,
            videos = videos,
            trailers = trailers,
            imdbRating = imdbRating,
            ageRating = ageRating,
            ratings = ratings,
            cast = cast,
            director = null,
            links = null,
            status = null,
            seasonsCount = seasonsCount,
            platforms = null,
            awards = awards,
            originalLanguage = originalLanguage,
            originalName = originalName,
            country = null,
            productionCompanies = null,
            networks = null,
            collectionName = null,
            collectionId = null,
            collectionParts = null,
            seasonPosters = null
        )
    }

    private suspend fun refreshStreamProviderState(profile: UserProfile?) {
        val detail = headlessRuntime.dispatch(
            mapOf(
                "type" to "detailLocalStateRequested",
                "primaryId" to (currentSeriesLookupId ?: ""),
                "fallbackId" to null,
                "contentType" to "series",
                "profile" to profile
            )
        ).state["detail"] as? Map<*, *>
        _uiState.update { it.copy(hasStreamProviders = detail?.get("hasStreamProviders") as? Boolean ?: false) }
    }

    fun loadSeason(id: String, seasonNumber: Int) {
        android.util.Log.d("CS3Detail", "loadSeason called: id=${id.take(40)}, season=$seasonNumber, detailVideos=${_uiState.value.detail?.videos?.size ?: "null"}")
        val lang = currentProfile?.language ?: "en"
        viewModelScope.launch {
            _uiState.update { it.copy(seasonEpisodes = emptyList()) }

            val allVideos = _uiState.value.detail?.videos.orEmpty()
            if (allVideos.isNotEmpty()) {
                android.util.Log.d("CS3Detail", "loadSeason from detail: detail=${_uiState.value.detail?.name}, videosInDetail=${allVideos.size}")
                _uiState.update { it.copy(seasonEpisodes = seasonVideosForSelection(allVideos, seasonNumber)) }
                return@launch
            }

            val seasonId = currentSeriesLookupId ?: normalizeSeriesLookupId(_uiState.value.detail?.id ?: id)
            val profile = currentProfile
            val result = headlessRuntime.dispatch(
                mapOf(
                    "type" to "detailSeasonRequested",
                    "seriesId" to seasonId,
                    "season" to seasonNumber,
                    "profile" to profile,
                    "language" to lang
                )
            )
            val detail = result.state["detail"] as? Map<*, *>
            _uiState.update { it.copy(seasonEpisodes = gson.fromStateList(detail?.get("seasonEpisodes"))) }
        }
    }

    private fun seasonVideosForSelection(videos: List<Video>, seasonNumber: Int): List<Video> {
        val hasSeasonData = videos.any { it.season != null && it.season > 0 }
        if (!hasSeasonData) return videos
        val filtered = videos.filter { it.season == seasonNumber }
        if (filtered.isNotEmpty()) return filtered
        val firstAvailable = videos.mapNotNull { it.season }.filter { it > 0 }.minOrNull()
        return if (firstAvailable != null) videos.filter { it.season == firstAvailable } else videos
    }

    fun markEpisodeWatched(seriesId: String, episode: Video, watched: Boolean = true) {
        val videoId = episode.id
        if (videoId.isBlank()) return
        viewModelScope.launch {
            _uiState.update {
                it.copy(localWatchedVideoIds = if (watched) it.localWatchedVideoIds + videoId
                                               else it.localWatchedVideoIds - videoId)
            }
            headlessRuntime.dispatch(
                mapOf(
                    "type" to "markWatchedRequested",
                    "seriesId" to seriesId,
                    "videoIds" to listOf(videoId),
                    "watched" to watched,
                    "meta" to (_uiState.value.detail?.toMeta()),
                    "episodes" to listOf(episode),
                    "profile" to currentProfile
                )
            )
        }
    }

    fun markSeasonWatched(seriesId: String, episodes: List<Video>, watched: Boolean? = null) {
        if (episodes.isEmpty()) return
        viewModelScope.launch {
            val currentlyWatched = _uiState.value.localWatchedVideoIds
            val targetIds = episodes.map { it.id }.filter { it.isNotBlank() }.toSet()
            if (targetIds.isEmpty()) return@launch
            val shouldMarkWatched = watched ?: !targetIds.all { it in currentlyWatched }
            _uiState.update {
                it.copy(localWatchedVideoIds = if (shouldMarkWatched) it.localWatchedVideoIds + targetIds
                                               else it.localWatchedVideoIds - targetIds)
            }
            headlessRuntime.dispatch(
                mapOf(
                    "type" to "markWatchedRequested",
                    "seriesId" to seriesId,
                    "videoIds" to targetIds.toList(),
                    "watched" to shouldMarkWatched,
                    "meta" to (_uiState.value.detail?.toMeta()),
                    "episodes" to episodes,
                    "profile" to currentProfile
                )
            )
        }
    }

    private fun normalizeSeriesLookupId(rawId: String): String {
        return StremioId.normalizeSeriesLookupId(rawId)
    }

    fun abandonShow(profile: UserProfile) {
        if (profile.isGuest) return
        viewModelScope.launch {
            val currentDetail = _uiState.value.detail ?: return@launch
            headlessRuntime.dispatch(
                mapOf(
                    "type" to "clearPlaybackProgressRequested",
                    "profile" to profile,
                    "meta" to currentDetail.toMeta()
                )
            )
            loadDetail(currentDetail.type, currentDetail.id, profile)
        }
    }

    fun toggleWatchlist() {
        viewModelScope.launch {
            val currentDetail = _uiState.value.detail ?: return@launch
            val meta = currentDetail.toMeta()
            headlessRuntime.dispatch(
                mapOf(
                    "type" to "toggleWatchlistRequested",
                    "item" to meta
                )
            )
            val detail = headlessRuntime.state.value["detail"] as? Map<*, *>
            _uiState.update { it.copy(isInWatchlist = detail?.get("isInWatchlist") as? Boolean ?: false) }
        }
    }

    fun setFeedback(isLike: Boolean) {
        if (currentProfile?.isGuest == true) return
        viewModelScope.launch {
            val currentDetail = _uiState.value.detail ?: return@launch
            val newValue = if (_uiState.value.feedback == isLike) null else isLike
            val result = headlessRuntime.dispatch(
                mapOf(
                    "type" to "setFeedbackRequested",
                    "id" to currentDetail.id,
                    "value" to newValue,
                    "meta" to currentDetail.toMeta()
                )
            )
            val detail = result.state["detail"] as? Map<*, *>
            _uiState.update { it.copy(feedback = detail?.get("feedback") as? Boolean) }
        }
    }

    private fun MetaDetail.toMeta() = Meta(
        id = id, name = name, type = type, poster = poster, background = background,
        logo = logo, description = description, imdbRating = imdbRating, releaseInfo = releaseInfo,
        released = released, originalLanguage = originalLanguage, originalName = originalName, videos = videos, trailers = trailers
    )

    fun setSelectedAddon(addon: String?) {
        val currentStreams = _uiState.value.streams
        _uiState.update {
            it.copy(
                selectedAddon = addon,
                filteredStreams = if (addon == null) currentStreams
                                 else currentStreams.filter { s -> s.addonName == addon }
            )
        }
    }

    fun fetchStreamsForSelection(type: String, id: String, context: android.content.Context? = null) {
        viewModelScope.launch {
            try {
                val language = currentProfile?.language ?: "en"
                val requestIds = buildStreamRequestIds(type, id, language)
                android.util.Log.d("Detail", " FETCHING STREAMS: type=$type, id=$id, requestIds=$requestIds")
                _uiState.update {
                    it.copy(
                        isLoadingStreams = true,
                        streams = emptyList(),
                        filteredStreams = emptyList(),
                        selectedAddon = null,
                        availableAddons = emptyList(),
                        loadingAddonNames = emptyList()
                    )
                }

                val isCs3Content = id.startsWith("cs3:")
                if (!_uiState.value.hasStreamProviders && !isCs3Content) {
                    showToast(context, AppStrings.t(currentProfile?.language, "auto.no_addons_found"))
                    _uiState.update { it.copy(isLoading = false) }
                    return@launch
                }

                val result = headlessRuntime.dispatch(
                    mapOf(
                        "type" to "detailStreamsRequested",
                        "contentType" to type,
                        "requestIds" to requestIds,
                        "detail" to _uiState.value.detail,
                        "seasonEpisodes" to _uiState.value.seasonEpisodes,
                        "language" to language,
                        "profile" to currentProfile
                    )
                )
                val detail = result.state["detail"] as? Map<*, *>
                applyDetailStreamState(detail)
                val fetchedStreams: List<Stream> = gson.fromStateList(detail?.get("streams"))
                val resolvedRequestId = detail?.get("resolvedRequestId")?.toString() ?: requestIds.first()

                if (fetchedStreams.isNotEmpty()) {
                    android.util.Log.d("Detail", " FOUND ${fetchedStreams.size} streams for $resolvedRequestId")
                } else {
                    android.util.Log.w("Detail", " NO STREAMS found for ${requestIds.joinToString()}")
                    showToast(context, AppStrings.t(currentProfile?.language, "auto.no_sources_found"))
                }
            } catch (e: Exception) {
                android.util.Log.e("Detail", " FATAL FETCH ERROR for $id", e)
            } finally {
                _uiState.update { it.copy(isLoadingStreams = false, loadingAddonNames = emptyList()) }
            }
        }
    }

    private fun applyDetailStreamState(detail: Map<*, *>?) {
        val allStreams: List<Stream> = gson.fromStateList(detail?.get("streams"))
        val sel = _uiState.value.selectedAddon
        _uiState.update {
            it.copy(
                streams = allStreams,
                filteredStreams = if (sel == null) allStreams else allStreams.filter { s -> s.addonName == sel },
                availableAddons = gson.fromStateList(detail?.get("availableAddons")),
                loadingAddonNames = gson.fromStateList(detail?.get("loadingAddonNames")),
                hasStreamProviders = detail?.get("hasStreamProviders") as? Boolean ?: it.hasStreamProviders
            )
        }
    }

    private fun buildStreamRequestIds(type: String, id: String, language: String): List<String> {
        if (id.startsWith("cs3:")) return listOf(id)
        val detailId = _uiState.value.detail?.id
        val canonicalBaseId = resolveCanonicalStreamBaseId(type, id, language)
        return StremioId.streamRequestIds(
            type = type,
            id = id,
            detailId = detailId,
            currentSeriesLookupId = currentSeriesLookupId,
            canonicalBaseId = canonicalBaseId
        )
    }

    private fun resolveCanonicalStreamBaseId(type: String, id: String, language: String): String? {
        StremioId.imdbId(id)?.let { return it }
        StremioId.imdbId(_uiState.value.detail?.id)?.let { return it }
        StremioId.imdbId(currentSeriesLookupId)?.let { return it }
        return null
    }

    suspend fun getSubtitlesFromAddon(baseUrl: String, type: String, id: String, extra: String = ""): List<SubtitleData> {
        val result = headlessRuntime.dispatch(
            mapOf(
                "type" to "addonResourceRequested",
                "transportUrl" to baseUrl,
                "resource" to "subtitles",
                "contentType" to type,
                "id" to id,
                "extra" to mapOf("extraArgs" to extra)
            )
        )
        val addons = result.state["addons"] as? Map<*, *>
        return gson.fromStateList(addons?.get("lastResourceResult"))
    }

    fun downloadEpisodes(episodes: List<Video?>, context: android.content.Context? = null) {
        val detail = _uiState.value.detail ?: return
        val profile = currentProfile
        val targets = episodes.filterNotNull().filter { !detailIsUpcoming(it.released) }
        if (targets.isEmpty()) return
        viewModelScope.launch {
            var queued = 0
            targets.forEach { episode ->
                if (enqueueEpisodeDownload(detail, episode, profile)) queued += 1
            }
            val key = if (queued > 0) "downloads.queued" else "downloads.failed"
            showToast(context, AppStrings.t(profile?.safeLanguage, key), android.widget.Toast.LENGTH_SHORT)
        }
    }

    private suspend fun enqueueEpisodeDownload(detail: MetaDetail, episode: Video, profile: UserProfile?): Boolean {
        val requestId = episode.id.takeIf { it.isNotBlank() } ?: return false
        val language = profile?.safeLanguage ?: "en"
        val streams = fetchStreamsForDownload(detail.type, requestId, language)
        if (streams.isEmpty()) return false
        val mode = profile?.safeDownloadSourceSelectionMode ?: STREAM_SOURCE_MODE_FIRST
        val effectiveMode = if (mode == STREAM_SOURCE_MODE_MANUAL) STREAM_SOURCE_MODE_FIRST else mode
        val selectedIndex = selectStreamIndex(
            streams = streams,
            currentVideoId = requestId,
            initialStreamIndex = 0,
            savedUrl = null,
            savedTitle = null,
            sourceSelectionMode = effectiveMode,
            regexPattern = profile?.safeDownloadSourceRegexPattern,
            preferredBingeGroup = null
        ).takeIf { it in streams.indices } ?: 0
        val stream = streams.getOrNull(selectedIndex) ?: return false
        val subtitle = selectDownloadSubtitle(profile, detail.type, requestId, stream)
        val result = headlessRuntime.dispatch(
            mapOf(
                "type" to "offlineDownloadRequested",
                "meta" to detail.toMeta(),
                "video" to episode,
                "videoId" to requestId,
                "stream" to stream,
                "subtitle" to subtitle,
                "profileId" to profile?.id,
                "language" to language
            )
        )
        val offline = result.state["offline"] as? Map<*, *>
        return offline?.get("error") == null
    }

    private suspend fun fetchStreamsForDownload(type: String, id: String, language: String): List<Stream> {
        val result = headlessRuntime.dispatch(
            mapOf(
                "type" to "detailStreamsRequested",
                "contentType" to type,
                "requestIds" to buildStreamRequestIds(type, id, language),
                "detail" to _uiState.value.detail,
                "seasonEpisodes" to _uiState.value.seasonEpisodes,
                "language" to language,
                "profile" to currentProfile
            )
        )
        val detail = result.state["detail"] as? Map<*, *>
        return gson.fromStateList(detail?.get("streams"))
    }

    private suspend fun selectDownloadSubtitle(
        profile: UserProfile?,
        type: String,
        id: String,
        stream: Stream
    ): OfflineSubtitleOption? {
        val setting = profile?.safeDownloadSubtitleLanguage ?: "preferred"
        if (setting == "off") return null
        val preferred = if (setting == "preferred") {
            profile?.safePreferredSubtitleLanguage
        } else {
            setting
        }?.substringBefore('-')?.substringBefore('_')?.lowercase(java.util.Locale.ROOT)
        val options = downloadSubtitleOptionsForStream(type, id, stream)
        return if (preferred.isNullOrBlank()) {
            options.firstOrNull()
        } else {
            options.firstOrNull { it.language?.substringBefore('-')?.substringBefore('_')?.lowercase(java.util.Locale.ROOT) == preferred }
                ?: options.firstOrNull()
        }
    }

    private suspend fun downloadSubtitleOptionsForStream(type: String, id: String, stream: Stream): List<OfflineSubtitleOption> {
        val inline = stream.subtitles.orEmpty().mapNotNull { subtitle ->
            val url = subtitle.subtitleUrl() ?: return@mapNotNull null
            val language = subtitle.subtitleLanguages().firstOrNull()?.lowercase(java.util.Locale.ROOT)
            OfflineSubtitleOption(
                label = listOfNotNull(language, stream.addonName).joinToString(" - ").ifBlank { url },
                language = language,
                url = url
            )
        }
        val remote = withContext(Dispatchers.IO) {
            supervisorScope {
                _uiState.value.userAddons
                    .filter { it.supportsStremioResource("subtitles", type, id) }
                    .map { addon ->
                        async {
                            getSubtitlesFromAddon(addon.transportUrl, type, id, stream.subtitleExtraArgs()).mapNotNull { subtitle ->
                                val url = subtitle.subtitleUrl() ?: return@mapNotNull null
                                val language = subtitle.subtitleLanguages().firstOrNull()?.lowercase(java.util.Locale.ROOT)
                                OfflineSubtitleOption(
                                    label = listOfNotNull(language, addon.manifest.name).joinToString(" - ").ifBlank { url },
                                    language = language,
                                    url = url
                                )
                            }
                        }
                    }
                    .map { it.await() }
                    .flatten()
            }
        }
        return inline + remote
    }

    private fun showToast(context: android.content.Context?, message: String, length: Int = android.widget.Toast.LENGTH_LONG) {
        context?.let { android.widget.Toast.makeText(it, message, length).show() }
    }

    fun resetAutoSelect() {
        _uiState.update { it.copy(autoSelectedStream = null) }
    }

    override fun onCleared() {
        headlessRuntime.close()
        super.onCleared()
    }
}
