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
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDropDown
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.blur
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.stableLazyKey
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.LocalDeviceType
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.LocalAccentColor

@Composable
internal fun CastSection(members: List<DetailCastMemberUiModel>, language: String?, modifier: Modifier = Modifier) {
    Column(modifier = modifier) {
        Text(
            text = AppStrings.t(language, "auto.cast"),
            color = Color.White,
            fontSize = 18.sp,
            fontWeight = FontWeight.Bold
        )
        LazyRow(
            contentPadding = PaddingValues(top = 12.dp, end = 20.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            items(members.take(20), key = { "${it.name}:${it.character.orEmpty()}" }) { member ->
                CastMemberCard(member)
            }
        }
    }
}

@Composable
internal fun CastMemberCard(member: DetailCastMemberUiModel) {
    val initials = remember(member.name) {
        member.name.split(' ').filter(String::isNotBlank).take(2).joinToString("") { it.take(1) }.uppercase()
    }
    Box(modifier = Modifier.width(104.dp), contentAlignment = Alignment.TopCenter) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(0.dp)
        ) {
            Box(
                modifier = Modifier.size(64.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.12f)),
                contentAlignment = Alignment.Center
            ) {
                Text(initials, color = Color.White.copy(alpha = 0.78f), fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                FluxaRemoteImage(
                    imageUrl = member.profileUrl,
                    cacheKey = "detail-cast:${member.profileUrl}",
                    contentDescription = member.name,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop
                )
            }
            Text(
                text = member.name,
                color = Color.White,
                fontSize = 12.sp,
                lineHeight = 13.sp,
                fontWeight = FontWeight.SemiBold,
                textAlign = TextAlign.Center,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.fillMaxWidth().padding(top = 4.dp)
            )
            member.character?.takeIf(String::isNotBlank)?.let { character ->
                Text(
                    text = character,
                    color = Color.White.copy(alpha = 0.68f),
                    fontSize = 11.sp,
                    lineHeight = 12.sp,
                    textAlign = TextAlign.Center,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.fillMaxWidth()
                )
            }
        }
    }
}

@Composable
internal fun TabRow(
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
internal fun DetailTabLabel(text: String, selected: Boolean, onClick: () -> Unit) {
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
                .background(if (selected) LocalAccentColor.current else Color.Transparent)
        )
    }
}

