package com.fluxa.app.shared.feature.detail

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.ArrowDropDown
import androidx.compose.material.icons.outlined.BookmarkBorder
import androidx.compose.material.icons.outlined.Shuffle
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.LocalHeroTrailerSurface
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.LocalDeviceType

private val CinematicPageColor = Color.Black
private val CinematicSecondaryActionColor = Color(0xFFDCE2E5)

@Composable
internal fun CinematicMobileDetailContent(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    presentation: DetailPresentationOptions,
    onStickyContextVisibilityChanged: (Boolean) -> Unit
) {
    val listState = rememberLazyListState()
    ObserveStickyContextVisibility(listState, onStickyContextVisibilityChanged, threshold = 245.dp)
    val heroExpansion by remember(listState, presentation.collapsingHero) {
        derivedStateOf {
            if (!presentation.collapsingHero || listState.firstVisibleItemIndex > 0) 0f
            else (1f - listState.firstVisibleItemScrollOffset / 500f).coerceIn(0f, 1f)
        }
    }
    val heroHeight by animateDpAsState(
        targetValue = (316f + 74f * heroExpansion).dp,
        label = "detail-cinematic-hero-height"
    )
    val isSeries = content.type == "series" && content.availableSeasons.isNotEmpty()

    LazyColumn(
        state = listState,
        modifier = Modifier.fillMaxSize().background(CinematicPageColor)
    ) {
        item(key = "cinematic-stage") {
            CinematicTopStage(
                content = content,
                language = language,
                presentation = presentation,
                alpha = 1f,
                heroHeight = heroHeight,
                onAction = onAction
            )
        }

        if (isSeries) {
            if (presentation.seasonSelectorMode != "dropdown") {
                item(key = "cinematic-season-title") {
                    Text(
                        text = "${AppStrings.t(language, "auto.season")} ${content.selectedSeason}",
                        color = Color.White,
                        fontSize = 20.sp,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.padding(start = 18.dp, end = 18.dp, top = 24.dp, bottom = 2.dp)
                    )
                }
            }
            item(key = "cinematic-season-selector") {
                SeasonSelector(
                    content = content,
                    language = language,
                    onAction = onAction,
                    mode = presentation.seasonSelectorMode,
                    headerStyle = presentation.seasonSelectorMode == "dropdown"
                )
            }
            detailEpisodeItems(content, language, presentation, onAction)
        }

        if (content.trailers.isNotEmpty()) {
            item(key = "cinematic-trailers") {
                CinematicTrailerRail(content.trailers, language)
            }
        }

        if (presentation.showCast && content.cast.isNotEmpty()) {
            item(key = "cinematic-cast") {
                CastSection(
                    members = content.cast,
                    language = language,
                    modifier = Modifier.padding(top = 24.dp)
                )
            }
        }
        if (presentation.showRecommendations && content.relatedItems.isNotEmpty()) {
            item(key = "cinematic-related-title") {
                DetailSectionTitle(
                    AppStrings.t(language, "auto.similar_titles"),
                    modifier = Modifier.padding(top = 26.dp)
                )
            }
            item(key = "cinematic-related") {
                RelatedGrid(content.relatedItems, onAction)
            }
        }
        if (content.traktComments.isNotEmpty()) {
            item(key = "cinematic-trakt-comments") {
                DiscussionSection(AppStrings.t(language, "detail.trakt_comments"), content.traktComments, language)
            }
        }
        if (content.mdblistDiscussion.isNotEmpty()) {
            item(key = "cinematic-mdblist-comments") {
                DiscussionSection(AppStrings.t(language, "detail.mdblist_discussion"), content.mdblistDiscussion, language)
            }
        }
        item(key = "cinematic-bottom-space") { Box(Modifier.height(72.dp)) }
    }
}

