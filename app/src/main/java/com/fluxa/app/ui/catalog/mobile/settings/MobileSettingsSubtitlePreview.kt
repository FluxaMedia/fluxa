@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.annotation.DrawableRes
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.graphics.drawscope.Stroke
import coil3.compose.AsyncImage
import java.util.Locale

@Composable
internal fun MobileSubtitlePreview(profile: UserProfile, lang: String) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(164.dp)
            .clip(RoundedCornerShape(16.dp))
            .background(
                Brush.linearGradient(
                    colors = listOf(
                        Color(0xFF121212),
                        Color(0xFF3C3125),
                        Color(0xFF8792A0),
                        Color(0xFF0D0D0D)
                    )
                )
            )
            .border(1.dp, Color.White.copy(alpha = 0.08f), RoundedCornerShape(16.dp)),
    ) {
        Box(
            modifier = Modifier
                .matchParentSize()
                .background(
                    Brush.radialGradient(
                        colors = listOf(
                            Color.White.copy(alpha = 0.26f),
                            Color.Transparent
                        ),
                        center = Offset(Float.POSITIVE_INFINITY, 0f),
                        radius = 520f
                    )
                )
        )
        Column(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .padding(horizontal = 18.dp, vertical = 16.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            SubtitlePreviewText(
                text = AppStrings.t(lang, "auto.subtitles_looks_like_this"),
                profile = profile
            )
        }
    }
}

@Composable
internal fun SubtitlePreviewText(text: String, profile: UserProfile) {
    val textAlpha = profile.safeSubtitleTextOpacity.coerceIn(0f, 1f)
    val backgroundAlpha = profile.safeSubtitleBackgroundOpacity.coerceIn(0f, 1f)
    val outlineAlpha = profile.safeSubtitleOutlineOpacity.coerceIn(0f, 1f)
    val fontSize = profile.safeSubtitleSize.sp
    val lineHeight = (profile.safeSubtitleSize + 4).sp
    val outlineWidth = (profile.safeSubtitleSize / 7f).coerceIn(2f, 5f)

    Box(contentAlignment = Alignment.Center) {
        if (backgroundAlpha > 0f) {
            Text(
                text = text,
                color = Color.Transparent,
                fontSize = fontSize,
                fontWeight = FontWeight.Bold,
                lineHeight = lineHeight,
                style = TextStyle(
                    background = Color(profile.safeSubtitleBackgroundColor).copy(alpha = backgroundAlpha)
                )
            )
        }
        if (outlineAlpha > 0f) {
            Text(
                text = text,
                color = Color(profile.safeSubtitleOutlineColor).copy(alpha = outlineAlpha),
                fontSize = fontSize,
                fontWeight = FontWeight.Bold,
                lineHeight = lineHeight,
                style = TextStyle(drawStyle = Stroke(width = outlineWidth))
            )
        }
        Text(
            text = text,
            color = Color(profile.safeSubtitleColor).copy(alpha = textAlpha),
            fontSize = fontSize,
            fontWeight = FontWeight.Bold,
            lineHeight = lineHeight
        )
    }
}
