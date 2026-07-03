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
import androidx.compose.ui.graphics.SolidColor
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
    val activity = context as? androidx.fragment.app.FragmentActivity

    fun selectMobileProfile(profile: UserProfile) {
        if (profile.pinHash.isNullOrBlank()) {
            onProfileSelected(profile)
            return
        }
        if (profile.biometricEnabled == true && activity != null && BiometricLockHelper.isAvailable(context)) {
            BiometricLockHelper.authenticate(
                activity = activity,
                lang = profile.safeLanguage,
                onSuccess = { onProfileSelected(profile) },
                onFailure = { profileNeedingPin = profile }
            )
        } else {
            profileNeedingPin = profile
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(if (isTv) Color(0xFF0C0C0C) else Color(0xFF09070B))
    ) {
        if (!isTv) {
            AsyncImage(
                model = currentHero?.background,
                contentDescription = null,
                modifier = Modifier
                    .fillMaxSize()
                    .alpha(0.32f),
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
        }

        if (isTv) {
            TvProfileSelectionScene(
                profiles = profiles,
                lang = lang,
                onProfileSelected = {
                    if (it.pinHash.isNullOrBlank()) onProfileSelected(it) else profileNeedingPin = it
                },
                onAddProfileClick = onAddProfileClick,
                onEditProfileClick = onEditProfileClick
            )
        } else {
            MobileProfileSelectionScene(
                profiles = profiles,
                onProfileSelected = { selectMobileProfile(it) },
                onAddProfileClick = onAddProfileClick,
                onEditProfileClick = onEditProfileClick,
                onDeleteClick = { profileManager.deleteProfileById(it.id) }
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

private const val PIN_LENGTH = 4

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

    fun submit() {
        if (PinHasher.hash(pin) == profile.pinHash) {
            onPinVerified()
        } else {
            showError = true
            pin = ""
        }
    }

    LaunchedEffect(profile.id) {
        runCatching { focusRequester.requestFocus() }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.78f)),
        contentAlignment = Alignment.Center
    ) {
        Column(
            modifier = Modifier
                .widthIn(max = 460.dp)
                .clip(RoundedCornerShape(32.dp))
                .background(Color(0xFF15121C))
                .border(1.dp, Color.White.copy(alpha = 0.10f), RoundedCornerShape(32.dp))
                .padding(horizontal = 40.dp, vertical = 36.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Box(
                modifier = Modifier
                    .size(72.dp)
                    .clip(CircleShape)
                    .background(Color(profile.safeColorArgb).copy(alpha = 0.95f))
                    .border(2.dp, Color.White.copy(alpha = 0.28f), CircleShape),
                contentAlignment = Alignment.Center
            ) {
                if (!profile.avatarUrl.isNullOrBlank()) {
                    AsyncImage(
                        model = profile.avatarUrl,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize().clip(CircleShape),
                        contentScale = androidx.compose.ui.layout.ContentScale.Crop
                    )
                } else {
                    DefaultProfileAvatar(modifier = Modifier.size(46.dp))
                }
            }
            Spacer(modifier = Modifier.height(18.dp))
            androidx.tv.material3.Text(
                text = profile.displayName,
                color = Color.White,
                fontSize = 24.sp,
                fontWeight = FontWeight.Bold
            )
            Spacer(modifier = Modifier.height(6.dp))
            androidx.tv.material3.Text(
                text = AppStrings.t(lang, "profiles.pin_prompt"),
                color = Color.White.copy(alpha = 0.6f),
                fontSize = 15.sp,
                textAlign = TextAlign.Center
            )
            Spacer(modifier = Modifier.height(32.dp))

            Box {
                BasicTextField(
                    value = pin,
                    onValueChange = {
                        val digits = it.filter(Char::isDigit).take(PIN_LENGTH)
                        pin = digits
                        showError = false
                        if (digits.length == PIN_LENGTH) submit()
                    },
                    modifier = Modifier
                        .focusRequester(focusRequester)
                        .size(1.dp),
                    singleLine = true,
                    textStyle = TextStyle(color = Color.Transparent, fontSize = 1.sp),
                    cursorBrush = SolidColor(Color.Transparent),
                    keyboardOptions = KeyboardOptions(
                        keyboardType = KeyboardType.NumberPassword,
                        imeAction = ImeAction.Done
                    ),
                    keyboardActions = KeyboardActions(onDone = { submit() })
                )
                Row(horizontalArrangement = Arrangement.spacedBy(14.dp)) {
                    repeat(PIN_LENGTH) { index ->
                        val filled = index < pin.length
                        val isCursor = index == pin.length
                        Box(
                            modifier = Modifier
                                .size(52.dp)
                                .clip(RoundedCornerShape(16.dp))
                                .background(Color.White.copy(alpha = if (filled) 0.14f else 0.06f))
                                .border(
                                    width = if (isCursor) 2.dp else 1.dp,
                                    color = when {
                                        showError -> FluxaColors.errorRed
                                        isCursor -> Color(0xFF63D7FF)
                                        else -> Color.White.copy(alpha = 0.16f)
                                    },
                                    shape = RoundedCornerShape(16.dp)
                                ),
                            contentAlignment = Alignment.Center
                        ) {
                            if (filled) {
                                Box(
                                    modifier = Modifier
                                        .size(12.dp)
                                        .clip(CircleShape)
                                        .background(Color.White)
                                )
                            }
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(18.dp))
            androidx.tv.material3.Text(
                text = if (showError) AppStrings.t(lang, "profiles.pin_error") else "",
                color = Color(0xFFFF8A8A),
                fontSize = 14.sp
            )
            Spacer(modifier = Modifier.height(18.dp))

            Surface(
                onClick = onDismiss,
                modifier = Modifier.width(160.dp),
                shape = androidx.tv.material3.ClickableSurfaceDefaults.shape(RoundedCornerShape(16.dp)),
                colors = androidx.tv.material3.ClickableSurfaceDefaults.colors(
                    containerColor = Color.White.copy(alpha = 0.06f),
                    focusedContainerColor = Color.White.copy(alpha = 0.16f)
                )
            ) {
                Box(modifier = Modifier.padding(vertical = 14.dp), contentAlignment = Alignment.Center) {
                    androidx.tv.material3.Text(text = AppStrings.t(lang, "common.cancel"), color = Color.White)
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