@Composable
internal fun CompactMobileDetailContent(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    presentation: DetailPresentationOptions,
    onStickyContextVisibilityChanged: (Boolean) -> Unit
) {
    val listState = rememberLazyListState()
    ObserveStickyContextVisibility(listState, onStickyContextVisibilityChanged, threshold = 235.dp)
    val isSeries = content.type == "series" && content.availableSeasons.isNotEmpty()

    LazyColumn(state = listState, modifier = Modifier.fillMaxSize()) {
        item(key = "compact-hero") {
            CompactHero(content, language, presentation)
        }
        item(key = "compact-actions") {
            CinematicActions(content, language, onAction, compact = true)
        }
        if (content.description.isNotBlank()) {
            item(key = "compact-description") {
                Text(
                    text = content.description,
                    color = Color.White.copy(alpha = 0.76f),
                    fontSize = 13.sp,
                    lineHeight = 18.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(horizontal = 18.dp, vertical = 12.dp)
                )
            }
        }
        if (isSeries) {
            item(key = "compact-season") {
                SeasonSelector(content, language, onAction, presentation.seasonSelectorMode)
            }
            items(content.seasonEpisodes, key = { it.id }) { episode ->
                EpisodeRow(
                    episode = episode,
                    content = content,
                    onAction = onAction,
                    showDescription = false,
                    compact = true,
                    blurUnwatched = presentation.blurUnwatchedEpisodes
                )
            }
        }
        if (presentation.showCast && content.cast.isNotEmpty()) {
            item(key = "compact-cast") {
                CastSection(content.cast, language, Modifier.padding(top = 18.dp))
            }
        }
        if (presentation.showRecommendations && content.relatedItems.isNotEmpty()) {
            item(key = "compact-related-title") {
                DetailSectionTitle(AppStrings.t(language, "auto.similar_titles"), Modifier.padding(top = 20.dp))
            }
            item(key = "compact-related") { RelatedGrid(content.relatedItems, onAction) }
        }
        item(key = "compact-bottom-space") { Box(Modifier.height(32.dp)) }
    }
}

