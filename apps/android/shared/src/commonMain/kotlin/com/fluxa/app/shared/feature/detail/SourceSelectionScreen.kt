package com.fluxa.app.shared.feature.detail

import com.fluxa.app.data.remote.*
import com.fluxa.app.data.stream.*

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.FluxaIcons
import com.fluxa.app.ui.catalog.LocalWindowWidthClass
import com.fluxa.app.ui.catalog.WindowWidthClass

@Composable
fun SourceSelectionScreen(
    content: DetailUiModel,
    language: String?,
    onBack: () -> Unit,
    onStreamSelected: (DetailStreamUiModel) -> Unit,
    onAddonSelected: (String?) -> Unit,
    onRetry: () -> Unit,
    modifier: Modifier = Modifier
) {
    val addonFilter = content.selectedAddon
    val widthClass = LocalWindowWidthClass.current
    val isExpanded = widthClass == WindowWidthClass.Expanded
    val contentWidthFraction = when (widthClass) {
        WindowWidthClass.Compact -> 1f
        WindowWidthClass.Medium -> 0.96f
        WindowWidthClass.Expanded -> 0.94f
    }
    val listBottomPadding = when (widthClass) {
        WindowWidthClass.Compact -> 220.dp
        WindowWidthClass.Medium -> 160.dp
        WindowWidthClass.Expanded -> 32.dp
    }
    fun addonPriority(name: String): Int = content.addonPriorityOrder.indexOf(name).let { if (it < 0) Int.MAX_VALUE else it }
    val addons = remember(content.streams, content.addonPriorityOrder) {
        content.streams.map { it.addonName }.filter { it.isNotBlank() }.distinct()
            .sortedBy(::addonPriority)
    }
    val addonCounts = remember(content.streams) {
        content.streams.groupingBy { it.addonName }.eachCount()
    }
    val visibleStreams = remember(content.streams, addonFilter, content.addonPriorityOrder) {
        val filter = addonFilter
        val streams = if (filter == null) content.streams else content.streams.filter { it.addonName == filter }
        if (filter == null) streams.sortedBy { addonPriority(it.addonName) } else streams
    }
    val episode = content.selectedEpisodeId?.let { id -> content.seasonEpisodes.firstOrNull { it.id == id } }

    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        Backdrop(content = content, episode = episode, compact = isExpanded)
        Column(modifier = Modifier.fillMaxSize()) {
            Header(content = content, episode = episode, language = language, onBack = onBack, compact = isExpanded)
            Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.TopCenter) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth(contentWidthFraction)
                        .widthIn(max = 1120.dp)
                ) {
                    AddonChips(
                        addons = addons,
                        addonCounts = addonCounts,
                        totalCount = content.streams.size,
                        loadingAddonNames = content.loadingAddonNames,
                        selected = addonFilter,
                        language = language,
                        onSelected = onAddonSelected,
                        onRetry = onRetry
                    )
                }
            }
            Box(modifier = Modifier.fillMaxWidth().weight(1f), contentAlignment = Alignment.TopCenter) {
                when {
                    visibleStreams.isEmpty() && content.isLoadingStreams -> Box(
                        modifier = Modifier.fillMaxWidth().widthIn(max = 1120.dp).fillMaxHeight(),
                        contentAlignment = Alignment.Center
                    ) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            CircularProgressIndicator(color = Color.White, modifier = Modifier.size(28.dp), strokeWidth = 2.dp)
                            if (content.loadingAddonNames.isNotEmpty()) {
                                Text(
                                    text = AppStrings.t(language, "auto.waiting_for_addon")
                                        .replace("{addon}", content.loadingAddonNames.joinToString(", ")),
                                    color = Color.White.copy(alpha = 0.58f),
                                    fontSize = 12.sp,
                                    modifier = Modifier.padding(top = 12.dp)
                                )
                            }
                        }
                    }
                    visibleStreams.isEmpty() -> Box(
                        modifier = Modifier.fillMaxWidth().widthIn(max = 1120.dp).fillMaxHeight(),
                        contentAlignment = Alignment.Center
                    ) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Text(
                                text = AppStrings.t(language, "auto.no_sources_found_3019f12c"),
                                color = Color.White.copy(alpha = 0.72f),
                                fontSize = 15.sp,
                                fontWeight = FontWeight.SemiBold
                            )
                        }
                    }
                    else -> LazyColumn(
                        modifier = Modifier
                            .fillMaxWidth(contentWidthFraction)
                            .widthIn(max = 1120.dp)
                            .fillMaxHeight(),
                        contentPadding = PaddingValues(
                            start = 16.dp,
                            end = 16.dp,
                            top = 4.dp,
                            bottom = listBottomPadding
                        ),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        items(visibleStreams, key = { "${it.addonName}:${it.playableUrl}" }) { stream ->
                            StreamCard(stream = stream, onClick = { onStreamSelected(stream) })
                        }
                        if (content.isLoadingStreams) {
                            item(key = "loading-more") {
                                LoadingAddonsRow(addonNames = content.loadingAddonNames, language = language)
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun Backdrop(content: DetailUiModel, episode: DetailEpisodeUiModel?, compact: Boolean) {
    val backdropUrl = episode?.thumbnailUrl?.takeIf { it.isNotBlank() } ?: content.backgroundUrl ?: content.posterUrl
    Box(modifier = Modifier.fillMaxWidth().height(if (compact) 230.dp else 300.dp)) {
        FluxaRemoteImage(
            imageUrl = backdropUrl,
            cacheKey = "sources-backdrop:${content.id}:${episode?.id.orEmpty()}",
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
                            0f to Color.Black.copy(alpha = 0.48f),
                            0.42f to FluxaColors.background.copy(alpha = 0.50f),
                            0.72f to FluxaColors.background.copy(alpha = 0.88f),
                            1f to FluxaColors.background
                        )
                    )
                )
        )
    }
}

