package com.fluxa.app.shared.feature.detail

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.ThumbUp
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.LocalHeroTrailerSurface
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.LocalDeviceType
import com.fluxa.app.ui.catalog.LocalWindowWidthClass
import com.fluxa.app.ui.catalog.WindowWidthClass
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.LocalAccentColor

@Composable
internal fun DiscussionSection(title: String, comments: List<DetailDiscussionCommentUiModel>, language: String?) {
    if (comments.isEmpty()) return
    Column(modifier = Modifier.padding(top = 24.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Text(title, color = Color.White, fontSize = 18.sp, fontWeight = FontWeight.Bold)
        comments.forEach { comment ->
            Column(
                modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp)).background(Color.White.copy(alpha = 0.07f)).padding(14.dp),
                verticalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                Text(comment.author, color = Color.White, fontSize = 13.sp, fontWeight = FontWeight.Bold)
                Text(
                    if (comment.spoiler) AppStrings.t(language, "detail.comments_spoiler") else comment.body,
                    color = Color.White.copy(alpha = 0.78f),
                    fontSize = 13.sp
                )
                if (comment.likes > 0) Text("♥ ${comment.likes}", color = Color.White.copy(alpha = 0.5f), fontSize = 12.sp)
            }
        }
    }
}

@Composable
internal fun Hero(content: DetailUiModel, language: String?, preferClearlogo: Boolean = true) {
    val isExpanded = LocalWindowWidthClass.current == WindowWidthClass.Expanded
    val heroHeight = if (isExpanded) 420.dp else 560.dp
    Box(modifier = Modifier.fillMaxWidth().height(heroHeight).clip(RoundedCornerShape(0.dp))) {
        FluxaRemoteImage(
            imageUrl = content.backgroundUrl ?: content.posterUrl,
            cacheKey = "detail-hero:${content.id}",
            contentDescription = null,
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Crop,
            alignment = if (isExpanded) Alignment.TopCenter else Alignment.Center
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
                    Brush.horizontalGradient(
                        colorStops = arrayOf(
                            0f to Color.Black.copy(alpha = 0.72f),
                            0.35f to Color.Black.copy(alpha = 0.34f),
                            0.68f to Color.Transparent,
                            1f to Color.Transparent
                        )
                    )
                )
        )
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.verticalGradient(
                        colorStops = arrayOf(
                            0f to Color.Transparent,
                            0.48f to Color.Transparent,
                            0.78f to FluxaColors.background.copy(alpha = 0.58f),
                            0.92f to FluxaColors.background.copy(alpha = 0.88f),
                            1f to FluxaColors.background
                        )
                    )
                )
        )
        Column(
            modifier = Modifier
                .align(Alignment.BottomStart)
                .then(if (isExpanded) Modifier.widthIn(max = 640.dp) else Modifier)
                .padding(start = 20.dp, end = 20.dp, top = 16.dp, bottom = 8.dp)
        ) {
            Text(
                text = if (content.type == "series") AppStrings.t(language, "auto.series") else AppStrings.t(language, "auto.movie"),
                color = Color.White.copy(alpha = 0.7f),
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = 1.5.sp
            )
            if (preferClearlogo && content.logoUrl != null) {
                FluxaRemoteImage(
                    imageUrl = content.logoUrl,
                    cacheKey = "detail-logo:${content.id}",
                    contentDescription = content.title,
                    modifier = Modifier
                        .padding(top = 8.dp)
                        .heightIn(max = 72.dp)
                        .widthIn(max = 280.dp),
                    contentScale = ContentScale.Fit,
                    alignment = Alignment.CenterStart,
                    trimTransparentPadding = true
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
                                    .background(LocalAccentColor.current, RoundedCornerShape(3.dp))
                                    .padding(horizontal = 6.dp, vertical = 2.dp)
                            )
                        } else {
                            Text(text = part, color = Color.White.copy(alpha = 0.8f), fontSize = 13.sp)
                        }
                    }
                }
            }
            val ratings = content.visibleRatings()
            if (ratings.isNotEmpty()) {
                val ratingLogo = LocalDetailRatingLogo.current
                LazyRow(
                    modifier = Modifier.padding(top = 10.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    items(ratings, key = { "${it.source}:${it.value}" }) { rating ->
                        Row(
                            modifier = Modifier
                                .background(Color.White.copy(alpha = 0.14f), RoundedCornerShape(8.dp))
                                .padding(horizontal = 8.dp, vertical = 4.dp),
                            horizontalArrangement = Arrangement.spacedBy(5.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            ratingLogo(
                                rating.source,
                                rating.value,
                                Modifier.height(if (rating.source == "imdb") 20.dp else 16.dp)
                            )
                            Text(rating.displayValue(), color = rating.scoreColor(), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                        }
                    }
                }
            }
        }
    }
}

