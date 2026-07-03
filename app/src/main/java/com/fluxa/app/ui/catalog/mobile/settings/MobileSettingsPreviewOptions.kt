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
import coil3.request.crossfade
import java.util.Locale

@Composable
internal fun MobileContinueWatchingPreview(
    profile: UserProfile,
    lang: String,
    onSelect: (String) -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    val isHorizontal = profile.safeContinueWatchingLayout != "vertical"
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp, vertical = 14.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        ContinueWatchingPreviewCard(
            title = verticalPosterLabel(lang),
            selected = !isHorizontal,
            horizontal = false,
            colors = colors,
            onClick = { onSelect("vertical") }
        )
        Spacer(Modifier.width(14.dp))
        ContinueWatchingPreviewCard(
            title = horizontalPosterLabel(lang),
            selected = isHorizontal,
            horizontal = true,
            colors = colors,
            onClick = { onSelect("horizontal") }
        )
    }
}

@Composable
internal fun ContinueWatchingPreviewCard(
    title: String,
    selected: Boolean,
    horizontal: Boolean,
    colors: MobileSettingsPalette,
    onClick: () -> Unit
) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Box(
            modifier = Modifier
                .size(if (horizontal) 116.dp else 70.dp, if (horizontal) 66.dp else 104.dp)
                .clip(RoundedCornerShape(12.dp))
                .background(Brush.linearGradient(listOf(Color(0xFF202630), Color(0xFF10141A))))
                .clickable { onClick() }
                .border(
                    1.dp,
                    if (selected) colors.accent.copy(alpha = 0.9f) else colors.text.copy(alpha = 0.16f),
                    RoundedCornerShape(12.dp)
                )
                .padding(8.dp),
            contentAlignment = Alignment.BottomStart
        ) {
            Column {
                Box(
                    modifier = Modifier
                        .fillMaxWidth(if (horizontal) 0.7f else 0.82f)
                        .height(5.dp)
                        .clip(CircleShape)
                        .background(Color.White.copy(alpha = 0.72f))
                )
                Spacer(Modifier.height(5.dp))
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(4.dp)
                        .clip(CircleShape)
                        .background(Color.White.copy(alpha = 0.18f))
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth(0.58f)
                            .fillMaxHeight()
                            .clip(CircleShape)
                            .background(colors.accent)
                    )
                }
            }
        }
        Spacer(Modifier.height(7.dp))
        Text(
            title,
            color = if (selected) colors.text else colors.mutedText,
            fontSize = 10.sp,
            fontWeight = FontWeight.Black
        )
    }
}

@Composable
internal fun MobilePosterPreview(profile: UserProfile, lang: String, meta: Meta?) {
    val colors = LocalMobileSettingsPalette.current
    val duration = when {
        !profile.safeAnimationsEnabled -> 0
        else -> 260
    }
    val landscape = profile.safePosterLandscapeMode
    val width by animateDpAsState(
        targetValue = mobilePosterWidth(profile.safePosterWidthPreset, landscape),
        animationSpec = tween(duration),
        label = "posterPreviewWidth"
    )
    val height by animateDpAsState(
        targetValue = mobilePosterHeight(profile.safePosterWidthPreset, landscape),
        animationSpec = tween(duration),
        label = "posterPreviewHeight"
    )
    val radius by animateDpAsState(
        targetValue = mobileCornerRadius(profile.safeCardCornerPreset),
        animationSpec = tween(duration),
        label = "posterPreviewRadius"
    )
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp, vertical = 14.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Box(
            modifier = Modifier
                .width(width)
                .height(height)
                .clip(RoundedCornerShape(radius))
                .background(Color.White.copy(alpha = 0.06f))
                .border(1.dp, colors.accent.copy(alpha = 0.38f), RoundedCornerShape(radius))
        ) {
            val targetUrl = remember(meta?.id, meta?.poster, meta?.background, landscape) {
                if (landscape && meta != null) horizontalArtworkCandidates(meta).firstOrNull() else meta?.poster
            }
            var displayUrl by remember { mutableStateOf(targetUrl) }
            LaunchedEffect(targetUrl) {
                if (targetUrl != null) displayUrl = targetUrl
            }
            val context = androidx.compose.ui.platform.LocalContext.current
            if (!displayUrl.isNullOrBlank()) {
                AsyncImage(
                    model = coil3.request.ImageRequest.Builder(context)
                        .data(displayUrl)
                        .crossfade(if (profile.safeAnimationsEnabled) 280 else 0)
                        .build(),
                    contentDescription = meta?.name,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop,
                    onError = { displayUrl = null }
                )
            } else {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(
                            Brush.linearGradient(
                                listOf(Color(0xFF2B3854), Color(0xFF7C3F54), Color(0xFF171B24))
                            )
                        )
                )
            }
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(
                        Brush.verticalGradient(
                            listOf(Color.Transparent, Color.Black.copy(alpha = if (profile.safePosterHideTitles) 0.12f else 0.76f))
                        )
                    )
            )
        }
        if (!profile.safePosterHideTitles) {
            Spacer(Modifier.height(8.dp))
            Text(
                text = meta?.name ?: AppStrings.t(lang, "auto.content_title"),
                color = colors.text.copy(alpha = 0.92f),
                fontSize = 12.sp,
                fontWeight = FontWeight.Black,
                maxLines = 1,
                overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis,
                modifier = Modifier.width(width)
            )
            Text(
                text = meta?.releaseInfo?.take(4) ?: meta?.released?.take(4) ?: "2026",
                color = colors.text.copy(alpha = 0.42f),
                fontSize = 10.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.width(width)
            )
        }
    }
}

