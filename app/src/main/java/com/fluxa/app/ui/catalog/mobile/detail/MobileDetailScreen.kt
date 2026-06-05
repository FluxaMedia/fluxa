@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.content.Intent
import android.net.Uri
import androidx.compose.animation.*
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items as mobileItems
import androidx.compose.foundation.lazy.rememberLazyListState
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
import androidx.compose.ui.unit.sp
import androidx.tv.material3.*
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.*

@Composable
internal fun MobileDetailScreen(
    detail: MetaDetail?,
    type: String,
    lang: String,
    isLoading: Boolean,
    isLoadingStreams: Boolean = false,
    isInWatchlist: Boolean,
    feedback: Boolean?,
    selectedSeason: Int,
    onSelectSeason: (Int) -> Unit,
    selectedEpisode: Video?,
    onSelectEpisode: (Video) -> Unit,
    availableSeasons: List<Int>,
    seasonEpisodes: List<Video>,
    watchedVideoIds: Set<String>,
    streams: List<Stream>,
    availableAddons: List<String>,
    hasStreamProviders: Boolean,
    selectedAddon: String?,
    onSelectAddon: (String?) -> Unit,
    similarItems: List<Meta>,
    trailers: List<DetailTrailer>,
    castList: List<CastMember>,
    accentColor: Color,
    onBack: () -> Unit,
    onToggleWatchlist: () -> Unit,
    onFeedback: (Boolean) -> Unit,
    onPlayPrimary: () -> Unit,
    onStreamClick: (Stream) -> Unit,
    onPlayEpisode: (Video) -> Unit,
    onDownloadEpisode: (Video?) -> Unit = {},
    onDownloadEpisodes: (List<Video>) -> Unit = {},
    onMarkEpisodeWatched: (Video, Boolean) -> Unit,
    onSetEpisodesWatched: (List<Video>, Boolean) -> Unit,
    resumeVideoId: String?,
    resumeProgress: Long,
    runtimeLabel: String?,
    activeTab: String,
    onActiveTabChange: (String) -> Unit,
    lastStreamUrl: String? = null,
    lastStreamTitle: String? = null,
    initialMeta: Meta? = null,
    trailerOnHero: Boolean = true,
    blurUnwatchedEpisodes: Boolean = false,
    episodeCardsLayout: String = "list",
    seasonSelectorMode: String = "dropdown",
    seasonPostersOnHero: Boolean = true
) {
    val context = LocalContext.current
    val effectiveSeasonSelectorMode = when (seasonSelectorMode) {
        "tabs", "posters" -> seasonSelectorMode
        else -> "dropdown"
    }
    val heroImage = detail?.seasonPosters?.get(selectedSeason.toString())?.takeIf { seasonPostersOnHero }
        ?: detail?.poster
        ?: detail?.background
        ?: initialMeta?.background
        ?: initialMeta?.poster
    var heroTrailer by remember(detail?.id) { mutableStateOf<DetailTrailer?>(null) }
    val listState = rememberLazyListState()
    val showStickyBar by remember { derivedStateOf { listState.firstVisibleItemIndex > 0 } }
    val scope = rememberCoroutineScope()
    var showSeasonSelector by remember(detail?.id) { mutableStateOf(false) }
    var showEpisodeSortSelector by remember(detail?.id, selectedSeason) { mutableStateOf(false) }
    var episodeSort by remember(detail?.id, selectedSeason) { mutableStateOf("number_asc") }
    var descriptionExpanded by remember(detail?.id, selectedEpisode?.id) { mutableStateOf(false) }
    var episodeDrawer by remember(detail?.id) { mutableStateOf<Video?>(null) }
    var seasonDrawer by remember(detail?.id) { mutableStateOf<MobileSeasonActionTarget?>(null) }
    val seasons = availableSeasons
    val titleText = detail?.name.orEmpty()
    val stickyLogoUrl = detail?.logo?.takeIf { it.isNotBlank() }
    var stickyLogoLoadFailed by remember(stickyLogoUrl) { mutableStateOf(false) }
    val scheduleLabel = remember(detail?.id, detail?.videos, lang) { releaseScheduleLabel(detail, lang) }
    val actionEpisodes = remember(detail?.videos, seasonEpisodes) {
        (detail?.videos.orEmpty() + seasonEpisodes)
            .distinctBy { it.id.ifBlank { "${it.season}:${it.number}:${it.name.orEmpty()}" } }
    }
    val visibleEpisodes = remember(actionEpisodes, selectedSeason, episodeSort) {
        actionEpisodes
            .filter { (it.season ?: selectedSeason) == selectedSeason }
            .let { sortedDetailEpisodes(it, episodeSort) }
    }
    val currentSeasonActionEpisodes = remember(visibleEpisodes) {
        visibleEpisodes.filter { !detailIsUpcoming(it.released) }
    }
    val episodesThroughSelectedSeason = remember(actionEpisodes, selectedSeason) {
        if (selectedSeason == 0) return@remember emptyList()
        actionEpisodes
            .filter { episode ->
                val season = episode.season ?: selectedSeason
                season in 1..selectedSeason
            }
            .filter { !detailIsUpcoming(it.released) }
            .sortedWith(compareBy<Video>({ it.season ?: selectedSeason }, { it.number ?: Int.MAX_VALUE }))
    }

    Box(modifier = Modifier.fillMaxSize().background(Color(0xFF050505))) {
        LazyColumn(
            state = listState,
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = 120.dp)
        ) {
            item(key = "detail-hero") {
                MobileDetailHero(
                    heroImage = heroImage,
                    onBack = onBack,
                    lang = lang,
                    activeTrailer = heroTrailer,
                    onStopTrailer = { heroTrailer = null },
                    onSelectTrailer = if (trailerOnHero) ({ heroTrailer = it }) else null,
                    accentColor = accentColor
                )
            }

            item {
                MobileDetailInfoSection(
                    detail = detail,
                    type = type,
                    lang = lang,
                    isLoading = isLoading,
                    selectedEpisode = selectedEpisode,
                    selectedSeason = selectedSeason,
                    titleText = titleText,
                    scheduleLabel = scheduleLabel,
                    descriptionExpanded = descriptionExpanded,
                    onDescriptionExpandedChange = { descriptionExpanded = it },
                    onPlayPrimary = onPlayPrimary,
                    isInWatchlist = isInWatchlist,
                    onToggleWatchlist = onToggleWatchlist,
                    onFeedback = onFeedback,
                    accentColor = accentColor
                )
            }

            item {
                val tabs = remember(type, trailers.isNotEmpty(), similarItems.isNotEmpty(), lang) {
                    buildList {
                        if (type == "series") add("episodes" to AppStrings.t(lang, "auto.episodes"))
                        if (trailers.isNotEmpty()) add("trailers" to AppStrings.t(lang, "auto.trailers"))
                        if (similarItems.isNotEmpty()) add("similar" to AppStrings.t(lang, "auto.similar_titles"))
                    }
                }
                if (tabs.isNotEmpty()) {
                    LazyRow(
                        modifier = Modifier.fillMaxWidth(),
                        contentPadding = PaddingValues(horizontal = 16.dp),
                        horizontalArrangement = Arrangement.spacedBy(18.dp)
                    ) {
                        mobileItems(tabs, key = { (key, _) -> key }) { (key, label) ->
                            val selected = activeTab == key
                            val textColor by animateColorAsState(
                                targetValue = if (selected) Color.White else Color.White.copy(alpha = 0.6f),
                                animationSpec = tween(180),
                                label = "detailTabText"
                            )
                            val indicatorWidth by animateDpAsState(
                                targetValue = if (selected) 44.dp else 0.dp,
                                animationSpec = tween(190),
                                label = "detailTabIndicator"
                            )
                            Column(
                                modifier = Modifier
                                    .animateItem()
                                    .clickable { onActiveTabChange(key) },
                                horizontalAlignment = Alignment.CenterHorizontally
                            ) {
                                Text(
                                    text = label,
                                    color = textColor,
                                    fontSize = 15.sp,
                                    fontWeight = FontWeight.Bold
                                )
                                Spacer(Modifier.height(10.dp))
                                Box(
                                    modifier = Modifier
                                        .width(indicatorWidth)
                                        .height(3.dp)
                                        .clip(RoundedCornerShape(999.dp))
                                        .background(if (activeTab == key) accentColor else Color.Transparent)
                                )
                            }
                        }
                    }
                }
            }

            when (activeTab) {
                "trailers" -> {
                    item {
                        LazyRow(
                            contentPadding = PaddingValues(horizontal = 16.dp, vertical = 16.dp),
                            horizontalArrangement = Arrangement.spacedBy(12.dp)
                        ) {
                            mobileItems(trailers, key = { "${it.id}:${it.url}:${it.title}" }) { trailer ->
                                TrailerCard(
                                    trailer = trailer,
                                    accentColor = accentColor,
                                    onPlay = {
                                        runCatching {
                                            context.startActivity(
                                                Intent(Intent.ACTION_VIEW, Uri.parse(trailer.url))
                                            )
                                        }
                                    }
                                )
                            }
                        }
                    }
                }

                "episodes" -> {
                    item {
                        Column(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(vertical = 14.dp),
                            verticalArrangement = Arrangement.spacedBy(12.dp)
                        ) {
                            when (effectiveSeasonSelectorMode) {
                                "tabs" -> MobileSeasonTabsSelector(
                                    seasons = seasons,
                                    selectedSeason = selectedSeason,
                                    lang = lang,
                                    accentColor = accentColor,
                                    onSelectSeason = onSelectSeason,
                                    onLongPressSeason = { season ->
                                        seasonDrawer = MobileSeasonActionTarget(
                                            season = season,
                                            seasonEpisodes = actionEpisodes
                                                .filter { (it.season ?: season) == season && !detailIsUpcoming(it.released) },
                                            throughEpisodes = mobileEpisodesThroughSeason(actionEpisodes, season)
                                        )
                                    }
                                )
                                "posters" -> MobileSeasonPosterSelector(
                                    detailId = detail?.id ?: titleText,
                                    seasons = seasons,
                                    selectedSeason = selectedSeason,
                                    seasonPosters = detail?.seasonPosters.orEmpty(),
                                    fallbackPoster = detail?.poster ?: detail?.background ?: initialMeta?.poster ?: initialMeta?.background,
                                    lang = lang,
                                    accentColor = accentColor,
                                    onSelectSeason = onSelectSeason,
                                    onLongPressSeason = { season ->
                                        seasonDrawer = MobileSeasonActionTarget(
                                            season = season,
                                            seasonEpisodes = actionEpisodes
                                                .filter { (it.season ?: season) == season && !detailIsUpcoming(it.released) },
                                            throughEpisodes = mobileEpisodesThroughSeason(actionEpisodes, season)
                                        )
                                    }
                                )
                                else -> Row(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(horizontal = 16.dp),
                                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                                    Box(
                                        modifier = Modifier
                                            .weight(1f, fill = false)
                                            .clip(RoundedCornerShape(12.dp))
                                            .background(Color.White.copy(alpha = 0.08f))
                                            .pointerInput(selectedSeason, episodesThroughSelectedSeason) {
                                                detectTapGestures(
                                                    onTap = { showSeasonSelector = true },
                                                    onLongPress = {
                                                        seasonDrawer = MobileSeasonActionTarget(
                                                            season = selectedSeason,
                                                            seasonEpisodes = currentSeasonActionEpisodes,
                                                            throughEpisodes = episodesThroughSelectedSeason
                                                        )
                                                    }
                                                )
                                            }
                                            .padding(horizontal = 14.dp, vertical = 10.dp)
                                    ) {
                                        Text(
                                            text = mobileSeasonDropdownLabel(selectedSeason, lang),
                                            color = Color.White,
                                            fontSize = 14.sp,
                                            fontWeight = FontWeight.SemiBold
                                        )
                                    }
                                    Text(
                                        text = AppStrings.format(lang, "format.episode_count", visibleEpisodes.size),
                                        color = Color.White.copy(alpha = 0.52f),
                                        fontSize = 13.sp,
                                        fontWeight = FontWeight.Bold
                                    )
                                }
                            }
                            if (effectiveSeasonSelectorMode != "dropdown") {
                                Text(
                                    text = AppStrings.format(lang, "format.episode_count", visibleEpisodes.size),
                                    color = Color.White.copy(alpha = 0.52f),
                                    fontSize = 13.sp,
                                    fontWeight = FontWeight.Bold,
                                    modifier = Modifier.padding(horizontal = 16.dp)
                                )
                            }
                        }
                    }
                    if (visibleEpisodes.isEmpty()) {
                        item {
                            Box(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(horizontal = 16.dp, vertical = 28.dp),
                                contentAlignment = Alignment.Center
                            ) {
                                if (isLoading) {
                                    CircularProgressIndicator(color = Color.White.copy(alpha = 0.7f))
                                } else {
                                    Text(
                                        text = AppStrings.t(lang, "auto.no_episode_information_found"),
                                        color = Color.White.copy(alpha = 0.58f),
                                        fontSize = 14.sp
                                    )
                                }
                            }
                        }
                    }
                    if (episodeCardsLayout == "horizontal" && visibleEpisodes.isNotEmpty()) {
                        item(key = "episodes-horizontal") {
                            LazyRow(
                                contentPadding = PaddingValues(horizontal = 16.dp, vertical = 8.dp),
                                horizontalArrangement = Arrangement.spacedBy(10.dp)
                            ) {
                                mobileItems(visibleEpisodes, key = { it.id }) { episode ->
                                    MobileHorizontalEpisodeCard(
                                        episode = episode,
                                        lang = lang,
                                        isSelected = selectedEpisode?.id == episode.id,
                                        isWatched = watchedVideoIds.contains(episode.id),
                                        showPoster = detail?.seasonPosters?.get(selectedSeason.toString())
                                            ?: detail?.poster
                                            ?: detail?.background,
                                        progress = when {
                                            watchedVideoIds.contains(episode.id) -> 1f
                                            episode.id == resumeVideoId && resumeProgress > 0L -> mobileEpisodeProgressFraction(resumeProgress, episode.episodeRuntime?.let { "${it}min" } ?: runtimeLabel)
                                            else -> 0f
                                        },
                                        durationLabel = runtimeLabel,
                                        accentColor = accentColor,
                                        blurUnwatched = blurUnwatchedEpisodes,
                                        onClick = {
                                            onSelectEpisode(episode)
                                            onPlayEpisode(episode)
                                        },
                                        onLongPress = { episodeDrawer = episode }
                                    )
                                }
                            }
                        }
                    } else {
                        mobileItems(visibleEpisodes, key = { it.id }) { episode ->
                            MobileEpisodeRow(
                                episode = episode,
                                lang = lang,
                                isSelected = selectedEpisode?.id == episode.id,
                                isWatched = watchedVideoIds.contains(episode.id),
                                showPoster = detail?.seasonPosters?.get(selectedSeason.toString())
                                    ?: detail?.poster
                                    ?: detail?.background,
                                progress = when {
                                    watchedVideoIds.contains(episode.id) -> 1f
                                    episode.id == resumeVideoId && resumeProgress > 0L -> mobileEpisodeProgressFraction(resumeProgress, episode.episodeRuntime?.let { "${it}min" } ?: runtimeLabel)
                                    else -> 0f
                                },
                                durationLabel = runtimeLabel,
                                accentColor = accentColor,
                                blurUnwatched = blurUnwatchedEpisodes,
                                onClick = {
                                    onSelectEpisode(episode)
                                    onPlayEpisode(episode)
                                },
                                onLongPress = { episodeDrawer = episode },
                                onDownloadClick = { onDownloadEpisode(episode) }
                            )
                        }
                    }
                }

                "sources" -> {
                    if (isLoadingStreams) {
                        item {
                            Box(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(horizontal = 16.dp, vertical = 24.dp),
                                contentAlignment = Alignment.Center
                            ) {
                                CircularProgressIndicator(color = Color.White.copy(alpha = 0.7f))
                            }
                        }
                    }
                    if (availableAddons.size > 1) {
                        item {
                            LazyRow(
                                contentPadding = PaddingValues(horizontal = 16.dp, vertical = 14.dp),
                                horizontalArrangement = Arrangement.spacedBy(8.dp)
                            ) {
                                item {
                                    SourceFilterPill(
                                        name = AppStrings.t(lang, "auto.all_24e2815b"),
                                        isSelected = selectedAddon == null,
                                        onClick = { onSelectAddon(null) }
                                    )
                                }
                                mobileItems(availableAddons, key = { it }) { addon ->
                                    SourceFilterPill(
                                        name = addon,
                                        isSelected = selectedAddon == addon,
                                        onClick = { onSelectAddon(addon) }
                                    )
                                }
                            }
                        }
                    }
                    if (!isLoadingStreams && streams.isEmpty()) {
                        item {
                            Box(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(horizontal = 16.dp, vertical = 24.dp),
                                contentAlignment = Alignment.Center
                            ) {
                                Text(
                                    text = if (availableAddons.isEmpty()) {
                                        AppStrings.t(lang, "auto.no_source_add_ons_found")
                                    } else {
                                        AppStrings.t(lang, "auto.no_sources_found_3019f12c")
                                    },
                                    color = Color.White.copy(alpha = 0.58f),
                                    fontSize = 14.sp
                                )
                            }
                        }
                    }
                    mobileItems(streams, key = { it.playableUrl ?: (it.title.orEmpty() + it.name.orEmpty()) }) { stream ->
                        Box(modifier = Modifier.padding(horizontal = 16.dp, vertical = 6.dp)) {
                            MobileStreamRow(stream = stream.toStreamUiModel(), onClick = { onStreamClick(stream) })
                        }
                    }
                }

                "similar" -> {
                    item {
                        LazyRow(
                            contentPadding = PaddingValues(horizontal = 16.dp, vertical = 16.dp),
                            horizontalArrangement = Arrangement.spacedBy(12.dp)
                        ) {
                            mobileItems(similarItems, key = { "${it.type}:${it.id}" }) { item ->
                                SimilarContentCard(item.toSimilarItemUiModel(), Color.White) {}
                            }
                        }
                    }
                }

            }
        }

        MobileDetailOverlays(
            showSeasonSelector = showSeasonSelector,
            seasons = seasons,
            selectedSeason = selectedSeason,
            lang = lang,
            onSelectSeason = onSelectSeason,
            onDismissSeasonSelector = { showSeasonSelector = false },
            episodeDrawer = episodeDrawer,
            actionEpisodes = actionEpisodes,
            watchedVideoIds = watchedVideoIds,
            runtimeLabel = runtimeLabel,
            onDismissEpisodeDrawer = { episodeDrawer = null },
            onMarkEpisodeWatched = onMarkEpisodeWatched,
            onSetEpisodesWatched = onSetEpisodesWatched,
            onDownloadEpisode = onDownloadEpisode,
            seasonDrawer = seasonDrawer,
            onDismissSeasonDrawer = { seasonDrawer = null },
            isLoading = isLoading,
            detail = detail,
            accentColor = accentColor
        )

        if (showEpisodeSortSelector) {
            ExploreOptionSelector(
                title = AppStrings.t(lang, "settings.sort_episodes"),
                options = episodeSortOptions(lang),
                selected = episodeSort,
                onSelect = {
                    episodeSort = it ?: "number_asc"
                    showEpisodeSortSelector = false
                },
                onDismiss = { showEpisodeSortSelector = false },
                accentColor = accentColor
            )
        }

        AnimatedVisibility(
            visible = showStickyBar,
            enter = fadeIn(tween(150)) + slideInVertically { -it },
            exit = fadeOut(tween(150)) + slideOutVertically { -it },
            modifier = Modifier.align(Alignment.TopStart).fillMaxWidth()
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(Color(0xFF050505).copy(alpha = 0.96f))
                    .statusBarsPadding()
                    .padding(start = 4.dp, end = 16.dp, top = 4.dp, bottom = 4.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Box(
                    modifier = Modifier
                        .size(44.dp)
                        .clip(CircleShape)
                        .clickable { onBack() },
                    contentAlignment = Alignment.Center
                ) {
                    Icon(FluxaIcons.ArrowBack, null, tint = Color.White, modifier = Modifier.size(22.dp))
                }
                Spacer(Modifier.width(4.dp))
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .height(38.dp),
                    contentAlignment = Alignment.CenterStart
                ) {
                    if (stickyLogoUrl != null && !stickyLogoLoadFailed) {
                        AsyncImage(
                            model = stickyLogoUrl,
                            contentDescription = titleText,
                            modifier = Modifier
                                .fillMaxWidth(0.62f)
                                .height(32.dp),
                            contentScale = ContentScale.Fit,
                            alignment = Alignment.CenterStart,
                            onError = { stickyLogoLoadFailed = true }
                        )
                    } else {
                        Text(
                            text = titleText,
                            color = Color.White,
                            fontSize = 17.sp,
                            fontWeight = FontWeight.Bold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                }
                Box(
                    modifier = Modifier
                        .size(44.dp)
                        .clip(CircleShape)
                        .clickable { onToggleWatchlist() },
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        if (isInWatchlist) FluxaIcons.Check else FluxaIcons.Add,
                        null,
                        tint = if (isInWatchlist) accentColor else Color.White,
                        modifier = Modifier.size(22.dp)
                    )
                }
            }
        }
    }
}

