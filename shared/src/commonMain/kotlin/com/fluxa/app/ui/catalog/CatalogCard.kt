package com.fluxa.app.ui.catalog

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.zIndex
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.shared.image.FluxaRemoteImage
import kotlinx.coroutines.delay

@Composable
fun CatalogCard(
    model: CatalogCardUiModel,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    onLongClick: (() -> Unit)? = null
) {
    val density = LocalDensity.current
    val deviceType = LocalDeviceType.current
    val interactionSource = remember { MutableInteractionSource() }
    val hovered by interactionSource.collectIsHoveredAsState()
    var focused by remember { mutableStateOf(false) }
    var animatedArtworkFailed by remember(model.animatedArtworkUrl) { mutableStateOf(false) }
    var artworkFailed by remember(model.artworkUrl) { mutableStateOf(model.artworkUrl.isNullOrBlank()) }
    val wantsAnimatedArtwork = (focused || hovered) &&
        !animatedArtworkFailed &&
        !model.animatedArtworkUrl.isNullOrBlank()
    var showAnimatedArtwork by remember(model.animatedArtworkUrl) { mutableStateOf(false) }
    LaunchedEffect(wantsAnimatedArtwork, focused, deviceType) {
        showAnimatedArtwork = false
        if (wantsAnimatedArtwork) {
            // TV must still play focus artwork. A short debounce avoids starting a decoder
            // for cards that the user crosses during a fast D-pad burst.
            val delayMs = when {
                focused && deviceType == DeviceType.TV -> 180L
                focused -> 120L
                else -> 350L
            }
            delay(delayMs)
            showAnimatedArtwork = true
        }
    }
    val displayingAnimatedArtwork = showAnimatedArtwork && !animatedArtworkFailed
    val displayedArtworkUrl = if (displayingAnimatedArtwork) {
        model.animatedArtworkUrl
    } else {
        model.artworkUrl
    }
    val displayedArtworkCacheKey = if (displayingAnimatedArtwork) {
        model.animatedArtworkMemoryCacheKey
    } else {
        model.artworkMemoryCacheKey
    }
    val displayedArtworkDiskCacheKey = if (displayingAnimatedArtwork) {
        displayedArtworkUrl
    } else {
        model.artworkDiskCacheKey
    }
    val displayedArtworkFailed = if (displayingAnimatedArtwork) {
        animatedArtworkFailed
    } else {
        artworkFailed
    }

    val targetScale = when {
        focused && deviceType == DeviceType.TV -> 1.10f
        focused -> 1.04f
        hovered && deviceType == DeviceType.Desktop -> 1.025f
        else -> 1f
    }
    val cardScale by animateFloatAsState(
        targetValue = targetScale,
        animationSpec = tween(durationMillis = 140),
        label = "catalog-card-scale"
    )

    val rankFontSize = remember(model.topTenRank, model.imageHeight, model.rankFontSizeRatio, density) {
        if (model.topTenRank != null) {
            with(density) { (model.imageHeight.toPx() * model.rankFontSizeRatio).toSp() }
        } else {
            0.sp
        }
    }

    Box(
        modifier = modifier
            .width(model.outerWidth)
            .height(
                model.imageHeight + if (model.showTitleBar) {
                    if (model.subtitle.isNotBlank()) {
                        FluxaDimensions.cardMetaBarWithEpisodeLabelHeight
                    } else {
                        FluxaDimensions.cardMetaBarHeight
                    }
                } else {
                    0.dp
                }
            )
            .onFocusChanged { focused = it.isFocused }
            .then(
                if (deviceType != DeviceType.Mobile) {
                    Modifier.hoverable(interactionSource)
                } else {
                    Modifier
                }
            )
            .zIndex(if (focused || (hovered && deviceType == DeviceType.Desktop)) 1f else 0f)
            .scale(cardScale)
            .then(
                if (focused) {
                    Modifier.border(3.dp, Color.White, RoundedCornerShape(8.dp))
                } else {
                    Modifier
                }
            )
            .combinedClickable(
                interactionSource = interactionSource,
                indication = null,
                onClick = onClick,
                onLongClick = onLongClick
            )
    ) {
        model.topTenRank?.let { rank ->
            Box(
                modifier = Modifier
                    .align(Alignment.TopStart)
                    .width(model.rankNumberBoxWidth)
                    .height(model.imageHeight),
                contentAlignment = Alignment.CenterEnd
            ) {
                Text(
                    text = rank.toString(),
                    color = Color(0xFF111111),
                    fontSize = rankFontSize,
                    fontWeight = FontWeight.Black,
                    maxLines = 1,
                    modifier = Modifier.offset(x = model.rankOffsetX, y = model.rankOffsetY)
                )
            }
        }

        Column(modifier = Modifier.align(Alignment.TopEnd).width(model.width)) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(model.imageHeight)
                    .background(
                        if (model.cardBackgroundIsSurfaceCard) {
                            FluxaColors.surfaceCard
                        } else {
                            Color.White.copy(alpha = FluxaDimensions.Alpha.emptyCardBackground)
                        }
                    ),
                contentAlignment = Alignment.Center
            ) {
                if (model.loadArtwork && !displayedArtworkFailed) {
                    FluxaRemoteImage(
                        imageUrl = displayedArtworkUrl,
                        cacheKey = displayedArtworkCacheKey,
                        diskCacheKey = displayedArtworkDiskCacheKey,
                        requestWidthPx = model.requestWidthPx,
                        requestHeightPx = model.requestHeightPx,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop,
                        onError = {
                            if (displayingAnimatedArtwork) {
                                // A broken GIF should fall back to the already-cached static cover,
                                // not trigger three immediate request/recomposition loops.
                                animatedArtworkFailed = true
                                showAnimatedArtwork = false
                            } else {
                                artworkFailed = true
                            }
                        }
                    )
                } else if (model.allowCoverFallback || displayedArtworkFailed) {
                    Text(
                        text = model.coverFallbackText,
                        color = Color.White.copy(
                            alpha = if (model.coverFallbackIsEmoji) {
                                FluxaDimensions.Alpha.coverEmoji
                            } else {
                                FluxaDimensions.Alpha.coverFallbackText
                            }
                        ),
                        fontSize = if (model.coverFallbackIsEmoji) {
                            FluxaDimensions.CardText.coverEmojiSize
                        } else {
                            FluxaDimensions.CardText.coverFallbackSize
                        },
                        fontWeight = FontWeight.Black
                    )
                }
                if (model.loadArtwork && model.showLogo && model.logoUrl != null) {
                    Box(modifier = Modifier.fillMaxSize().background(MOBILE_CARD_BOTTOM_GRADIENT))
                    FluxaRemoteImage(
                        imageUrl = model.logoUrl,
                        cacheKey = model.logoMemoryCacheKey,
                        contentDescription = null,
                        modifier = Modifier
                            .align(Alignment.BottomStart)
                            .padding(start = 9.dp, bottom = 9.dp)
                            .widthIn(max = model.width * 0.42f)
                            .heightIn(max = model.imageHeight * 0.30f),
                        contentScale = ContentScale.Fit
                    )
                }
                if (model.overlayTitleBar && (model.title.isNotBlank() || model.subtitle.isNotBlank())) {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .background(CONTINUE_WATCHING_LABEL_GRADIENT)
                    )
                    Column(
                        modifier = Modifier
                            .align(Alignment.BottomStart)
                            .fillMaxWidth()
                            .padding(
                                start = 9.dp,
                                end = 9.dp,
                                bottom = if (model.showProgressBar) 17.dp else 8.dp
                            )
                    ) {
                        if (model.title.isNotBlank()) {
                            Text(
                                text = model.title,
                                color = Color.White,
                                fontSize = 12.sp,
                                fontWeight = FontWeight.SemiBold,
                                lineHeight = 13.sp,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                        if (model.subtitle.isNotBlank()) {
                            Text(
                                text = model.subtitle,
                                color = Color.White.copy(alpha = 0.78f),
                                fontSize = 10.sp,
                                fontWeight = FontWeight.Medium,
                                lineHeight = 11.sp,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                    }
                }
                if (model.showProgressBar) {
                    Box(
                        modifier = Modifier
                            .align(Alignment.BottomCenter)
                            .fillMaxWidth()
                            .padding(horizontal = 8.dp, vertical = 6.dp)
                            .height(FluxaDimensions.cardProgressBarHeight)
                            .clip(RoundedCornerShape(999.dp))
                            .background(Color.Black.copy(alpha = 0.50f))
                    ) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth(model.progress)
                                .fillMaxHeight()
                                .background(FluxaColors.progressFill)
                        )
                    }
                }
                if (model.showUpNextBadge) {
                    CardCornerBadge(
                        text = model.upNextLabel,
                        modifier = Modifier.align(Alignment.TopEnd),
                        accent = model.upNextBadgeAccent
                    )
                } else if (model.progressLabel != null) {
                    CardCornerBadge(
                        text = model.progressLabel,
                        modifier = Modifier.align(Alignment.TopEnd)
                    )
                }
            }
            if (model.showTitleBar) {
                Column(modifier = Modifier.padding(top = 3.dp)) {
                    Text(
                        text = model.title,
                        color = Color.White,
                        fontSize = FluxaDimensions.CardText.titleSize,
                        fontWeight = FontWeight.SemiBold,
                        lineHeight = 13.sp,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis
                    )
                    if (model.subtitle.isNotBlank()) {
                        Text(
                            text = model.subtitle,
                            color = Color.White.copy(alpha = FluxaDimensions.Alpha.cardSubtitle),
                            fontSize = FluxaDimensions.CardText.subtitleSize,
                            fontWeight = FontWeight.Medium,
                            lineHeight = 11.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                }
            }
        }
    }
}

private val CONTINUE_WATCHING_LABEL_GRADIENT = androidx.compose.ui.graphics.Brush.verticalGradient(
    colors = listOf(
        Color.Transparent,
        Color.Black.copy(alpha = 0.18f),
        Color.Black.copy(alpha = 0.82f)
    ),
    startY = 45f
)

private val MOBILE_CARD_BOTTOM_GRADIENT = androidx.compose.ui.graphics.Brush.verticalGradient(
    colors = listOf(Color.Transparent, Color.Black.copy(alpha = 0.6f)),
    startY = 150f
)

@Composable
private fun CardCornerBadge(text: String, modifier: Modifier = Modifier, accent: Boolean = false) {
    Text(
        text = text,
        color = Color.White,
        fontSize = 9.sp,
        fontWeight = FontWeight.SemiBold,
        letterSpacing = 0.2.sp,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier
            .padding(6.dp)
            .clip(RoundedCornerShape(5.dp))
            .background(
                if (accent) FluxaColors.progressFill else Color.Black.copy(alpha = FluxaDimensions.Alpha.upNextBadge)
            )
            .padding(horizontal = 6.dp, vertical = 3.dp)
    )
}