@Composable
private fun CinematicTopStage(
    content: DetailUiModel,
    language: String?,
    presentation: DetailPresentationOptions,
    alpha: Float,
    heroHeight: Dp,
    onAction: (DetailAction) -> Unit
) {
    BoxWithConstraints(
        modifier = Modifier
            .fillMaxWidth()
            .height(heroHeight)
            .alpha(alpha)
    ) {
        FluxaRemoteImage(
            imageUrl = content.backgroundUrl ?: content.posterUrl,
            cacheKey = "detail-cinematic-hero:${content.id}",
            contentDescription = null,
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Crop,
            alignment = Alignment.TopCenter
        )
        val trailerSurface = LocalHeroTrailerSurface.current
        if (content.trailerUrl != null && trailerSurface != null) {
            trailerSurface(content.trailerUrl, emptyList(), {}, Modifier.fillMaxSize())
        }

        // A long fade is the important visual transition here. Do not terminate the
        // artwork and then start a flat surface; the content should appear to emerge
        // from the backdrop just like the reference screen.
        Box(
            Modifier
                .fillMaxSize()
                .background(
                    Brush.verticalGradient(
                        0f to Color.Black.copy(alpha = 0.12f),
                        0.42f to Color.Black.copy(alpha = 0.02f),
                        0.58f to CinematicPageColor.copy(alpha = 0.04f),
                        0.76f to CinematicPageColor.copy(alpha = 0.26f),
                        0.92f to CinematicPageColor.copy(alpha = 0.70f),
                        1f to CinematicPageColor
                    )
                )
        )
        Box(
            Modifier
                .fillMaxSize()
                .background(
                    Brush.horizontalGradient(
                        0f to Color.Black.copy(alpha = 0.10f),
                        0.50f to Color.Transparent,
                        1f to Color.Black.copy(alpha = 0.10f)
                    )
                )
        )

        Column(
            modifier = Modifier
                .align(Alignment.TopCenter)
                .fillMaxWidth()
                .padding(top = maxHeight * 0.31f)
                .padding(horizontal = 22.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            if (presentation.preferClearlogo && content.logoUrl != null) {
                FluxaRemoteImage(
                    imageUrl = content.logoUrl,
                    cacheKey = "detail-cinematic-logo:${content.id}",
                    contentDescription = content.title,
                    modifier = Modifier
                        .fillMaxWidth(0.88f)
                        .heightIn(max = 72.dp),
                    contentScale = ContentScale.Fit,
                    trimTransparentPadding = true
                )
            } else {
                Text(
                    text = content.title,
                    color = Color.White,
                    fontSize = 26.sp,
                    lineHeight = 29.sp,
                    textAlign = TextAlign.Center,
                    fontWeight = FontWeight.ExtraBold,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
            }

            val metadata = buildList {
                content.releaseLabel.takeIf(String::isNotBlank)?.let(::add)
                content.genres.firstOrNull()?.takeIf(String::isNotBlank)?.let(::add)
                content.ageRating?.takeIf(String::isNotBlank)?.let(::add)
                content.runtimeLabel?.takeIf(String::isNotBlank)?.let(::add)
            }
            if (metadata.isNotEmpty()) {
                Text(
                    text = metadata.take(4).joinToString("  ·  "),
                    color = Color.White.copy(alpha = 0.84f),
                    fontSize = 14.sp,
                    lineHeight = 18.sp,
                    textAlign = TextAlign.Center,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 6.dp)
                )
            }
        }

        Column(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .padding(bottom = 3.dp)
        ) {
            CinematicActions(content, language, onAction, embeddedInHero = true)
            CinematicSummary(content, embeddedInHero = true)
        }
    }
}

@Composable
private fun CompactHero(content: DetailUiModel, language: String?, presentation: DetailPresentationOptions) {
    Box(Modifier.fillMaxWidth().height(285.dp)) {
        FluxaRemoteImage(
            imageUrl = content.backgroundUrl ?: content.posterUrl,
            cacheKey = "detail-compact-hero:${content.id}",
            contentDescription = null,
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Crop,
            alignment = Alignment.TopCenter
        )
        Box(
            Modifier.fillMaxSize().background(
                Brush.verticalGradient(
                    0f to Color.Black.copy(alpha = 0.18f),
                    0.62f to Color.Black.copy(alpha = 0.42f),
                    1f to FluxaColors.background
                )
            )
        )
        Column(Modifier.align(Alignment.BottomStart).padding(horizontal = 18.dp, vertical = 14.dp)) {
            if (presentation.preferClearlogo && content.logoUrl != null) {
                FluxaRemoteImage(
                    imageUrl = content.logoUrl,
                    cacheKey = "detail-compact-logo:${content.id}",
                    contentDescription = content.title,
                    modifier = Modifier.fillMaxWidth(0.58f).heightIn(max = 62.dp),
                    contentScale = ContentScale.Fit,
                    trimTransparentPadding = true
                )
            } else {
                Text(content.title, color = Color.White, fontSize = 25.sp, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            Text(
                text = listOfNotNull(
                    content.releaseLabel.takeIf(String::isNotBlank),
                    content.ageRating?.takeIf(String::isNotBlank),
                    content.runtimeLabel?.takeIf(String::isNotBlank)
                ).joinToString(" · "),
                color = Color.White.copy(alpha = 0.72f),
                fontSize = 12.sp,
                modifier = Modifier.padding(top = 7.dp)
            )
        }
    }
}

@Composable
private fun CinematicActions(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    compact: Boolean = false,
    embeddedInHero: Boolean = false
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 18.dp, vertical = if (embeddedInHero) 3.dp else if (compact) 7.dp else 6.dp),
        horizontalArrangement = Arrangement.spacedBy(9.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        CinematicPlayButton(
            content = content,
            language = language,
            onAction = onAction,
            compact = compact,
            embeddedInHero = embeddedInHero,
            modifier = Modifier.weight(1f)
        )
        if (content.supportsWatchlist) {
            DetailRoundedAction(
                selected = content.isInWatchlist,
                onClick = { onAction(DetailAction.ToggleWatchlist) },
                compact = embeddedInHero
            ) {
                Icon(
                    imageVector = if (content.isInWatchlist) Icons.Filled.Check else Icons.Outlined.BookmarkBorder,
                    contentDescription = AppStrings.t(language, if (content.isInWatchlist) "auto.in_list" else "auto.my_list"),
                    tint = Color(0xFF1B1C1D),
                    modifier = Modifier.size(20.dp)
                )
            }
        }
        if (content.supportsLike) {
            DetailRoundedAction(
                selected = content.isLiked,
                onClick = { onAction(DetailAction.ToggleLike) },
                compact = embeddedInHero
            ) {
                Icon(
                    imageVector = Icons.Outlined.Shuffle,
                    contentDescription = AppStrings.t(language, "common.shuffle"),
                    tint = Color(0xFF1B1C1D),
                    modifier = Modifier.size(19.dp)
                )
            }
        }
    }
    if (!embeddedInHero && content.resumeProgress > 0L && content.resumeVideoId != null) {
        Text(
            text = AppStrings.t(language, "auto.restart"),
            color = Color.White.copy(alpha = 0.60f),
            fontSize = 11.sp,
            fontWeight = FontWeight.Medium,
            modifier = Modifier
                .padding(start = 20.dp, top = 0.dp)
                .clickable { onAction(DetailAction.Play(fromStart = true)) }
                .padding(vertical = 5.dp)
        )
    }
}

@Composable
private fun CinematicPlayButton(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    compact: Boolean,
    embeddedInHero: Boolean = false,
    modifier: Modifier = Modifier
) {
    val resumeEpisode = content.resumeVideoId?.let { id -> content.seasonEpisodes.firstOrNull { it.id == id } }
    val label = if (content.resumeProgress > 0L && resumeEpisode?.season != null && resumeEpisode.number != null) {
        AppStrings.t(language, "auto.resume") + " S${resumeEpisode.season}E${resumeEpisode.number}"
    } else if (content.resumeProgress > 0L) {
        AppStrings.t(language, "auto.resume")
    } else {
        AppStrings.t(language, "common.play")
    }
    Row(
        modifier = modifier
            .height(if (embeddedInHero) 40.dp else if (compact) 42.dp else 44.dp)
            .clip(RoundedCornerShape(if (embeddedInHero) 11.dp else 10.dp))
            .background(Color.White)
            .clickable { onAction(DetailAction.Play()) }
            .padding(horizontal = 16.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(Icons.Rounded.PlayArrow, contentDescription = null, tint = Color.Black, modifier = Modifier.size(19.dp))
        Text(
            text = label,
            color = Color.Black,
            fontWeight = FontWeight.SemiBold,
            fontSize = 14.sp,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(start = 5.dp)
        )
    }
}

@Composable
private fun DetailRoundedAction(
    selected: Boolean,
    onClick: () -> Unit,
    compact: Boolean = false,
    content: @Composable () -> Unit
) {
    Box(
        modifier = Modifier
            .size(if (compact) 40.dp else 44.dp)
            .clip(RoundedCornerShape(10.dp))
            .background(if (selected) Color.White else CinematicSecondaryActionColor)
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center
    ) { content() }
}

@Composable
private fun CinematicSummary(content: DetailUiModel, embeddedInHero: Boolean = false) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = if (embeddedInHero) 28.dp else 24.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        if (content.description.isNotBlank()) {
            Text(
                text = content.description,
                color = Color.White.copy(alpha = 0.88f),
                fontSize = if (embeddedInHero) 13.5.sp else 14.sp,
                lineHeight = if (embeddedInHero) 18.sp else 19.sp,
                fontWeight = FontWeight.Normal,
                textAlign = TextAlign.Center,
                maxLines = 3,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = if (embeddedInHero) 3.dp else 4.dp)
            )
        }
        val ratings = content.visibleRatings().take(5)
        if (ratings.isNotEmpty()) {
            val ratingLogo = LocalDetailRatingLogo.current
            Row(
                modifier = Modifier.fillMaxWidth().padding(top = if (embeddedInHero) 6.dp else 8.dp),
                horizontalArrangement = Arrangement.Center,
                verticalAlignment = Alignment.CenterVertically
            ) {
                ratings.forEachIndexed { index, rating ->
                    if (index > 0) Box(Modifier.width(10.dp))
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(3.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        ratingLogo(rating.source, rating.value, Modifier.height(if (rating.source == "imdb") 15.dp else 13.dp))
                        Text(
                            rating.displayValue(),
                            color = rating.scoreColor().copy(alpha = 0.90f),
                            fontSize = 10.5.sp,
                            fontWeight = FontWeight.Medium
                        )
                    }
                }
            }
        }
    }
}

