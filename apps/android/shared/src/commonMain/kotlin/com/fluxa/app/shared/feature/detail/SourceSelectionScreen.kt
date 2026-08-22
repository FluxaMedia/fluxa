package com.fluxa.app.shared.feature.detail

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
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
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
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
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
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.LocalDeviceType
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.FluxaIcons

@Composable
fun SourceSelectionScreen(
    content: DetailUiModel,
    language: String?,
    onBack: () -> Unit,
    onStreamSelected: (DetailStreamUiModel) -> Unit,
    onRetry: () -> Unit,
    modifier: Modifier = Modifier
) {
    var addonFilter by remember(content.id) { mutableStateOf<String?>(null) }
    val isDesktop = LocalDeviceType.current == DeviceType.Desktop
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
        Backdrop(content = content, episode = episode, compact = isDesktop)
        Column(modifier = Modifier.fillMaxSize()) {
            Header(content = content, episode = episode, language = language, onBack = onBack, compact = isDesktop)
            Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.TopCenter) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth(if (isDesktop) 0.94f else 1f)
                        .widthIn(max = 1120.dp)
                ) {
                    SourceSummary(
                        total = content.streams.size,
                        visible = visibleStreams.size,
                        selectedAddon = addonFilter,
                        language = language
                    )
                    AddonChips(
                        addons = addons,
                        addonCounts = addonCounts,
                        totalCount = content.streams.size,
                        loadingAddonNames = content.loadingAddonNames,
                        selected = addonFilter,
                        language = language,
                        onSelected = { addonFilter = it },
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
                            Text(
                                text = AppStrings.t(language, "player.source_selection_subtitle"),
                                color = Color.White.copy(alpha = 0.38f),
                                fontSize = 12.sp,
                                modifier = Modifier.padding(top = 6.dp)
                            )
                        }
                    }
                    else -> LazyColumn(
                        modifier = Modifier
                            .fillMaxWidth(if (isDesktop) 0.94f else 1f)
                            .widthIn(max = 1120.dp)
                            .fillMaxHeight(),
                        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 32.dp),
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
private fun SourceSummary(
    total: Int,
    visible: Int,
    selectedAddon: String?,
    language: String?
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 4.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = AppStrings.t(language, "player.source_selection_title"),
                color = Color.White,
                fontWeight = FontWeight.Bold,
                fontSize = 17.sp
            )
            Text(
                text = AppStrings.t(language, "player.source_selection_subtitle"),
                color = Color.White.copy(alpha = 0.48f),
                fontSize = 11.sp,
                modifier = Modifier.padding(top = 2.dp)
            )
        }
        val count = if (selectedAddon == null) total else visible
        Box(
            modifier = Modifier
                .clip(CircleShape)
                .background(Color.White.copy(alpha = 0.08f))
                .padding(horizontal = 10.dp, vertical = 6.dp)
        ) {
            Text(
                text = "$count ${AppStrings.t(language, "auto.sources").lowercase()}",
                color = Color.White.copy(alpha = 0.72f),
                fontSize = 11.sp,
                fontWeight = FontWeight.SemiBold
            )
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
            focused -> Color.White.copy(alpha = 0.22f)
            else -> Color.White.copy(alpha = 0.07f)
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
    val provider = stream.addonName.trim().ifBlank { "Source" }
    val rawTitle = stream.title.trim().ifBlank { stream.name.trim() }.ifBlank { provider }
    val cleanName = stream.name.trim().takeIf { it.isNotBlank() && !it.equals(provider, ignoreCase = true) }
    val primary = cleanName ?: rawTitle.lineSequence().firstOrNull()?.trim().orEmpty().ifBlank { provider }
    val secondary = rawTitle.takeIf { it.isNotBlank() && !it.equals(primary, ignoreCase = true) }
    val quality = remember(rawTitle, stream.name) { sourceQualityLabel("${stream.name} $rawTitle") }
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
            .padding(horizontal = 14.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(
            modifier = Modifier.widthIn(min = 96.dp, max = 150.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            Text(
                text = provider,
                color = Color.White,
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            if (stream.sourceKind == "plugin") {
                Text(
                    text = "PLUGIN",
                    color = Color.White.copy(alpha = 0.42f),
                    fontSize = 9.sp,
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 0.8.sp
                )
            }
        }
        Spacer(modifier = Modifier.width(16.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = primary,
                color = Color.White,
                fontSize = 13.sp,
                lineHeight = 18.sp,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            secondary?.let {
                Text(
                    text = it,
                    color = Color.White.copy(alpha = 0.52f),
                    fontSize = 11.sp,
                    lineHeight = 16.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 3.dp)
                )
            }
        }
        quality?.let {
            Spacer(modifier = Modifier.width(12.dp))
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(6.dp))
                    .background(Color.White.copy(alpha = 0.09f))
                    .padding(horizontal = 8.dp, vertical = 5.dp)
            ) {
                Text(
                    text = it,
                    color = Color.White.copy(alpha = 0.78f),
                    fontSize = 10.sp,
                    fontWeight = FontWeight.Bold
                )
            }
        }
        Spacer(modifier = Modifier.width(12.dp))
        Box(
            modifier = Modifier
                .size(34.dp)
                .clip(CircleShape)
                .background(if (focused) Color.White else Color.White.copy(alpha = 0.08f)),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                imageVector = FluxaIcons.PlayArrow,
                contentDescription = null,
                tint = if (focused) Color.Black else Color.White.copy(alpha = 0.72f),
                modifier = Modifier.size(18.dp)
            )
        }
    }
}

private fun sourceQualityLabel(value: String): String? {
    val normalized = value.uppercase()
    return when {
        "2160P" in normalized || "4K" in normalized || "UHD" in normalized -> "4K"
        "1440P" in normalized -> "1440p"
        "1080P" in normalized -> "1080p"
        "720P" in normalized -> "720p"
        "480P" in normalized -> "480p"
        "360P" in normalized -> "360p"
        else -> null
    }
}
