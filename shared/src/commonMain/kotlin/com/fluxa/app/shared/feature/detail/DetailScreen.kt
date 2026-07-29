package com.fluxa.app.shared.feature.detail

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.ArrowDropDown
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Share
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.LocalHeroTrailerSurface
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.LocalDeviceType
import com.fluxa.app.ui.catalog.LocalWindowWidthClass
import com.fluxa.app.ui.catalog.WindowWidthClass
import com.fluxa.app.ui.catalog.FluxaColors

private enum class DetailTab { Episodes, MoreLikeThis }

@Composable
fun DetailScreen(
    state: DetailUiState,
    language: String?,
    onAction: (DetailAction) -> Unit,
    onBack: () -> Unit = {},
    onShareRequested: () -> Unit = {},
    modifier: Modifier = Modifier
) {
    val content = state.content
    val isMobile = LocalDeviceType.current == DeviceType.Mobile
    var showStickyContext by remember(content?.id) { mutableStateOf(false) }
    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        when {
            content != null -> DetailContent(
                content = content,
                language = language,
                onAction = onAction,
                onStickyContextVisibilityChanged = { showStickyContext = it }
            )
            state.isLoading -> DetailLoading()
            else -> DetailEmpty(text = AppStrings.t(language, state.errorKey ?: "auto.no_results_found"))
        }
        TopBar(
            content = content,
            language = language,
            showStickyContext = isMobile && showStickyContext,
            onAction = onAction,
            onBack = onBack,
            onShareRequested = onShareRequested
        )
    }
}

@Composable
private fun TopBar(
    content: DetailUiModel?,
    language: String?,
    showStickyContext: Boolean,
    onAction: (DetailAction) -> Unit,
    onBack: () -> Unit,
    onShareRequested: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .then(
                if (showStickyContext) Modifier.background(Color.Black.copy(alpha = 0.94f)) else Modifier.background(
                    Brush.verticalGradient(
                    colorStops = arrayOf(0f to Color.Black.copy(alpha = 0.45f), 1f to Color.Transparent)
                    )
                )
            )
            .padding(top = 44.dp, bottom = if (showStickyContext) 12.dp else 24.dp, start = 12.dp, end = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
            contentDescription = AppStrings.t(language, "common.back"),
            tint = Color.White,
            modifier = Modifier.size(32.dp).clickable(onClick = onBack)
        )
        if (showStickyContext && content != null) {
            StickyContentIdentity(content = content, modifier = Modifier.weight(1f).padding(horizontal = 16.dp))
            Icon(
                imageVector = if (content.isInWatchlist) Icons.Filled.Check else Icons.Filled.Add,
                contentDescription = AppStrings.t(language, if (content.isInWatchlist) "auto.in_list" else "auto.my_list"),
                tint = Color.White,
                modifier = Modifier.size(28.dp).clickable { onAction(DetailAction.ToggleWatchlist) }
            )
        } else {
            Box(modifier = Modifier.weight(1f))
        }
        Icon(
            imageVector = Icons.Filled.Share,
            contentDescription = AppStrings.t(language, "common.share"),
            tint = Color.White,
            modifier = Modifier.padding(start = 18.dp).size(28.dp).clickable(onClick = onShareRequested)
        )
    }
}

@Composable
private fun StickyContentIdentity(content: DetailUiModel, modifier: Modifier = Modifier) {
    if (content.logoUrl != null) {
        FluxaRemoteImage(
            imageUrl = content.logoUrl,
            cacheKey = "detail-sticky-logo:${content.id}",
            contentDescription = content.title,
            modifier = modifier.height(28.dp),
            contentScale = ContentScale.Fit
        )
    } else {
        Text(
            text = content.title,
            color = Color.White,
            fontSize = 16.sp,
            fontWeight = FontWeight.Bold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = modifier
        )
    }
}

