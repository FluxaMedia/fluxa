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

@Composable
fun ProfileScreen(
    context: android.content.Context,
    onProfileSelected: (UserProfile) -> Unit,
    onAddProfileClick: () -> Unit,
    onEditProfileClick: (UserProfile) -> Unit,
    viewModel: HomeViewModel? = null
) {
    val profileManager = remember { AppContainer.profileManager }
    var profiles by remember { mutableStateOf(profileManager.getProfiles()) }
    var profileNeedingPin by remember { mutableStateOf<UserProfile?>(null) }

    DisposableEffect(Unit) {
        val listener = android.content.SharedPreferences.OnSharedPreferenceChangeListener { _, _ ->
            profiles = profileManager.getProfiles()
        }
        profileManager.registerOnChangeListener(listener)
        profiles = profileManager.getProfiles()
        onDispose { profileManager.unregisterOnChangeListener(listener) }
    }

    val billboardPool by (viewModel?.billboardPool?.collectAsState() ?: remember { mutableStateOf(emptyList<Meta>()) })
    val currentHero = remember(billboardPool) {
        billboardPool.firstOrNull { !it.background.isNullOrBlank() || !it.poster.isNullOrBlank() }
    }
    val deviceType = LocalDeviceType.current
    val isTv = deviceType == DeviceType.TV
    val lang = profiles.firstOrNull()?.safeLanguage ?: "en"

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFF09070B))
    ) {
        AsyncImage(
            model = currentHero?.background,
            contentDescription = null,
            modifier = Modifier
                .fillMaxSize()
                .alpha(if (isTv) 0.22f else 0.32f),
            contentScale = ContentScale.Crop
        )
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.radialGradient(
                        colors = listOf(
                            Color(0xFF7D1E3A).copy(alpha = 0.85f),
                            Color(0xFF30111E).copy(alpha = 0.72f),
                            Color(0xFF09070B)
                        ),
                        radius = 1200f
                    )
                )
        )
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.verticalGradient(
                        colors = listOf(
                            Color.Transparent,
                            Color(0xFF09070B).copy(alpha = 0.48f),
                            Color(0xFF09070B)
                        )
                    )
                )
        )

        if (isTv) {
            TvProfileSelectionScene(
                profiles = profiles,
                lang = lang,
                onProfileSelected = {
                    if (it.pin.isNullOrBlank()) onProfileSelected(it) else profileNeedingPin = it
                },
                onAddProfileClick = onAddProfileClick
            )
        } else {
            MobileProfileSelectionScene(
                profiles = profiles,
                onProfileSelected = {
                    if (it.pin.isNullOrBlank()) onProfileSelected(it) else profileNeedingPin = it
                },
                onAddProfileClick = onAddProfileClick,
                onEditProfileClick = onEditProfileClick
            )
        }

        profileNeedingPin?.let { profile ->
            ProfilePinOverlay(
                profile = profile,
                onDismiss = { profileNeedingPin = null },
                onPinVerified = {
                    profileNeedingPin = null
                    onProfileSelected(profile)
                }
            )
        }
    }
}

