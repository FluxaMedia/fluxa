package com.fluxa.app.shared.feature.profile

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun ProfileSettingsScreen(
    profileState: ProfileUiState,
    settingsState: SettingsUiState,
    language: String?,
    onProfileSelected: (ProfileUiModel) -> Unit,
    onSettingsChanged: (SettingsUiState) -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(FluxaColors.background)
            .padding(20.dp),
        verticalArrangement = Arrangement.spacedBy(18.dp)
    ) {
        Text(
            text = AppStrings.t(language, "auto.profile"),
            color = Color.White,
            fontWeight = FontWeight.Bold
        )
        profileState.profiles.forEach { profile ->
            Text(
                text = profile.name,
                color = if (profile.id == profileState.activeProfile?.id) Color.White else Color.White.copy(alpha = 0.7f),
                fontWeight = if (profile.id == profileState.activeProfile?.id) FontWeight.Bold else FontWeight.Normal,
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable { onProfileSelected(profile) }
                    .padding(vertical = 6.dp)
            )
        }
        Text(
            text = AppStrings.t(language, "auto.interface"),
            color = Color.White,
            fontWeight = FontWeight.Bold
        )
        SettingsToggle(
            label = AppStrings.t(language, "auto.auto_play_next_episode"),
            value = settingsState.autoPlayNextEpisode,
            onValueChanged = { onSettingsChanged(settingsState.copy(autoPlayNextEpisode = it)) }
        )
        SettingsToggle(
            label = AppStrings.t(language, "auto.auto_enable_subtitles"),
            value = settingsState.subtitlesEnabled,
            onValueChanged = { onSettingsChanged(settingsState.copy(subtitlesEnabled = it)) }
        )
    }
}

@Composable
private fun SettingsToggle(label: String, value: Boolean, onValueChanged: (Boolean) -> Unit) {
    androidx.compose.foundation.layout.Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
    ) {
        Text(text = label, color = Color.White)
        Switch(checked = value, onCheckedChange = onValueChanged)
    }
}
