package com.fluxa.app.ui.catalog

import androidx.compose.animation.Crossfade
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.CompositingStrategy
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.shared.image.FluxaRemoteImage

@Composable
internal fun PlayerTerminalRecommendations(
    items: List<Meta>,
    language: String?,
    onSelected: (Meta) -> Unit,
    onDismiss: () -> Unit,
) {
    val visibleItems = remember(items) { items.take(10) }
    if (visibleItems.isEmpty()) return

    var selectedIndex by remember(visibleItems.map { it.id }) { mutableIntStateOf(0) }
    val activeIndex = selectedIndex.coerceIn(0, visibleItems.lastIndex)
    val activeMeta = visibleItems[activeIndex]
    val widthClass = LocalWindowWidthClass.current
    val activeArtwork = activeMeta.background ?: activeMeta.poster

    BoxWithConstraints(
        modifier = Modifier
            .fillMaxSize()
            .graphicsLayer { compositingStrategy = CompositingStrategy.Offscreen }
            .drawWithContent {
                drawContent()
                val miniWidth = size.width * when {
                    size.width > size.height * 1.15f && widthClass == WindowWidthClass.Compact -> 0.30f
                    widthClass == WindowWidthClass.Compact -> 0.78f
                    widthClass == WindowWidthClass.Medium -> 0.30f
                    else -> 0.24f
                }
                val miniHeight = miniWidth * 9f / 16f
                drawRect(
                    color = Color.Transparent,
                    topLeft = androidx.compose.ui.geometry.Offset(24.dp.toPx(), 24.dp.toPx()),
                    size = androidx.compose.ui.geometry.Size(miniWidth, miniHeight),
                    blendMode = BlendMode.Clear,
                )
            }
            .background(FluxaColors.background),
    ) {
        val isLandscape = maxWidth > maxHeight * 1.15f
        val layout = rememberTerminalRecommendationLayout(
            if (widthClass == WindowWidthClass.Compact && isLandscape) WindowWidthClass.Medium else widthClass,
        )
        Crossfade(
            targetState = activeArtwork,
            animationSpec = tween(360),
            label = "terminal-recommendation-backdrop",
        ) { artworkUrl ->
            FluxaRemoteImage(
                imageUrl = artworkUrl,
                cacheKey = artworkUrl?.let { "terminal-recommendation-backdrop:$it" },
                contentDescription = activeMeta.name,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
                alignment = Alignment.Center,
            )
        }

        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.verticalGradient(
                        colorStops = arrayOf(
                            0f to Color.Black.copy(alpha = 0.08f),
                            0.48f to Color.Transparent,
                            0.78f to FluxaColors.background.copy(alpha = 0.62f),
                            1f to FluxaColors.background.copy(alpha = 0.98f),
                        ),
                    ),
                ),
        )

        RecommendationCloseButton(
            language = language,
            onDismiss = onDismiss,
            modifier = Modifier
                .align(Alignment.TopEnd)
                .padding(layout.closePadding),
        )

        Column(
            modifier = Modifier
                .align(layout.heroAlignment)
                .fillMaxWidth(layout.heroWidthFraction)
                .widthIn(max = layout.heroMaxWidth)
                .padding(layout.heroPadding),
            verticalArrangement = Arrangement.spacedBy(layout.heroSpacing),
            horizontalAlignment = Alignment.Start,
        ) {
            Crossfade(
                targetState = activeMeta.logo?.takeIf { it.isNotBlank() },
                animationSpec = tween(260),
                label = "terminal-recommendation-logo",
            ) { logoUrl ->
                if (logoUrl != null) {
                    FluxaRemoteImage(
                        imageUrl = logoUrl,
                        cacheKey = "terminal-recommendation-logo:$logoUrl",
                        contentDescription = activeMeta.name,
                        modifier = Modifier
                            .heightIn(max = layout.logoMaxHeight)
                            .widthIn(max = layout.logoMaxWidth),
                        contentScale = ContentScale.Fit,
                        trimTransparentPadding = true,
                    )
                } else {
                    Text(
                        text = activeMeta.name,
                        color = Color.White,
                        fontSize = layout.titleSize,
                        lineHeight = layout.titleLineHeight,
                        fontWeight = FontWeight.ExtraBold,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }

            RecommendationMetadata(meta = activeMeta, textSize = layout.metaTextSize)

            activeMeta.description
                ?.takeIf { it.isNotBlank() }
                ?.let { description ->
                    Text(
                        text = description,
                        color = Color.White.copy(alpha = 0.92f),
                        fontSize = layout.descriptionSize,
                        lineHeight = layout.descriptionLineHeight,
                        fontWeight = FontWeight.Medium,
                        maxLines = layout.descriptionMaxLines,
                        overflow = TextOverflow.Ellipsis,
                    )
                }

            Row(
                modifier = Modifier
                    .height(layout.playButtonHeight)
                    .clip(RoundedCornerShape(layout.buttonCornerRadius))
                    .background(Color.White)
                    .clickable { onSelected(activeMeta) }
                    .padding(horizontal = layout.playButtonHorizontalPadding),
                horizontalArrangement = Arrangement.Center,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(
                    imageVector = FluxaIcons.Filled.PlayArrow,
                    contentDescription = null,
                    tint = Color.Black,
                    modifier = Modifier.size(layout.playIconSize),
                )
                Text(
                    text = AppStrings.t(language, "common.play"),
                    color = Color.Black,
                    fontSize = layout.playTextSize,
                    fontWeight = FontWeight.ExtraBold,
                    modifier = Modifier.padding(start = 6.dp),
                )
            }
        }

        RecommendationIndicators(
            count = visibleItems.size,
            selectedIndex = activeIndex,
            onSelected = { index -> selectedIndex = index },
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .padding(bottom = layout.indicatorBottomPadding),
        )
    }
}

