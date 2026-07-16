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
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
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
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.FluxaIcons

@Composable
fun SourceSelectionScreen(
    content: DetailUiModel,
    language: String?,
    onBack: () -> Unit,
    onStreamSelected: (DetailStreamUiModel) -> Unit,
    modifier: Modifier = Modifier
) {
    var addonFilter by remember(content.id) { mutableStateOf<String?>(null) }
    val addons = remember(content.streams) {
        content.streams.map { it.addonName }.filter { it.isNotBlank() }.distinct()
    }
    val visibleStreams = remember(content.streams, addonFilter) {
        val filter = addonFilter
        if (filter == null) content.streams else content.streams.filter { it.addonName == filter }
    }

    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        Backdrop(content = content)
        Column(modifier = Modifier.fillMaxSize()) {
            Header(content = content, language = language, onBack = onBack)
            if (addons.size > 1) {
                AddonChips(
                    addons = addons,
                    selected = addonFilter,
                    language = language,
                    onSelected = { addonFilter = it }
                )
            }
            when {
                content.isLoadingStreams -> Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator(color = Color.White)
                }
                visibleStreams.isEmpty() -> Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = AppStrings.t(language, "auto.no_sources_found_3019f12c"),
                        color = Color.White.copy(alpha = 0.6f),
                        fontSize = 14.sp
                    )
                }
                else -> LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(start = 20.dp, end = 20.dp, top = 4.dp, bottom = 28.dp),
                    verticalArrangement = Arrangement.spacedBy(10.dp)
                ) {
                    items(visibleStreams, key = { it.playableUrl }) { stream ->
                        StreamCard(stream = stream, onClick = { onStreamSelected(stream) })
                    }
                }
            }
        }
    }
}

@Composable
private fun Backdrop(content: DetailUiModel) {
    Box(modifier = Modifier.fillMaxWidth().height(300.dp)) {
        FluxaRemoteImage(
            imageUrl = content.backgroundUrl ?: content.posterUrl,
            cacheKey = "sources-backdrop:${content.id}",
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
                            0f to FluxaColors.background.copy(alpha = 0.55f),
                            0.6f to FluxaColors.background.copy(alpha = 0.86f),
                            1f to FluxaColors.background
                        )
                    )
                )
        )
    }
}

@Composable
private fun Header(content: DetailUiModel, language: String?, onBack: () -> Unit) {
    val episode = content.selectedEpisodeId?.let { id ->
        content.seasonEpisodes.firstOrNull { it.id == id }
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = 12.dp, end = 20.dp, top = 44.dp, bottom = 20.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .size(40.dp)
                .clip(CircleShape)
                .background(Color.White.copy(alpha = 0.08f))
                .clickable(onClick = onBack),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                imageVector = FluxaIcons.ArrowBack,
                contentDescription = AppStrings.t(language, "common.back"),
                tint = Color.White,
                modifier = Modifier.size(20.dp)
            )
        }
        Column(modifier = Modifier.padding(start = 14.dp).weight(1f)) {
            Text(
                text = AppStrings.t(language, "auto.sources"),
                color = Color.White,
                fontSize = 22.sp,
                fontWeight = FontWeight.Bold
            )
            val subtitle = buildString {
                append(content.title)
                if (episode?.season != null && episode.number != null) {
                    append("  ·  S${episode.season} E${episode.number}")
                }
            }
            Text(
                text = subtitle,
                color = Color.White.copy(alpha = 0.65f),
                fontSize = 13.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = 2.dp)
            )
        }
    }
}

