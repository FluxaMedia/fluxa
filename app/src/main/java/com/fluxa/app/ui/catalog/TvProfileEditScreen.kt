@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.UserProfile

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
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
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
internal fun TvProfileEditScreen(
    initialProfile: UserProfile? = null,
    onSave: (UserProfile) -> Unit,
    onDelete: ((UserProfile) -> Unit)? = null,
    onCancel: () -> Unit
) {
    val context = LocalContext.current
    val lang = initialProfile?.safeLanguage ?: "en"
    var name by remember { mutableStateOf(initialProfile?.displayName ?: "") }
    var selectedAvatarUrl by remember { mutableStateOf(initialProfile?.avatarUrl) }
    var pin by remember { mutableStateOf("") }
    var removePin by remember { mutableStateOf(false) }
    val pinValid = pin.isEmpty() || pin.length == 4
    fun resolvePinHash(): String? = when {
        removePin -> null
        pin.length == 4 -> com.fluxa.app.data.local.PinHasher.hash(pin)
        else -> initialProfile?.pinHash
    }
    val scope = rememberCoroutineScope()

    val imagePicker = rememberLauncherForActivityResult(ActivityResultContracts.GetContent()) { uri ->
        uri?.let {
            scope.launch {
                selectedAvatarUrl = withContext(Dispatchers.IO) {
                    copyProfileImageToLocalUri(context, it) ?: it.toString()
                }
            }
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
            .padding(horizontal = 58.dp, vertical = 40.dp)
    ) {
        Row(modifier = Modifier.fillMaxSize()) {
            Column(modifier = Modifier.widthIn(min = 320.dp)) {
                IconButton(onClick = onCancel) {
                    Icon(FluxaIcons.ArrowBack, null, tint = Color.White)
                }
                Spacer(Modifier.height(12.dp))
                Text(
                    AppStrings.t(lang, if (initialProfile == null) "profiles.add" else "profiles.edit"),
                    color = Color.White,
                    fontSize = 26.sp,
                    fontWeight = FontWeight.Black
                )
                Spacer(Modifier.height(28.dp))
                Box(
                    modifier = Modifier
                        .size(140.dp)
                        .clip(CircleShape)
                        .background(Color.White.copy(alpha = 0.12f)),
                    contentAlignment = Alignment.Center
                ) {
                    if (selectedAvatarUrl != null) {
                        AsyncImage(
                            model = selectedAvatarUrl,
                            contentDescription = null,
                            modifier = Modifier.fillMaxSize(),
                            contentScale = ContentScale.Crop
                        )
                    } else {
                        DefaultProfileAvatar(modifier = Modifier.size(90.dp))
                    }
                }
                Spacer(Modifier.height(28.dp))
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text(AppStrings.t(lang, "profiles.name")) },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true,
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedTextColor = Color.White,
                        unfocusedTextColor = Color.White,
                        focusedBorderColor = Color.White,
                        unfocusedBorderColor = Color.White.copy(alpha = 0.3f),
                        focusedLabelColor = Color.White,
                        unfocusedLabelColor = Color.White.copy(alpha = 0.5f),
                        cursorColor = Color.White
                    )
                )
                Spacer(Modifier.height(16.dp))
                OutlinedTextField(
                    value = pin,
                    onValueChange = {
                        pin = it.filter(Char::isDigit).take(4)
                        if (pin.isNotEmpty()) removePin = false
                    },
                    label = { Text(AppStrings.t(lang, "profiles.pin_lock")) },
                    placeholder = {
                        Text(
                            if (initialProfile?.pinHash != null && !removePin) {
                                AppStrings.t(lang, "profiles.pin_set_placeholder")
                            } else {
                                AppStrings.t(lang, "profiles.pin_placeholder")
                            }
                        )
                    },
                    isError = !pinValid,
                    supportingText = if (!pinValid) {
                        { Text(AppStrings.t(lang, "profiles.pin_invalid")) }
                    } else null,
                    visualTransformation = androidx.compose.ui.text.input.PasswordVisualTransformation(),
                    keyboardOptions = androidx.compose.foundation.text.KeyboardOptions(keyboardType = androidx.compose.ui.text.input.KeyboardType.NumberPassword),
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedTextColor = Color.White,
                        unfocusedTextColor = Color.White,
                        focusedBorderColor = Color.White,
                        unfocusedBorderColor = Color.White.copy(alpha = 0.3f),
                        focusedLabelColor = Color.White,
                        unfocusedLabelColor = Color.White.copy(alpha = 0.5f),
                        cursorColor = Color.White
                    )
                )
                if (initialProfile?.pinHash != null && !removePin) {
                    TextButton(onClick = { removePin = true; pin = "" }) {
                        Text(AppStrings.t(lang, "profiles.pin_remove"), color = Color(0xFFFF8A8A))
                    }
                }
                Spacer(Modifier.height(28.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                    androidx.tv.material3.Button(
                        onClick = {
                            val profile = initialProfile?.copy(
                                profileName = name,
                                avatarUrl = selectedAvatarUrl,
                                pinHash = resolvePinHash()
                            ) ?: UserProfile(
                                id = java.util.UUID.randomUUID().toString(),
                                email = name,
                                profileName = name,
                                authKey = "",
                                isGuest = false,
                                language = "en",
                                avatarUrl = selectedAvatarUrl,
                                pinHash = resolvePinHash(),
                                localAddons = listOf("https://v3-cinemeta.strem.io/manifest.json")
                            )
                            onSave(profile)
                        },
                        enabled = name.isNotBlank() && pinValid
                    ) {
                        Text(AppStrings.t(lang, "profiles.done"))
                    }
                    if (initialProfile != null && onDelete != null) {
                        TextButton(onClick = { onDelete(initialProfile) }) {
                            Text(AppStrings.t(lang, "profiles.delete"), color = Color(0xFFFF8A8A), fontWeight = FontWeight.Bold)
                        }
                    }
                }
            }

            Spacer(Modifier.width(48.dp))

            Column(modifier = Modifier.fillMaxSize()) {
                Text(
                    AppStrings.t(lang, "profiles.choose_image"),
                    color = Color.White,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Bold
                )
                Spacer(Modifier.height(4.dp))
                Text(
                    AppStrings.t(lang, "profiles.choose_image_desc"),
                    color = Color.Gray,
                    fontSize = 13.sp
                )
                Spacer(Modifier.height(16.dp))
                OutlinedButton(onClick = { imagePicker.launch("image/*") }) {
                    Icon(FluxaIcons.Upload, null, modifier = Modifier.size(18.dp))
                    Spacer(Modifier.width(8.dp))
                    Text(AppStrings.t(lang, "profiles.upload_image"))
                }
                if (selectedAvatarUrl != null) {
                    Spacer(Modifier.height(8.dp))
                    TextButton(onClick = { selectedAvatarUrl = null }) {
                        Text(AppStrings.t(lang, "profiles.remove_image"), color = Color.White.copy(alpha = 0.6f))
                    }
                }
            }
        }
    }
}