@Composable
private fun Header(content: DetailUiModel, episode: DetailEpisodeUiModel?, language: String?, onBack: () -> Unit, compact: Boolean) {
    var backFocused by remember { mutableStateOf(false) }
    var descriptionExpanded by remember(episode?.id) { mutableStateOf(false) }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = if (compact) 20.dp else 12.dp, end = 20.dp, top = if (compact) 22.dp else 44.dp, bottom = if (compact) 14.dp else 20.dp),
        verticalAlignment = Alignment.Top
    ) {
        Box(
            modifier = Modifier
                .size(40.dp)
                .clip(CircleShape)
                .background(if (backFocused) Color.White else Color.White.copy(alpha = 0.08f))
                .onFocusChanged { backFocused = it.isFocused }
                .clickable(onClick = onBack),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                imageVector = FluxaIcons.ArrowBack,
                contentDescription = AppStrings.t(language, "common.back"),
                tint = if (backFocused) Color.Black else Color.White,
                modifier = Modifier.size(20.dp)
            )
        }
        Column(modifier = Modifier.padding(start = 14.dp).weight(1f)) {
            if (!content.logoUrl.isNullOrBlank()) {
                FluxaRemoteImage(
                    imageUrl = content.logoUrl,
                    cacheKey = "sources-logo:${content.id}",
                    contentDescription = content.title,
                    modifier = Modifier.heightIn(max = if (compact) 48.dp else 40.dp).widthIn(max = if (compact) 260.dp else 220.dp),
                    contentScale = ContentScale.Fit,
                    trimTransparentPadding = true
                )
            } else {
                Text(
                    text = content.title,
                    color = Color.White,
                    fontSize = 22.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
            val episodeLine = if (episode?.season != null && episode.number != null) {
                val code = "S${episode.season}, E${episode.number}"
                if (!episode.title.isNullOrBlank()) "$code: ${episode.title}" else code
            } else {
                null
            }
            if (episodeLine != null) {
                Text(
                    text = episodeLine,
                    color = Color.White.copy(alpha = 0.85f),
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Medium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 2.dp)
                )
            }
            if (!episode?.description.isNullOrBlank()) {
                Text(
                    text = episode?.description.orEmpty(),
                    color = Color.White.copy(alpha = 0.6f),
                    fontSize = 12.sp,
                    lineHeight = 16.sp,
                    maxLines = if (descriptionExpanded) Int.MAX_VALUE else 2,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier
                        .padding(top = 3.dp)
                        .clickable(interactionSource = null, indication = null) {
                            descriptionExpanded = !descriptionExpanded
                        }
                )
            }
        }
    }
}

