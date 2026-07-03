@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
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
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.Border
import androidx.tv.material3.ClickableSurfaceDefaults
import androidx.tv.material3.Surface
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import coil3.request.crossfade

@Composable
internal fun TvPosterPreview(profile: UserProfile, lang: String, meta: Meta?) {
    val landscape = profile.safePosterLandscapeMode
    val width by animateDpAsState(
        targetValue = mobilePosterWidth(profile.safePosterWidthPreset, landscape),
        animationSpec = tween(FluxaDimensions.AnimDuration.contentExpand),
        label = "tvPosterPreviewWidth"
    )
    val height by animateDpAsState(
        targetValue = mobilePosterHeight(profile.safePosterWidthPreset, landscape),
        animationSpec = tween(FluxaDimensions.AnimDuration.contentExpand),
        label = "tvPosterPreviewHeight"
    )
    val radius by animateDpAsState(
        targetValue = mobileCornerRadius(profile.safeCardCornerPreset),
        animationSpec = tween(FluxaDimensions.AnimDuration.contentExpand),
        label = "tvPosterPreviewRadius"
    )
    Column(
        modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Box(
            modifier = Modifier
                .width(width)
                .height(height)
                .clip(RoundedCornerShape(radius))
                .background(Color.White.copy(alpha = 0.06f))
                .border(1.dp, Color.White.copy(alpha = 0.38f), RoundedCornerShape(radius))
        ) {
            val targetUrl = remember(meta?.id, meta?.poster, meta?.background, landscape) {
                if (landscape && meta != null) horizontalArtworkCandidates(meta).firstOrNull() else meta?.poster
            }
            var displayUrl by remember { mutableStateOf(targetUrl) }
            LaunchedEffect(targetUrl) {
                if (targetUrl != null) displayUrl = targetUrl
            }
            val context = LocalContext.current
            if (!displayUrl.isNullOrBlank()) {
                AsyncImage(
                    model = ImageRequest.Builder(context)
                        .data(displayUrl)
                        .crossfade(280)
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
                            Brush.linearGradient(listOf(Color(0xFF2B3854), Color(0xFF7C3F54), Color(0xFF171B24)))
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
                color = Color.White.copy(alpha = 0.92f),
                fontSize = 13.sp,
                fontWeight = FontWeight.Black,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.width(width)
            )
            Text(
                text = meta?.releaseInfo?.take(4) ?: meta?.released?.take(4) ?: "2026",
                color = Color.White.copy(alpha = 0.5f),
                fontSize = 11.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.width(width)
            )
        }
    }
}

@Composable
internal fun TvEpisodeLayoutPreview(profile: UserProfile, lang: String, onSelect: (String) -> Unit) {
    val isHorizontal = profile.safeEpisodeCardsLayout == "horizontal"
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        TvLayoutPreviewCard(
            title = AppStrings.t(lang, "settings.episode_layout_list"),
            selected = !isHorizontal,
            horizontal = false,
            onClick = { onSelect("list") }
        )
        Spacer(Modifier.width(20.dp))
        TvLayoutPreviewCard(
            title = AppStrings.t(lang, "settings.episode_layout_horizontal"),
            selected = isHorizontal,
            horizontal = true,
            onClick = { onSelect("horizontal") }
        )
    }
}

@Composable
private fun TvLayoutPreviewCard(
    title: String,
    selected: Boolean,
    horizontal: Boolean,
    onClick: () -> Unit
) {
    var focused by remember { mutableStateOf(false) }
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Surface(
            onClick = onClick,
            modifier = Modifier
                .size(if (horizontal) 168.dp else 140.dp, if (horizontal) 106.dp else 118.dp)
                .onFocusChanged { focused = it.isFocused },
            shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(14.dp)),
            colors = ClickableSurfaceDefaults.colors(
                containerColor = Color(0xFF171B24),
                contentColor = Color.White
            ),
            border = ClickableSurfaceDefaults.border(
                border = Border(
                    androidx.compose.foundation.BorderStroke(1.dp, if (selected) Color.White.copy(alpha = 0.75f) else Color.White.copy(alpha = 0.14f)),
                    shape = RoundedCornerShape(14.dp)
                ),
                focusedBorder = Border(
                    androidx.compose.foundation.BorderStroke(2.dp, Color.White),
                    shape = RoundedCornerShape(14.dp)
                )
            )
        ) {
            Box(modifier = Modifier.fillMaxSize().padding(10.dp)) {
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
                                .width(48.dp)
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
        }
        Spacer(Modifier.height(8.dp))
        Text(
            title,
            color = if (selected || focused) Color.White else Color.White.copy(alpha = 0.6f),
            fontSize = 12.sp,
            fontWeight = FontWeight.Black
        )
    }
}