@Composable
private fun ProfilePinOverlay(
    profile: UserProfile,
    onDismiss: () -> Unit,
    onPinVerified: () -> Unit
) {
    var pin by remember { mutableStateOf("") }
    var showError by remember { mutableStateOf(false) }
    val focusRequester = remember { FocusRequester() }
    val lang = profile.safeLanguage

    LaunchedEffect(profile.id) {
        runCatching { focusRequester.requestFocus() }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.72f)),
        contentAlignment = Alignment.Center
    ) {
        Column(
            modifier = Modifier
                .widthIn(max = 420.dp)
                .clip(RoundedCornerShape(28.dp))
                .background(Color(0xFF16111B))
                .border(1.dp, Color.White.copy(alpha = 0.12f), RoundedCornerShape(28.dp))
                .padding(28.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            androidx.tv.material3.Text(
                text = profile.displayName,
                color = Color.White,
                fontSize = 28.sp,
                fontWeight = FontWeight.Bold
            )
            Spacer(modifier = Modifier.height(8.dp))
            androidx.tv.material3.Text(
                text = AppStrings.t(lang, "profiles.pin_prompt"),
                color = Color.White.copy(alpha = 0.74f),
                textAlign = TextAlign.Center
            )
            Spacer(modifier = Modifier.height(22.dp))
            BasicTextField(
                value = pin,
                onValueChange = {
                    pin = it.filter(Char::isDigit).take(6)
                    showError = false
                },
                modifier = Modifier
                    .focusRequester(focusRequester)
                    .width(220.dp)
                    .clip(RoundedCornerShape(18.dp))
                    .background(Color.White.copy(alpha = 0.08f))
                    .border(1.dp, Color.White.copy(alpha = 0.15f), RoundedCornerShape(18.dp))
                    .padding(vertical = 16.dp),
                singleLine = true,
                textStyle = TextStyle(
                    color = Color.White,
                    fontSize = 28.sp,
                    fontWeight = FontWeight.Bold,
                    textAlign = TextAlign.Center
                ),
                keyboardOptions = KeyboardOptions(
                    keyboardType = KeyboardType.NumberPassword,
                    imeAction = ImeAction.Done
                ),
                keyboardActions = KeyboardActions(
                    onDone = {
                        if (pin == profile.pin) onPinVerified() else showError = true
                    }
                ),
                decorationBox = { innerTextField ->
                    Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
                        if (pin.isBlank()) {
                            androidx.tv.material3.Text(
                                text = "",
                                color = Color.White.copy(alpha = 0.24f),
                                fontSize = 28.sp,
                                fontWeight = FontWeight.Medium
                            )
                        }
                        innerTextField()
                    }
                }
            )
            if (showError) {
                Spacer(modifier = Modifier.height(12.dp))
                androidx.tv.material3.Text(
                    text = AppStrings.t(lang, "profiles.pin_error"),
                    color = Color(0xFFFF8A8A),
                    fontSize = 14.sp
                )
            }
            Spacer(modifier = Modifier.height(22.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Surface(
                    onClick = onDismiss,
                    modifier = Modifier.width(140.dp),
                    colors = androidx.tv.material3.ClickableSurfaceDefaults.colors(
                        containerColor = Color.White.copy(alpha = 0.08f),
                        focusedContainerColor = Color.White.copy(alpha = 0.14f)
                    )
                ) {
                    Box(modifier = Modifier.padding(vertical = 14.dp), contentAlignment = Alignment.Center) {
                        androidx.tv.material3.Text(text = AppStrings.t(lang, "common.cancel"), color = Color.White)
                    }
                }
                Surface(
                    onClick = {
                        if (pin == profile.pin) onPinVerified() else showError = true
                    },
                    modifier = Modifier.width(140.dp),
                    colors = androidx.tv.material3.ClickableSurfaceDefaults.colors(
                        containerColor = Color(0xFF63D7FF),
                        focusedContainerColor = Color(0xFF9AE7FF)
                    )
                ) {
                    Box(modifier = Modifier.padding(vertical = 14.dp), contentAlignment = Alignment.Center) {
                        androidx.tv.material3.Text(text = AppStrings.t(lang, "profiles.enter"), color = Color(0xFF04131C), fontWeight = FontWeight.Bold)
                    }
                }
            }
        }
    }
}

@Composable
fun ProfileItem(profile: UserProfile, size: Dp, onClick: () -> Unit, onEditClick: () -> Unit) {
    TvProfileOrb(
        profile = profile,
        modifier = Modifier.width(size + 34.dp),
        onClick = onClick
    )
}

@Composable
fun AddProfileItem(size: Dp, onClick: () -> Unit) {
    TvAddProfileOrb(
        modifier = Modifier.width(size + 34.dp),
        onClick = onClick
    )
}

internal fun profileRoleLabel(profile: UserProfile): String {
    return when {
        profile.isGuest -> "GUEST"
        !profile.pin.isNullOrBlank() -> "KILITLI"
        else -> "YONETICI"
    }
}

internal fun roleColor(profile: UserProfile): Color {
    return when {
        profile.isGuest -> Color(0xFFD8D1E7)
        else -> Color(0xFFFFD95B)
    }
}