internal fun LazyListScope.detailEpisodeItems(
    content: DetailUiModel,
    language: String?,
    presentation: DetailPresentationOptions,
    onAction: (DetailAction) -> Unit
) {
    when (presentation.episodeCardsLayout) {
        "list" -> items(content.seasonEpisodes, key = { "cinematic-list:${it.id}" }) { episode ->
            EpisodeRow(
                episode = episode,
                content = content,
                onAction = onAction,
                showDescription = presentation.showEpisodeDescriptions,
                blurUnwatched = presentation.blurUnwatchedEpisodes
            )
        }
        "carousel", "horizontal", "grid" -> item(key = "cinematic-episode-rail") {
            LazyRow(
                contentPadding = PaddingValues(horizontal = 18.dp, vertical = 8.dp),
                horizontalArrangement = Arrangement.spacedBy(13.dp)
            ) {
                items(content.seasonEpisodes, key = { it.id }) { episode ->
                    CinematicEpisodeCard(
                        episode = episode,
                        language = language,
                        showDescription = presentation.showEpisodeDescriptions,
                        blurUnwatched = presentation.blurUnwatchedEpisodes,
                        onClick = { onAction(DetailAction.EpisodeSelected(episode.id)) },
                        modifier = Modifier.width(218.dp)
                    )
                }
            }
        }
        "desktop_grid" -> {
            val rows = content.seasonEpisodes.chunked(3)
            items(rows, key = { row -> "desktop:${row.joinToString("|") { it.id }}" }) { rowEpisodes ->
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 18.dp, vertical = 10.dp),
                    horizontalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    rowEpisodes.forEach { episode ->
                        CinematicEpisodeCard(
                            episode = episode,
                            language = language,
                            showDescription = presentation.showEpisodeDescriptions,
                            blurUnwatched = presentation.blurUnwatchedEpisodes,
                            onClick = { onAction(DetailAction.EpisodeSelected(episode.id)) },
                            modifier = Modifier.weight(1f)
                        )
                    }
                    repeat(3 - rowEpisodes.size) {
                        Box(modifier = Modifier.weight(1f))
                    }
                }
            }
        }
        else -> {
            val rows = content.seasonEpisodes.chunked(2)
            items(rows, key = { row -> row.joinToString("|") { it.id } }) { rowEpisodes ->
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 18.dp, vertical = 7.dp),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    rowEpisodes.forEach { episode ->
                        CinematicEpisodeCard(
                            episode = episode,
                            language = language,
                            showDescription = presentation.showEpisodeDescriptions,
                            blurUnwatched = presentation.blurUnwatchedEpisodes,
                            onClick = { onAction(DetailAction.EpisodeSelected(episode.id)) },
                            modifier = Modifier.weight(1f)
                        )
                    }
                    if (rowEpisodes.size == 1) Box(Modifier.weight(1f))
                }
            }
        }
    }
}