@Composable
private fun DetailContent(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    onStickyContextVisibilityChanged: (Boolean) -> Unit
) {
    val isSeries = content.type == "series" && content.availableSeasons.isNotEmpty()
    val hasSecondary = isSeries || content.relatedItems.isNotEmpty()
    val twoPane = LocalWindowWidthClass.current == WindowWidthClass.Expanded &&
        LocalDeviceType.current != DeviceType.TV
    if (twoPane && hasSecondary) {
        val listState = rememberLazyListState()
        ObserveStickyContextVisibility(listState, onStickyContextVisibilityChanged)
        Row(modifier = Modifier.fillMaxSize()) {
            LazyColumn(state = listState, modifier = Modifier.width(440.dp).fillMaxHeight()) {
                item(key = "hero") { Hero(content = content, language = language) }
                item(key = "body") { DetailBody(content = content, language = language, onAction = onAction) }
                item(key = "bottom-spacer") { Box(modifier = Modifier.height(32.dp)) }
            }
            DetailSecondaryPane(
                content = content,
                language = language,
                onAction = onAction,
                isSeries = isSeries,
                modifier = Modifier.weight(1f).fillMaxHeight()
            )
        }
    } else {
        var activeTab by remember(content.id) { mutableStateOf(DetailTab.Episodes) }
        val listState = rememberLazyListState()
        ObserveStickyContextVisibility(listState, onStickyContextVisibilityChanged)
        LazyColumn(state = listState, modifier = Modifier.fillMaxSize()) {
            item(key = "hero") { Hero(content = content, language = language) }
            item(key = "body") { DetailBody(content = content, language = language, onAction = onAction) }
            detailSecondaryItems(content, language, onAction, isSeries, activeTab) { activeTab = it }
            item(key = "bottom-spacer") { Box(modifier = Modifier.height(32.dp)) }
        }
    }
}

@Composable
private fun ObserveStickyContextVisibility(
    listState: LazyListState,
    onStickyContextVisibilityChanged: (Boolean) -> Unit
) {
    val logoExitOffset = with(LocalDensity.current) { 390.dp.roundToPx() }
    val showStickyContext by remember(listState, logoExitOffset) {
        derivedStateOf {
            listState.firstVisibleItemIndex > 0 ||
                (listState.firstVisibleItemIndex == 0 && listState.firstVisibleItemScrollOffset >= logoExitOffset)
        }
    }
    LaunchedEffect(showStickyContext) {
        onStickyContextVisibilityChanged(showStickyContext)
    }
}

@Composable
private fun DetailSecondaryPane(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    isSeries: Boolean,
    modifier: Modifier = Modifier
) {
    var activeTab by remember(content.id) { mutableStateOf(DetailTab.Episodes) }
    LazyColumn(modifier = modifier) {
        detailSecondaryItems(content, language, onAction, isSeries, activeTab) { activeTab = it }
        item(key = "bottom-spacer") { Box(modifier = Modifier.height(32.dp)) }
    }
}

private fun androidx.compose.foundation.lazy.LazyListScope.detailSecondaryItems(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    isSeries: Boolean,
    activeTab: DetailTab,
    onTabSelected: (DetailTab) -> Unit
) {
    if (isSeries || content.relatedItems.isNotEmpty()) {
        item(key = "tabs") {
            TabRow(
                isSeries = isSeries,
                hasRelated = content.relatedItems.isNotEmpty(),
                activeTab = activeTab,
                language = language,
                onTabSelected = onTabSelected,
                modifier = Modifier.padding(top = 24.dp)
            )
        }
    }
    when {
        isSeries && activeTab == DetailTab.Episodes -> {
            item(key = "season-selector") {
                SeasonSelector(content = content, language = language, onAction = onAction)
            }
            items(content.seasonEpisodes, key = { it.id }) { episode ->
                EpisodeRow(episode = episode, content = content, onAction = onAction)
            }
        }
        content.relatedItems.isNotEmpty() -> {
            item(key = "related") {
                RelatedGrid(items = content.relatedItems, onAction = onAction)
            }
        }
        else -> Unit
    }
}

@Composable
private fun DetailBody(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit
) {
    Column(modifier = Modifier.padding(horizontal = 20.dp)) {
        Column(verticalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.padding(top = 16.dp)) {
            ResumeButton(content = content, language = language, onAction = onAction)
            if (content.resumeProgress > 0L && content.resumeVideoId != null) {
                RestartButton(language = language, onClick = { onAction(DetailAction.Play(fromStart = true)) })
            }
            DownloadButton(content = content, language = language, onAction = onAction)
        }
        if (content.description.isNotBlank()) {
            Text(
                text = content.description,
                color = Color.White,
                fontSize = 14.sp,
                lineHeight = 19.sp,
                modifier = Modifier.padding(top = 16.dp)
            )
        }
        if (content.castNames.isNotEmpty()) {
            CastLine(names = content.castNames, language = language, modifier = Modifier.padding(top = 12.dp))
        }
        ActionRow(
            content = content,
            language = language,
            onAction = onAction,
            modifier = Modifier.padding(top = 20.dp)
        )
    }
}

