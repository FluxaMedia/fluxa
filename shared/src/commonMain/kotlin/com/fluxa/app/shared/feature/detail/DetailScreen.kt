package com.fluxa.app.shared.feature.detail

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Share
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.CatalogCard
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
    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        when {
            content != null -> DetailContent(content = content, language = language, onAction = onAction)
            state.isLoading -> DetailLoading()
            else -> DetailEmpty(text = AppStrings.t(language, state.errorKey ?: "auto.no_results_found"))
        }
        TopBar(onBack = onBack, onShareRequested = onShareRequested)
    }
}

@Composable
private fun TopBar(onBack: () -> Unit, onShareRequested: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(
                Brush.verticalGradient(
                    colorStops = arrayOf(0f to Color.Black.copy(alpha = 0.45f), 1f to Color.Transparent)
                )
            )
            .padding(top = 44.dp, bottom = 24.dp, start = 12.dp, end = 12.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Icon(
            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
            contentDescription = AppStrings.t(null, "common.close"),
            tint = Color.White,
            modifier = Modifier.size(28.dp).clickable(onClick = onBack)
        )
        Icon(
            imageVector = Icons.Filled.Share,
            contentDescription = null,
            tint = Color.White,
            modifier = Modifier.size(24.dp).clickable(onClick = onShareRequested)
        )
    }
}

@Composable
private fun DetailContent(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit
) {
    var activeTab by remember(content.id) { mutableStateOf(DetailTab.Episodes) }
    val isSeries = content.type == "series" && content.availableSeasons.isNotEmpty()

    LazyColumn(modifier = Modifier.fillMaxSize()) {
        item(key = "hero") {
            Hero(content = content, language = language)
        }
        item(key = "body") {
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
        if (isSeries || content.relatedItems.isNotEmpty()) {
            item(key = "tabs") {
                TabRow(
                    isSeries = isSeries,
                    hasRelated = content.relatedItems.isNotEmpty(),
                    activeTab = activeTab,
                    language = language,
                    onTabSelected = { activeTab = it },
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
        item(key = "bottom-spacer") {
            Box(modifier = Modifier.height(32.dp))
        }
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
            Text(
                text = content.title,
                color = Color.White,
                fontSize = 26.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(top = 4.dp)
            )
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
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(6.dp))
            .background(Color(0xFFE0E0E0))
            .clickable { onAction(DetailAction.Play()) }
            .padding(horizontal = 16.dp, vertical = 12.dp)
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Icon(Icons.Filled.PlayArrow, contentDescription = null, tint = Color.Black, modifier = Modifier.size(20.dp))
            Text(
                text = label,
                color = Color.Black,
                fontWeight = FontWeight.Bold,
                fontSize = 16.sp,
                modifier = Modifier.padding(start = 8.dp)
            )
        }
    }
}

@Composable
private fun RestartButton(language: String?, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(46.dp)
            .clip(RoundedCornerShape(6.dp))
            .background(FluxaColors.surfaceRaised)
            .clickable(onClick = onClick),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(Icons.Filled.Refresh, contentDescription = null, tint = Color.White, modifier = Modifier.size(18.dp))
        Text(
            text = AppStrings.t(language, "auto.restart"),
            color = Color.White,
            fontWeight = FontWeight.Bold,
            fontSize = 15.sp,
            modifier = Modifier.padding(start = 8.dp)
        )
    }
}

@Composable
private fun DownloadButton(content: DetailUiModel, language: String?, onAction: (DetailAction) -> Unit) {
    val selectedEpisodeId = content.selectedEpisodeId
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(46.dp)
            .clip(RoundedCornerShape(6.dp))
            .background(FluxaColors.surfaceRaised)
            .clickable {
                if (selectedEpisodeId != null) onAction(DetailAction.DownloadEpisode(selectedEpisodeId))
                else onAction(DetailAction.DownloadSeason(content.selectedSeason))
            },
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            text = AppStrings.t(language, "auto.download"),
            color = Color.White,
            fontWeight = FontWeight.Bold,
            fontSize = 15.sp
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
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier.clickable(onClick = onClick)
    ) {
        Icon(icon, contentDescription = null, tint = Color.White, modifier = Modifier.size(24.dp))
        Text(text = label, color = Color.White.copy(alpha = 0.8f), fontSize = 11.sp, modifier = Modifier.padding(top = 6.dp))
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

@Composable
private fun SeasonSelector(content: DetailUiModel, language: String?, onAction: (DetailAction) -> Unit) {
    var expanded by remember(content.id) { mutableStateOf(false) }
    Box(modifier = Modifier.padding(horizontal = 20.dp, vertical = 12.dp)) {
        Row(
            modifier = Modifier.clickable { expanded = !expanded },
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "${AppStrings.t(language, "auto.season")} ${content.selectedSeason}",
                color = Color.White,
                fontWeight = FontWeight.Bold,
                fontSize = 16.sp
            )
        }
        if (expanded) {
            Column(
                modifier = Modifier
                    .padding(top = 32.dp)
                    .background(FluxaColors.surfaceRaised, RoundedCornerShape(8.dp))
            ) {
                content.availableSeasons.forEach { season ->
                    Text(
                        text = "${AppStrings.t(language, "auto.season")} $season",
                        color = Color.White,
                        fontSize = 14.sp,
                        modifier = Modifier
                            .clickable {
                                expanded = false
                                onAction(DetailAction.SeasonSelected(season))
                            }
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp, vertical = 10.dp)
                    )
                }
            }
        }
    }
}

@Composable
private fun EpisodeRow(episode: DetailEpisodeUiModel, content: DetailUiModel, onAction: (DetailAction) -> Unit) {
    val selected = episode.id == content.selectedEpisodeId
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = !episode.isUpcoming) { onAction(DetailAction.EpisodeSelected(episode.id)) }
            .padding(horizontal = 20.dp, vertical = 10.dp),
        verticalAlignment = Alignment.Top
    ) {
        Box(
            modifier = Modifier
                .width(120.dp)
                .aspectRatio(16f / 9f)
                .clip(RoundedCornerShape(4.dp))
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
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            episode.runtimeLabel?.let {
                Text(text = it, color = Color.White.copy(alpha = 0.6f), fontSize = 12.sp, modifier = Modifier.padding(top = 2.dp))
            }
            episode.description?.takeIf { it.isNotBlank() }?.let {
                Text(
                    text = it,
                    color = Color.White.copy(alpha = 0.6f),
                    fontSize = 12.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 4.dp)
                )
            }
        }
        if (!episode.isUpcoming) {
            Icon(
                imageVector = Icons.Filled.Add,
                contentDescription = null,
                tint = Color.White.copy(alpha = 0.7f),
                modifier = Modifier
                    .padding(start = 8.dp, top = 4.dp)
                    .size(20.dp)
                    .clickable { onAction(DetailAction.DownloadEpisode(episode.id)) }
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
