package com.fluxa.app.shared.feature.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
internal fun SettingsTvRailRow(label: String, selected: Boolean, onClick: () -> Unit) {
    var focused by remember { mutableStateOf(false) }
    Row(
        modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp)).onFocusChanged { focused = it.isFocused }
            .background(if (focused) Color.White else if (selected) LocalSettingsAccentColor.current.copy(alpha = 0.16f) else Color.Transparent)
            .clickable(onClick = onClick).padding(horizontal = 14.dp, vertical = 12.dp)
    ) {
        Text(
            label,
            color = if (focused) Color.Black else if (selected) LocalSettingsAccentColor.current else Color.White.copy(alpha = 0.6f),
            fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium
        )
    }
}

internal fun settingsCategoryTitle(category: SettingsCategory, lang: String?): String = when (category) {
    SettingsCategory.Hub -> AppStrings.t(lang, "nav.settings")
    SettingsCategory.Account -> AppStrings.t(lang, "auto.account")
    SettingsCategory.TmdbFeatures -> AppStrings.t(lang, "settings.apis")
    SettingsCategory.Notifications -> AppStrings.t(lang, "settings.notifications_title")
    SettingsCategory.General -> AppStrings.t(lang, "auto.general")
    SettingsCategory.Appearance -> AppStrings.t(lang, "auto.appearance")
    SettingsCategory.AppearanceHome -> AppStrings.t(lang, "settings.appearance_home_screen")
    SettingsCategory.AppearanceDetail -> AppStrings.t(lang, "settings.appearance_detail_screen")
    SettingsCategory.Playback -> AppStrings.t(lang, "auto.playback")
    SettingsCategory.Subtitles -> AppStrings.t(lang, "auto.subtitles")
    SettingsCategory.Advanced -> AppStrings.t(lang, "settings.advanced_settings")
    SettingsCategory.Content -> AppStrings.t(lang, "auto.catalogs")
    SettingsCategory.Downloads -> AppStrings.t(lang, "auto.downloads")
    SettingsCategory.Developer -> AppStrings.t(lang, "settings.developer")
}

@Composable
internal fun SettingsTopBarTitle(title: String, showBack: Boolean, onBack: () -> Unit) {
    if (!showBack) {
        Text(
            title,
            color = Color.White,
            fontWeight = FontWeight.Black,
            fontSize = 32.sp,
            modifier = Modifier.padding(top = 8.dp, bottom = 20.dp)
        )
        return
    }
    Row(modifier = Modifier.fillMaxWidth().padding(vertical = 16.dp), verticalAlignment = Alignment.CenterVertically) {
        Box(
            modifier = Modifier
                .size(38.dp)
                .clip(CircleShape)
                .background(Color.White.copy(alpha = 0.06f))
                .clickable(onClick = onBack),
            contentAlignment = Alignment.Center
        ) {
            Icon(imageVector = Icons.AutoMirrored.Filled.ArrowBack, contentDescription = null, tint = Color.White, modifier = Modifier.size(19.dp))
        }
        Spacer(Modifier.width(12.dp))
        Text(title, color = Color.White, fontWeight = FontWeight.Bold, fontSize = 20.sp)
    }
}

@Composable
internal fun SettingsTopBarDivider() {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(1.dp)
            .background(Color.White.copy(alpha = 0.08f))
    )
}
