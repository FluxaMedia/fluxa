package com.fluxa.app.ui.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import coil3.compose.LocalPlatformContext
import coil3.request.ImageRequest
import coil3.request.crossfade

@Composable
fun CatalogCard(
    model: CatalogCardUiModel,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    onLongClick: (() -> Unit)? = null
) {
    val context = LocalPlatformContext.current
    val density = LocalDensity.current

    val artworkRequest = remember(context, model.artworkUrl, model.requestWidthPx, model.requestHeightPx) {
        ImageRequest.Builder(context)
            .data(model.artworkUrl)
            .crossfade(false)
            .memoryCacheKey(model.artworkMemoryCacheKey)
            .diskCacheKey(model.artworkDiskCacheKey)
            .size(model.requestWidthPx, model.requestHeightPx)
            .build()
    }
    var failed by remember(model.artworkUrl) { mutableStateOf(model.artworkUrl.isNullOrBlank()) }

    val logoRequest = remember(model.logoUrl, model.showLogo) {
        if (!model.showLogo || model.logoUrl == null) null else ImageRequest.Builder(context)
            .data(model.logoUrl)
            .crossfade(false)
            .memoryCacheKey(model.logoMemoryCacheKey)
            .diskCacheKey(model.logoUrl)
            .build()
    }

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
            .height(model.imageHeight + if (model.showTitleBar) FluxaDimensions.cardMetaBarHeight else 0.dp)
            .combinedClickable(
                interactionSource = null,
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
                if (model.loadArtwork && !failed) {
                    AsyncImage(
                        model = artworkRequest,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop,
                        onError = { failed = true }
                    )
                } else if (model.allowCoverFallback) {
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
                if (model.loadArtwork && model.showLogo && logoRequest != null) {
                    Box(modifier = Modifier.fillMaxSize().background(MOBILE_CARD_BOTTOM_GRADIENT))
                    AsyncImage(
                        model = logoRequest,
                        contentDescription = null,
                        modifier = Modifier
                            .align(Alignment.BottomStart)
                            .padding(start = 9.dp, bottom = 9.dp)
                            .widthIn(max = model.width * 0.42f)
                            .heightIn(max = model.imageHeight * 0.30f),
                        contentScale = ContentScale.Fit
                    )
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
                    Text(
                        text = model.upNextLabel,
                        color = Color.White,
                        fontSize = FluxaDimensions.CardText.subtitleSize,
                        fontWeight = FontWeight.Black,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier
                            .align(Alignment.TopEnd)
                            .padding(8.dp)
                            .background(Color.Black.copy(alpha = FluxaDimensions.Alpha.upNextBadge))
                            .padding(horizontal = 8.dp, vertical = 5.dp)
                    )
                }
            }
            if (model.showTitleBar) {
                Text(
                    text = model.title,
                    color = Color.White,
                    fontSize = FluxaDimensions.CardText.titleSize,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 6.dp)
                )
                if (model.subtitle.isNotBlank()) {
                    Text(
                        text = model.subtitle,
                        color = Color.White.copy(alpha = FluxaDimensions.Alpha.cardSubtitle),
                        fontSize = FluxaDimensions.CardText.subtitleSize,
                        fontWeight = FontWeight.Medium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.padding(top = 1.dp)
                    )
                }
            }
        }
    }
}

private val MOBILE_CARD_BOTTOM_GRADIENT = androidx.compose.ui.graphics.Brush.verticalGradient(
    colors = listOf(Color.Transparent, Color.Black.copy(alpha = 0.6f)),
    startY = 150f
)
