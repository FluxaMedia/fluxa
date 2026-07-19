package com.fluxa.app.shared.feature.profile

import androidx.compose.foundation.background
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
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.foundation.border
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun ProfileListScreen(
    state: ProfileUiState,
    language: String?,
    biometricAvailable: Boolean,
    onAction: (ProfileAction) -> Unit,
    onBiometricRequested: (ProfileUiModel) -> Unit,
    modifier: Modifier = Modifier
) {
    var isManaging by remember { mutableStateOf(false) }
    val firstItemFocusRequester = remember { FocusRequester() }
    LaunchedEffect(state.profiles.firstOrNull()?.id) {
        runCatching { firstItemFocusRequester.requestFocus() }
    }

    LaunchedEffect(state.pendingPinProfile?.id) {
        val pending = state.pendingPinProfile
        if (pending != null && pending.biometricEnabled && biometricAvailable) {
            onBiometricRequested(pending)
        }
    }

    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        Column(
            modifier = Modifier.fillMaxSize().padding(top = 48.dp, bottom = 24.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text(
                text = AppStrings.t(language, "profiles.who_watching"),
                color = Color.White,
                fontSize = 24.sp,
                fontWeight = FontWeight.Bold
            )
            Spacer(Modifier.height(32.dp))
            LazyVerticalGrid(
                columns = GridCells.Fixed(2),
                modifier = Modifier.fillMaxWidth().weight(1f),
                contentPadding = PaddingValues(horizontal = 20.dp, vertical = 8.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalArrangement = Arrangement.spacedBy(24.dp)
            ) {
                itemsIndexed(state.profiles, key = { _, profile -> profile.id }) { index, profile ->
                    ProfileGridItem(
                        profile = profile,
                        isManaging = isManaging,
                        onClick = { onAction(ProfileAction.Selected(profile)) },
                        onEditClick = { onAction(ProfileAction.EditRequested(profile)) },
                        focusRequester = if (index == 0) firstItemFocusRequester else null
                    )
                }
                item {
                    AddProfileGridItem(
                        language = language,
                        onClick = { onAction(ProfileAction.AddRequested) }
                    )
                }
            }
            TextButton(onClick = { isManaging = true }, enabled = !isManaging) {
                Text(
                    text = AppStrings.t(language, "profiles.manage"),
                    color = Color.White,
                    fontWeight = FontWeight.Bold
                )
            }
        }

        state.pendingPinProfile?.let { profile ->
            ProfilePinOverlay(
                profile = profile,
                language = language,
                pinError = state.pinError,
                onSubmit = { pin -> onAction(ProfileAction.PinAttempt(profile.id, pin)) },
                onDismiss = { onAction(ProfileAction.PinCancelled) }
            )
        }
    }
}

@Composable
private fun ProfileGridItem(
    profile: ProfileUiModel,
    isManaging: Boolean,
    onClick: () -> Unit,
    onEditClick: () -> Unit,
    focusRequester: FocusRequester? = null
) {
    Column(
        modifier = Modifier.fillMaxWidth().wrapContentWidth(Alignment.CenterHorizontally),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        var focused by remember { mutableStateOf(false) }
        Box(
            modifier = Modifier
                .size(100.dp)
                .clip(CircleShape)
                .let { if (focusRequester != null) it.focusRequester(focusRequester) else it }
                .onFocusChanged { focused = it.isFocused }
                .background(Color(profile.accentColorArgb))
                .then(if (focused) Modifier.border(3.dp, Color.White, CircleShape) else Modifier)
                .clickable(enabled = !isManaging, onClick = onClick),
            contentAlignment = Alignment.Center
        ) {
            if (!profile.avatarUrl.isNullOrBlank()) {
                FluxaRemoteImage(
                    imageUrl = profile.avatarUrl,
                    cacheKey = "profile-avatar:${profile.avatarUrl}",
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize().clip(CircleShape),
                    contentScale = androidx.compose.ui.layout.ContentScale.Crop
                )
            } else {
                ProfileDefaultAvatar(modifier = Modifier.size(66.dp))
            }
        }
        Spacer(Modifier.height(10.dp))
        Text(profile.name, color = Color.White, fontSize = 16.sp, fontWeight = FontWeight.Medium)
        if (isManaging) {
            Spacer(Modifier.height(6.dp))
            var editFocused by remember { mutableStateOf(false) }
            Box(
                modifier = Modifier
                    .size(30.dp)
                    .clip(CircleShape)
                    .onFocusChanged { editFocused = it.isFocused }
                    .background(if (editFocused) Color.White else Color.White.copy(alpha = 0.1f))
                    .clickable(onClick = onEditClick),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    Icons.Filled.Edit,
                    contentDescription = null,
                    tint = if (editFocused) Color.Black else Color.White.copy(alpha = 0.82f),
                    modifier = Modifier.size(15.dp)
                )
            }
        }
    }
}

@Composable
private fun AddProfileGridItem(language: String?, onClick: () -> Unit) {
    var focused by remember { mutableStateOf(false) }
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier.fillMaxWidth().wrapContentWidth(Alignment.CenterHorizontally).onFocusChanged { focused = it.isFocused }.clickable(onClick = onClick)
    ) {
        Box(
            modifier = Modifier
                .size(100.dp)
                .clip(CircleShape)
                .background(Color.White.copy(alpha = 0.12f))
                .then(if (focused) Modifier.border(3.dp, Color.White, CircleShape) else Modifier),
            contentAlignment = Alignment.Center
        ) {
            Text("+", color = Color.White, fontSize = 32.sp, fontWeight = FontWeight.Bold)
        }
        Spacer(Modifier.height(12.dp))
        Text(AppStrings.t(language, "profiles.add_profile"), color = Color.White, fontSize = 16.sp)
    }
}

