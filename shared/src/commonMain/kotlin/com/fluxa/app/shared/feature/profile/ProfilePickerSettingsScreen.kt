package com.fluxa.app.shared.feature.profile

import androidx.compose.foundation.background
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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.settings.SettingsGroupCard
import com.fluxa.app.shared.feature.settings.SettingsSectionHeader
import com.fluxa.app.shared.feature.settings.SettingsTextFieldRow
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun ProfilePickerSettingsScreen(
    state: ProfileUiState,
    language: String?,
    onAction: (ProfileAction) -> Unit,
    onPickBackgroundClick: () -> Unit,
    onBack: () -> Unit,
    modifier: Modifier = Modifier
) {
    Column(modifier = modifier.fillMaxSize().background(Color.Black).verticalScroll(rememberScrollState()).padding(horizontal = 20.dp)) {
        Row(modifier = Modifier.fillMaxWidth().padding(vertical = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(
                modifier = Modifier.size(40.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.05f)).clickable(onClick = onBack),
                contentAlignment = Alignment.Center
            ) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = null, tint = Color.White, modifier = Modifier.size(20.dp))
            }
            Spacer(Modifier.width(12.dp))
            Text(
                AppStrings.t(language, "profiles.picker_settings"),
                color = Color.White,
                fontWeight = FontWeight.Bold,
                fontSize = 18.sp
            )
        }

        SettingsSectionHeader(AppStrings.t(language, "profiles.picker_background"))
        SettingsGroupCard {
            Text(
                AppStrings.t(language, "profiles.picker_background_desc"),
                color = Color.White.copy(alpha = 0.5f),
                fontSize = 12.sp,
                modifier = Modifier.padding(vertical = 8.dp)
            )
            if (!state.pickerBackgroundUrl.isNullOrBlank()) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(120.dp)
                        .clip(RoundedCornerShape(12.dp))
                        .background(Color.White.copy(alpha = 0.08f))
                ) {
                    FluxaRemoteImage(
                        imageUrl = state.pickerBackgroundUrl,
                        cacheKey = "profile-picker-background:${state.pickerBackgroundUrl}",
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = androidx.compose.ui.layout.ContentScale.Crop
                    )
                }
                Spacer(Modifier.height(12.dp))
            }
            SettingsTextFieldRow(
                AppStrings.t(language, "profiles.picker_background_placeholder"),
                state.pickerBackgroundUrl.orEmpty()
            ) { onAction(ProfileAction.BackgroundUrlChanged(it.takeIf(String::isNotBlank))) }
            Spacer(Modifier.height(8.dp))
            OutlinedButton(
                onClick = onPickBackgroundClick,
                modifier = Modifier.fillMaxWidth(),
                colors = ButtonDefaults.outlinedButtonColors(contentColor = Color.White)
            ) {
                Text(AppStrings.t(language, "profiles.upload_image"))
            }
            if (!state.pickerBackgroundUrl.isNullOrBlank()) {
                TextButton(onClick = { onAction(ProfileAction.BackgroundUrlChanged(null)) }) {
                    Text(AppStrings.t(language, "profiles.clear_background"), color = Color.White.copy(alpha = 0.6f))
                }
            }
        }

        Spacer(Modifier.height(8.dp))
        SettingsSectionHeader(AppStrings.t(language, "profiles.avatar_packs"))
        SettingsGroupCard {
            Text(
                AppStrings.t(language, "profiles.avatar_packs_desc"),
                color = Color.White.copy(alpha = 0.5f),
                fontSize = 12.sp,
                modifier = Modifier.padding(vertical = 8.dp)
            )
            var repositoryUrl by remember { mutableStateOf("") }
            SettingsTextFieldRow(
                AppStrings.t(language, "profiles.avatar_pack_repository_placeholder"),
                repositoryUrl
            ) { repositoryUrl = it }
            Spacer(Modifier.height(8.dp))
            OutlinedButton(
                onClick = {
                    onAction(ProfileAction.AddAvatarPackRequested(repositoryUrl))
                    repositoryUrl = ""
                },
                enabled = repositoryUrl.isNotBlank() && !state.avatarPackDiscoveryInProgress,
                modifier = Modifier.fillMaxWidth(),
                colors = ButtonDefaults.outlinedButtonColors(contentColor = Color.White)
            ) {
                if (state.avatarPackDiscoveryInProgress) {
                    CircularProgressIndicator(modifier = Modifier.size(16.dp), color = Color.White, strokeWidth = 2.dp)
                } else {
                    Text(AppStrings.t(language, "profiles.add_pack"))
                }
            }
            if (state.avatarPackDiscoveryError) {
                Spacer(Modifier.height(8.dp))
                Text(AppStrings.t(language, "profiles.avatar_pack_error"), color = FluxaColors.errorRed, fontSize = 12.sp)
            }
        }

        Spacer(Modifier.height(8.dp))
        if (state.avatarPacks.isEmpty()) {
            Text(
                AppStrings.t(language, "profiles.no_avatar_packs"),
                color = Color.White.copy(alpha = 0.4f),
                fontSize = 13.sp,
                modifier = Modifier.padding(vertical = 12.dp)
            )
        } else {
            state.avatarPacks.forEach { pack ->
                AvatarPackRow(pack, language, onRemove = { onAction(ProfileAction.RemoveAvatarPackRequested(pack.id)) })
                Spacer(Modifier.height(12.dp))
            }
        }

        Spacer(Modifier.height(32.dp))
    }
}

@Composable
private fun AvatarPackRow(pack: ProfileAvatarPackUiModel, language: String?, onRemove: () -> Unit) {
    SettingsGroupCard {
        Row(modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
            Column(modifier = Modifier.weight(1f)) {
                Text(pack.title, color = Color.White, fontWeight = FontWeight.Medium)
                Text(
                    AppStrings.format(language, "profiles.avatar_pack_count", pack.avatars.size),
                    color = Color.White.copy(alpha = 0.5f),
                    fontSize = 12.sp
                )
            }
            Row(horizontalArrangement = Arrangement.spacedBy((-8).dp)) {
                pack.avatars.take(4).forEach { avatar ->
                    Box(
                        modifier = Modifier
                            .size(28.dp)
                            .clip(CircleShape)
                            .background(FluxaColors.surfaceCard)
                    ) {
                        FluxaRemoteImage(
                            imageUrl = avatar.url,
                            cacheKey = "avatar-pack-thumb:${avatar.url}",
                            contentDescription = null,
                            modifier = Modifier.fillMaxSize().clip(CircleShape),
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop
                        )
                    }
                }
            }
            Spacer(Modifier.width(12.dp))
            Box(
                modifier = Modifier.size(28.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.06f)).clickable(onClick = onRemove),
                contentAlignment = Alignment.Center
            ) {
                Icon(Icons.Filled.Close, contentDescription = null, tint = FluxaColors.errorRed, modifier = Modifier.size(14.dp))
            }
        }
    }
}
