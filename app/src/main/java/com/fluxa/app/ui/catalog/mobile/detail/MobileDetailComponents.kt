package com.fluxa.app.ui.catalog

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import com.fluxa.app.R
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video
import java.util.Locale

@Composable
internal fun MobileDetailTopIcon(icon: ImageVector, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .size(34.dp)
            .clip(CircleShape)
            .background(Color.Black.copy(alpha = 0.34f))
            .clickable { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Icon(icon, null, tint = Color.White, modifier = Modifier.size(18.dp))
    }
}

@Composable
internal fun MobilePrimaryButton(label: String, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .background(Color.White)
            .clickable { onClick() }
            .padding(vertical = 12.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(FluxaIcons.PlayArrow, null, tint = Color.Black)
        Spacer(Modifier.width(6.dp))
        Text(label, color = Color.Black, fontSize = 18.sp, fontWeight = FontWeight.Black)
    }
}

@Composable
internal fun MobileSecondaryButton(label: String, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .background(Color.White.copy(alpha = 0.10f))
            .clickable { onClick() }
            .padding(horizontal = 14.dp, vertical = 12.dp)
    ) {
        Text(label, color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

@Composable
internal fun MobileDetailAction(icon: ImageVector, label: String, onClick: () -> Unit) {
    Column(
        modifier = Modifier.clickable { onClick() },
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(6.dp)
    ) {
        Icon(icon, null, tint = Color.White, modifier = Modifier.size(22.dp))
        Text(label, color = Color.White.copy(alpha = 0.82f), fontSize = 12.sp)
    }
}

@Composable
internal fun MobileEpisodeActionsSheet(
    episode: Video,
    lang: String,
    isWatched: Boolean,
    isThroughWatched: Boolean,
    hasThroughEpisodes: Boolean,
    durationLabel: String?,
    onDismiss: () -> Unit,
    onToggleWatched: () -> Unit,
    onToggleThroughWatched: () -> Unit
) {
    val runtimeText = durationLabel?.takeIf { it.isNotBlank() }?.let { formatRuntimeLabel(it, lang) }
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.48f))
            .clickable { onDismiss() },
        contentAlignment = Alignment.BottomCenter
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp))
                .background(Color(0xFF171717))
                .pointerInput(episode.id) {
                    detectTapGestures(onTap = {})
                }
                .padding(horizontal = 18.dp, vertical = 18.dp)
                .navigationBarsPadding(),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.Top) {
                Box(
                    modifier = Modifier
                        .width(118.dp)
                        .height(68.dp)
                        .clip(RoundedCornerShape(12.dp))
                ) {
                    AsyncImage(
                        model = episode.thumbnail,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize().alpha(if (isWatched) 0.45f else 1f),
                        contentScale = ContentScale.Crop
                    )
                    if (isWatched) {
                        Icon(
                            FluxaIcons.CheckCircle,
                            null,
                            tint = Color.White,
                            modifier = Modifier.align(Alignment.Center).size(24.dp)
                        )
                    }
                }
                Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    Text(
                        text = "${episode.number ?: 0}. ${episode.name.orEmpty()}",
                        color = Color.White,
                        fontSize = 18.sp,
                        lineHeight = 22.sp,
                        fontWeight = FontWeight.Black,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis
                    )
                    Text(
                        text = buildList {
                            add("S${episode.season ?: 1}:E${episode.number ?: 0}")
                            runtimeText?.let { add(it) }
                            if (isWatched) add(AppStrings.t(lang, "detail.watched_status"))
                        }.joinToString("  "),
                        color = Color.White.copy(alpha = 0.62f),
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                }
            }
            episode.overview?.takeIf { it.isNotBlank() }?.let {
                Text(
                    text = it,
                    color = Color.White.copy(alpha = 0.72f),
                    fontSize = 13.sp,
                    lineHeight = 19.sp,
                    maxLines = 5,
                    overflow = TextOverflow.Ellipsis
                )
            }
            MobileSheetActionButton(
                icon = FluxaIcons.CheckCircle,
                label = AppStrings.t(lang, if (isWatched) "detail.mark_unwatched" else "detail.mark_watched"),
                enabled = true,
                onClick = onToggleWatched
            )
            MobileSheetActionButton(
                icon = FluxaIcons.CheckCircle,
                label = AppStrings.t(lang, if (isThroughWatched) "detail.mark_episode_through_unwatched" else "detail.mark_episode_through_watched"),
                enabled = hasThroughEpisodes,
                onClick = onToggleThroughWatched
            )
        }
    }
}

