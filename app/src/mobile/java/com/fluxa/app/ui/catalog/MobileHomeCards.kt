@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)

package com.fluxa.app.ui.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import coil3.request.crossfade
import com.fluxa.app.data.remote.Meta

internal fun collectionFolderCardImageHeight(
    fallbackLayout: String,
    widthPreset: String
): androidx.compose.ui.unit.Dp {
    return when (fallbackLayout) {
        "horizontal" -> horizontalCardHeight(widthPreset, DeviceType.Mobile)
        "square" -> posterCardWidth(widthPreset)
        else -> posterCardHeight(widthPreset)
    }
}

internal fun collectionFolderTitleSlotHeight(hidePosterTitles: Boolean): androidx.compose.ui.unit.Dp {
    return if (hidePosterTitles) 0.dp else 30.dp
}

internal fun collectionFolderLayout(reason: String?, fallbackLayout: String): String {
    return when (reason) {
        "wide" -> "horizontal"
        "square" -> "square"
        "poster" -> "vertical"
        else -> fallbackLayout
    }
}

internal const val HOME_CATALOG_LOAD_MORE_THRESHOLD = 5

@Composable
internal fun HomeCatalogCard(
    artwork: String?,
    width: androidx.compose.ui.unit.Dp,
    imageHeight: androidx.compose.ui.unit.Dp,
    showTitle: Boolean,
    title: String,
    secondary: String,
    showProgressBar: Boolean,
    progress: Float,
    isUpNext: Boolean,
    upNextLabel: String,
    topTenRank: Int?,
    coverEmoji: String?,
    name: String,
    onClick: () -> Unit,
    onLongClick: (() -> Unit)?
) {
    val context = LocalContext.current
    val request = remember(context, artwork) {
        ImageRequest.Builder(context)
            .data(artwork)
            .crossfade(false)
            .memoryCacheKey(artwork?.let { "home-catalog:$it" })
            .diskCacheKey(artwork)
            .build()
    }
    val clickModifier = if (onLongClick != null) {
        Modifier.combinedClickable(interactionSource = null, indication = null, onClick = onClick, onLongClick = onLongClick)
    } else {
        Modifier.clickable(interactionSource = null, indication = null, onClick = onClick)
    }
    Column(
        modifier = Modifier
            .width(width)
            .height(imageHeight + if (showTitle) 42.dp else 0.dp)
            .then(clickModifier)
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(imageHeight)
                .background(Color.White.copy(alpha = 0.05f)),
            contentAlignment = Alignment.Center
        ) {
            if (!artwork.isNullOrBlank()) {
                AsyncImage(
                    model = request,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop
                )
            } else {
                androidx.compose.material3.Text(
                    text = coverEmoji?.takeIf { it.isNotBlank() } ?: name.take(1).uppercase(),
                    color = Color.White.copy(alpha = if (coverEmoji.isNullOrBlank()) 0.2f else 0.82f),
                    fontSize = if (coverEmoji.isNullOrBlank()) 42.sp else 36.sp,
                    fontWeight = FontWeight.Black
                )
            }
            if (showProgressBar) {
                Box(
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .padding(horizontal = 8.dp, vertical = 6.dp)
                        .height(4.dp)
                        .clip(RoundedCornerShape(999.dp))
                        .background(Color.White.copy(alpha = 0.2f))
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth(progress)
                            .fillMaxHeight()
                            .background(FluxaColors.progressFill)
                    )
                }
            }
            if (isUpNext) {
                androidx.compose.material3.Text(
                    text = upNextLabel,
                    color = Color.White,
                    fontSize = 10.sp,
                    fontWeight = FontWeight.Black,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .padding(8.dp)
                        .background(Color.Black.copy(alpha = 0.72f))
                        .padding(horizontal = 8.dp, vertical = 5.dp)
                )
            }
            if (topTenRank != null) {
                androidx.compose.material3.Text(
                    text = topTenRank.toString(),
                    color = Color.White,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Black,
                    modifier = Modifier
                        .align(Alignment.TopStart)
                        .padding(7.dp)
                        .background(Color.Black.copy(alpha = 0.66f), RoundedCornerShape(6.dp))
                        .padding(horizontal = 7.dp, vertical = 3.dp)
                )
            }
        }
        if (showTitle) {
            androidx.compose.material3.Text(
                text = title,
                color = Color.White,
                fontSize = 12.sp,
                lineHeight = 13.sp,
                fontWeight = FontWeight.Black,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = 4.dp)
            )
            if (secondary.isNotBlank()) {
                androidx.compose.material3.Text(
                    text = secondary,
                    color = Color.White.copy(alpha = 0.58f),
                    fontSize = 10.sp,
                    lineHeight = 11.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 0.dp)
                )
            }
        }
    }
}

@Composable
internal fun MobileCollectionFolderCard(
    title: String,
    poster: String?,
    coverEmoji: String?,
    hideTitle: Boolean,
    reason: String?,
    hideTitlesByProfile: Boolean,
    fallbackLayout: String,
    widthPreset: String,
    titleSlotHeight: androidx.compose.ui.unit.Dp,
    onClick: () -> Unit
) {
    val context = LocalContext.current
    val layout = collectionFolderLayout(reason, fallbackLayout)
    val width = when (layout) {
        "horizontal" -> horizontalCardWidth(widthPreset, DeviceType.Mobile)
        else -> posterCardWidth(widthPreset)
    }
    val imageHeight = when (layout) {
        "horizontal" -> horizontalCardHeight(widthPreset, DeviceType.Mobile)
        "square" -> posterCardWidth(widthPreset)
        else -> posterCardHeight(widthPreset)
    }
    val displayPoster = poster?.takeIf { it.isNotBlank() }
    val request = remember(context, displayPoster) {
        ImageRequest.Builder(context)
            .data(displayPoster)
            .crossfade(false)
            .memoryCacheKey(displayPoster?.let { "home-collection:$it" })
            .diskCacheKey(displayPoster)
            .build()
    }
    val showTitle = !(hideTitlesByProfile || hideTitle)
    val totalHeight = imageHeight + titleSlotHeight

    Column(
        modifier = Modifier
            .width(width)
            .height(totalHeight)
            .clickable(
                interactionSource = null,
                indication = null,
                onClick = onClick
            ),
        verticalArrangement = Arrangement.spacedBy(4.dp)
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(imageHeight)
                .background(Color.White.copy(alpha = 0.05f)),
            contentAlignment = Alignment.Center
        ) {
            if (displayPoster != null) {
                AsyncImage(
                    model = request,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop
                )
            } else {
                androidx.compose.material3.Text(
                    text = coverEmoji?.takeIf { it.isNotBlank() } ?: title.take(1).uppercase(),
                    color = Color.White.copy(alpha = if (coverEmoji.isNullOrBlank()) 0.2f else 0.82f),
                    fontSize = if (coverEmoji.isNullOrBlank()) 42.sp else 36.sp,
                    fontWeight = FontWeight.Black
                )
            }
        }
        if (showTitle) {
            androidx.compose.material3.Text(
                text = title,
                color = Color.White,
                fontSize = 12.sp,
                lineHeight = 15.sp,
                fontWeight = FontWeight.Black,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.height(titleSlotHeight - 4.dp)
            )
        }
    }
}