@Composable
internal fun ExpandedDetailIdentityBlock(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    preferClearlogo: Boolean = true
) {
    Column(modifier = Modifier.widthIn(max = 640.dp).padding(horizontal = 40.dp)) {
        Text(
            text = if (content.type == "series") AppStrings.t(language, "auto.series") else AppStrings.t(language, "auto.movie"),
            color = Color.White.copy(alpha = 0.7f),
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 1.5.sp
        )
        if (preferClearlogo && content.logoUrl != null) {
            FluxaRemoteImage(
                imageUrl = content.logoUrl,
                cacheKey = "detail-logo:${content.id}",
                contentDescription = content.title,
                modifier = Modifier
                    .padding(top = 10.dp)
                    .heightIn(max = 92.dp)
                    .widthIn(max = 380.dp),
                contentScale = ContentScale.Fit,
                alignment = Alignment.CenterStart,
                trimTransparentPadding = true
            )
        } else {
            Text(
                text = content.title,
                color = Color.White,
                fontSize = 34.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(top = 8.dp)
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
            content.genres.filter { it.isNotBlank() }.takeIf { it.isNotEmpty() }?.let {
                add(it.joinToString(" · ") to false)
            }
        }
        if (metaParts.isNotEmpty()) {
            Row(
                modifier = Modifier.padding(top = 12.dp),
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
                                .background(LocalAccentColor.current, RoundedCornerShape(3.dp))
                                .padding(horizontal = 6.dp, vertical = 2.dp)
                        )
                    } else {
                        Text(text = part, color = Color.White.copy(alpha = 0.75f), fontSize = 13.sp)
                    }
                }
            }
        }
        val ratings = content.visibleRatings()
        if (ratings.isNotEmpty()) {
            val ratingLogo = LocalDetailRatingLogo.current
            Row(modifier = Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                ratings.forEach { rating ->
                    Row(
                        modifier = Modifier
                            .background(Color.White.copy(alpha = 0.14f), RoundedCornerShape(8.dp))
                            .padding(horizontal = 8.dp, vertical = 4.dp),
                        horizontalArrangement = Arrangement.spacedBy(5.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        ratingLogo(
                            rating.source,
                            rating.value,
                            Modifier.height(if (rating.source == "imdb") 20.dp else 16.dp)
                        )
                        Text(rating.displayValue(), color = rating.scoreColor(), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                    }
                }
            }
        }
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            modifier = Modifier.padding(top = 22.dp)
        ) {
            CompactPlayButton(content = content, language = language, onAction = onAction)
            if (content.supportsWatchlist) {
                DetailCircleIconButton(
                    icon = if (content.isInWatchlist) Icons.Filled.Check else Icons.Filled.Add,
                    contentDescription = AppStrings.t(language, if (content.isInWatchlist) "auto.in_list" else "auto.my_list"),
                    selected = content.isInWatchlist,
                    onClick = { onAction(DetailAction.ToggleWatchlist) }
                )
            }
            if (content.supportsLike) {
                DetailCircleIconButton(
                    icon = Icons.Filled.ThumbUp,
                    contentDescription = AppStrings.t(language, "common.like"),
                    selected = content.isLiked,
                    onClick = { onAction(DetailAction.ToggleLike) }
                )
            }
            DetailCircleIconButton(
                icon = Icons.Filled.Download,
                contentDescription = AppStrings.t(language, "auto.download"),
                selected = false,
                onClick = {
                    val episodeId = content.selectedEpisodeId
                    if (episodeId != null) onAction(DetailAction.DownloadEpisode(episodeId))
                    else onAction(DetailAction.DownloadSeason(content.selectedSeason))
                }
            )
        }
        if (content.resumeProgress > 0L && content.resumeVideoId != null) {
            Box(modifier = Modifier.padding(top = 10.dp)) {
                RestartButton(language = language, onClick = { onAction(DetailAction.Play(fromStart = true)) })
            }
        }
        if (content.description.isNotBlank()) {
            Text(
                text = content.description,
                color = Color(0xFFB8B8B8),
                fontSize = 15.sp,
                lineHeight = 22.sp,
                modifier = Modifier.padding(top = 18.dp)
            )
        }
    }
}

@Composable
internal fun CompactPlayButton(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    modifier: Modifier = Modifier
) {
    val resumeEpisode = content.resumeVideoId?.let { id -> content.seasonEpisodes.firstOrNull { it.id == id } }
    val label = if (content.resumeProgress > 0L && resumeEpisode?.season != null && resumeEpisode.number != null) {
        AppStrings.t(language, "auto.resume") + " S${resumeEpisode.season} E${resumeEpisode.number}"
    } else if (content.resumeProgress > 0L) {
        AppStrings.t(language, "auto.resume")
    } else {
        AppStrings.t(language, "common.play")
    }
    Row(
        modifier = modifier
            .height(52.dp)
            .clip(RoundedCornerShape(10.dp))
            .background(Color.White)
            .clickable { onAction(DetailAction.Play()) }
            .padding(horizontal = 22.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(imageVector = Icons.Rounded.PlayArrow, contentDescription = null, tint = Color.Black, modifier = Modifier.size(22.dp))
        Text(text = label, color = Color.Black, fontWeight = FontWeight.Bold, fontSize = 15.sp)
    }
}

@Composable
internal fun DetailCircleIconButton(
    icon: ImageVector,
    contentDescription: String?,
    selected: Boolean,
    onClick: () -> Unit
) {
    Box(
        modifier = Modifier
            .size(52.dp)
            .clip(CircleShape)
            .background(if (selected) Color.White else Color.White.copy(alpha = 0.14f))
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Icon(
            imageVector = icon,
            contentDescription = contentDescription,
            tint = if (selected) Color.Black else Color.White,
            modifier = Modifier.size(22.dp)
        )
    }
}

@Composable
internal fun ResumeButton(content: DetailUiModel, language: String?, onAction: (DetailAction) -> Unit) {
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
internal fun RestartButton(language: String?, onClick: () -> Unit) {
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
internal fun DownloadButton(content: DetailUiModel, language: String?, onAction: (DetailAction) -> Unit) {
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