@Composable
internal fun MobileSeasonActionsSheet(
    season: Int,
    lang: String,
    isSeasonWatched: Boolean,
    isThroughWatched: Boolean,
    hasSeasonEpisodes: Boolean,
    hasThroughEpisodes: Boolean,
    onDismiss: () -> Unit,
    onToggleSeasonWatched: () -> Unit,
    onToggleThroughWatched: () -> Unit
) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.48f))
            .clickable { onDismiss() },
        contentAlignment = Alignment.BottomCenter
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp))
                .background(Color(0xFF171717))
                .pointerInput(season) {
                    detectTapGestures(onTap = {})
                }
                .padding(horizontal = 18.dp, vertical = 18.dp)
                .navigationBarsPadding(),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                text = if (season == 0) AppStrings.t(lang, "auto.specials") else AppStrings.format(lang, "format.season_number", season),
                color = Color.White,
                fontSize = 20.sp,
                lineHeight = 24.sp,
                fontWeight = FontWeight.Black,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            MobileSheetActionButton(
                icon = FluxaIcons.CheckCircle,
                label = AppStrings.t(lang, if (isSeasonWatched) "detail.mark_season_unwatched" else "detail.mark_season_watched"),
                enabled = hasSeasonEpisodes,
                onClick = onToggleSeasonWatched
            )
            MobileSheetActionButton(
                icon = FluxaIcons.CheckCircle,
                label = AppStrings.t(lang, if (isThroughWatched) "detail.mark_through_season_unwatched" else "detail.mark_through_season_watched"),
                enabled = hasThroughEpisodes,
                onClick = onToggleThroughWatched
            )
        }
    }
}