private fun mobileSeasonDropdownLabel(season: Int, lang: String): String {
    return if (season == 0) {
        AppStrings.t(lang, "format.specials_dropdown")
    } else {
        AppStrings.format(lang, "format.season_dropdown", season)
    }
}

@Composable
private fun MobileSeasonTabsSelector(
    seasons: List<Int>,
    selectedSeason: Int,
    lang: String,
    accentColor: Color,
    onSelectSeason: (Int) -> Unit,
    onLongPressSeason: (Int) -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .horizontalScroll(rememberScrollState())
            .padding(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        seasons.forEach { season ->
            val selected = season == selectedSeason
            val label = mobileSeasonPlainLabel(season, lang)
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(12.dp))
                    .background(if (selected) accentColor else Color.White.copy(alpha = 0.08f))
                    .border(
                        1.dp,
                        if (selected) Color.White.copy(alpha = 0.28f) else Color.White.copy(alpha = 0.08f),
                        RoundedCornerShape(12.dp)
                    )
                    .pointerInput(season) {
                        detectTapGestures(
                            onTap = { onSelectSeason(season) },
                            onLongPress = { onLongPressSeason(season) }
                        )
                    }
                    .padding(horizontal = 16.dp, vertical = 10.dp)
            ) {
                Text(
                    text = label,
                    color = if (selected) Color.Black else Color.White,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1
                )
            }
        }
    }
}

