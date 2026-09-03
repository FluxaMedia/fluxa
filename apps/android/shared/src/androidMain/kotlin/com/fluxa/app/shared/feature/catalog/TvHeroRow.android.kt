@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)

package com.fluxa.app.shared.feature.catalog

import com.fluxa.app.ui.catalog.FluxaIcons

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.material3.Text
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.focusGroup
import androidx.tv.material3.Carousel
import androidx.tv.material3.rememberCarouselState
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.shared.shortenHeroSynopsis
import com.fluxa.app.shared.LocalHeroTrailerSurface
import com.fluxa.app.shared.feature.player.TrailerCue
import com.fluxa.app.ui.catalog.LocalWindowWidthClass
import com.fluxa.app.ui.catalog.WindowWidthClass

@Composable
actual fun TvHeroRow(
    items: List<CatalogItemUiModel>,
    language: String?,
    onItemClick: (CatalogItemUiModel) -> Unit,
    onPlayClick: (CatalogItemUiModel) -> Unit,
    modifier: Modifier,
    trailerItemId: String?,
    trailerUrl: String?,
    trailerSubtitleCues: List<TrailerCue>,
    focusedItemOverride: CatalogItemUiModel?,
    posterFocused: Boolean,
    onActiveItemChanged: (CatalogItemUiModel) -> Unit,
    onFocusChanged: (Boolean) -> Unit,
    focusRequester: FocusRequester?,
    downFocusRequester: FocusRequester?,
    onDownPressed: () -> Unit,
    leftFocusRequester: FocusRequester?
) {
    if (items.isEmpty()) return
    val hero = items.take(10)
    val isExpanded = LocalWindowWidthClass.current == WindowWidthClass.Expanded
    val heroShape = if (isExpanded) RectangleShape else RoundedCornerShape(16.dp)
    val carouselState = rememberCarouselState()
    var heroFocused by remember { mutableStateOf(false) }
    val isPosterDriven = isExpanded && focusedItemOverride != null && !heroFocused
    val showPosterScrim = posterFocused && !heroFocused
    val showsHeroDetails = heroFocused || !isExpanded || focusedItemOverride != null
    val sideGradientAlpha by animateFloatAsState(
        targetValue = if (showPosterScrim) 1f else 0f,
        animationSpec = tween(420),
        label = "tv-hero-side-gradient-alpha"
    )
    val activeItem = focusedItemOverride ?: hero[carouselState.activeItemIndex]
    LaunchedEffect(activeItem.id) {
        onActiveItemChanged(activeItem)
    }
    Carousel(
        itemCount = hero.size,
        carouselState = carouselState,
        contentTransformStartToEnd = (
            fadeIn(animationSpec = tween(360))
            ).togetherWith(fadeOut(animationSpec = tween(240))),
        contentTransformEndToStart = (
            fadeIn(animationSpec = tween(360))
            ).togetherWith(fadeOut(animationSpec = tween(240))),
        carouselIndicator = {},
        modifier = modifier
            .fillMaxSize()
            .clip(heroShape)
            .then(if (focusRequester != null) Modifier.focusRequester(focusRequester) else Modifier)
            .focusProperties {
                downFocusRequester?.let { down = it }
                leftFocusRequester?.let { left = it }
            }
            .onPreviewKeyEvent { event ->
                if (event.type == KeyEventType.KeyDown && event.key == Key.DirectionDown) {
                    onDownPressed()
                    true
                } else {
                    false
                }
            }
            .onFocusChanged {
                heroFocused = it.hasFocus
                onFocusChanged(it.hasFocus)
            }
    ) { index ->
        val item = focusedItemOverride ?: hero[index]
        Box(
            modifier = Modifier
                .fillMaxSize()
                .clickable { onItemClick(item) }
        ) {
            AnimatedVisibility(
                visible = !heroFocused,
                enter = fadeIn(animationSpec = tween(280)),
                exit = fadeOut(animationSpec = tween(180))
            ) {
                AnimatedContent(
                    targetState = item,
                    transitionSpec = {
                        fadeIn(animationSpec = tween(320)) togetherWith
                            fadeOut(animationSpec = tween(220))
                    },
                    label = "tv-hero-artwork"
                ) { artworkItem ->
                    FluxaRemoteImage(
                        imageUrl = artworkItem.backdropUrl ?: artworkItem.card.artworkUrl,
                        cacheKey = "tv-hero:${artworkItem.id}",
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop
                    )
                }
            }
            val trailerSurface = LocalHeroTrailerSurface.current
            AnimatedVisibility(
                visible = heroFocused && index == carouselState.activeItemIndex && item.id == trailerItemId && trailerUrl != null && trailerSurface != null,
                enter = fadeIn(animationSpec = tween(durationMillis = 420, delayMillis = 180)),
                exit = fadeOut(animationSpec = tween(180))
            ) {
                trailerSurface?.invoke(
                    trailerUrl.orEmpty(),
                    trailerSubtitleCues,
                    {},
                    Modifier.fillMaxSize()
                )
            }
            if (showPosterScrim) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(
                            Brush.horizontalGradient(
                                0f to Color.Black.copy(alpha = 0.98f * sideGradientAlpha),
                                0.28f to Color.Black.copy(alpha = 0.92f * sideGradientAlpha),
                                0.58f to Color.Black.copy(alpha = 0.48f * sideGradientAlpha),
                                0.88f to Color.Transparent
                            )
                        )
                )
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(
                            Brush.verticalGradient(
                                0f to Color.Transparent,
                                0.36f to Color.Transparent,
                                0.72f to Color.Black.copy(alpha = 0.78f),
                                1f to Color.Black.copy(alpha = 0.96f)
                            )
                        )
                )
            }
            if (showsHeroDetails) {
                Column(
                    modifier = Modifier
                        .align(if (isPosterDriven) Alignment.TopStart else Alignment.BottomStart)
                        .padding(
                            start = if (isPosterDriven) 192.dp else if (isExpanded) 72.dp else 36.dp,
                            top = if (isPosterDriven) 80.dp else 0.dp,
                            bottom = if (isPosterDriven) 0.dp else if (isExpanded) 28.dp else 40.dp,
                            end = 40.dp
                        )
                ) {
                if (!item.card.logoUrl.isNullOrBlank()) {
                    FluxaRemoteImage(
                        imageUrl = item.card.logoUrl,
                        cacheKey = "tv-hero-logo:${item.card.logoUrl}",
                        contentDescription = item.card.title,
                        modifier = Modifier
                            .width(440.dp)
                            .height(104.dp),
                        contentScale = ContentScale.Fit,
                        alignment = Alignment.CenterStart,
                        trimTransparentPadding = true
                    )
                } else {
                    Text(
                        text = item.card.title,
                        color = Color.White,
                        fontSize = 36.sp,
                        fontWeight = FontWeight.Bold,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis
                    )
                }
                val metadataParts = remember(item.genres, item.releaseLabel, item.type, item.seasonsCount, item.runtimeLabel, item.ageRating, language) {
                    buildList {
                        item.genres.firstOrNull { it.isNotBlank() }?.let { add(it) }
                        item.releaseLabel?.takeIf { it.isNotBlank() }?.let { add(it) }
                        if (item.type == "series" && (item.seasonsCount ?: 0) > 0) {
                            add("${item.seasonsCount} ${AppStrings.t(language, "auto.seasons")}")
                        } else {
                            item.runtimeLabel?.takeIf { it.isNotBlank() }?.let { add(it) }
                        }
                        item.ageRating?.takeIf { it.isNotBlank() }?.let { add(it) }
                    }
                }
                if (metadataParts.isNotEmpty()) {
                    Row(
                        modifier = Modifier.padding(top = 8.dp),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        metadataParts.forEachIndexed { index, part ->
                            if (index > 0) {
                                Text(text = "•", color = Color.White.copy(alpha = 0.5f), fontSize = 13.sp, fontWeight = FontWeight.Bold)
                            }
                            Text(text = part, color = Color.White.copy(alpha = 0.85f), fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                        }
                    }
                }
                val description = item.description
                if (!description.isNullOrBlank()) {
                    val shortenedDescription = remember(item.id, description) { shortenHeroSynopsis(description) }
                    Text(
                        text = shortenedDescription,
                        color = Color.White.copy(alpha = 0.8f),
                        fontSize = 15.sp,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.padding(top = 10.dp).widthIn(max = 440.dp)
                    )
                }
                if (heroFocused) {
                    Row(
                        modifier = Modifier.padding(top = 16.dp),
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Button(
                            onClick = { onPlayClick(item) },
                            colors = ButtonDefaults.buttonColors(
                                containerColor = Color.White,
                                contentColor = Color.Black
                            ),
                            shape = RoundedCornerShape(4.dp),
                            contentPadding = PaddingValues(horizontal = 24.dp, vertical = 0.dp),
                            modifier = Modifier.height(44.dp)
                        ) {
                            Icon(FluxaIcons.Filled.PlayArrow, contentDescription = null)
                            Spacer(Modifier.width(6.dp))
                            Text(
                                AppStrings.t(language, "common.play"),
                                fontSize = 18.sp,
                                fontWeight = FontWeight.Medium
                            )
                        }
                        Button(
                            onClick = { onItemClick(item) },
                            colors = ButtonDefaults.buttonColors(
                                containerColor = Color.White.copy(alpha = 0.55f),
                                contentColor = Color.White
                            ),
                            shape = RoundedCornerShape(4.dp),
                            contentPadding = PaddingValues(horizontal = 24.dp, vertical = 0.dp),
                            modifier = Modifier.height(44.dp)
                        ) {
                            Text(
                                AppStrings.t(language, "hero.more_info"),
                                fontSize = 18.sp,
                                fontWeight = FontWeight.Medium
                            )
                        }
                    }
                }
                }
            }
        }
    }
}
