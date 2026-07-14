package com.fluxa.app.shared.feature.detail

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun DetailScreen(
    state: DetailUiState,
    language: String?,
    onAction: (DetailAction) -> Unit,
    modifier: Modifier = Modifier
) {
    val content = state.content
    when {
        content != null -> DetailContent(
            content = content,
            language = language,
            onAction = onAction,
            modifier = modifier
        )
        state.isLoading -> DetailLoading(modifier)
        else -> DetailEmpty(
            text = AppStrings.t(language, state.errorKey ?: "auto.no_results_found"),
            modifier = modifier
        )
    }
}

@Composable
private fun DetailContent(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    modifier: Modifier
) {
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(FluxaColors.background)
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(280.dp)
                .background(FluxaColors.surface)
        ) {
            FluxaRemoteImage(
                imageUrl = content.backgroundUrl ?: content.posterUrl,
                cacheKey = "detail-artwork:${content.id}",
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop
            )
        }
        Column(
            modifier = Modifier.padding(horizontal = 20.dp, vertical = 16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(text = content.title, color = Color.White, fontWeight = FontWeight.Bold)
            if (content.releaseLabel.isNotBlank() || content.ratingLabel.isNotBlank()) {
                Text(
                    text = listOf(content.releaseLabel, content.ratingLabel)
                        .filter { it.isNotBlank() }
                        .joinToString(AppStrings.t(language, "auto.detail_metadata_separator")),
                    color = Color.White.copy(alpha = 0.7f)
                )
            }
            if (content.description.isNotBlank()) {
                Text(
                    text = content.description,
                    color = Color.White.copy(alpha = 0.86f),
                    maxLines = 5,
                    overflow = TextOverflow.Ellipsis
                )
            }
            Button(onClick = { onAction(DetailAction.Play) }) {
                Text(AppStrings.t(language, "common.play"))
            }
            Button(onClick = { onAction(DetailAction.ToggleWatchlist) }) {
                Text(AppStrings.t(language, if (content.isInWatchlist) "auto.in_list" else "auto.my_list"))
            }
        }
        if (content.type == "series" && content.availableSeasons.isNotEmpty()) {
            SeasonSelector(content = content, language = language, onAction = onAction)
            EpisodeList(content = content, language = language, onAction = onAction)
        }
        SourcesSection(content = content, language = language, onAction = onAction)
        if (content.relatedItems.isNotEmpty()) {
            Text(
                text = AppStrings.t(language, "auto.similar_titles"),
                color = Color.White,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(horizontal = 20.dp)
            )
            RelatedItems(items = content.relatedItems, onAction = onAction)
        }
    }
}

@Composable
private fun SeasonSelector(content: DetailUiModel, language: String?, onAction: (DetailAction) -> Unit) {
    Column(modifier = Modifier.padding(top = 8.dp)) {
        Text(
            text = AppStrings.t(language, "auto.seasons"),
            color = Color.White,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.padding(horizontal = 20.dp)
        )
        LazyRow(
            contentPadding = PaddingValues(horizontal = 20.dp, vertical = 8.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            items(content.availableSeasons, key = { it }) { season ->
                val selected = season == content.selectedSeason
                Text(
                    text = "${AppStrings.t(language, "auto.season")} $season",
                    color = if (selected) FluxaColors.background else Color.White,
                    modifier = Modifier
                        .background(if (selected) Color.White else FluxaColors.surface)
                        .clickable { onAction(DetailAction.SeasonSelected(season)) }
                        .padding(horizontal = 14.dp, vertical = 8.dp)
                )
            }
        }
    }
}

@Composable
private fun EpisodeList(content: DetailUiModel, language: String?, onAction: (DetailAction) -> Unit) {
    LazyColumn(
        modifier = Modifier
            .fillMaxWidth()
            .height(240.dp),
        contentPadding = PaddingValues(horizontal = 20.dp, vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        items(content.seasonEpisodes, key = { it.id }) { episode ->
            val selected = episode.id == content.selectedEpisodeId
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(if (selected) FluxaColors.surface else Color.Transparent)
                    .clickable(enabled = !episode.isUpcoming) { onAction(DetailAction.EpisodeSelected(episode.id)) }
                    .padding(12.dp),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Column {
                    Text(
                        text = listOfNotNull(episode.number?.let { "$it." }, episode.title)
                            .joinToString(" "),
                        color = Color.White
                    )
                    episode.description?.takeIf { it.isNotBlank() }?.let {
                        Text(text = it, color = Color.White.copy(alpha = 0.6f), maxLines = 2, overflow = TextOverflow.Ellipsis)
                    }
                }
                Text(
                    text = AppStrings.t(language, "auto.download"),
                    color = Color.White.copy(alpha = 0.7f),
                    modifier = Modifier.clickable(enabled = !episode.isUpcoming) {
                        onAction(DetailAction.DownloadEpisode(episode.id))
                    }
                )
            }
        }
    }
}

@Composable
private fun SourcesSection(content: DetailUiModel, language: String?, onAction: (DetailAction) -> Unit) {
    if (!content.hasStreamProviders && content.streams.isEmpty() && !content.isLoadingStreams) return
    Column(modifier = Modifier.padding(top = 8.dp)) {
        Text(
            text = AppStrings.t(language, "auto.sources"),
            color = Color.White,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.padding(horizontal = 20.dp)
        )
        if (content.availableAddons.size > 1) {
            LazyRow(
                contentPadding = PaddingValues(horizontal = 20.dp, vertical = 8.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                items(content.availableAddons, key = { it }) { addon ->
                    val selected = addon == content.selectedAddon
                    Text(
                        text = addon,
                        color = if (selected) FluxaColors.background else Color.White,
                        modifier = Modifier
                            .background(if (selected) Color.White else FluxaColors.surface)
                            .clickable {
                                onAction(DetailAction.AddonFilterSelected(if (selected) null else addon))
                            }
                            .padding(horizontal = 14.dp, vertical = 8.dp)
                    )
                }
            }
        }
        if (content.isLoadingStreams) {
            CircularProgressIndicator(
                color = Color.White,
                modifier = Modifier.padding(horizontal = 20.dp, vertical = 12.dp)
            )
        } else {
            Column(
                modifier = Modifier.padding(horizontal = 20.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                content.streams.forEach { stream ->
                    Text(
                        text = listOf(stream.addonName, stream.title).filter { it.isNotBlank() }.joinToString(" · "),
                        color = Color.White,
                        modifier = Modifier
                            .fillMaxWidth()
                            .background(FluxaColors.surface)
                            .clickable {
                                onAction(DetailAction.StreamSelected(stream, content.selectedEpisodeId))
                            }
                            .padding(12.dp)
                    )
                }
            }
        }
    }
}

@Composable
private fun RelatedItems(items: List<CatalogItemUiModel>, onAction: (DetailAction) -> Unit) {
    LazyRow(
        contentPadding = PaddingValues(horizontal = 20.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        items(items, key = { it.id }) { item ->
            CatalogCard(model = item.card, onClick = { onAction(DetailAction.RelatedItemSelected(item)) })
        }
    }
}

@Composable
private fun DetailLoading(modifier: Modifier) {
    Box(
        modifier = modifier
            .fillMaxSize()
            .background(FluxaColors.background),
        contentAlignment = Alignment.Center
    ) {
        CircularProgressIndicator(color = Color.White)
    }
}

@Composable
private fun DetailEmpty(text: String, modifier: Modifier) {
    Box(
        modifier = modifier
            .fillMaxSize()
            .background(FluxaColors.background),
        contentAlignment = Alignment.Center
    ) {
        Text(text = text, color = Color.White)
    }
}