@Composable
private fun MobileSeasonPosterSelector(
    detailId: String,
    seasons: List<Int>,
    selectedSeason: Int,
    seasonPosters: Map<String, String>,
    fallbackPoster: String?,
    lang: String,
    accentColor: Color,
    onSelectSeason: (Int) -> Unit,
    onLongPressSeason: (Int) -> Unit
) {
    val context = LocalContext.current
    LazyRow(
        contentPadding = PaddingValues(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        mobileItems(seasons, key = { it }) { season ->
            val selected = season == selectedSeason
            val label = mobileSeasonShortLabel(season, lang)
            val poster = seasonPosters[season.toString()] ?: fallbackPoster
            val posterRequest = poster?.takeIf { it.isNotBlank() }?.let { url ->
                remember(detailId, season, url) {
                    ImageRequest.Builder(context)
                        .data(url)
                        .memoryCacheKey("detail-season:$detailId:$season:$url")
                        .diskCacheKey(url)
                        .build()
                }
            }
            Column(
                modifier = Modifier
                    .width(88.dp)
                    .pointerInput(season) {
                        detectTapGestures(
                            onTap = { onSelectSeason(season) },
                            onLongPress = { onLongPressSeason(season) }
                        )
                    },
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(7.dp)
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(132.dp)
                        .clip(RoundedCornerShape(12.dp))
                        .background(Color.White.copy(alpha = 0.08f))
                        .border(
                            2.dp,
                            if (selected) accentColor else Color.White.copy(alpha = 0.10f),
                            RoundedCornerShape(12.dp)
                        )
                ) {
                    if (posterRequest != null) {
                        AsyncImage(
                            model = posterRequest,
                            contentDescription = label,
                            modifier = Modifier.fillMaxSize(),
                            contentScale = ContentScale.Crop
                        )
                    }
                    Box(
                        modifier = Modifier
                            .align(Alignment.BottomCenter)
                            .fillMaxWidth()
                            .background(
                                Brush.verticalGradient(
                                    listOf(Color.Transparent, Color.Black.copy(alpha = 0.78f))
                                )
                            )
                            .padding(horizontal = 6.dp, vertical = 8.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            text = label,
                            color = Color.White,
                            fontSize = 12.sp,
                            fontWeight = FontWeight.Black,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            textAlign = TextAlign.Center
                        )
                    }
                }
                Box(
                    modifier = Modifier
                        .height(3.dp)
                        .width(38.dp)
                        .clip(RoundedCornerShape(999.dp))
                        .background(if (selected) accentColor else Color.Transparent)
                )
            }
        }
    }
}

private fun mobileSeasonPlainLabel(season: Int, lang: String): String {
    return if (season == 0) {
        AppStrings.t(lang, "auto.specials")
    } else {
        AppStrings.format(lang, "format.season_number", season)
    }
}

private fun mobileSeasonShortLabel(season: Int, lang: String): String {
    return if (season == 0) {
        AppStrings.t(lang, "auto.specials")
    } else {
        AppStrings.format(lang, "format.season_short", season)
    }
}

private fun mobileEpisodesThroughSeason(episodes: List<Video>, season: Int): List<Video> {
    if (season == 0) return emptyList()
    return episodes
        .filter { episode ->
            val episodeSeason = episode.season ?: season
            episodeSeason in 1..season && !detailIsUpcoming(episode.released)
        }
        .sortedWith(compareBy<Video>({ it.season ?: season }, { it.number ?: Int.MAX_VALUE }))
}
