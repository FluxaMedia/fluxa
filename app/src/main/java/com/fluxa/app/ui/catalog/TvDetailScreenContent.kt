@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
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
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.*
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.Dp
import kotlinx.coroutines.CoroutineScope
import androidx.compose.ui.unit.sp
import androidx.tv.material3.*
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.*

@Composable
internal fun TvDetailScreenContent(
    detail: MetaDetail?,
    id: String,
    type: String,
    activeProfile: UserProfile?,
    horizontalPadding: Dp,
    lazyListState: LazyListState,
    coroutineScope: CoroutineScope,
    selectedSeason: Int,
    onSelectedSeasonChange: (Int) -> Unit,
    selectedEpisode: Video?,
    onSelectedEpisodeChange: (Video?) -> Unit,
    focusedEpisode: Video?,
    onFocusedEpisodeChange: (Video?) -> Unit,
    isInWatchlist: Boolean,
    feedback: Boolean?,
    lang: String,
    isLoading: Boolean,
    isLoadingStreams: Boolean,
    seasonEpisodes: List<Video>,
    effectiveWatchedVideoIds: Set<String>,
    effectiveLastVideoId: String?,
    effectiveInitialProgress: Long?,
    streams: List<Stream>,
    availableAddons: List<String>,
    selectedAddon: String?,
    similarItems: List<Meta>,
    trailers: List<DetailTrailer>,
    availableSeasons: List<Int>,
    lastStreamIndex: Int?,
    lastStreamUrl: String?,
    lastStreamTitle: String?,
    viewModel: DetailViewModel,
    accentColor: Color,
    onBack: () -> Unit,
    onStreamClick: (Stream, Video?) -> Unit,
    onPlayEpisode: (Video) -> Unit
) {
    val deviceType = DeviceType.TV
    val context = LocalContext.current
    var showEpisodeSortSelector by remember(detail?.id, selectedSeason) { mutableStateOf(false) }
    var episodeSort by remember(detail?.id, selectedSeason) { mutableStateOf("number_asc") }
    val detailBgRequest = remember(detail?.background) {
        detail?.background?.let { bg ->
            ImageRequest.Builder(context).data(bg).memoryCacheKey("detail-bg:$bg").diskCacheKey(bg).build()
        }
    }
Box(modifier = Modifier.fillMaxSize().background(FluxaColors.backgroundAmoled)) {
    detailBgRequest?.let { AsyncImage(model = it, contentDescription = null, modifier = Modifier.fillMaxSize().alpha(0.25f), contentScale = ContentScale.Crop) }
    Box(modifier = Modifier.fillMaxSize().background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(0.9f), Color.Black))))

    LazyColumn(state = lazyListState, modifier = Modifier.fillMaxSize(), contentPadding = PaddingValues(bottom = 100.dp)) {
        item {
            DetailHeaderContentOfficial(
                detail = detail, 
                selectedEpisode = selectedEpisode,
                isInWatchlist = isInWatchlist,
                feedback = feedback,
                lang = lang,
                onBack = onBack,
                onToggleWatchlist = { viewModel.toggleWatchlist() },
                onFeedback = { viewModel.setFeedback(it) },
                onPlayClick = {
                    val playStream = preferredPlayStream(streams, lastStreamIndex, lastStreamUrl, lastStreamTitle)
                    if (playStream != null) {
                        onStreamClick(playStream, selectedEpisode)
                    } else {
                        coroutineScope.launch {
                            val target = if (detail?.type == "series") 3 else 2
                            lazyListState.animateScrollToItem(target)
                        }
                    }
                }
            )
        }

        item {
            val overview = selectedEpisode?.overview ?: focusedEpisode?.overview ?: detail?.description
            var expanded by remember { mutableStateOf(false) }
            if (overview != null) {
                Column(modifier = Modifier.padding(horizontal = horizontalPadding, vertical = 16.dp)) {
                    Text(text = overview, style = androidx.compose.material3.MaterialTheme.typography.bodyLarge, color = Color.White.copy(alpha = 0.85f), maxLines = if (expanded) Int.MAX_VALUE else 3, overflow = TextOverflow.Ellipsis, modifier = Modifier.width(if (deviceType == DeviceType.TV) 800.dp else 400.dp), lineHeight = 24.sp)
                    if (overview.length > 150) {
                        Surface(
                            onClick = { expanded = !expanded },
                            modifier = Modifier.padding(top = 8.dp),
                            shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(6.dp)),
                            colors = ClickableSurfaceDefaults.colors(containerColor = Color.Transparent, focusedContainerColor = Color.White.copy(alpha = 0.08f))
                        ) {
                            Text(
                                text = if (expanded) (AppStrings.t(lang, "auto.read_less")) else (AppStrings.t(lang, "auto.read_more")),
                                color = Color.White.copy(alpha = 0.5f),
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                            )
                        }
                    }
                }
            }
        }

        if (trailers.isNotEmpty()) {
            item {
                Column(modifier = Modifier.padding(top = 28.dp)) {
                    Text(text = AppStrings.t(lang, "auto.trailers"), style = androidx.compose.material3.MaterialTheme.typography.titleMedium, color = Color.White.copy(alpha = 0.4f), fontWeight = FontWeight.Bold, modifier = Modifier.padding(start = horizontalPadding))
                    LazyRow(horizontalArrangement = Arrangement.spacedBy(20.dp), contentPadding = PaddingValues(horizontal = horizontalPadding, vertical = 20.dp)) {
                        items(trailers, key = { "${it.id}:${it.url}:${it.title}" }) { trailer ->
                            TrailerCard(trailer = trailer, accentColor = accentColor)
                        }
                    }
                }
            }
        }

        if (detail?.type == "series") {
            val seasons = availableSeasons
            item {
                val episodes = remember(seasonEpisodes, detail.videos, selectedSeason) {
                    if (seasonEpisodes.isNotEmpty()) seasonEpisodes else detail.videos?.filter { it.season == selectedSeason } ?: emptyList()
                }
                val downloadableEpisodes = remember(episodes) {
                    episodes.filter { !detailIsUpcoming(it.released) }
                }
                Column(modifier = Modifier.padding(top = 32.dp)) {
                    Row(
                        modifier = Modifier.padding(start = horizontalPadding),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Text(text = AppStrings.t(lang, "auto.seasons_da41bd44"), style = androidx.compose.material3.MaterialTheme.typography.titleMedium, color = Color.White.copy(alpha = 0.4f), fontWeight = FontWeight.Bold)
                        Text(
                            text = AppStrings.format(lang, "format.episode_count", episodes.size),
                            color = Color.White.copy(alpha = 0.48f),
                            fontSize = 13.sp,
                            fontWeight = FontWeight.Bold
                        )
                        Surface(
                            onClick = {
                                viewModel.markSeasonWatched(detail.id, episodes.filter { !detailIsUpcoming(it.released) })
                            },
                            modifier = Modifier.size(42.dp),
                            shape = ClickableSurfaceDefaults.shape(CircleShape),
                            colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(alpha = 0.08f), contentColor = Color.White, focusedContainerColor = Color.White, focusedContentColor = Color.Black)
                        ) {
                            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                Icon(FluxaIcons.CheckCircle, null, modifier = Modifier.size(20.dp))
                            }
                        }
                        Surface(
                            onClick = { viewModel.downloadEpisodes(downloadableEpisodes, context) },
                            modifier = Modifier.size(42.dp),
                            shape = ClickableSurfaceDefaults.shape(CircleShape),
                            colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(alpha = 0.08f), contentColor = Color.White, focusedContainerColor = Color.White, focusedContentColor = Color.Black)
                        ) {
                            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                Icon(FluxaIcons.Download, null, modifier = Modifier.size(20.dp))
                            }
                        }
                        Surface(
                            onClick = { showEpisodeSortSelector = true },
                            modifier = Modifier.size(42.dp),
                            shape = ClickableSurfaceDefaults.shape(CircleShape),
                            colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(alpha = 0.08f), contentColor = Color.White, focusedContainerColor = Color.White, focusedContentColor = Color.Black)
                        ) {
                            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                Icon(FluxaIcons.GraphicEq, null, modifier = Modifier.size(20.dp))
                            }
                        }
                    }
                    LazyRow(horizontalArrangement = Arrangement.spacedBy(16.dp), contentPadding = PaddingValues(horizontal = horizontalPadding, vertical = 16.dp)) {
                        items(seasons, key = { it }) { season -> SeasonPill(seasonNumber = season, isSelected = selectedSeason == season, accentColor = accentColor, lang = lang, onClick = { onSelectedSeasonChange(season) }) }
                    }
                }
            }
            item {
                val episodes = remember(seasonEpisodes, detail.videos, selectedSeason, episodeSort) {
                    sortedDetailEpisodes(
                        if (seasonEpisodes.isNotEmpty()) seasonEpisodes else detail.videos?.filter { it.season == selectedSeason } ?: emptyList(),
                        episodeSort
                    )
                }
                Column(modifier = Modifier.padding(top = 24.dp)) {
                    Text(text = AppStrings.t(lang, "auto.episodes"), style = androidx.compose.material3.MaterialTheme.typography.titleMedium, color = Color.White.copy(alpha = 0.4f), fontWeight = FontWeight.Bold, modifier = Modifier.padding(start = horizontalPadding))
                    LazyRow(horizontalArrangement = Arrangement.spacedBy(20.dp), contentPadding = PaddingValues(horizontal = horizontalPadding, vertical = 20.dp)) {
                        items(episodes, key = { it.id }) { episode ->
                EpisodeCard(
                    episode = episode, 
                    isSelected = selectedEpisode?.id == episode.id, 
                    isWatched = effectiveWatchedVideoIds.contains(episode.id), 
                    progress = when {
                        effectiveWatchedVideoIds.contains(episode.id) -> 1f
                        episode.id == effectiveLastVideoId && (effectiveInitialProgress ?: 0L) > 0L -> episodeProgressFraction(effectiveInitialProgress ?: 0L, detail.runtime)
                        else -> 0f
                    },
                    durationLabel = detail.runtime,
                    accentColor = accentColor, 
                    lang = lang, 
                    onFocus = { onFocusedEpisodeChange(episode) }, 
                    onClick = {
                        if (!detailIsUpcoming(episode.released)) {
                            onSelectedEpisodeChange(episode)
                            onFocusedEpisodeChange(episode)
                            onPlayEpisode(episode)
                        }
                    },
                    onDownloadClick = { viewModel.downloadEpisodes(listOf(episode), context) }
                ) 
            }
                    }
                    
                    if (deviceType == DeviceType.TV) {
                        AnimatedVisibility(
                            visible = focusedEpisode != null,
                            enter = fadeIn() + expandVertically(),
                            exit = fadeOut() + shrinkVertically()
                        ) {
                            Column(modifier = Modifier.padding(horizontal = horizontalPadding, vertical = 8.dp)) {
                                Text(
                                    text = focusedEpisode?.overview ?: "",
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = Color.White.copy(alpha = 0.6f),
                                    maxLines = 3,
                                    overflow = TextOverflow.Ellipsis,
                                    modifier = Modifier.width(800.dp),
                                    lineHeight = 20.sp
                                )
                            }
                        }
                    }
                }
            }
        }

        if (showEpisodeSortSelector) {
            item {
                ExploreOptionSelector(
                    title = AppStrings.t(lang, "settings.sort_episodes"),
                    options = episodeSortOptions(lang),
                    selected = episodeSort,
                    onSelect = {
                        episodeSort = it ?: "number_asc"
                        showEpisodeSortSelector = false
                    },
                    onDismiss = { showEpisodeSortSelector = false }
                )
            }
        }

        item {
            if (availableAddons.isNotEmpty() || streams.isNotEmpty() || isLoadingStreams) {
                Column(modifier = Modifier.padding(top = 24.dp)) {
                    val title = if(detail?.type == "movie") (AppStrings.t(lang, "auto.sources")) else {
                        val ep = selectedEpisode ?: focusedEpisode
                        if (ep != null) "S${ep.season} B${ep.number} - ${AppStrings.t(lang, "auto.sources")}" 
                        else (AppStrings.t(lang, "auto.sources"))
                    }
                    Text(text = title, style = androidx.compose.material3.MaterialTheme.typography.titleLarge, color = Color.White, fontWeight = FontWeight.Black, modifier = Modifier.padding(start = horizontalPadding, bottom = 12.dp))

                    if (isLoadingStreams) {
                        Box(modifier = Modifier.fillMaxWidth().height(80.dp), contentAlignment = Alignment.Center) {
                            CircularProgressIndicator(color = Color.White.copy(alpha = 0.5f))
                        }
                    }

                    if (availableAddons.size > 1 && !isLoadingStreams) {
                        LazyRow(
                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                            contentPadding = PaddingValues(start = horizontalPadding, end = horizontalPadding, bottom = 16.dp)
                        ) {
                            item {
                                SourceFilterPill(
                                    name = AppStrings.t(lang, "auto.all_24e2815b"),
                                    isSelected = selectedAddon == null,
                                    onClick = { viewModel.setSelectedAddon(null) }
                                )
                            }
                            items(availableAddons, key = { it }) { addon ->
                                SourceFilterPill(
                                    name = addon,
                                    isSelected = selectedAddon == addon,
                                    onClick = { viewModel.setSelectedAddon(addon) }
                                )
                            }
                        }
                    }

                    if (!isLoadingStreams) {
                        if (streams.isNotEmpty()) {
                            LazyRow(horizontalArrangement = Arrangement.spacedBy(16.dp), contentPadding = PaddingValues(horizontal = horizontalPadding)) {
                                items(streams, key = { it.playableUrl ?: (it.title.orEmpty() + it.name.orEmpty()) }) { stream -> StreamCard(stream.toStreamUiModel(), Color.White) { onStreamClick(stream, selectedEpisode) } }
                            }
                        } else {
                            Text(
                                text = AppStrings.t(lang, "auto.no_sources_found_for_this_filter"),
                                color = Color.White.copy(alpha = 0.5f),
                                modifier = Modifier.padding(start = horizontalPadding, top = 8.dp)
                            )
                        }
                    }
                }
            }
        }

        val rawCastList = detail?.cast.orEmpty().filter { !it.profilePath.isNullOrBlank() }.take(12)
        if (rawCastList.isNotEmpty()) {
            item {
                val castList = remember(detail?.cast) {
                    detail?.cast.orEmpty().filter { !it.profilePath.isNullOrBlank() }.take(12)
                }
                Column(modifier = Modifier.padding(top = 24.dp)) {
                    Text(text = AppStrings.t(lang, "auto.cast"), style = androidx.compose.material3.MaterialTheme.typography.titleMedium, color = Color.White.copy(alpha = 0.4f), fontWeight = FontWeight.Bold, modifier = Modifier.padding(start = horizontalPadding))
                    LazyRow(horizontalArrangement = Arrangement.spacedBy(24.dp), contentPadding = PaddingValues(horizontal = horizontalPadding, vertical = 16.dp)) {
                        items(castList, key = { it.name }) { member -> CastMemberCard(member) }
                    }
                }
            }
        }

        if (similarItems.isNotEmpty()) {
            item {
                Column(modifier = Modifier.padding(top = 24.dp)) {
                    Text(text = AppStrings.t(lang, "auto.similar_content"), style = androidx.compose.material3.MaterialTheme.typography.titleMedium, color = Color.White.copy(alpha = 0.4f), fontWeight = FontWeight.Bold, modifier = Modifier.padding(start = horizontalPadding))
                    LazyRow(horizontalArrangement = Arrangement.spacedBy(20.dp), contentPadding = PaddingValues(horizontal = horizontalPadding, vertical = 16.dp)) {
                        items(similarItems, key = { "${it.type}:${it.id}" }) { item -> SimilarContentCard(item.toSimilarItemUiModel(), Color.White) { viewModel.loadDetail(item.type, item.id, activeProfile) } }
                    }
                }
            }
        }
    }
    
    if (isLoading && detail == null) Box(modifier = Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.6f)), contentAlignment = Alignment.Center) { CircularProgressIndicator(color = Color.White) }
}
}