@Composable
private fun AddonChips(
    addons: List<String>,
    selected: String?,
    language: String?,
    onSelected: (String?) -> Unit
) {
    LazyRow(
        contentPadding = PaddingValues(horizontal = 20.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        modifier = Modifier.fillMaxWidth().padding(bottom = 16.dp)
    ) {
        item(key = "all") {
            AddonChip(
                label = AppStrings.t(language, "auto.all"),
                selected = selected == null,
                onClick = { onSelected(null) }
            )
        }
        items(addons, key = { it }) { addon ->
            AddonChip(
                label = addon,
                selected = selected == addon,
                onClick = { onSelected(addon) }
            )
        }
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

private data class StreamCardInfo(
    val headline: String,
    val detail: String?,
    val badges: List<String>
)

private val qualityPattern = Regex("(2160p|4k|1440p|1080p|720p|480p|360p)", RegexOption.IGNORE_CASE)
private val tagPatterns = listOf(
    Regex("\\bdolby\\s*vision\\b|\\bdv\\b", RegexOption.IGNORE_CASE) to "DV",
    Regex("\\bhdr10\\+?|\\bhdr\\b", RegexOption.IGNORE_CASE) to "HDR",
    Regex("\\bhevc\\b|\\bx265\\b|\\bh\\.?265\\b", RegexOption.IGNORE_CASE) to "HEVC",
    Regex("\\bremux\\b", RegexOption.IGNORE_CASE) to "REMUX"
)

private fun parseStreamCard(stream: DetailStreamUiModel): StreamCardInfo {
    val lines = stream.title.lines().map { it.trim() }.filter { it.isNotBlank() }
    val headline = lines.firstOrNull() ?: stream.addonName
    val extra = lines.drop(1).joinToString("  ·  ")
    val detail = listOf(stream.addonName, extra)
        .filter { it.isNotBlank() && !headline.contains(it) }
        .joinToString("  ·  ")
        .takeIf { it.isNotBlank() }

    val badges = buildList {
        qualityPattern.find(stream.title)?.let { match ->
            val value = match.value.lowercase()
            add(if (value == "2160p" || value == "4k") "4K" else value)
        }
        tagPatterns.forEach { (pattern, label) ->
            if (pattern.containsMatchIn(stream.title)) add(label)
        }
    }
    return StreamCardInfo(headline = headline, detail = detail, badges = badges)
}

@Composable
private fun StreamCard(stream: DetailStreamUiModel, onClick: () -> Unit) {
    val info = remember(stream) { parseStreamCard(stream) }
    var focused by remember { mutableStateOf(false) }
    val bg by animateColorAsState(
        targetValue = if (focused) Color.White.copy(alpha = 0.16f) else Color.White.copy(alpha = 0.05f),
        animationSpec = tween(150),
        label = "cardBg"
    )
    val scale by animateFloatAsState(
        targetValue = if (focused) 1.01f else 1f,
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
            .clip(RoundedCornerShape(14.dp))
            .background(bg)
            .clickable(onClick = onClick)
            .onFocusChanged { focused = it.isFocused }
            .focusable()
            .padding(horizontal = 16.dp, vertical = 14.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            if (info.badges.isNotEmpty()) {
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    info.badges.forEach { badge -> QualityBadge(text = badge) }
                }
                Spacer(modifier = Modifier.height(8.dp))
            }
            Text(
                text = info.headline,
                color = Color.White,
                fontSize = 14.sp,
                fontWeight = FontWeight.SemiBold,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis
            )
            info.detail?.let {
                Text(
                    text = it,
                    color = Color.White.copy(alpha = 0.55f),
                    fontSize = 12.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 4.dp)
                )
            }
        }
        Spacer(modifier = Modifier.width(12.dp))
        Icon(
            imageVector = FluxaIcons.PlayArrow,
            contentDescription = null,
            tint = Color.White.copy(alpha = if (focused) 1f else 0.5f),
            modifier = Modifier.size(22.dp)
        )
    }
}

@Composable
private fun QualityBadge(text: String) {
    Text(
        text = text,
        color = Color.White,
        fontSize = 10.sp,
        fontWeight = FontWeight.Bold,
        letterSpacing = 0.5.sp,
        modifier = Modifier
            .background(Color.White.copy(alpha = 0.12f), RoundedCornerShape(4.dp))
            .padding(horizontal = 6.dp, vertical = 3.dp)
    )
}