@Composable
private fun CinematicTrailerRail(trailers: List<DetailTrailerUiModel>, language: String?) {
    Column(modifier = Modifier.padding(top = 20.dp)) {
        Row(
            modifier = Modifier.padding(horizontal = 18.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = AppStrings.t(language, "auto.trailer"),
                color = Color.White,
                fontSize = 19.sp,
                fontWeight = FontWeight.Bold
            )
            Icon(
                imageVector = Icons.Filled.ArrowDropDown,
                contentDescription = null,
                tint = Color.White.copy(alpha = 0.65f),
                modifier = Modifier.size(21.dp)
            )
        }
        LazyRow(
            contentPadding = PaddingValues(start = 18.dp, top = 12.dp, end = 18.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            itemsIndexed(trailers, key = { index, trailer -> "detail-trailer:${trailer.id}:$index" }) { _, trailer ->
                CinematicTrailerCard(trailer)
            }
        }
    }
}

@Composable
private fun CinematicTrailerCard(trailer: DetailTrailerUiModel) {
    Column(modifier = Modifier.width(200.dp)) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(16f / 9f)
                .clip(RoundedCornerShape(7.dp))
                .background(FluxaColors.surfaceCard)
        ) {
            FluxaRemoteImage(
                imageUrl = trailer.thumbnailUrl,
                cacheKey = "detail-trailer:${trailer.id}",
                contentDescription = trailer.title,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop
            )
        }
        Text(
            text = trailer.title,
            color = Color.White,
            fontSize = 14.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(top = 8.dp)
        )
        if (trailer.sourceLabel.isNotBlank()) {
            Text(
                text = trailer.sourceLabel,
                color = Color.White.copy(alpha = 0.6f),
                fontSize = 11.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = 2.dp)
            )
        }
    }
}

