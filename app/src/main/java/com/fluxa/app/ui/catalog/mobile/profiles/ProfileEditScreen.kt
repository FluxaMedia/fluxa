package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.*
import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MobileProfileEditScreen(
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
    var biometricEnabled by remember { mutableStateOf(initialProfile?.biometricEnabled == true) }
    val pinValid = pin.isEmpty() || pin.length == 4
    fun resolvePinHash(): String? = when {
        removePin -> null
        pin.length == 4 -> PinHasher.hash(pin)
        else -> initialProfile?.pinHash
    }
    fun resolveBiometricEnabled(): Boolean = resolvePinHash() != null && biometricEnabled
    val biometricAvailable = remember { BiometricLockHelper.isAvailable(context) }
    val scope = rememberCoroutineScope()
    val scrollState = rememberScrollState()

    val imagePicker = rememberLauncherForActivityResult(ActivityResultContracts.GetContent()) { uri ->
        uri?.let {
            scope.launch {
                selectedAvatarUrl = withContext(Dispatchers.IO) {
                    copyProfileImageToLocalUri(context, it) ?: it.toString()
                }
            }
        }
    }

    val categories = remember { AvatarLibrary.categories }
    val avatarData = remember { mutableStateMapOf<String, List<AvatarCharacter>>() }

    LaunchedEffect(categories) {
        categories.forEach { category ->
            if (avatarData[category.imdbId].isNullOrEmpty()) {
                scope.launch {
                    val characters = AvatarProvider.fetchAvatarsForShow(context, category.imdbId)
                    avatarData[category.imdbId] = characters
                }
            }
        }
    }

    Scaffold(
        topBar = {
            CenterAlignedTopAppBar(
                title = {
                    Text(
                        AppStrings.t(lang, if (initialProfile == null) "profiles.add" else "profiles.edit"),
                        fontWeight = FontWeight.Bold
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onCancel) {
                        Icon(FluxaIcons.ArrowBack, null)
                    }
                },
                actions = {
                    TextButton(
                        onClick = {
                            val profile = initialProfile?.copy(
                                email = name,
                                avatarUrl = selectedAvatarUrl,
                                pinHash = resolvePinHash(),
                                biometricEnabled = resolveBiometricEnabled()
                            ) ?: UserProfile(
                                id = java.util.UUID.randomUUID().toString(),
                                email = name,
                                authKey = "",
                                isGuest = false,
                                language = "en",
                                avatarUrl = selectedAvatarUrl,
                                pinHash = resolvePinHash(),
                                biometricEnabled = resolveBiometricEnabled(),
                                localAddons = listOf("https://v3-cinemeta.strem.io/manifest.json")
                            )
                            onSave(profile)
                        },
                        enabled = name.isNotBlank() && pinValid
                    ) {
                        Text(
                            AppStrings.t(lang, "profiles.done"),
                            fontWeight = FontWeight.Bold,
                            color = if (name.isNotBlank() && pinValid) Color.White else Color.White.copy(alpha = 0.35f)
                        )
                    }
                },
                colors = TopAppBarDefaults.centerAlignedTopAppBarColors(
                    containerColor = Color.Black,
                    titleContentColor = Color.White,
                    navigationIconContentColor = Color.White,
                    actionIconContentColor = Color.White
                )
            )
        },
        containerColor = Color.Black
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(scrollState)
                .padding(horizontal = 20.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Spacer(Modifier.height(24.dp))

            // Avatar with edit badge
            Box(
                contentAlignment = Alignment.BottomEnd,
                modifier = Modifier.size(140.dp)
            ) {
                Box(
                    modifier = Modifier
                        .size(140.dp)
                        .clip(CircleShape)
                        .background(Color.White.copy(alpha = 0.12f))
                        .clickable { scope.launch { scrollState.animateScrollTo(scrollState.maxValue) } },
                    contentAlignment = Alignment.Center
                ) {
                    if (selectedAvatarUrl != null) {
                        AsyncImage(
                            model = selectedAvatarUrl,
                            contentDescription = null,
                            modifier = Modifier.fillMaxSize(),
                            contentScale = ContentScale.Crop,
                            alignment = Alignment.Center
                        )
                    } else {
                        DefaultProfileAvatar(modifier = Modifier.size(90.dp))
                    }
                }

                // Pencil badge at bottom-right
                Box(
                    modifier = Modifier
                        .size(36.dp)
                        .clip(CircleShape)
                        .background(Color(0xFF2C2C2C))
                        .border(2.dp, Color.Black, CircleShape),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        FluxaIcons.Edit,
                        contentDescription = null,
                        tint = Color.White,
                        modifier = Modifier.size(18.dp)
                    )
                }
            }

            Spacer(Modifier.height(32.dp))

            OutlinedTextField(
                value = name,
                onValueChange = { name = it },
                label = { Text(AppStrings.t(lang, "profiles.name")) },
                modifier = Modifier.fillMaxWidth(),
                colors = OutlinedTextFieldDefaults.colors(
                    focusedTextColor = Color.White,
                    unfocusedTextColor = Color.White,
                    focusedBorderColor = Color.White,
                    unfocusedBorderColor = Color.White.copy(alpha = 0.3f),
                    focusedLabelColor = Color.White,
                    unfocusedLabelColor = Color.White.copy(alpha = 0.5f),
                    cursorColor = Color.White
                ),
                singleLine = true
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
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword),
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

            if (biometricAvailable && resolvePinHash() != null) {
                Spacer(Modifier.height(8.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            AppStrings.t(lang, "profiles.biometric_lock"),
                            color = Color.White,
                            fontWeight = FontWeight.Medium
                        )
                        Text(
                            AppStrings.t(lang, "profiles.biometric_lock_desc"),
                            color = Color.Gray,
                            fontSize = 12.sp
                        )
                    }
                    Switch(checked = biometricEnabled, onCheckedChange = { biometricEnabled = it })
                }
            }

            Spacer(Modifier.height(32.dp))

            HorizontalDivider(color = Color.White.copy(alpha = 0.1f))

            Spacer(Modifier.height(20.dp))

            Text(
                AppStrings.t(lang, "profiles.choose_image"),
                color = Color.White,
                fontSize = 18.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.align(Alignment.Start)
            )

            Spacer(Modifier.height(4.dp))

            Text(
                text = AppStrings.t(lang, "profiles.choose_image_desc"),
                color = Color.Gray,
                fontSize = 12.sp,
                modifier = Modifier.align(Alignment.Start)
            )

            Spacer(Modifier.height(16.dp))

            OutlinedButton(
                onClick = { imagePicker.launch("image/*") },
                modifier = Modifier.fillMaxWidth(),
                border = BorderStroke(1.dp, Color.White.copy(alpha = 0.25f)),
                colors = ButtonDefaults.outlinedButtonColors(contentColor = Color.White)
            ) {
                Icon(FluxaIcons.Upload, null, modifier = Modifier.size(18.dp))
                Spacer(Modifier.width(8.dp))
                Text(AppStrings.t(lang, "profiles.upload_image"))
            }

            Spacer(Modifier.height(20.dp))

            categories.forEach { category ->
                val characters = avatarData[category.imdbId]
                if (!characters.isNullOrEmpty()) {
                    Text(
                        category.showName,
                        color = Color.Gray,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Medium,
                        modifier = Modifier
                            .align(Alignment.Start)
                            .padding(vertical = 8.dp)
                    )

                    LazyRow(
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                        modifier = Modifier.fillMaxWidth(),
                        contentPadding = PaddingValues(bottom = 16.dp)
                    ) {
                        items(characters, key = { it.url }) { character ->
                            AvatarItem(
                                character = character,
                                isSelected = selectedAvatarUrl == character.url,
                                onClick = { selectedAvatarUrl = character.url }
                            )
                        }
                    }
                }
            }

            if (initialProfile != null && onDelete != null) {
                Spacer(Modifier.height(8.dp))
                TextButton(onClick = { onDelete(initialProfile) }) {
                    Text(AppStrings.t(lang, "profiles.delete"), color = Color(0xFFFF8A8A), fontWeight = FontWeight.Bold)
                }
            }

            Spacer(Modifier.height(32.dp))
        }
    }
}

