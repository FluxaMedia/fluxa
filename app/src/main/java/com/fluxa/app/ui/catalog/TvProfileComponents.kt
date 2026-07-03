@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)

package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
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
import androidx.compose.foundation.layout.fillMaxHeight
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
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
    onAddProfileClick: () -> Unit,
    onEditProfileClick: (UserProfile) -> Unit = {}
) {
    var editMode by remember { mutableStateOf(false) }
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
        Spacer(modifier = Modifier.weight(1f))
        androidx.tv.material3.Text(
            text = AppStrings.t(lang, if (editMode) "profiles.manage_profiles" else "profiles.who_watching"),
            style = MaterialTheme.typography.displayMedium,
            color = Color.White,
            fontWeight = FontWeight.Bold
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
                    editMode = editMode,
                    onClick = {
                        if (editMode) onEditProfileClick(profile) else onProfileSelected(profile)
                    }
                )
            }
            TvAddProfileOrb(
                lang = lang,
                modifier = Modifier.focusRequester(focusRequesters.last()),
                onClick = onAddProfileClick
            )
        }

        Spacer(modifier = Modifier.weight(1f))
        ManageProfilesButton(
            editMode = editMode,
            lang = lang,
            onClick = { editMode = !editMode }
        )
    }
}

@Composable
private fun ManageProfilesButton(editMode: Boolean, lang: String, onClick: () -> Unit) {
    var isFocused by remember { mutableStateOf(false) }
    Surface(
        onClick = onClick,
        modifier = Modifier
            .height(32.dp)
            .onFocusChanged { isFocused = it.isFocused },
        shape = androidx.tv.material3.ClickableSurfaceDefaults.shape(RoundedCornerShape(7.dp)),
        colors = androidx.tv.material3.ClickableSurfaceDefaults.colors(
            containerColor = if (isFocused) Color.White.copy(alpha = 0.16f) else Color.White.copy(alpha = 0.06f),
            contentColor = Color.White
        ),
        border = androidx.tv.material3.ClickableSurfaceDefaults.border(
            border = androidx.tv.material3.Border(
                androidx.compose.foundation.BorderStroke(1.dp, Color.White.copy(alpha = 0.14f)),
                shape = RoundedCornerShape(7.dp)
            )
        )
    ) {
        Box(modifier = Modifier.fillMaxHeight().padding(horizontal = 14.dp), contentAlignment = Alignment.Center) {
            androidx.tv.material3.Text(
                text = AppStrings.t(lang, if (editMode) "profiles.done" else "profiles.manage_profiles"),
                fontWeight = FontWeight.Medium,
                fontSize = 12.sp
            )
        }
    }
}

@Composable
internal fun TvProfileOrb(
    profile: UserProfile,
    modifier: Modifier = Modifier,
    editMode: Boolean = false,
    onClick: () -> Unit
) {
    var isFocused by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isFocused) 1.04f else 1f, tween(180), label = "profileScale")

    Column(
        modifier = modifier
            .width(180.dp)
            .scale(scale)
            .alpha(if (isFocused) 1f else 0.85f)
            .onFocusChanged { isFocused = it.isFocused }
            .clickable { onClick() },
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Box(contentAlignment = Alignment.Center) {
            Box(
                modifier = Modifier
                    .size(146.dp)
                    .clip(CircleShape)
                    .background(Color(profile.safeColorArgb).copy(alpha = 0.95f))
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
                    DefaultProfileAvatar(modifier = Modifier.size(94.dp))
                }
            }

            if (editMode) {
                Box(
                    modifier = Modifier
                        .size(146.dp)
                        .clip(CircleShape)
                        .background(Color.Black.copy(alpha = 0.45f)),
                    contentAlignment = Alignment.Center
                ) {
                    androidx.tv.material3.Icon(
                        imageVector = FluxaIcons.Edit,
                        contentDescription = null,
                        tint = Color.White,
                        modifier = Modifier.size(34.dp)
                    )
                }
            } else if (!profile.pinHash.isNullOrBlank()) {
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
    }
}

@Composable
internal fun TvAddProfileOrb(
    lang: String = "en",
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    var isFocused by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isFocused) 1.04f else 1f, tween(180), label = "addProfileScale")

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
                .background(Color.White.copy(alpha = if (isFocused) 0.1f else 0.04f))
                .border(
                    width = if (isFocused) 2.dp else 1.dp,
                    color = if (isFocused) Color.White else Color.White.copy(alpha = 0.1f),
                    shape = CircleShape
                ),
            contentAlignment = Alignment.Center
        ) {
            androidx.tv.material3.Icon(
                imageVector = FluxaIcons.Add,
                contentDescription = null,
                tint = Color.White.copy(alpha = if (isFocused) 0.92f else 0.55f),
                modifier = Modifier.size(36.dp)
            )
        }
        Spacer(modifier = Modifier.height(16.dp))
        androidx.tv.material3.Text(
            text = AppStrings.t(lang, "profiles.new_profile"),
            color = Color.White.copy(alpha = if (isFocused) 1f else 0.65f),
            fontSize = 24.sp,
            fontWeight = FontWeight.Medium
        )
    }
}