@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
internal fun SeasonSelector(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    mode: String = "dropdown",
    headerStyle: Boolean = false
) {
    when (mode) {
        "tabs" -> LazyRow(
            contentPadding = PaddingValues(horizontal = 20.dp, vertical = 12.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            items(content.availableSeasons, key = { it }) { season ->
                val selected = season == content.selectedSeason
                Text(
                    text = "${AppStrings.t(language, "auto.season")} $season",
                    color = if (selected) Color.Black else Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 14.sp,
                    modifier = Modifier
                        .clip(RoundedCornerShape(999.dp))
                        .background(if (selected) Color.White else Color.White.copy(alpha = 0.10f))
                        .clickable { onAction(DetailAction.SeasonSelected(season)) }
                        .padding(horizontal = 14.dp, vertical = 9.dp)
                )
            }
        }
        "posters" -> LazyRow(
            contentPadding = PaddingValues(horizontal = 20.dp, vertical = 12.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            items(content.availableSeasons, key = { it }) { season ->
                val selected = season == content.selectedSeason
                Box(
                    modifier = Modifier
                        .width(76.dp)
                        .height(104.dp)
                        .clip(RoundedCornerShape(10.dp))
                        .background(if (selected) Color.White else FluxaColors.surfaceRaised)
                        .clickable { onAction(DetailAction.SeasonSelected(season)) },
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = "${AppStrings.t(language, "auto.season")}\n$season",
                        color = if (selected) Color.Black else Color.White,
                        fontWeight = FontWeight.Bold,
                        fontSize = 13.sp,
                        textAlign = TextAlign.Center
                    )
                }
            }
        }
        else -> DropdownSeasonSelector(content, language, onAction, headerStyle)
    }
}

@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
private fun DropdownSeasonSelector(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    headerStyle: Boolean
) {
    var sheetOpen by remember(content.id) { mutableStateOf(false) }
    var triggerFocused by remember { mutableStateOf(false) }
    val triggerModifier = if (headerStyle) {
        Modifier
            .fillMaxWidth()
            .padding(start = 18.dp, end = 18.dp, top = 24.dp, bottom = 7.dp)
            .clip(RoundedCornerShape(8.dp))
            .onFocusChanged { triggerFocused = it.isFocused }
            .background(if (triggerFocused) Color.White.copy(alpha = 0.12f) else Color.Transparent)
            .clickable { sheetOpen = true }
            .padding(vertical = 3.dp)
    } else {
        Modifier
            .padding(horizontal = 20.dp, vertical = 12.dp)
            .clip(RoundedCornerShape(6.dp))
            .onFocusChanged { triggerFocused = it.isFocused }
            .background(if (triggerFocused) Color.White.copy(alpha = 0.9f) else FluxaColors.surfaceRaised)
            .clickable { sheetOpen = true }
            .padding(horizontal = 14.dp, vertical = 10.dp)
    }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = triggerModifier
    ) {
        Text(
            text = "${AppStrings.t(language, "auto.season")} ${content.selectedSeason}",
            color = if (triggerFocused && !headerStyle) Color.Black else Color.White,
            fontWeight = FontWeight.Bold,
            fontSize = if (headerStyle) 17.sp else 15.sp
        )
        Icon(
            imageVector = Icons.Filled.ArrowDropDown,
            contentDescription = null,
            tint = if (triggerFocused && !headerStyle) Color.Black else Color.White,
            modifier = Modifier.padding(start = 3.dp).size(if (headerStyle) 17.dp else 20.dp)
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
                        color = if (rowFocused) Color.Black else if (selected) LocalAccentColor.current else Color.White,
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
internal fun EpisodeRow(
    episode: DetailEpisodeUiModel,
    content: DetailUiModel,
    onAction: (DetailAction) -> Unit,
    showDescription: Boolean = true,
    compact: Boolean = false,
    blurUnwatched: Boolean = false
) {
    val selected = episode.id == content.selectedEpisodeId
    val isTv = LocalDeviceType.current == DeviceType.TV
    val isDesktop = LocalDeviceType.current == DeviceType.Desktop
    val description = if (showDescription) episode.description?.takeIf(String::isNotBlank) else null
    var focused by remember { mutableStateOf(false) }
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .background(if (isTv && focused) Color.White.copy(alpha = 0.16f) else Color.Transparent)
            .onFocusChanged { focused = it.isFocused }
            .clickable(enabled = !episode.isUpcoming) { onAction(DetailAction.EpisodeSelected(episode.id)) }
            .padding(horizontal = 20.dp, vertical = if (compact) 8.dp else 12.dp)
    ) {
        Row(verticalAlignment = Alignment.Top) {
            Box(
                modifier = Modifier
                    .then(if (isDesktop) Modifier.width(480.dp) else Modifier.weight(1f))
                    .aspectRatio(16f / 9f)
                    .clip(RoundedCornerShape(6.dp))
                    .background(FluxaColors.surfaceCard)
            ) {
                FluxaRemoteImage(
                    imageUrl = episode.thumbnailUrl,
                    cacheKey = "detail-episode:${episode.id}",
                    contentDescription = null,
                    modifier = Modifier
                        .fillMaxSize()
                        .then(if (blurUnwatched && !episode.isWatched) Modifier.blur(9.dp) else Modifier),
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
                if (isDesktop) description?.let {
                    Text(
                        text = it,
                        color = Color.White.copy(alpha = 0.66f),
                        fontSize = 14.sp,
                        lineHeight = 20.sp,
                        maxLines = 6,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.padding(top = 12.dp)
                    )
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
        if (!isDesktop) description?.let {
            Text(
                text = it,
                color = Color.White.copy(alpha = 0.6f),
                fontSize = 13.sp,
                maxLines = if (compact) 2 else 3,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = 8.dp)
            )
        }
    }
}

@Composable
internal fun RelatedGrid(items: List<CatalogItemUiModel>, onAction: (DetailAction) -> Unit) {
    LazyRow(
        contentPadding = PaddingValues(horizontal = 20.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        items(items, key = { it.stableLazyKey() }, contentType = { "catalog-card" }) { item ->
            CatalogCard(model = item.card, onClick = { onAction(DetailAction.RelatedItemSelected(item)) })
        }
    }
}

@Composable
internal fun DetailLoading() {
    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        CircularProgressIndicator(color = Color.White)
    }
}

@Composable
internal fun DetailEmpty(text: String) {
    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Text(text = text, color = Color.White)
    }
}
