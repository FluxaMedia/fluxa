@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)

package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.MaterialTheme
import androidx.tv.material3.Surface
import coil3.compose.AsyncImage
import com.fluxa.app.R

@Composable
internal fun TvProfileSelectionScene(
    profiles: List<UserProfile>,
    lang: String,
    onProfileSelected: (UserProfile) -> Unit,
    onAddProfileClick: () -> Unit
) {
    val focusRequesters = remember(profiles.size) {
        List(profiles.size + 1) { FocusRequester() }
    }

    LaunchedEffect(profiles.size) {
        if (focusRequesters.isNotEmpty()) {
            runCatching { focusRequesters.first().requestFocus() }
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 72.dp, vertical = 48.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        TvBrandHeader()
        Spacer(modifier = Modifier.height(38.dp))
        androidx.tv.material3.Text(
            text = AppStrings.t(lang, "profiles.who_watching"),
            style = MaterialTheme.typography.displayMedium,
            color = Color.White,
            fontWeight = FontWeight.Bold
        )
        Spacer(modifier = Modifier.height(10.dp))
        androidx.tv.material3.Text(
            text = AppStrings.t(lang, "profiles.select_profile_hint"),
            style = MaterialTheme.typography.titleMedium,
            color = Color.White.copy(alpha = 0.78f),
            textAlign = TextAlign.Center
        )
        Spacer(modifier = Modifier.height(56.dp))

        Row(
            horizontalArrangement = Arrangement.spacedBy(40.dp),
            verticalAlignment = Alignment.Top
        ) {
            profiles.forEachIndexed { index, profile ->
                TvProfileOrb(
                    profile = profile,
                    modifier = Modifier.focusRequester(focusRequesters[index]),
                    onClick = { onProfileSelected(profile) }
                )
            }
            TvAddProfileOrb(
                lang = lang,
                modifier = Modifier.focusRequester(focusRequesters.last()),
                onClick = onAddProfileClick
            )
        }

        Spacer(modifier = Modifier.height(52.dp))
        androidx.tv.material3.Text(
            text = AppStrings.t(lang, "profiles.remote_hint"),
            color = Color.White.copy(alpha = 0.64f),
            style = MaterialTheme.typography.bodyLarge
        )
    }
}

@Composable
private fun TvBrandHeader() {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        Box(
            modifier = Modifier
                .size(42.dp)
                .clip(RoundedCornerShape(14.dp))
                .background(
                    Brush.linearGradient(
                        colors = listOf(
                            Color(0xFF7F5FFF),
                            Color(0xFF2AB7FF)
                        )
                    )
                ),
            contentAlignment = Alignment.Center
        ) {
            androidx.tv.material3.Icon(
                imageVector = FluxaIcons.PlayCircleFilled,
                contentDescription = null,
                tint = Color.White,
                modifier = Modifier.size(26.dp)
            )
        }
        androidx.tv.material3.Text(
            text = stringResource(R.string.app_name),
            color = Color.White,
            fontSize = 28.sp,
            fontWeight = FontWeight.Black
        )
    }
}

@Composable
internal fun TvProfileOrb(
    profile: UserProfile,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    var isFocused by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isFocused) 1.08f else 1f, tween(180), label = "profileScale")
    val glowAlpha by animateFloatAsState(if (isFocused) 1f else 0.28f, tween(180), label = "profileGlow")

    Column(
        modifier = modifier
            .width(180.dp)
            .scale(scale)
            .onFocusChanged { isFocused = it.isFocused }
            .clickable { onClick() },
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Box(contentAlignment = Alignment.Center) {
            Box(
                modifier = Modifier
                    .size(174.dp)
                    .clip(CircleShape)
                    .background(Color.White.copy(alpha = 0.08f * glowAlpha))
            )
            Box(
                modifier = Modifier
                    .size(146.dp)
                    .clip(CircleShape)
                    .background(Color(profile.colorArgb).copy(alpha = 0.95f))
                    .border(
                        width = if (isFocused) 4.dp else 2.dp,
                        color = if (isFocused) Color.White else Color.White.copy(alpha = 0.28f),
                        shape = CircleShape
                    ),
                contentAlignment = Alignment.Center
            ) {
                if (!profile.avatarUrl.isNullOrBlank()) {
                    AsyncImage(
                        model = profile.avatarUrl,
                        contentDescription = null,
                        modifier = Modifier
                            .fillMaxSize()
                            .clip(CircleShape),
                        contentScale = ContentScale.Crop
                    )
                } else {
                    androidx.tv.material3.Text(
                        text = profile.displayName.take(1).uppercase(),
                        color = Color.White,
                        fontSize = 58.sp,
                        fontWeight = FontWeight.Black
                    )
                }
            }

            if (!profile.pin.isNullOrBlank()) {
                Box(
                    modifier = Modifier
                        .align(Alignment.BottomEnd)
                        .offset(x = (-6).dp, y = (-10).dp)
                        .size(34.dp)
                        .clip(CircleShape)
                        .background(Color(0xFFFFC62B)),
                    contentAlignment = Alignment.Center
                ) {
                    androidx.tv.material3.Icon(
                        imageVector = FluxaIcons.Lock,
                        contentDescription = null,
                        tint = Color(0xFF422A00),
                        modifier = Modifier.size(18.dp)
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(16.dp))
        androidx.tv.material3.Text(
            text = profile.displayName,
            color = Color.White,
            fontSize = 26.sp,
            fontWeight = if (isFocused) FontWeight.Bold else FontWeight.Medium
        )
        Spacer(modifier = Modifier.height(6.dp))
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            if (!profile.isGuest) {
                androidx.tv.material3.Icon(
                    imageVector = FluxaIcons.Star,
                    contentDescription = null,
                    tint = Color(0xFFFFD95B),
                    modifier = Modifier.size(14.dp)
                )
            }
            androidx.tv.material3.Text(
                text = profileRoleLabel(profile),
                color = roleColor(profile),
                fontSize = 14.sp,
                fontWeight = FontWeight.Bold
            )
        }
    }
}

@Composable
internal fun TvAddProfileOrb(
    lang: String = "en",
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    var isFocused by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isFocused) 1.08f else 1f, tween(180), label = "addProfileScale")

    Column(
        modifier = modifier
            .width(180.dp)
            .scale(scale)
            .onFocusChanged { isFocused = it.isFocused }
            .clickable { onClick() },
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Box(
            modifier = Modifier
                .size(146.dp)
                .clip(CircleShape)
                .background(Color(0xFF785061).copy(alpha = if (isFocused) 0.74f else 0.48f))
                .border(
                    width = if (isFocused) 3.dp else 1.dp,
                    color = if (isFocused) Color.White else Color.White.copy(alpha = 0.14f),
                    shape = CircleShape
                ),
            contentAlignment = Alignment.Center
        ) {
            androidx.tv.material3.Icon(
                imageVector = FluxaIcons.Add,
                contentDescription = null,
                tint = Color.White.copy(alpha = 0.92f),
                modifier = Modifier.size(42.dp)
            )
        }
        Spacer(modifier = Modifier.height(16.dp))
        androidx.tv.material3.Text(
            text = AppStrings.t(lang, "profiles.new_profile"),
            color = Color.White.copy(alpha = if (isFocused) 1f else 0.8f),
            fontSize = 24.sp,
            fontWeight = FontWeight.Medium
        )
    }
}