@Composable
private fun Hero(content: DetailUiModel, language: String?) {
    Box(modifier = Modifier.fillMaxWidth().height(560.dp)) {
        FluxaRemoteImage(
            imageUrl = content.backgroundUrl ?: content.posterUrl,
            cacheKey = "detail-hero:${content.id}",
            contentDescription = null,
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Crop
        )
        val trailerSurface = LocalHeroTrailerSurface.current
        if (content.trailerUrl != null && trailerSurface != null) {
            val trailerAlpha by animateFloatAsState(targetValue = 1f, label = "detail-hero-trailer-fade")
            trailerSurface(content.trailerUrl, emptyList(), {}, Modifier.fillMaxSize().alpha(trailerAlpha))
        }
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.verticalGradient(
                        colorStops = arrayOf(
                            0f to Color.Transparent,
                            0.55f to Color.Transparent,
                            0.85f to FluxaColors.background.copy(alpha = 0.75f),
                            1f to FluxaColors.background
                        )
                    )
                )
        )
        Column(
            modifier = Modifier
                .align(Alignment.BottomStart)
                .padding(horizontal = 20.dp, vertical = 16.dp)
        ) {
            Text(
                text = if (content.type == "series") AppStrings.t(language, "auto.series") else AppStrings.t(language, "auto.movie"),
                color = Color.White.copy(alpha = 0.7f),
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = 1.5.sp
            )
            if (content.logoUrl != null) {
                FluxaRemoteImage(
                    imageUrl = content.logoUrl,
                    cacheKey = "detail-logo:${content.id}",
                    contentDescription = content.title,
                    modifier = Modifier
                        .padding(top = 8.dp)
                        .heightIn(max = 90.dp)
                        .fillMaxWidth(0.7f),
                    contentScale = ContentScale.Fit
                )
            } else {
                Text(
                    text = content.title,
                    color = Color.White,
                    fontSize = 26.sp,
                    fontWeight = FontWeight.Bold,
                    modifier = Modifier.padding(top = 4.dp)
                )
            }
            val metaParts = buildList {
                content.ageRating?.takeIf { it.isNotBlank() }?.let { add(it to true) }
                if (content.releaseLabel.isNotBlank()) add(content.releaseLabel to false)
                if (content.type == "series" && content.availableSeasons.isNotEmpty()) {
                    add("${content.availableSeasons.size} ${AppStrings.t(language, "auto.seasons")}" to false)
                } else {
                    content.runtimeLabel?.takeIf { it.isNotBlank() }?.let { add(it to false) }
                }
            }
            if (metaParts.isNotEmpty()) {
                Row(
                    modifier = Modifier.padding(top = 10.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    metaParts.forEach { (part, isBadge) ->
                        if (isBadge) {
                            Text(
                                text = part,
                                color = Color.White,
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier
                                    .background(FluxaColors.accent, RoundedCornerShape(3.dp))
                                    .padding(horizontal = 6.dp, vertical = 2.dp)
                            )
                        } else {
                            Text(text = part, color = Color.White.copy(alpha = 0.8f), fontSize = 13.sp)
                        }
                    }
                }
            }
            val ratings = content.ratings.ifEmpty {
                content.ratingLabel.takeIf { it.isNotBlank() }?.let { listOf(DetailRatingUiModel("IMDb", it)) }.orEmpty()
            }
            if (ratings.isNotEmpty()) {
                LazyRow(
                    modifier = Modifier.padding(top = 10.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    items(ratings, key = { "${it.source}:${it.value}" }) { rating ->
                        Text(
                            text = "${rating.source} ${rating.value}",
                            color = Color.White,
                            fontSize = 12.sp,
                            fontWeight = FontWeight.SemiBold,
                            modifier = Modifier
                                .background(Color.White.copy(alpha = 0.14f), RoundedCornerShape(8.dp))
                                .padding(horizontal = 8.dp, vertical = 4.dp)
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun ResumeButton(content: DetailUiModel, language: String?, onAction: (DetailAction) -> Unit) {
    val resumeEpisode = content.resumeVideoId?.let { id -> content.seasonEpisodes.firstOrNull { it.id == id } }
    val label = if (content.resumeProgress > 0L && resumeEpisode?.season != null && resumeEpisode.number != null) {
        AppStrings.t(language, "auto.resume") + " S${resumeEpisode.season} E${resumeEpisode.number}"
    } else if (content.resumeProgress > 0L) {
        AppStrings.t(language, "auto.resume")
    } else {
        AppStrings.t(language, "common.play")
    }
    val isTv = LocalDeviceType.current == DeviceType.TV
    var focused by remember { mutableStateOf(false) }
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier
            .fillMaxWidth()
            .height(54.dp)
            .clip(RoundedCornerShape(14.dp))
            .background(Color.White)
            .onFocusChanged { focused = it.isFocused }
            .then(if (isTv && focused) Modifier.scale(1.03f).border(3.dp, Color.White, RoundedCornerShape(14.dp)) else Modifier)
            .clickable { onAction(DetailAction.Play()) }
            .padding(horizontal = 18.dp),
        verticalArrangement = Arrangement.Center
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Icon(
                Icons.Rounded.PlayArrow,
                contentDescription = null,
                tint = Color.Black,
                modifier = Modifier.size(27.dp)
            )
            Text(
                text = label,
                color = Color.Black,
                fontWeight = FontWeight.ExtraBold,
                fontSize = 16.sp,
                modifier = Modifier.padding(start = 8.dp)
            )
        }
    }
}

@Composable
private fun RestartButton(language: String?, onClick: () -> Unit) {
    val isTv = LocalDeviceType.current == DeviceType.TV
    var focused by remember { mutableStateOf(false) }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(46.dp)
            .clip(RoundedCornerShape(6.dp))
            .background(if (isTv && focused) Color.White else FluxaColors.surfaceRaised)
            .onFocusChanged { focused = it.isFocused }
            .then(if (isTv && focused) Modifier.scale(1.03f).border(2.dp, Color.White, RoundedCornerShape(6.dp)) else Modifier)
            .clickable(onClick = onClick),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(Icons.Filled.Refresh, contentDescription = null, tint = if (isTv && focused) Color.Black else Color.White, modifier = Modifier.size(18.dp))
        Text(
            text = AppStrings.t(language, "auto.restart"),
            color = if (isTv && focused) Color.Black else Color.White,
            fontWeight = FontWeight.Bold,
            fontSize = 15.sp,
            modifier = Modifier.padding(start = 8.dp)
        )
    }
}

@Composable
private fun DownloadButton(content: DetailUiModel, language: String?, onAction: (DetailAction) -> Unit) {
    val selectedEpisodeId = content.selectedEpisodeId
    val isTv = LocalDeviceType.current == DeviceType.TV
    var focused by remember { mutableStateOf(false) }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(46.dp)
            .clip(RoundedCornerShape(6.dp))
            .background(if (isTv && focused) Color.White else FluxaColors.surfaceRaised)
            .onFocusChanged { focused = it.isFocused }
            .then(if (isTv && focused) Modifier.scale(1.03f).border(2.dp, Color.White, RoundedCornerShape(6.dp)) else Modifier)
            .clickable {
                if (selectedEpisodeId != null) onAction(DetailAction.DownloadEpisode(selectedEpisodeId))
                else onAction(DetailAction.DownloadSeason(content.selectedSeason))
            },
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(Icons.Filled.Download, contentDescription = null, tint = if (isTv && focused) Color.Black else Color.White, modifier = Modifier.size(18.dp))
        Text(
            text = AppStrings.t(language, "auto.download"),
            color = if (isTv && focused) Color.Black else Color.White,
            fontWeight = FontWeight.Bold,
            fontSize = 15.sp,
            modifier = Modifier.padding(start = 8.dp)
        )
    }
}

@Composable
private fun CastLine(names: List<String>, language: String?, modifier: Modifier = Modifier) {
    var expanded by remember(names) { mutableStateOf(false) }
    val visibleNames = if (expanded) names else names.take(3)
    val canExpand = !expanded && names.size > visibleNames.size
    Text(
        modifier = modifier.clickable(enabled = canExpand) { expanded = true },
        fontSize = 13.sp,
        color = Color.White.copy(alpha = 0.6f),
        maxLines = if (expanded) Int.MAX_VALUE else 2,
        overflow = TextOverflow.Ellipsis,
        text = buildString {
            append(AppStrings.t(language, "auto.cast"))
            append(": ")
            append(visibleNames.joinToString(", "))
            if (canExpand) {
                append("  ")
                append(AppStrings.t(language, "common.more"))
            }
        }
    )
}

@Composable
private fun ActionRow(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    modifier: Modifier = Modifier
) {
    Row(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceEvenly
    ) {
        ActionItem(
            icon = if (content.isInWatchlist) Icons.Filled.Check else Icons.Filled.Add,
            label = AppStrings.t(language, if (content.isInWatchlist) "auto.in_list" else "auto.my_list"),
            onClick = { onAction(DetailAction.ToggleWatchlist) }
        )
    }
}

@Composable
private fun ActionItem(icon: androidx.compose.ui.graphics.vector.ImageVector, label: String, onClick: () -> Unit) {
    val isTv = LocalDeviceType.current == DeviceType.TV
    var focused by remember { mutableStateOf(false) }
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier
            .clip(RoundedCornerShape(10.dp))
            .background(if (isTv && focused) Color.White else Color.Transparent)
            .onFocusChanged { focused = it.isFocused }
            .clickable(onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 8.dp)
    ) {
        Icon(icon, contentDescription = null, tint = if (isTv && focused) Color.Black else Color.White, modifier = Modifier.size(24.dp))
        Text(text = label, color = if (isTv && focused) Color.Black else Color.White.copy(alpha = 0.8f), fontSize = 11.sp, modifier = Modifier.padding(top = 6.dp))
    }
}

@Composable
private fun TabRow(
    isSeries: Boolean,
    hasRelated: Boolean,
    activeTab: DetailTab,
    language: String?,
    onTabSelected: (DetailTab) -> Unit,
    modifier: Modifier = Modifier
) {
    Row(modifier = modifier.fillMaxWidth().padding(horizontal = 20.dp), horizontalArrangement = Arrangement.spacedBy(24.dp)) {
        if (isSeries) {
            DetailTabLabel(
                text = AppStrings.t(language, "auto.episodes"),
                selected = activeTab == DetailTab.Episodes,
                onClick = { onTabSelected(DetailTab.Episodes) }
            )
        }
        if (hasRelated) {
            DetailTabLabel(
                text = AppStrings.t(language, "auto.similar_titles"),
                selected = activeTab == DetailTab.MoreLikeThis || !isSeries,
                onClick = { onTabSelected(DetailTab.MoreLikeThis) }
            )
        }
    }
}

@Composable
private fun DetailTabLabel(text: String, selected: Boolean, onClick: () -> Unit) {
    Column(modifier = Modifier.clickable(onClick = onClick)) {
        Text(
            text = text,
            color = if (selected) Color.White else Color.White.copy(alpha = 0.5f),
            fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal,
            fontSize = 14.sp
        )
        Box(
            modifier = Modifier
                .padding(top = 8.dp)
                .height(2.dp)
                .width(if (selected) 24.dp else 0.dp)
                .background(if (selected) FluxaColors.accent else Color.Transparent)
        )
    }
}

@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
private fun SeasonSelector(content: DetailUiModel, language: String?, onAction: (DetailAction) -> Unit) {
    var sheetOpen by remember(content.id) { mutableStateOf(false) }
    var triggerFocused by remember { mutableStateOf(false) }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .padding(horizontal = 20.dp, vertical = 12.dp)
            .clip(RoundedCornerShape(6.dp))
            .onFocusChanged { triggerFocused = it.isFocused }
            .background(if (triggerFocused) Color.White.copy(alpha = 0.9f) else FluxaColors.surfaceRaised)
            .clickable { sheetOpen = true }
            .padding(horizontal = 14.dp, vertical = 10.dp)
    ) {
        Text(
            text = "${AppStrings.t(language, "auto.season")} ${content.selectedSeason}",
            color = if (triggerFocused) Color.Black else Color.White,
            fontWeight = FontWeight.Bold,
            fontSize = 15.sp
        )
        Icon(
            imageVector = Icons.Filled.ArrowDropDown,
            contentDescription = null,
            tint = if (triggerFocused) Color.Black else Color.White,
            modifier = Modifier.padding(start = 4.dp).size(20.dp)
        )
    }
    if (sheetOpen) {
        val selectedRowFocusRequester = remember(content.id) { FocusRequester() }
        LaunchedEffect(sheetOpen) {
            if (sheetOpen) runCatching { selectedRowFocusRequester.requestFocus() }
        }
        androidx.compose.material3.ModalBottomSheet(
            onDismissRequest = { sheetOpen = false },
            containerColor = FluxaColors.surfaceRaised
        ) {
            Column(modifier = Modifier.padding(bottom = 24.dp)) {
                content.availableSeasons.forEach { season ->
                    val selected = season == content.selectedSeason
                    var rowFocused by remember { mutableStateOf(false) }
                    Text(
                        text = "${AppStrings.t(language, "auto.season")} $season",
                        color = if (rowFocused) Color.Black else if (selected) FluxaColors.accent else Color.White,
                        fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal,
                        fontSize = 16.sp,
                        modifier = Modifier
                            .let { if (selected) it.focusRequester(selectedRowFocusRequester) else it }
                            .onFocusChanged { rowFocused = it.isFocused }
                            .background(if (rowFocused) Color.White else Color.Transparent)
                            .clickable {
                                sheetOpen = false
                                onAction(DetailAction.SeasonSelected(season))
                            }
                            .fillMaxWidth()
                            .padding(horizontal = 20.dp, vertical = 14.dp)
                    )
                }
            }
        }
    }
}

@Composable
private fun EpisodeRow(episode: DetailEpisodeUiModel, content: DetailUiModel, onAction: (DetailAction) -> Unit) {
    val selected = episode.id == content.selectedEpisodeId
    val isTv = LocalDeviceType.current == DeviceType.TV
    var focused by remember { mutableStateOf(false) }
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .background(if (isTv && focused) Color.White.copy(alpha = 0.16f) else Color.Transparent)
            .onFocusChanged { focused = it.isFocused }
            .clickable(enabled = !episode.isUpcoming) { onAction(DetailAction.EpisodeSelected(episode.id)) }
            .padding(horizontal = 20.dp, vertical = 12.dp)
    ) {
        Row(verticalAlignment = Alignment.Top) {
            Box(
                modifier = Modifier
                    .weight(1f)
                    .aspectRatio(16f / 9f)
                    .clip(RoundedCornerShape(6.dp))
                    .background(FluxaColors.surfaceCard)
            ) {
                FluxaRemoteImage(
                    imageUrl = episode.thumbnailUrl,
                    cacheKey = "detail-episode:${episode.id}",
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop
                )
                if (selected) {
                    Box(
                        modifier = Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.35f)),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(Icons.Filled.PlayArrow, contentDescription = null, tint = Color.White)
                    }
                }
            }
            Column(modifier = Modifier.padding(start = 12.dp).weight(1f)) {
                Text(
                    text = listOfNotNull(episode.number?.let { "$it." }, episode.title).joinToString(" "),
                    color = Color.White,
                    fontWeight = FontWeight.Medium,
                    fontSize = 14.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
                episode.runtimeLabel?.let {
                    Text(text = it, color = Color.White.copy(alpha = 0.6f), fontSize = 12.sp, modifier = Modifier.padding(top = 4.dp))
                }
            }
            if (!episode.isUpcoming) {
                Icon(
                    imageVector = Icons.Filled.Download,
                    contentDescription = null,
                    tint = Color.White.copy(alpha = 0.7f),
                    modifier = Modifier
                        .padding(start = 8.dp, top = 4.dp)
                        .size(20.dp)
                        .clickable { onAction(DetailAction.DownloadEpisode(episode.id)) }
                )
            }
        }
        episode.description?.takeIf { it.isNotBlank() }?.let {
            Text(
                text = it,
                color = Color.White.copy(alpha = 0.6f),
                fontSize = 13.sp,
                maxLines = 3,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = 8.dp)
            )
        }
    }
}

@Composable
private fun RelatedGrid(items: List<CatalogItemUiModel>, onAction: (DetailAction) -> Unit) {
    LazyRow(
        contentPadding = PaddingValues(horizontal = 20.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        items(items, key = { it.id }) { item ->
            CatalogCard(model = item.card, onClick = { onAction(DetailAction.RelatedItemSelected(item)) })
        }
    }
}

@Composable
private fun DetailLoading() {
    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        CircularProgressIndicator(color = Color.White)
    }
}

@Composable
private fun DetailEmpty(text: String) {
    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Text(text = text, color = Color.White)
    }
}
