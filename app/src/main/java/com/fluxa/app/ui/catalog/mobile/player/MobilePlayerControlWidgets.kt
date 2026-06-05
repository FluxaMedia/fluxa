@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
internal fun PlayerTopIconButton(icon: ImageVector, onClick: () -> Unit, modifier: Modifier = Modifier) {
    Box(
        modifier = modifier
            .size(34.dp)
            .clip(CircleShape)
            .background(Color.Black.copy(alpha = 0.38f))
            .border(1.dp, Color.White.copy(alpha = 0.10f), CircleShape)
            .clickable { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Icon(icon, null, tint = Color.White, modifier = Modifier.size(18.dp))
    }
}

@Composable
internal fun MobileTransportButton(icon: ImageVector, enabled: Boolean, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .size(48.dp)
            .clip(CircleShape)
            .background(Color.Black.copy(alpha = if (enabled) 0.40f else 0.18f))
            .border(1.dp, Color.White.copy(alpha = if (enabled) 0.10f else 0.04f), CircleShape)
            .clickable(enabled = enabled) { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Icon(
            icon,
            null,
            tint = Color.White.copy(alpha = if (enabled) 0.92f else 0.26f),
            modifier = Modifier.size(24.dp)
        )
    }
}

@Composable
internal fun MobileBottomAction(
    icon: ImageVector,
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Row(
        modifier = modifier
            .clip(RoundedCornerShape(16.dp))
            .clickable { onClick() }
            .padding(horizontal = 4.dp, vertical = 6.dp),
        horizontalArrangement = androidx.compose.foundation.layout.Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = Color.White,
            modifier = Modifier.size(24.dp)
        )
        Spacer(modifier = Modifier.width(6.dp))
        Text(
            text = label,
            color = Color.White,
            fontSize = 13.sp,
            fontWeight = FontWeight.Medium,
            textAlign = TextAlign.Start,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis
        )
    }
}

@Composable
internal fun MobilePlayerSeekbar(
    position: Long,
    duration: Long,
    bufferedFraction: Float,
    onSeek: (Long) -> Unit,
    accentColor: Color = Color(0xFFE53935),
    onScrubbingChange: (Boolean, Long) -> Unit = { _, _ -> },
    seekPreviewBitmap: ImageBitmap? = null
) {
    var sliderPosition by remember(duration) { mutableFloatStateOf(position.toFloat()) }
    var isDragging by remember { mutableStateOf(false) }

    LaunchedEffect(position, duration) {
        if (!isDragging) {
            sliderPosition = position.coerceIn(0L, duration.coerceAtLeast(0L)).toFloat()
        }
    }

    BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
        if (isDragging && seekPreviewBitmap != null) {
            val fraction = if (duration > 0L) (sliderPosition / duration.toFloat()).coerceIn(0f, 1f) else 0f
            val cardWidth = 160.dp
            val rawLeft = maxWidth * fraction - cardWidth / 2
            val clampedLeft = rawLeft.coerceIn(0.dp, maxOf(0.dp, maxWidth - cardWidth))

            Column(
                modifier = Modifier
                    .offset(x = clampedLeft, y = (-116).dp)
                    .width(cardWidth)
                    .clip(RoundedCornerShape(6.dp))
                    .background(Color.Black.copy(alpha = 0.82f))
                    .border(1.dp, Color.White.copy(alpha = 0.18f), RoundedCornerShape(6.dp)),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Image(
                    bitmap = seekPreviewBitmap,
                    contentDescription = null,
                    modifier = Modifier
                        .fillMaxWidth()
                        .aspectRatio(16f / 9f),
                    contentScale = ContentScale.Crop
                )
                Text(
                    text = formatTime(sliderPosition.toLong()),
                    color = Color.White,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Medium,
                    modifier = Modifier.padding(vertical = 3.dp)
                )
            }
        }

        Slider(
            value = sliderPosition,
            onValueChange = {
                if (!isDragging) {
                    isDragging = true
                }
                sliderPosition = it
                onScrubbingChange(true, it.toLong())
            },
            onValueChangeFinished = {
                isDragging = false
                onScrubbingChange(false, sliderPosition.toLong())
                onSeek(sliderPosition.toLong())
            },
            valueRange = 0f..duration.coerceAtLeast(1L).toFloat(),
            colors = SliderDefaults.colors(
                thumbColor = accentColor,
                activeTrackColor = accentColor,
                inactiveTrackColor = Color.Black
            ),
            modifier = Modifier.fillMaxWidth(),
            thumb = {
                Box(
                    modifier = Modifier
                        .size(18.dp)
                        .clip(CircleShape)
                        .background(accentColor)
                )
            },
            track = { sliderState ->
                Canvas(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(4.dp)
                ) {
                    val trackHeight = 4.dp.toPx()
                    val radius = androidx.compose.ui.geometry.CornerRadius(trackHeight / 2f)
                    val activeFraction = if (duration > 0L) {
                        (sliderState.value / duration.toFloat()).coerceIn(0f, 1f)
                    } else {
                        0f
                    }
                    val visibleBufferedFraction = bufferedFraction.coerceIn(0f, 1f).coerceAtLeast(activeFraction)
                    drawRoundRect(
                        color = Color.Black,
                        size = androidx.compose.ui.geometry.Size(size.width, trackHeight),
                        cornerRadius = radius
                    )
                    drawRoundRect(
                        color = Color(0xFFBDBDBD),
                        size = androidx.compose.ui.geometry.Size(size.width * visibleBufferedFraction, trackHeight),
                        cornerRadius = radius
                    )
                    drawRoundRect(
                        color = accentColor,
                        size = androidx.compose.ui.geometry.Size(size.width * activeFraction, trackHeight),
                        cornerRadius = radius
                    )
                }
            }
        )
    }
}
