package com.fluxa.app.shared.feature.profile

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.settings.SettingsGroupCard
import com.fluxa.app.shared.feature.settings.SettingsSectionHeader
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaColors

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ProfileEditScreen(
    initialProfile: ProfileUiModel?,
    avatarUrl: String?,
    biometricAvailable: Boolean,
    language: String?,
    onPickAvatarClick: () -> Unit,
    onRemoveAvatarClick: () -> Unit,
    onSave: (ProfileEditUiModel) -> Unit,
    onDelete: (suspend (String?) -> Boolean)?,
    onCancel: () -> Unit,
    avatarPacks: List<ProfileAvatarPackUiModel> = emptyList(),
    onPickPackAvatarClick: (String) -> Unit = {},
    modifier: Modifier = Modifier
) {
    var name by remember(initialProfile?.id) { mutableStateOf(initialProfile?.name.orEmpty()) }
    var pin by remember(initialProfile?.id) { mutableStateOf("") }
    var currentPin by remember(initialProfile?.id) { mutableStateOf("") }
    var removePin by remember(initialProfile?.id) { mutableStateOf(false) }
    var biometricEnabled by remember(initialProfile?.id) { mutableStateOf(initialProfile?.biometricEnabled == true) }
    var showAvatarSheet by remember { mutableStateOf(false) }
    var confirmDelete by remember(initialProfile?.id) { mutableStateOf(false) }
    var deletePin by remember(initialProfile?.id) { mutableStateOf("") }
    var deletePinError by remember(initialProfile?.id) { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    val pinValid = pin.isEmpty() || pin.length == 4
    val willHavePin = !removePin && (pin.length == 4 || initialProfile?.hasPin == true)

    Column(modifier = modifier.fillMaxSize().background(Color.Black).verticalScroll(rememberScrollState()).padding(horizontal = 20.dp)) {
        Row(modifier = Modifier.fillMaxWidth().padding(vertical = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(
                modifier = Modifier.size(40.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.05f)).clickable(onClick = onCancel),
                contentAlignment = Alignment.Center
            ) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = null, tint = Color.White, modifier = Modifier.size(20.dp))
            }
            Spacer(Modifier.width(12.dp))
            Text(
                AppStrings.t(language, if (initialProfile == null) "profiles.add" else "profiles.edit"),
                color = Color.White,
                fontWeight = FontWeight.Bold,
                fontSize = 18.sp,
                modifier = Modifier.weight(1f)
            )
            TextButton(
                onClick = {
                    onSave(
                        ProfileEditUiModel(
                            id = initialProfile?.id,
                            name = name,
                            avatarUrl = avatarUrl,
                            newPin = pin.takeIf { it.length == 4 },
                            currentPin = currentPin.takeIf { it.length == 4 },
                            keepExistingPin = !removePin && pin.length != 4 && initialProfile?.hasPin == true,
                            biometricEnabled = willHavePin && biometricEnabled
                        )
                    )
                },
                enabled = name.isNotBlank() && pinValid
            ) {
                Text(
                    AppStrings.t(language, "profiles.done"),
                    fontWeight = FontWeight.Bold,
                    color = if (name.isNotBlank() && pinValid) Color.White else Color.White.copy(alpha = 0.35f)
                )
            }
        }

        Spacer(Modifier.height(8.dp))

        Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.fillMaxWidth()) {
            Box(contentAlignment = Alignment.BottomEnd, modifier = Modifier.size(110.dp)) {
                Box(
                    modifier = Modifier.size(110.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.12f)),
                    contentAlignment = Alignment.Center
                ) {
                    if (!avatarUrl.isNullOrBlank()) {
                        FluxaRemoteImage(
                            imageUrl = avatarUrl,
                            cacheKey = "profile-avatar:$avatarUrl",
                            contentDescription = null,
                            modifier = Modifier.fillMaxSize().clip(CircleShape),
                            contentScale = ContentScale.Crop
                        )
                    } else {
                        ProfileDefaultAvatar(modifier = Modifier.size(70.dp))
                    }
                }
                Box(
                    modifier = Modifier
                        .size(32.dp)
                        .clip(CircleShape)
                        .background(Color(0xFF2C2C2C))
                        .border(2.dp, Color.Black, CircleShape)
                        .clickable { showAvatarSheet = true },
                    contentAlignment = Alignment.Center
                ) {
                    Icon(Icons.Filled.Edit, contentDescription = null, tint = Color.White, modifier = Modifier.size(14.dp))
                }
            }
            Spacer(Modifier.height(10.dp))
            TextButton(onClick = { showAvatarSheet = true }) {
                Text(AppStrings.t(language, "profiles.change_image"), color = Color.White.copy(alpha = 0.7f), fontWeight = FontWeight.Medium)
            }
        }

        Spacer(Modifier.height(16.dp))

        SettingsSectionHeader(AppStrings.t(language, "profiles.name"))
        SettingsGroupCard {
            OutlinedTextField(
                value = name,
                onValueChange = { name = it },
                modifier = Modifier.fillMaxWidth().padding(vertical = 6.dp),
                colors = profileFieldColors(),
                singleLine = true
            )
        }

        SettingsSectionHeader(AppStrings.t(language, "profiles.pin_lock"))
        SettingsGroupCard {
            OutlinedTextField(
                value = pin,
                onValueChange = {
                    pin = it.filter(Char::isDigit).take(4)
                    if (pin.isNotEmpty()) removePin = false
                },
                placeholder = {
                    Text(
                        if (initialProfile?.hasPin == true && !removePin) {
                            AppStrings.t(language, "profiles.pin_set_placeholder")
                        } else {
                            AppStrings.t(language, "profiles.pin_placeholder")
                        }
                    )
                },
                isError = !pinValid,
                supportingText = if (!pinValid) { { Text(AppStrings.t(language, "profiles.pin_invalid")) } } else null,
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword),
                singleLine = true,
                modifier = Modifier.fillMaxWidth().padding(vertical = 6.dp),
                colors = profileFieldColors()
            )

            if (initialProfile?.nuvioPinEnabled == true) {
                OutlinedTextField(
                    value = currentPin,
                    onValueChange = { currentPin = it.filter(Char::isDigit).take(4) },
                    placeholder = { Text("Current Nuvio PIN (for changing/removing)") },
                    visualTransformation = PasswordVisualTransformation(),
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword),
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth().padding(vertical = 6.dp),
                    colors = profileFieldColors(),
                )
            }
            if (initialProfile?.hasPin == true && !removePin) {
                TextButton(onClick = { removePin = true; pin = "" }) {
                    Text(AppStrings.t(language, "profiles.pin_remove"), color = FluxaColors.errorRed)
                }
            }
            if (biometricAvailable && willHavePin) {
                Row(modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text(AppStrings.t(language, "profiles.biometric_lock"), color = Color.White, fontWeight = FontWeight.Medium)
                        Text(AppStrings.t(language, "profiles.biometric_lock_desc"), color = Color.Gray, fontSize = 12.sp)
                    }
                    Switch(checked = biometricEnabled, onCheckedChange = { biometricEnabled = it })
                }
            }
        }

        if (initialProfile != null && onDelete != null) {
            SettingsSectionHeader(AppStrings.t(language, "profiles.danger_zone"))
            SettingsGroupCard {
                TextButton(onClick = { confirmDelete = true }, modifier = Modifier.fillMaxWidth()) {
                    Text(AppStrings.t(language, "profiles.delete"), color = FluxaColors.errorRed, fontWeight = FontWeight.Bold)
                }
            }
        }

        Spacer(Modifier.height(32.dp))
    }

    if (confirmDelete && onDelete != null) {
        AlertDialog(
            onDismissRequest = { confirmDelete = false },
            title = { Text(AppStrings.t(language, "profiles.delete_confirm_title")) },
            text = {
                Column {
                    Text(AppStrings.t(language, "profiles.delete_confirm_message"))
                    if (initialProfile?.hasPin == true) {
                        OutlinedTextField(
                            value = deletePin,
                            onValueChange = {
                                deletePin = it.filter(Char::isDigit).take(4)
                                deletePinError = false
                            },
                            label = { Text(AppStrings.t(language, "profiles.pin_prompt")) },
                            isError = deletePinError,
                            visualTransformation = PasswordVisualTransformation(),
                            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword)
                        )
                    }
                }
            },
            confirmButton = {
                TextButton(onClick = {
                    scope.launch {
                        val deleted = onDelete(if (initialProfile?.hasPin == true) deletePin else null)
                        if (deleted) confirmDelete = false else deletePinError = true
                    }
                }) { Text(AppStrings.t(language, "profiles.delete"), color = FluxaColors.errorRed) }
            },
            dismissButton = {
                TextButton(onClick = { confirmDelete = false }) { Text(AppStrings.t(language, "common.cancel")) }
            }
        )
    }

    if (showAvatarSheet) {
        ModalBottomSheet(onDismissRequest = { showAvatarSheet = false }) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = 20.dp)
                    .padding(bottom = 24.dp)
            ) {
                Text(
                    AppStrings.t(language, "profiles.choose_image"),
                    color = Color.White,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Bold
                )
                Spacer(Modifier.height(4.dp))
                Text(AppStrings.t(language, "profiles.choose_image_desc"), color = Color.Gray, fontSize = 12.sp)
                Spacer(Modifier.height(16.dp))

                OutlinedButton(
                    onClick = onPickAvatarClick,
                    modifier = Modifier.fillMaxWidth(),
                    colors = ButtonDefaults.outlinedButtonColors(contentColor = Color.White)
                ) {
                    Text(AppStrings.t(language, "profiles.upload_image"))
                }

                if (!avatarUrl.isNullOrBlank()) {
                    Spacer(Modifier.height(8.dp))
                    TextButton(onClick = onRemoveAvatarClick, modifier = Modifier.fillMaxWidth()) {
                        Text(AppStrings.t(language, "profiles.remove_image"), color = Color.White.copy(alpha = 0.6f))
                    }
                }

                if (avatarPacks.isNotEmpty()) {
                    Spacer(Modifier.height(16.dp))
                    Text(
                        AppStrings.t(language, "profiles.choose_from_pack"),
                        color = Color.White,
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Bold
                    )
                    avatarPacks.forEach { pack ->
                        Spacer(Modifier.height(12.dp))
                        Text(pack.title, color = Color.White.copy(alpha = 0.6f), fontSize = 13.sp)
                        Spacer(Modifier.height(8.dp))
                        pack.avatars.chunked(4).forEach { row ->
                            Row(horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.padding(bottom = 12.dp)) {
                                row.forEach { avatar ->
                                    AvatarPackThumbnail(
                                        avatar = avatar,
                                        selected = avatar.url == avatarUrl,
                                        onClick = { onPickPackAvatarClick(avatar.url) }
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun AvatarPackThumbnail(avatar: ProfileAvatarUiModel, selected: Boolean, onClick: () -> Unit) {
    var failed by remember(avatar.url) { mutableStateOf(false) }
    Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.width(64.dp)) {
        Box(
            modifier = Modifier
                .size(56.dp)
                .clip(CircleShape)
                .background(Color.White.copy(alpha = 0.08f))
                .then(if (selected) Modifier.border(2.dp, Color.White, CircleShape) else Modifier)
                .clickable(onClick = onClick)
        ) {
            if (failed) {
                ProfileDefaultAvatar(modifier = Modifier.size(34.dp).align(Alignment.Center), tint = Color.White.copy(alpha = 0.35f))
            } else {
                FluxaRemoteImage(
                    imageUrl = avatar.url,
                    cacheKey = "profile-avatar-pack:${avatar.url}",
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize().clip(CircleShape),
                    contentScale = ContentScale.Crop,
                    onError = { failed = true }
                )
            }
        }
        if (avatar.name.isNotBlank()) {
            Spacer(Modifier.height(4.dp))
            Text(
                avatar.name,
                color = Color.White.copy(alpha = 0.6f),
                fontSize = 10.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                textAlign = TextAlign.Center
            )
        }
    }
}

@Composable
private fun profileFieldColors() = OutlinedTextFieldDefaults.colors(
    focusedTextColor = Color.White,
    unfocusedTextColor = Color.White,
    focusedBorderColor = Color.White,
    unfocusedBorderColor = Color.White.copy(alpha = 0.3f),
    focusedLabelColor = Color.White,
    unfocusedLabelColor = Color.White.copy(alpha = 0.5f),
    cursorColor = Color.White
)