@Composable
private fun AddonChips(
    addons: List<String>,
    addonCounts: Map<String, Int>,
    totalCount: Int,
    loadingAddonNames: List<String>,
    selected: String?,
    language: String?,
    onSelected: (String?) -> Unit,
    onRetry: () -> Unit
) {
    Column(modifier = Modifier.padding(bottom = 16.dp)) {
        LazyRow(
            contentPadding = PaddingValues(horizontal = 20.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            modifier = Modifier.fillMaxWidth()
        ) {
            item(key = "retry") {
                RetryChip(onClick = onRetry)
            }
            item(key = "all") {
                AddonChip(
                    label = "${AppStrings.t(language, "auto.all")}  $totalCount",
                    selected = selected == null,
                    onClick = { onSelected(null) }
                )
            }
            itemsIndexed(addons, key = { index, addon -> "addon-chip:$addon:$index" }) { _, addon ->
                AddonChip(
                    label = "$addon  ${addonCounts[addon] ?: 0}",
                    selected = selected == addon,
                    onClick = { onSelected(addon) }
                )
            }
        }
        if (loadingAddonNames.isNotEmpty()) {
            Row(
                modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                CircularProgressIndicator(
                    color = Color.White.copy(alpha = 0.6f),
                    strokeWidth = 1.5.dp,
                    modifier = Modifier.size(12.dp)
                )
                Text(
                    text = AppStrings.t(language, "auto.waiting_for_addon")
                        .replace("{addon}", loadingAddonNames.joinToString(", ")),
                    color = Color.White.copy(alpha = 0.5f),
                    fontSize = 12.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(start = 8.dp)
                )
            }
        }
    }
}

@Composable
private fun RetryChip(onClick: () -> Unit) {
    var focused by remember { mutableStateOf(false) }
    Box(
        modifier = Modifier
            .size(34.dp)
            .clip(CircleShape)
            .background(if (focused) Color.White.copy(alpha = 0.22f) else Color.White.copy(alpha = 0.07f))
            .onFocusChanged { focused = it.isFocused }
            .focusable()
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Icon(
            imageVector = FluxaIcons.Refresh,
            contentDescription = null,
            tint = Color.White,
            modifier = Modifier.size(16.dp)
        )
    }
}

@Composable
private fun AddonChip(label: String, selected: Boolean, onClick: () -> Unit) {
    var focused by remember { mutableStateOf(false) }
    val bg by animateColorAsState(
        targetValue = when {
            selected -> Color.White
            focused -> Color(0xFF252525)
            else -> Color(0xFF111111)
        },
        animationSpec = tween(150),
        label = "chipBg"
    )
    Box(
        modifier = Modifier
            .clip(CircleShape)
            .background(bg)
            .clickable(onClick = onClick)
            .onFocusChanged { focused = it.isFocused }
            .focusable()
            .padding(horizontal = 16.dp, vertical = 8.dp)
    ) {
        Text(
            text = label,
            color = if (selected) Color.Black else Color.White,
            fontSize = 13.sp,
            fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
            maxLines = 1
        )
    }
}

@Composable
private fun LoadingAddonsRow(addonNames: List<String>, language: String?) {
    Column(
        modifier = Modifier.fillMaxWidth().padding(vertical = 16.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        CircularProgressIndicator(
            color = Color.White.copy(alpha = 0.7f),
            modifier = Modifier.size(22.dp)
        )
        if (addonNames.isNotEmpty()) {
            Text(
                text = AppStrings.t(language, "auto.waiting_for_addon")
                    .replace("{addon}", addonNames.joinToString(", ")),
                color = Color.White.copy(alpha = 0.5f),
                fontSize = 12.sp,
                modifier = Modifier.padding(top = 8.dp)
            )
        }
    }
}

@Composable
private fun StreamCard(stream: DetailStreamUiModel, onClick: () -> Unit) {
    var focused by remember { mutableStateOf(false) }
    val bg by animateColorAsState(
        targetValue = if (focused) Color.White.copy(alpha = 0.14f) else Color.White.copy(alpha = 0.045f),
        animationSpec = tween(150),
        label = "cardBg"
    )
    val scale by animateFloatAsState(
        targetValue = if (focused) 1.008f else 1f,
        animationSpec = tween(150, easing = FastOutSlowInEasing),
        label = "cardScale"
    )
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .graphicsLayer {
                scaleX = scale
                scaleY = scale
            }
            .clip(RoundedCornerShape(12.dp))
            .background(bg)
            .clickable(onClick = onClick)
            .onFocusChanged { focused = it.isFocused }
            .focusable()
            .padding(horizontal = 14.dp, vertical = 16.dp),
        verticalAlignment = Alignment.Top
    ) {
        Column(modifier = Modifier.weight(1f)) {
            if (stream.name.isNotBlank()) {
                Text(
                    text = stream.name,
                    color = Color.White,
                    fontSize = 15.sp,
                    lineHeight = 20.sp,
                    fontWeight = FontWeight.SemiBold,
                    softWrap = true,
                )
            }
            if (stream.description.isNotBlank()) {
                Text(
                    text = stream.description,
                    color = Color.White.copy(alpha = 0.52f),
                    fontSize = 12.sp,
                    lineHeight = 17.sp,
                    softWrap = true,
                    modifier = Modifier.padding(top = 5.dp)
                )
            }
        }
    }
}