@Composable
private fun CinematicEpisodeCard(
    episode: DetailEpisodeUiModel,
    language: String?,
    showDescription: Boolean,
    blurUnwatched: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    val isDesktop = LocalDeviceType.current == DeviceType.Desktop
    Column(
        modifier = modifier
            .alpha(if (episode.isUpcoming) 0.58f else 1f)
            .clickable(enabled = !episode.isUpcoming, onClick = onClick)
    ) {
        Box(
            Modifier
                .fillMaxWidth()
                .aspectRatio(16f / 9f)
                .clip(RoundedCornerShape(8.dp))
                .background(FluxaColors.surfaceCard)
        ) {
            FluxaRemoteImage(
                imageUrl = episode.thumbnailUrl,
                cacheKey = "detail-cinematic-episode:${episode.id}",
                contentDescription = episode.title,
                modifier = Modifier
                    .fillMaxSize()
                    .then(if (blurUnwatched && !episode.isWatched) Modifier.blur(9.dp) else Modifier),
                contentScale = ContentScale.Crop
            )
            if (episode.isWatched) {
                Box(
                    modifier = Modifier
                        .padding(7.dp)
                        .size(20.dp)
                        .clip(CircleShape)
                        .background(Color.Black.copy(alpha = 0.42f))
                        .border(1.dp, Color.White.copy(alpha = 0.84f), CircleShape),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(Icons.Filled.Check, contentDescription = null, tint = Color.White, modifier = Modifier.size(17.dp))
                }
            }
        }
        Column(
            modifier = if (isDesktop) {
                Modifier
                    .fillMaxWidth()
                    .background(FluxaColors.background.copy(alpha = 0.92f))
                    .padding(horizontal = 10.dp, vertical = 8.dp)
            } else {
                Modifier
            }
        ) {
            Text(
                text = episode.number?.let { "${AppStrings.t(language, "auto.episode")} $it" } ?: episode.title,
                color = Color.White.copy(alpha = 0.58f),
                fontSize = 11.sp,
                modifier = if (isDesktop) Modifier else Modifier.padding(top = 7.dp)
            )
            Text(
                text = episode.title,
                color = Color.White,
                fontSize = 14.sp,
                lineHeight = 17.sp,
                fontWeight = FontWeight.SemiBold,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = 2.dp)
            )
            if (showDescription) {
                episode.description?.takeIf(String::isNotBlank)?.let { description ->
                    Text(
                        text = description,
                        color = Color.White.copy(alpha = 0.72f),
                        fontSize = 11.5.sp,
                        lineHeight = 15.sp,
                        maxLines = 3,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.padding(top = 5.dp)
                    )
                }
            }
            val footer = listOfNotNull(episode.releaseLabel, episode.runtimeLabel).filter(String::isNotBlank).joinToString(" · ")
            if (footer.isNotBlank()) {
                Text(
                    text = footer,
                    color = Color.White.copy(alpha = 0.58f),
                    fontSize = 11.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 5.dp)
                )
            }
        }
    }
}

@Composable
private fun DetailSectionTitle(text: String, modifier: Modifier = Modifier) {
    Text(
        text = text,
        color = Color.White,
        fontSize = 19.sp,
        fontWeight = FontWeight.Bold,
        modifier = modifier
            .padding(horizontal = 18.dp)
            .padding(bottom = 8.dp)
    )
}
