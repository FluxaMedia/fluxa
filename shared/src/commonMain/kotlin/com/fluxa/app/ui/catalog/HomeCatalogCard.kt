package com.fluxa.app.ui.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.shared.image.FluxaRemoteImage

@Composable
fun HomeCatalogCard(
    artwork: String?,
    width: Dp,
    imageHeight: Dp,
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
                FluxaRemoteImage(
                    imageUrl = artwork,
                    cacheKey = "home-catalog:$artwork",
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
                        .padding(4.dp)
                        .background(Color.Black.copy(alpha = 0.72f))
                        .padding(horizontal = 6.dp, vertical = 3.dp)
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
