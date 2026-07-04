package com.fluxa.app.ui.catalog

import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

@Composable
fun rememberShimmerBrush(): Brush {
    val transition = rememberInfiniteTransition(label = "shimmer")
    val translate by transition.animateFloat(
        initialValue = 0f,
        targetValue = 1000f,
        animationSpec = infiniteRepeatable(
            animation = tween(durationMillis = 1100, easing = androidx.compose.animation.core.LinearEasing),
            repeatMode = RepeatMode.Restart
        ),
        label = "shimmerTranslate"
    )
    return Brush.linearGradient(
        colors = listOf(
            Color.White.copy(alpha = 0.045f),
            Color.White.copy(alpha = 0.10f),
            Color.White.copy(alpha = 0.045f)
        ),
        start = Offset(translate - 320f, translate - 320f),
        end = Offset(translate, translate)
    )
}

@Composable
fun SkeletonBox(modifier: Modifier, cornerRadius: Dp = 10.dp, brush: Brush = rememberShimmerBrush()) {
    Spacer(modifier = modifier.clip(RoundedCornerShape(cornerRadius)).background(brush))
}

@Composable
fun PosterCardSkeleton(width: Dp, imageHeight: Dp, showTitle: Boolean, brush: Brush = rememberShimmerBrush()) {
    Column(modifier = Modifier.width(width)) {
        SkeletonBox(
            modifier = Modifier.width(width).height(imageHeight),
            cornerRadius = FluxaDimensions.CornerPresets.soft,
            brush = brush
        )
        if (showTitle) {
            Spacer(Modifier.height(8.dp))
            SkeletonBox(Modifier.fillMaxWidth().height(11.dp), cornerRadius = 4.dp, brush = brush)
            Spacer(Modifier.height(5.dp))
            SkeletonBox(Modifier.width(width * 0.6f).height(9.dp), cornerRadius = 4.dp, brush = brush)
        }
    }
}

@Composable
fun PosterRowSkeleton(
    cardWidth: Dp,
    cardImageHeight: Dp,
    showTitle: Boolean = true,
    count: Int = 6,
    contentPadding: PaddingValues = PaddingValues(horizontal = 16.dp)
) {
    val brush = rememberShimmerBrush()
    LazyRow(
        contentPadding = contentPadding,
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        items(count = count) {
            PosterCardSkeleton(cardWidth, cardImageHeight, showTitle, brush)
        }
    }
}