private const val PIN_LENGTH = 4

@Composable
private fun ProfilePinOverlay(
    profile: ProfileUiModel,
    language: String?,
    pinError: Boolean,
    onSubmit: (String) -> Unit,
    onDismiss: () -> Unit
) {
    var pin by remember(profile.id) { mutableStateOf("") }
    val focusRequester = remember { FocusRequester() }

    LaunchedEffect(profile.id) {
        runCatching { focusRequester.requestFocus() }
    }
    LaunchedEffect(pinError) {
        if (pinError) pin = ""
    }

    Box(modifier = Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.78f)), contentAlignment = Alignment.Center) {
        Column(
            modifier = Modifier
                .widthIn(max = 400.dp)
                .clip(RoundedCornerShape(32.dp))
                .background(Color(0xFF15121C))
                .padding(horizontal = 40.dp, vertical = 36.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Box(
                modifier = Modifier.size(72.dp).clip(CircleShape).background(Color(profile.accentColorArgb)),
                contentAlignment = Alignment.Center
            ) {
                if (!profile.avatarUrl.isNullOrBlank()) {
                    FluxaRemoteImage(
                        imageUrl = profile.avatarUrl,
                        cacheKey = "profile-avatar:${profile.avatarUrl}",
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize().clip(CircleShape),
                        contentScale = androidx.compose.ui.layout.ContentScale.Crop
                    )
                } else {
                    ProfileDefaultAvatar(modifier = Modifier.size(46.dp))
                }
            }
            Spacer(Modifier.height(18.dp))
            Text(profile.name, color = Color.White, fontSize = 24.sp, fontWeight = FontWeight.Bold)
            Spacer(Modifier.height(6.dp))
            Text(
                AppStrings.t(language, "profiles.pin_prompt"),
                color = Color.White.copy(alpha = 0.6f),
                fontSize = 15.sp,
                textAlign = TextAlign.Center
            )
            Spacer(Modifier.height(32.dp))

            Box {
                BasicTextField(
                    value = pin,
                    onValueChange = {
                        val digits = it.filter(Char::isDigit).take(PIN_LENGTH)
                        pin = digits
                        if (digits.length == PIN_LENGTH) onSubmit(digits)
                    },
                    modifier = Modifier.focusRequester(focusRequester).size(1.dp),
                    singleLine = true,
                    textStyle = TextStyle(color = Color.Transparent, fontSize = 1.sp),
                    cursorBrush = SolidColor(Color.Transparent),
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword, imeAction = ImeAction.Done),
                    keyboardActions = KeyboardActions(onDone = { if (pin.length == PIN_LENGTH) onSubmit(pin) })
                )
                Row(horizontalArrangement = Arrangement.spacedBy(14.dp)) {
                    repeat(PIN_LENGTH) { index ->
                        val filled = index < pin.length
                        Box(
                            modifier = Modifier
                                .size(52.dp)
                                .clip(RoundedCornerShape(16.dp))
                                .background(Color.White.copy(alpha = if (filled) 0.14f else 0.06f)),
                            contentAlignment = Alignment.Center
                        ) {
                            if (filled) {
                                Box(modifier = Modifier.size(12.dp).clip(CircleShape).background(Color.White))
                            }
                        }
                    }
                }
            }

            Spacer(Modifier.height(18.dp))
            Text(
                text = if (pinError) AppStrings.t(language, "profiles.pin_error") else "",
                color = FluxaColors.errorRed,
                fontSize = 14.sp
            )
            Spacer(Modifier.height(18.dp))

            TextButton(onClick = onDismiss) {
                Text(AppStrings.t(language, "common.cancel"), color = Color.White)
            }
        }
    }
}

@Composable
fun ProfileDefaultAvatar(modifier: Modifier = Modifier, tint: Color = Color.White) {
    androidx.compose.foundation.Canvas(modifier = modifier.fillMaxSize()) {
        val w = size.width
        val h = size.height
        val strokeWidth = size.minDimension * 0.075f
        drawCircle(color = tint, radius = w * 0.085f, center = androidx.compose.ui.geometry.Offset(w * 0.30f, h * 0.37f))
        drawCircle(color = tint, radius = w * 0.075f, center = androidx.compose.ui.geometry.Offset(w * 0.67f, h * 0.34f))
        val smile = androidx.compose.ui.graphics.Path().apply {
            moveTo(w * 0.22f, h * 0.62f)
            cubicTo(w * 0.20f, h * 0.54f, w * 0.26f, h * 0.57f, w * 0.30f, h * 0.65f)
            cubicTo(w * 0.42f, h * 0.75f, w * 0.66f, h * 0.68f, w * 0.74f, h * 0.57f)
        }
        drawPath(
            path = smile,
            color = tint,
            style = androidx.compose.ui.graphics.drawscope.Stroke(width = strokeWidth, cap = androidx.compose.ui.graphics.StrokeCap.Round)
        )
    }
}