@Composable
private fun MobileSheetActionButton(
    icon: ImageVector,
    label: String,
    enabled: Boolean,
    onClick: () -> Unit
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(48.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(Color.White.copy(alpha = if (enabled) 0.10f else 0.045f))
            .alpha(if (enabled) 1f else 0.48f)
            .clickable(enabled = enabled) { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            Icon(icon, null, tint = Color.White, modifier = Modifier.size(20.dp))
            Text(
                text = label,
                color = Color.White,
                fontSize = 14.sp,
                fontWeight = FontWeight.Bold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
    }
}

@Composable
internal fun MobileEpisodeRow(
    episode: Video,
    lang: String,
    isSelected: Boolean,
    isWatched: Boolean,
    progress: Float,
    durationLabel: String?,
    accentColor: Color,
    showPoster: String? = null,
    blurUnwatched: Boolean = false,
    onClick: () -> Unit,
    onLongPress: () -> Unit,
    onDownloadClick: () -> Unit = {}
) {
    val episodeRuntimeLabel = remember(episode.episodeRuntime, durationLabel) {
        episode.episodeRuntime?.let { "${it}min" } ?: durationLabel
    }
    val runtimeText = episodeRuntimeLabel?.takeIf { it.isNotBlank() }?.let { formatRuntimeLabel(it, lang) }
    val releaseDateLabel = remember(episode.released, lang) { formatEpisodeReleaseDate(episode.released, lang) }
    val effectiveProgress = if (isWatched) 1f else progress.coerceIn(0f, 1f)
    val thumbnailRequest = rememberEpisodeThumbnailRequest(episode)
    val hasMeta = releaseDateLabel != null || runtimeText != null ||
        episode.rating?.takeIf { it.isNotBlank() && it.toDoubleOrNull().let { v -> v != null && v > 0.0 } } != null
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 8.dp)
            .clip(RoundedCornerShape(14.dp))
            .alpha(when {
                blurUnwatched && !isWatched -> 0.5f
                !blurUnwatched && isWatched -> 0.78f
                else -> 1f
            })
            .pointerInput(episode.id) {
                detectTapGestures(
                    onTap = { onClick() },
                    onLongPress = { onLongPress() }
                )
            }
            .padding(10.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(112.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .width(148.dp)
                    .fillMaxHeight()
                    .clip(RoundedCornerShape(12.dp))
            ) {
                if (showPoster != null && thumbnailRequest == null) {
                    AsyncImage(
                        model = showPoster,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop
                    )
                }
                thumbnailRequest?.let { request ->
                    AsyncImage(
                        model = request,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize().alpha(if (isWatched) 0.52f else 1f),
                        contentScale = ContentScale.Crop
                    )
                }
                if (isWatched) {
                    Icon(
                        FluxaIcons.CheckCircle,
                        contentDescription = null,
                        tint = Color.White.copy(alpha = 0.92f),
                        modifier = Modifier
                            .align(Alignment.BottomStart)
                            .padding(start = 7.dp, bottom = 8.dp)
                            .size(18.dp)
                    )
                }
                if (effectiveProgress > 0f) {
                    Box(
                        modifier = Modifier
                            .align(Alignment.BottomCenter)
                            .fillMaxWidth()
                            .height(3.dp)
                            .background(Color.White.copy(alpha = 0.22f))
                    ) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth(effectiveProgress)
                                .fillMaxHeight()
                                .background(accentColor)
                        )
                    }
                }
            }
            Column(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight(),
                verticalArrangement = Arrangement.spacedBy(3.dp)
            ) {
                Text(
                    text = "${episode.number ?: 0}. ${episode.name.orEmpty()}",
                    color = Color.White,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.fillMaxWidth()
                )
                if (hasMeta) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        releaseDateLabel?.let {
                            Text(
                                text = it,
                                color = Color.White.copy(alpha = 0.50f),
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Medium,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                        runtimeText?.let {
                            Text(text = it, color = Color.White.copy(alpha = 0.50f), fontSize = 11.sp, fontWeight = FontWeight.Medium)
                        }
                        episode.rating?.takeIf { it.isNotBlank() && it.toDoubleOrNull().let { v -> v != null && v > 0.0 } }?.let { rating ->
                            Row(
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(4.dp)
                            ) {
                                Image(
                                    painter = painterResource(id = R.drawable.imdb_logo),
                                    contentDescription = null,
                                    modifier = Modifier.height(14.dp).widthIn(max = 52.dp),
                                    contentScale = ContentScale.Fit
                                )
                                Text(text = rating, color = Color(0xFFF5C518), fontSize = 11.sp, fontWeight = FontWeight.Bold)
                            }
                        }
                    }
                }
                episode.overview?.takeIf { it.isNotBlank() }?.let {
                    Text(
                        text = it,
                        color = Color.White.copy(alpha = 0.62f),
                        fontSize = 12.sp,
                        maxLines = 3,
                        lineHeight = 17.sp,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
        }
    }
}

@Composable
internal fun MobileHorizontalEpisodeCard(
    episode: Video,
    lang: String,
    isSelected: Boolean,
    isWatched: Boolean,
    progress: Float,
    durationLabel: String?,
    accentColor: Color,
    showPoster: String? = null,
    blurUnwatched: Boolean = false,
    onClick: () -> Unit,
    onLongPress: () -> Unit
) {
    val runtimeLabel = remember(episode.episodeRuntime, durationLabel) {
        episode.episodeRuntime?.let { "${it}min" } ?: durationLabel
    }
    val runtimeText = runtimeLabel?.takeIf { it.isNotBlank() }?.let { formatRuntimeLabel(it, lang) }
    val releaseDateLabel = remember(episode.released, lang) { formatEpisodeReleaseDate(episode.released, lang) }
    val effectiveProgress = if (isWatched) 1f else progress.coerceIn(0f, 1f)
    val thumbnailRequest = rememberEpisodeThumbnailRequest(episode)
    val episodeAlpha = when {
        blurUnwatched && !isWatched -> 0.5f
        !blurUnwatched && isWatched -> 0.78f
        else -> 1f
    }
    Column(
        modifier = Modifier
            .width(164.dp)
            .clip(RoundedCornerShape(14.dp))
            .alpha(episodeAlpha)
            .pointerInput(episode.id) {
                detectTapGestures(
                    onTap = { onClick() },
                    onLongPress = { onLongPress() }
                )
            }
            .padding(bottom = 8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(100.dp)
                .clip(RoundedCornerShape(12.dp))
        ) {
            when {
                thumbnailRequest != null -> AsyncImage(
                    model = thumbnailRequest,
                    contentDescription = episode.name,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop
                )
                !showPoster.isNullOrBlank() -> AsyncImage(
                    model = showPoster,
                    contentDescription = episode.name,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop
                )
                else -> Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(Color.White.copy(alpha = 0.07f))
                )
            }
            if (isSelected) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(accentColor.copy(alpha = 0.18f))
                )
            }
            if (isWatched) {
                Box(
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .padding(6.dp)
                        .size(20.dp)
                        .clip(CircleShape)
                        .background(Color.Black.copy(alpha = 0.56f)),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        FluxaIcons.Check,
                        contentDescription = null,
                        tint = Color.White,
                        modifier = Modifier.size(13.dp)
                    )
                }
            }
            if (effectiveProgress > 0f && effectiveProgress < 1f) {
                LinearProgressIndicator(
                    progress = { effectiveProgress },
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .height(3.dp),
                    color = accentColor,
                    trackColor = Color.White.copy(alpha = 0.2f)
                )
            }
        }
        Column(
            modifier = Modifier.padding(horizontal = 4.dp),
            verticalArrangement = Arrangement.spacedBy(3.dp)
        ) {
            val epNum = episode.number
            val epTitle = episode.name?.takeIf { it.isNotBlank() }
            if (epNum != null || epTitle != null) {
                Text(
                    text = buildString {
                        if (epNum != null) append("$epNum. ")
                        if (epTitle != null) append(epTitle)
                    },
                    color = Color.White.copy(alpha = 0.92f),
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 2,
                    lineHeight = 16.sp,
                    overflow = TextOverflow.Ellipsis
                )
            }
            episode.overview?.takeIf { it.isNotBlank() }?.let {
                Text(
                    text = it,
                    color = Color.White.copy(alpha = 0.5f),
                    fontSize = 11.sp,
                    maxLines = 2,
                    lineHeight = 15.sp,
                    overflow = TextOverflow.Ellipsis
                )
            }
            val metaParts = listOfNotNull(runtimeText, releaseDateLabel)
            if (metaParts.isNotEmpty()) {
                Text(
                    text = metaParts.joinToString("  "),
                    color = Color.White.copy(alpha = 0.38f),
                    fontSize = 10.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
    }
}

internal fun mobileEpisodeProgressFraction(progressMs: Long, runtimeLabel: String?): Float {
    val minutes = runtimeLabel
        ?.lowercase(Locale.ROOT)
        ?.let { label ->
            Regex("""(\d+)""").find(label)?.groupValues?.getOrNull(1)?.toIntOrNull()
        }
        ?: return 0f
    val durationMs = minutes * 60_000L
    if (durationMs <= 0L) return 0f
    return (progressMs.toFloat() / durationMs.toFloat()).coerceIn(0f, 1f)
}

internal data class MobileSeasonActionTarget(
    val season: Int,
    val seasonEpisodes: List<Video>,
    val throughEpisodes: List<Video>
)

internal fun episodesThroughEpisode(
    episodes: List<Video>,
    target: Video,
    selectedSeason: Int
): List<Video> {
    val targetSeason = target.season ?: selectedSeason
    val targetNumber = target.number ?: return emptyList()
    return episodes
        .filter { (it.season ?: targetSeason) == targetSeason }
        .filter { (it.number ?: Int.MAX_VALUE) <= targetNumber }
        .filter { !detailIsUpcoming(it.released) }
        .sortedWith(compareBy<Video>({ it.number ?: Int.MAX_VALUE }, { it.name.orEmpty() }))
}

@Composable
internal fun MobileStreamRow(stream: StreamUiModel, onClick: () -> Unit) {
    val sourceName = stream.header
    val rawTitle = stream.body
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(8.dp))
            .background(Color(0xFF2B2A34))
            .border(1.dp, Color.White.copy(alpha = 0.05f), RoundedCornerShape(8.dp))
            .clickable { onClick() }
            .padding(horizontal = 14.dp, vertical = 13.dp),
        verticalAlignment = Alignment.Top
    ) {
        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(7.dp)) {
            Text(
                text = sourceName,
                color = Color.White,
                fontSize = 13.sp,
                lineHeight = 15.sp,
                fontWeight = FontWeight.Black,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            rawTitle?.let {
                AddonStreamBodyText(
                    text = it,
                    bodyMaxLines = 6
                )
            }
        }
    }
}