private fun copyProfileImageToLocalUri(context: android.content.Context, source: android.net.Uri): String? {
    return runCatching {
        val mimeType = context.contentResolver.getType(source) ?: "image/jpeg"
        val ext = when {
            mimeType.contains("gif") -> "gif"
            mimeType.contains("png") -> "png"
            mimeType.contains("webp") -> "webp"
            else -> "jpg"
        }
        val directory = File(context.filesDir, "profile_images").apply { mkdirs() }
        val target = File(directory, "profile_${System.currentTimeMillis()}.$ext")
        context.contentResolver.openInputStream(source)?.use { input ->
            target.outputStream().use { output -> input.copyTo(output) }
        } ?: return null
        target.toURI().toString()
    }.getOrNull()
}

@Composable
fun AvatarItem(
    character: AvatarCharacter,
    isSelected: Boolean,
    onClick: () -> Unit
) {
    Box(
        modifier = Modifier
            .size(FluxaDimensions.Profile.avatarPickerThumbSize)
            .clip(CircleShape)
            .border(
                width = if (isSelected) 3.dp else 0.dp,
                color = if (isSelected) Color(0xFFE50914) else Color.Transparent,
                shape = CircleShape
            )
            .clickable { onClick() }
    ) {
        AsyncImage(
            model = character.url,
            contentDescription = character.name,
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Crop,
            alignment = Alignment.Center
        )
        if (isSelected) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(Color.Black.copy(alpha = 0.3f)),
                contentAlignment = Alignment.Center
            ) {
                Icon(FluxaIcons.Check, null, tint = Color.White)
            }
        }
    }
}
