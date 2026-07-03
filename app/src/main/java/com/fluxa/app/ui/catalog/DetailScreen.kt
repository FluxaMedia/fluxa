@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.content.Intent
import android.net.Uri
import androidx.compose.animation.*
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.*
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.*
import coil3.compose.AsyncImage
import com.fluxa.app.R
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.*

internal fun preferredPlayStream(streams: List<Stream>, fallbackIndex: Int? = null, savedUrl: String? = null, savedTitle: String? = null): Stream? {
    if (!savedUrl.isNullOrEmpty()) {
        streams.find { it.playableUrl == savedUrl }?.let { return it }
    }
    if (!savedTitle.isNullOrEmpty()) {
        streams.find { it.title == savedTitle }?.let { return it }
    }
    return streams.getOrNull(fallbackIndex ?: -1)
        ?: streams.firstOrNull()
}

@Composable
fun DetailScreen(
    type: String, 
    id: String, 
    activeProfile: UserProfile?,
    initialProgress: Long? = null,
    lastVideoId: String? = null,
    lastStreamIndex: Int? = null,
    autoPlay: Boolean = false,
    targetSeason: Int? = null,
    targetEpisode: Int? = null,
    onBack: () -> Unit,
    onStreamClick: (Stream, Video?) -> Unit,
    onPlayClick: (Video?) -> Unit,
    onPlayEpisode: (Video) -> Unit,
    onDownloadEpisode: (Video?) -> Unit = {},
    onDownloadEpisodes: (List<Video>) -> Unit = {},
    viewModel: DetailViewModel,
    lastStreamUrl: String? = null,
    lastStreamTitle: String? = null,
    sourceAddonTransportUrl: String? = null,
    sourceAddonCatalogType: String? = null,
    initialMeta: Meta? = null
) {
    val context = LocalContext.current
    val deviceType = LocalDeviceType.current
    val horizontalPadding = if (deviceType == DeviceType.TV) 58.dp else 16.dp

    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val detail = state.detail
    val isLoading = state.isLoading
    val isLoadingStreams = state.isLoadingStreams
    val isInWatchlist = state.isInWatchlist
    val feedback = state.feedback
    val seasonEpisodes = state.seasonEpisodes
    val watchedVideoIds = state.watchedVideoIds
    val localWatchedVideoIds = state.localWatchedVideoIds
    val savedPlayback = state.savedPlayback
    val streams = state.filteredStreams
    val availableAddons = state.availableAddons
    val selectedAddon = state.selectedAddon
    val hasStreamProviders = state.hasStreamProviders
    val similarItems = state.similarItems
    val trailers = state.trailers

    val coroutineScope = rememberCoroutineScope()
    val lang = activeProfile?.safeLanguage ?: "en"
    // For CS3 content the catalog type ("movie") may differ from the loaded detail type ("series").
    // Once detail loads, prefer detail.type so episode/season logic works correctly.
    val effectiveType = detail?.type ?: type
    var selectedSeason by remember(id, type, targetSeason) { mutableIntStateOf(targetSeason ?: 1) }
    var selectedEpisode by remember(id, type, lastVideoId, targetSeason, targetEpisode) { mutableStateOf<Video?>(null) }
    var focusedEpisode by remember(id, type, lastVideoId, targetSeason, targetEpisode) { mutableStateOf<Video?>(null) }
    var pendingStreamEpisodeId by remember(id, type) { mutableStateOf<String?>(null) }
    var loadedStreamEpisodeId by remember(id, type) { mutableStateOf<String?>(null) }
    val displaySelectedEpisode = remember(selectedEpisode, seasonEpisodes, selectedSeason) {
        selectedEpisode?.let { episode ->
            seasonEpisodes.firstOrNull { it.id == episode.id }
                ?: seasonEpisodes.firstOrNull { it.number == episode.number && (it.season ?: selectedSeason) == (episode.season ?: selectedSeason) }
                ?: episode
        }
    }
    val effectiveLastVideoId = lastVideoId ?: savedPlayback?.lastVideoId
    val effectiveInitialProgress = initialProgress ?: savedPlayback?.timeOffset
    // Tracks whether the resume-from-savedPlayback auto-selection has been applied or is no longer needed.
    // Pre-consumed when the caller already provides an explicit target (targetSeason / lastVideoId).
    var autoResumeConsumed by remember(id, type, targetSeason, lastVideoId) {
        mutableStateOf(targetSeason != null || lastVideoId != null)
    }
    val effectiveWatchedVideoIds = remember(watchedVideoIds, localWatchedVideoIds) {
        watchedVideoIds.toSet() + localWatchedVideoIds
    }

    val lazyListState = rememberLazyListState()

    LaunchedEffect(id, type, targetSeason, targetEpisode, lastVideoId, sourceAddonTransportUrl, sourceAddonCatalogType, initialMeta) {
        selectedEpisode = null
        focusedEpisode = null
        pendingStreamEpisodeId = null
        loadedStreamEpisodeId = null
        viewModel.setSelectedAddon(null)
        viewModel.loadDetail(type, id, activeProfile, sourceAddonTransportUrl, sourceAddonCatalogType, initialMeta)
        selectedSeason = targetSeason ?: 1
    }

    LaunchedEffect(detail, seasonEpisodes) {
        if (effectiveType == "series") {
            val currentSeasonEpisodes = seasonEpisodes.filter { (it.season ?: selectedSeason) == selectedSeason }
            if (currentSeasonEpisodes.isEmpty()) return@LaunchedEffect
        }
        if (selectedEpisode == null) {
            val currentSeasonEpisodes = seasonEpisodes.filter { (it.season ?: selectedSeason) == selectedSeason }
            if (targetEpisode != null) {
                selectedEpisode = currentSeasonEpisodes.find { it.number == targetEpisode }
            }
            if (selectedEpisode == null && lastVideoId != null) {
                selectedEpisode = currentSeasonEpisodes.find { it.id == lastVideoId }
            }
            if (selectedEpisode == null) {
                selectedEpisode = currentSeasonEpisodes.firstOrNull { !detailIsUpcoming(it.released) }
                    ?: currentSeasonEpisodes.firstOrNull()
            }
            selectedEpisode?.season?.takeIf { it == selectedSeason }?.let {
                selectedSeason = it 
                //  AUTO-SCROLL TO SEASON
                if (it > 1) {
                    coroutineScope.launch {
                        lazyListState.animateScrollToItem(if (effectiveType == "series") 1 else 0) // Focus back on meta info if needed OR to episodes row
                    }
                }
            }
        }
    }

    var activeTab by remember(detail?.id, effectiveType) { mutableStateOf(if (effectiveType == "series") "episodes" else "similar") }
    var hasFetchedStreams by remember(detail?.id, selectedEpisode?.id) { mutableStateOf(false) }

    LaunchedEffect(detail?.id, effectiveType, trailers, similarItems) {
        val validTabs = buildSet {
            if (trailers.isNotEmpty()) add("trailers")
            if (effectiveType == "series") add("episodes")
            if (hasStreamProviders || streams.isNotEmpty()) add("sources")
            if (similarItems.isNotEmpty()) add("similar")
        }
        if (activeTab !in validTabs) {
            activeTab = when {
                effectiveType == "series" -> "episodes"
                trailers.isNotEmpty() -> "trailers"
                similarItems.isNotEmpty() -> "similar"
                else -> activeTab
            }
        }
    }
    
    LaunchedEffect(activeTab, detail, selectedEpisode) {
        // Only fetch when user explicitly clicks "sources" tab
        if (activeTab != "sources") return@LaunchedEffect
        
        val currentDetail = detail ?: return@LaunchedEffect
        val effectiveEpisode = displaySelectedEpisode ?: selectedEpisode
        
        // Don't re-fetch if already fetched for this episode
        val currentEpId = if (effectiveType == "movie") currentDetail.id else effectiveEpisode?.id
        if (hasFetchedStreams && loadedStreamEpisodeId == currentEpId) return@LaunchedEffect
        
        if (effectiveType == "series") {
            val episodeSeason = effectiveEpisode?.season ?: selectedSeason
            val hasSeasonMetadata = seasonEpisodes.any { (it.season ?: episodeSeason) == episodeSeason }
            if (!hasSeasonMetadata) return@LaunchedEffect
        }
        
        val epId = if (effectiveType == "movie") currentDetail.id else effectiveEpisode?.id
        
        if (epId != null) {
            pendingStreamEpisodeId = epId
            loadedStreamEpisodeId = null
            hasFetchedStreams = false
            viewModel.fetchStreamsForSelection(type, epId, context)
            hasFetchedStreams = true
        }
    }

    LaunchedEffect(streams, isLoadingStreams, pendingStreamEpisodeId) {
        if (!isLoadingStreams && pendingStreamEpisodeId != null) {
            loadedStreamEpisodeId = pendingStreamEpisodeId
        }
    }

    var autoPlayConsumed by remember(id, type, targetSeason, targetEpisode, lastVideoId, autoPlay) { mutableStateOf(false) }
    LaunchedEffect(autoPlay, autoPlayConsumed, isLoadingStreams, detail?.id, displaySelectedEpisode?.id, loadedStreamEpisodeId, streams) {
        if (!autoPlay || autoPlayConsumed || isLoadingStreams) return@LaunchedEffect
        val playStream = preferredPlayStream(streams, lastStreamIndex, lastStreamUrl, lastStreamTitle) ?: return@LaunchedEffect
        val canAutoplay = when {
            effectiveType == "movie" -> true
            displaySelectedEpisode == null -> false
            else -> loadedStreamEpisodeId == displaySelectedEpisode.id
        }
        if (!canAutoplay) return@LaunchedEffect
        autoPlayConsumed = true
        onStreamClick(playStream, displaySelectedEpisode)
    }

    LaunchedEffect(detail?.id, detail?.videos, selectedSeason, effectiveType) {
        if (effectiveType == "series") {
            viewModel.loadSeason(detail?.id ?: id, selectedSeason)
        }
    }

    LaunchedEffect(selectedSeason, effectiveType) {
        if (effectiveType == "series") {
            selectedEpisode = null
            focusedEpisode = null
            pendingStreamEpisodeId = null
            loadedStreamEpisodeId = null
        }
    }

    LaunchedEffect(selectedSeason, seasonEpisodes, effectiveType) {
        if (effectiveType != "series") return@LaunchedEffect
        val currentSeasonEpisodes = seasonEpisodes.filter { (it.season ?: selectedSeason) == selectedSeason }
        if (currentSeasonEpisodes.isEmpty()) return@LaunchedEffect
        val currentSelection = selectedEpisode
        if (currentSelection == null || (currentSelection.season ?: selectedSeason) != selectedSeason) {
            selectedEpisode = currentSeasonEpisodes.firstOrNull { !detailIsUpcoming(it.released) } ?: currentSeasonEpisodes.firstOrNull()
        }
    }

    // Auto-select the resume season + episode from savedPlayback when no explicit target was provided.
    // Fires each time savedPlayback, detail videos, or season episodes change until resume is consumed.
    LaunchedEffect(savedPlayback?.lastVideoId, detail?.id, seasonEpisodes) {
        if (autoResumeConsumed) return@LaunchedEffect
        if (effectiveType != "series") return@LaunchedEffect
        val resumeId = savedPlayback?.lastVideoId ?: return@LaunchedEffect

        val resumeSeason = detail?.videos?.find { it.id == resumeId }?.season
            ?: parseSeasonFromVideoId(resumeId)

        if (resumeSeason != null && resumeSeason > 0 && resumeSeason != selectedSeason) {
            selectedSeason = resumeSeason
            selectedEpisode = null
            return@LaunchedEffect
        }

        val resumeEpisode = seasonEpisodes.find { it.id == resumeId }
        if (resumeEpisode != null) {
            selectedEpisode = resumeEpisode
            autoResumeConsumed = true
        }
    }

    val availableSeasons = remember(detail?.seasonsCount, detail?.videos) {
        buildList {
            val seasonsCount = detail?.seasonsCount ?: 0
            if (seasonsCount > 0) addAll(1..seasonsCount)
            addAll(detail?.videos?.mapNotNull { it.season }.orEmpty().filter { it > 0 })
            if (detail?.videos?.any { it.season == 0 } == true) add(0)
        }.distinct().sortedWith(compareBy<Int> { if (it == 0) 1 else 0 }.thenBy { it }).ifEmpty { listOf(1) }
    }

    if (deviceType == DeviceType.Mobile) {
        MobileDetailScreen(
            detail = detail,
            type = effectiveType,
            lang = lang,
            isLoading = isLoading,
            isLoadingStreams = isLoadingStreams,
            isInWatchlist = isInWatchlist,
            feedback = feedback,
            selectedSeason = selectedSeason,
            onSelectSeason = {
                autoResumeConsumed = true
                selectedSeason = it
            },
            selectedEpisode = displaySelectedEpisode,
            onSelectEpisode = { episode ->
                autoResumeConsumed = true
                selectedEpisode = episode
                focusedEpisode = episode
                viewModel.setSelectedAddon(null)
            },
            availableSeasons = availableSeasons,
            seasonEpisodes = seasonEpisodes,
            watchedVideoIds = effectiveWatchedVideoIds,
            streams = if (effectiveType == "movie" || loadedStreamEpisodeId == selectedEpisode?.id || selectedEpisode == null) streams else emptyList(),
            availableAddons = availableAddons,
            hasStreamProviders = hasStreamProviders,
            selectedAddon = selectedAddon,
            onSelectAddon = { viewModel.setSelectedAddon(it) },
            similarItems = similarItems,
            trailers = trailers,
            castList = detail?.cast.orEmpty().filter { !it.profilePath.isNullOrBlank() }.take(12),
            accentColor = Color(activeProfile?.safeAccentColorArgb ?: 0xFFFFFFFF.toInt()),
            onBack = onBack,
            onToggleWatchlist = { viewModel.toggleWatchlist() },
            onFeedback = { viewModel.setFeedback(it) },
            onPlayPrimary = {
                val currentEpisodeId = selectedEpisode?.id
                val canUseStreams = currentEpisodeId != null && loadedStreamEpisodeId == currentEpisodeId
                val playStream = if (canUseStreams) preferredPlayStream(streams, lastStreamIndex, lastStreamUrl, lastStreamTitle) else null
                if (playStream != null) onStreamClick(playStream, selectedEpisode) else onPlayClick(selectedEpisode)
            },
            onStreamClick = { stream -> onStreamClick(stream, selectedEpisode) },
            onPlayEpisode = onPlayEpisode,
            onSimilarClick = { item ->
                viewModel.loadDetail(item.type, item.id, activeProfile, initialMeta = item)
            },
            onDownloadEpisode = onDownloadEpisode,
            onDownloadEpisodes = onDownloadEpisodes,
            onMarkEpisodeWatched = { episode, watched -> viewModel.markEpisodeWatched(detail?.id ?: id, episode, watched) },
            onSetEpisodesWatched = { episodes, watched -> viewModel.markSeasonWatched(detail?.id ?: id, episodes, watched) },
            resumeVideoId = effectiveLastVideoId,
            resumeProgress = effectiveInitialProgress ?: 0L,
            runtimeLabel = detail?.runtime,
            activeTab = activeTab,
            onActiveTabChange = { activeTab = it },
            lastStreamUrl = lastStreamUrl,
            lastStreamTitle = lastStreamTitle,
            initialMeta = initialMeta,
            trailerOnHero = activeProfile?.safeTrailerOnHero ?: true,
            blurUnwatchedEpisodes = activeProfile?.safeBlurUnwatchedEpisodes ?: false,
            episodeCardsLayout = activeProfile?.safeEpisodeCardsLayout ?: "list",
            seasonSelectorMode = activeProfile?.safeDetailSeasonSelectorMode ?: "dropdown",
            seasonPostersOnHero = activeProfile?.safeDetailSeasonPostersOnHero ?: true
        )
        return
    }

    TvDetailScreenContent(
        detail = detail,
        initialMeta = initialMeta,
        id = id,
        type = effectiveType,
        activeProfile = activeProfile,
        horizontalPadding = horizontalPadding,
        lazyListState = lazyListState,
        coroutineScope = coroutineScope,
        selectedSeason = selectedSeason,
        onSelectedSeasonChange = { selectedSeason = it },
        selectedEpisode = selectedEpisode,
        onSelectedEpisodeChange = { selectedEpisode = it },
        focusedEpisode = focusedEpisode,
        onFocusedEpisodeChange = { focusedEpisode = it },
        isInWatchlist = isInWatchlist,
        feedback = feedback,
        lang = lang,
        isLoading = isLoading,
        isLoadingStreams = isLoadingStreams,
        seasonEpisodes = seasonEpisodes,
        effectiveWatchedVideoIds = effectiveWatchedVideoIds,
        effectiveLastVideoId = effectiveLastVideoId,
        effectiveInitialProgress = effectiveInitialProgress,
        streams = streams,
        availableAddons = availableAddons,
        selectedAddon = selectedAddon,
        similarItems = similarItems,
        trailers = trailers,
        availableSeasons = availableSeasons,
        lastStreamIndex = lastStreamIndex,
        lastStreamUrl = lastStreamUrl,
        lastStreamTitle = lastStreamTitle,
        viewModel = viewModel,
        accentColor = Color(activeProfile?.safeAccentColorArgb ?: 0xFFFFFFFF.toInt()),
        onBack = onBack,
        onStreamClick = onStreamClick,
        onPlayEpisode = onPlayEpisode
    )
}

// Parses the season number from Stremio-style video IDs ("imdbId:season:episode").
private fun parseSeasonFromVideoId(videoId: String): Int? {
    val parts = videoId.split(":")
    if (parts.size < 3) return null
    return parts[parts.size - 2].toIntOrNull()?.takeIf { it > 0 }
}
