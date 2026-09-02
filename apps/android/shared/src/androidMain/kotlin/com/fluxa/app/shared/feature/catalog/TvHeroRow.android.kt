@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)

package com.fluxa.app.shared.feature.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
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
import androidx.tv.material3.CarouselDefaults
import androidx.tv.material3.rememberCarouselState
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.shared.shortenHeroSynopsis
import com.fluxa.app.shared.LocalHeroTrailerSurface
import com.fluxa.app.shared.feature.player.TrailerCue

@Composable
actual fun TvHeroRow(
    items: List<CatalogItemUiModel>,
    language: String?,
    onItemClick: (CatalogItemUiModel) -> Unit,
    modifier: Modifier,
    trailerItemId: String?,
    trailerUrl: String?,
    trailerSubtitleCues: List<TrailerCue>,
    onFocusChanged: (Boolean) -> Unit,
    focusRequester: FocusRequester?,
    downFocusRequester: FocusRequester?,
    onDownPressed: () -> Unit
) {
    if (items.isEmpty()) return
    val hero = items.take(10)
    val carouselState = rememberCarouselState()
    var heroFocused by remember { mutableStateOf(false) }
    val sideGradientAlpha by animateFloatAsState(
        targetValue = if (heroFocused) 0f else 1f,
        animationSpec = tween(420),
        label = "tv-hero-side-gradient-alpha"
    )
    Carousel(
        itemCount = hero.size,
        carouselState = carouselState,
        contentTransformStartToEnd = (
            fadeIn(animationSpec = tween(450)) +
                slideInHorizontally(animationSpec = tween(450)) { it / 10 }
            ).togetherWith(fadeOut(animationSpec = tween(300))),
        contentTransformEndToStart = (
            fadeIn(animationSpec = tween(450)) +
                slideInHorizontally(animationSpec = tween(450)) { -it / 10 }
            ).togetherWith(fadeOut(animationSpec = tween(300))),
        carouselIndicator = {
            CarouselDefaults.IndicatorRow(
                itemCount = hero.size,
                activeItemIndex = carouselState.activeItemIndex,
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(bottom = 20.dp),
                spacing = 8.dp,
                indicator = { isActive ->
                    val indicatorWidth by animateDpAsState(
                        targetValue = if (isActive) 28.dp else 8.dp,
                        animationSpec = tween(300),
                        label = "hero-indicator-width"
                    )
                    Box(
                        modifier = Modifier
                            .width(indicatorWidth)
                            .height(6.dp)
                            .background(Color.White, RoundedCornerShape(50))
                    )
                }
            )
        },
        modifier = modifier
            .fillMaxWidth()
            .height(432.dp)
            .clip(RoundedCornerShape(16.dp))
            .then(if (focusRequester != null) Modifier.focusRequester(focusRequester) else Modifier)
            .focusProperties {
                downFocusRequester?.let { down = it }
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
            .then(
                if (heroFocused) {
                    Modifier.border(4.dp, Color.White, RoundedCornerShape(16.dp))
                } else {
                    Modifier
                }
            )
    ) { index ->
        val item = hero[index]
        Box(
            modifier = Modifier
                .fillMaxSize()
                .clickable { onItemClick(item) }
        ) {
            FluxaRemoteImage(
                imageUrl = item.backdropUrl ?: item.card.artworkUrl,
                cacheKey = "tv-hero:${item.id}",
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop
            )
            val trailerSurface = LocalHeroTrailerSurface.current
            if (heroFocused && index == carouselState.activeItemIndex && item.id == trailerItemId && trailerUrl != null && trailerSurface != null) {
                trailerSurface(
                    trailerUrl,
                    trailerSubtitleCues,
                    {},
                    Modifier.fillMaxSize()
                )
            }
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(
                        Brush.horizontalGradient(
                            0f to Color.Black.copy(alpha = 0.9f * sideGradientAlpha),
                            0.42f to Color.Black.copy(alpha = 0.45f * sideGradientAlpha),
                            0.72f to Color.Transparent
                        )
                    )
            )
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(
                        Brush.verticalGradient(
                            0f to Color.Transparent,
                            0.68f to Color.Transparent,
                            1f to Color.Black.copy(alpha = 0.95f)
                        )
                    )
            )
            Column(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(start = 36.dp, bottom = 40.dp, end = 40.dp)
            ) {
                if (!item.card.logoUrl.isNullOrBlank()) {
                    FluxaRemoteImage(
                        imageUrl = item.card.logoUrl,
                        cacheKey = "tv-hero-logo:${item.card.logoUrl}",
                        contentDescription = item.card.title,
                        modifier = Modifier
                            .width(360.dp)
                            .height(82.dp),
                        contentScale = ContentScale.Fit,
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
                    val shortenedDescription = remember(description) { shortenHeroSynopsis(description) }
                    Text(
                        text = shortenedDescription,
                        color = Color.White.copy(alpha = 0.8f),
                        fontSize = 15.sp,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.padding(top = 10.dp).widthIn(max = 620.dp)
                    )
                }
            }
        }
    }
}