@Composable
private fun RecommendationCloseButton(
    language: String?,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var focused by remember { mutableIntStateOf(0) }
    val scale by animateFloatAsState(if (focused == 1) 1.08f else 1f, label = "recommendation-close-scale")
    Box(
        modifier = modifier
            .scale(scale)
            .size(42.dp)
            .clip(CircleShape)
            .onFocusChanged { focused = if (it.isFocused) 1 else 0 }
            .clickable(onClick = onDismiss)
            .focusable(),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = FluxaIcons.Filled.Close,
            contentDescription = AppStrings.t(language, "common.close"),
            tint = Color.White,
            modifier = Modifier.size(24.dp),
        )
    }
}

@Composable
private fun RecommendationMetadata(meta: Meta, textSize: TextUnit) {
    val parts = remember(meta.id, meta.ageRating, meta.imdbRating, meta.genres) {
        buildList {
            meta.ageRating?.takeIf { it.isNotBlank() }?.let(::add)
            meta.imdbRating?.takeIf { it.isNotBlank() }?.let { add("IMDb $it") }
            meta.genres.orEmpty().filter { it.isNotBlank() }.take(2).forEach(::add)
        }
    }
    if (parts.isEmpty()) return

    Row(
        horizontalArrangement = Arrangement.spacedBy(7.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        parts.forEachIndexed { index, part ->
            if (index > 0) {
                Text(text = "•", color = Color.White.copy(alpha = 0.52f), fontSize = textSize)
            }
            Text(
                text = part,
                color = Color.White.copy(alpha = 0.9f),
                fontSize = textSize,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
private fun RecommendationIndicators(
    count: Int,
    selectedIndex: Int,
    onSelected: (Int) -> Unit,
    modifier: Modifier,
) {
    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        repeat(count) { index ->
            Box(
                modifier = Modifier
                    .size(width = if (index == selectedIndex) 20.dp else 6.dp, height = 6.dp)
                    .clip(RoundedCornerShape(50))
                    .background(Color.White.copy(alpha = if (index == selectedIndex) 1f else 0.58f))
                    .clickable { onSelected(index) },
            )
        }
    }
}

private data class TerminalRecommendationLayout(
    val heroAlignment: Alignment,
    val heroWidthFraction: Float,
    val heroMaxWidth: Dp,
    val heroPadding: PaddingValues,
    val heroSpacing: Dp,
    val closePadding: PaddingValues,
    val indicatorBottomPadding: Dp,
    val logoMaxHeight: Dp,
    val logoMaxWidth: Dp,
    val titleSize: TextUnit,
    val titleLineHeight: TextUnit,
    val metaTextSize: TextUnit,
    val descriptionSize: TextUnit,
    val descriptionLineHeight: TextUnit,
    val descriptionMaxLines: Int,
    val playButtonHeight: Dp,
    val playButtonHorizontalPadding: Dp,
    val playIconSize: Dp,
    val playTextSize: TextUnit,
    val buttonCornerRadius: Dp,
)

@Composable
private fun rememberTerminalRecommendationLayout(
    widthClass: WindowWidthClass,
): TerminalRecommendationLayout = when (widthClass) {
    WindowWidthClass.Compact -> TerminalRecommendationLayout(
        heroAlignment = Alignment.BottomStart,
        heroWidthFraction = 0.96f,
        heroMaxWidth = 520.dp,
        heroPadding = PaddingValues(start = 16.dp, end = 16.dp, bottom = 150.dp),
        heroSpacing = 8.dp,
        closePadding = PaddingValues(top = 18.dp, end = 16.dp),
        indicatorBottomPadding = 18.dp,
        logoMaxHeight = 50.dp,
        logoMaxWidth = 230.dp,
        titleSize = 28.sp,
        titleLineHeight = 31.sp,
        metaTextSize = 12.sp,
        descriptionSize = 14.sp,
        descriptionLineHeight = 19.sp,
        descriptionMaxLines = 3,
        playButtonHeight = 44.dp,
        playButtonHorizontalPadding = 18.dp,
        playIconSize = 24.dp,
        playTextSize = 14.sp,
        buttonCornerRadius = 10.dp,
    )
    WindowWidthClass.Medium -> TerminalRecommendationLayout(
        heroAlignment = Alignment.BottomEnd,
        heroWidthFraction = 0.46f,
        heroMaxWidth = 560.dp,
        heroPadding = PaddingValues(start = 20.dp, end = 56.dp, bottom = 176.dp),
        heroSpacing = 10.dp,
        closePadding = PaddingValues(top = 24.dp, end = 24.dp),
        indicatorBottomPadding = 24.dp,
        logoMaxHeight = 66.dp,
        logoMaxWidth = 300.dp,
        titleSize = 36.sp,
        titleLineHeight = 40.sp,
        metaTextSize = 14.sp,
        descriptionSize = 16.sp,
        descriptionLineHeight = 22.sp,
        descriptionMaxLines = 4,
        playButtonHeight = 48.dp,
        playButtonHorizontalPadding = 22.dp,
        playIconSize = 27.dp,
        playTextSize = 16.sp,
        buttonCornerRadius = 11.dp,
    )
    WindowWidthClass.Expanded -> TerminalRecommendationLayout(
        heroAlignment = Alignment.BottomEnd,
        heroWidthFraction = 0.43f,
        heroMaxWidth = 560.dp,
        heroPadding = PaddingValues(start = 16.dp, end = 72.dp, bottom = 170.dp),
        heroSpacing = 12.dp,
        closePadding = PaddingValues(top = 20.dp, end = 24.dp),
        indicatorBottomPadding = 28.dp,
        logoMaxHeight = 86.dp,
        logoMaxWidth = 380.dp,
        titleSize = 48.sp,
        titleLineHeight = 52.sp,
        metaTextSize = 15.sp,
        descriptionSize = 17.sp,
        descriptionLineHeight = 23.sp,
        descriptionMaxLines = 5,
        playButtonHeight = 50.dp,
        playButtonHorizontalPadding = 22.dp,
        playIconSize = 28.dp,
        playTextSize = 16.sp,
        buttonCornerRadius = 12.dp,
    )
}