@Composable
internal fun MobileEpisodeLayoutPreview(
    profile: UserProfile,
    lang: String,
    onSelect: (String) -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    val isHorizontal = profile.safeEpisodeCardsLayout == "horizontal"
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp, vertical = 14.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        EpisodeLayoutPreviewCard(
            title = AppStrings.t(lang, "settings.episode_layout_list"),
            selected = !isHorizontal,
            horizontal = false,
            colors = colors,
            onClick = { onSelect("list") }
        )
        Spacer(Modifier.width(14.dp))
        EpisodeLayoutPreviewCard(
            title = AppStrings.t(lang, "settings.episode_layout_horizontal"),
            selected = isHorizontal,
            horizontal = true,
            colors = colors,
            onClick = { onSelect("horizontal") }
        )
    }
}

@Composable
private fun EpisodeLayoutPreviewCard(
    title: String,
    selected: Boolean,
    horizontal: Boolean,
    colors: MobileSettingsPalette,
    onClick: () -> Unit
) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Box(
            modifier = Modifier
                .size(if (horizontal) 148.dp else 160.dp, if (horizontal) 106.dp else 110.dp)
                .clip(RoundedCornerShape(12.dp))
                .background(Brush.linearGradient(listOf(Color(0xFF202630), Color(0xFF10141A))))
                .clickable { onClick() }
                .border(
                    1.dp,
                    if (selected) colors.accent.copy(alpha = 0.9f) else colors.text.copy(alpha = 0.16f),
                    RoundedCornerShape(12.dp)
                )
                .padding(8.dp)
        ) {
            if (horizontal) {
                Column(
                    verticalArrangement = Arrangement.spacedBy(6.dp),
                    modifier = Modifier.fillMaxSize()
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .weight(1f)
                            .clip(RoundedCornerShape(6.dp))
                            .background(Color.White.copy(alpha = 0.14f))
                    )
                    Box(
                        modifier = Modifier
                            .fillMaxWidth(0.88f)
                            .height(4.dp)
                            .clip(RoundedCornerShape(2.dp))
                            .background(Color.White.copy(alpha = 0.55f))
                    )
                    Box(
                        modifier = Modifier
                            .fillMaxWidth(0.65f)
                            .height(3.dp)
                            .clip(RoundedCornerShape(2.dp))
                            .background(Color.White.copy(alpha = 0.22f))
                    )
                }
            } else {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.fillMaxSize()
                ) {
                    Box(
                        modifier = Modifier
                            .width(52.dp)
                            .fillMaxHeight()
                            .clip(RoundedCornerShape(6.dp))
                            .background(Color.White.copy(alpha = 0.14f))
                    )
                    Column(
                        verticalArrangement = Arrangement.spacedBy(5.dp),
                        modifier = Modifier.weight(1f)
                    ) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth(0.85f)
                                .height(4.dp)
                                .clip(RoundedCornerShape(2.dp))
                                .background(Color.White.copy(alpha = 0.55f))
                        )
                        Box(
                            modifier = Modifier
                                .fillMaxWidth(0.6f)
                                .height(3.dp)
                                .clip(RoundedCornerShape(2.dp))
                                .background(Color.White.copy(alpha = 0.22f))
                        )
                    }
                }
            }
        }
        Spacer(Modifier.height(7.dp))
        Text(
            title,
            color = if (selected) colors.text else colors.mutedText,
            fontSize = 10.sp,
            fontWeight = FontWeight.Black
        )
    }
}
