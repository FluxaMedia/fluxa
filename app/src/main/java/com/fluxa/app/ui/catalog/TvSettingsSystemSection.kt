@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch

@Composable
internal fun DownloadSettings(profile: UserProfile, lang: String, onUpdateProfile: (UserProfile) -> Unit) {
    SettingsSection(AppStrings.t(lang, "auto.downloads"), AppStrings.t(lang, "settings.downloads_desc")) {
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.download_source_selection"),
            subtitle = AppStrings.t(lang, "settings.download_source_selection_desc"),
            options = streamSourceSelectionOptions(lang),
            selected = profile.safeDownloadSourceSelectionMode,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(downloadSourceSelectionMode = it)) }
        )
        if (profile.safeDownloadSourceSelectionMode == STREAM_SOURCE_MODE_REGEX) {
            SettingsTextFieldTile(
                title = AppStrings.t(lang, "settings.regex_pattern"),
                subtitle = AppStrings.t(lang, "settings.regex_pattern_desc"),
                value = profile.safeDownloadSourceRegexPattern,
                placeholder = AppStrings.t(lang, "settings.regex_pattern_placeholder"),
                onValueChange = { onUpdateProfile(profile.sanitizedUpdate(downloadSourceRegexPattern = it)) }
            )
        }
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.download_subtitle"),
            subtitle = AppStrings.t(lang, "settings.download_subtitle_desc"),
            options = downloadSubtitleOptions(lang),
            selected = profile.safeDownloadSubtitleLanguage,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(downloadSubtitleLanguage = it)) }
        )
    }
}

@Composable
internal fun SubtitleSettings(profile: UserProfile, lang: String, onUpdateProfile: (UserProfile) -> Unit) {
    SettingsSection(AppStrings.t(lang, "auto.subtitles_fc449c82"), AppStrings.t(lang, "auto.subtitle_language_size_and_readability")) {
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.preferred_audio_language"),
            subtitle = AppStrings.t(lang, "settings.preferred_audio_language_desc"),
            options = audioLanguageOptions(lang),
            selected = profile.safePreferredAudioLanguage,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(preferredAudioLanguage = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.secondary_audio_language"),
            subtitle = AppStrings.t(lang, "settings.secondary_audio_language_desc"),
            options = audioLanguageOptions(lang),
            selected = profile.safeSecondaryAudioLanguage,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(secondaryAudioLanguage = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "auto.auto_enable_subtitles"),
            subtitle = AppStrings.t(lang, "auto.enable_subtitles_automatically_when_availabl"),
            checked = profile.safeAutoEnableSubtitles,
            onToggle = { onUpdateProfile(profile.copy(autoEnableSubtitles = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.subtitle_shadow"),
            subtitle = AppStrings.t(lang, "settings.subtitle_shadow_desc"),
            checked = profile.safeSubtitleShadow,
            onToggle = { onUpdateProfile(profile.copy(subtitleShadow = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.preferred_subtitle_language"),
            subtitle = AppStrings.t(lang, "auto.default_subtitle_track_preference"),
            options = subtitleLanguageOptions(lang),
            selected = profile.safePreferredSubtitleLanguage,
            onSelect = { onUpdateProfile(profile.copy(preferredSubtitleLanguage = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.secondary_subtitle_language"),
            subtitle = AppStrings.t(lang, "settings.secondary_subtitle_language_desc"),
            options = subtitleLanguageOptions(lang),
            selected = profile.safeSecondarySubtitleLanguage,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(secondarySubtitleLanguage = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.subtitle_size"),
            subtitle = AppStrings.t(lang, "auto.tune_readability_on_tv_and_mobile"),
            options = listOf(
                ChoiceOption("18", "18"),
                ChoiceOption("20", "20"),
                ChoiceOption("24", "24"),
                ChoiceOption("28", "28")
            ),
            selected = profile.safeSubtitleSize.toInt().toString(),
            onSelect = { onUpdateProfile(profile.copy(subtitleSize = it.toFloat())) }
        )
        SettingsColorOpacityTile(
            title = AppStrings.t(lang, "settings.subtitle_text"),
            subtitle = AppStrings.t(lang, "settings.subtitle_text_desc"),
            opacityTitle = AppStrings.t(lang, "auto.text_transparency"),
            colorOptions = mobileSubtitleColorOptions(lang),
            selectedColor = profile.safeSubtitleColor,
            opacity = profile.safeSubtitleTextOpacity,
            onColorSelect = { onUpdateProfile(profile.copy(subtitleColor = it)) },
            onOpacity = { onUpdateProfile(profile.copy(subtitleTextOpacity = it)) }
        )
        SettingsColorOpacityTile(
            title = AppStrings.t(lang, "settings.subtitle_background"),
            subtitle = AppStrings.t(lang, "settings.subtitle_background_desc"),
            opacityTitle = AppStrings.t(lang, "auto.background_transparency"),
            colorOptions = mobileSubtitleColorOptions(lang),
            selectedColor = profile.safeSubtitleBackgroundColor,
            opacity = profile.safeSubtitleBackgroundOpacity,
            onColorSelect = { onUpdateProfile(profile.copy(subtitleBackgroundColor = it)) },
            onOpacity = { onUpdateProfile(profile.copy(subtitleBackgroundOpacity = it)) }
        )
        SettingsColorOpacityTile(
            title = AppStrings.t(lang, "settings.subtitle_outline"),
            subtitle = AppStrings.t(lang, "settings.subtitle_outline_desc"),
            opacityTitle = AppStrings.t(lang, "settings.subtitle.outline_opacity"),
            colorOptions = mobileSubtitleColorOptions(lang),
            selectedColor = profile.safeSubtitleOutlineColor,
            opacity = profile.safeSubtitleOutlineOpacity,
            onColorSelect = { onUpdateProfile(profile.copy(subtitleOutlineColor = it)) },
            onOpacity = { onUpdateProfile(profile.copy(subtitleOutlineOpacity = it)) }
        )
        androidx.compose.foundation.layout.Box(
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = 16.dp)
                .clip(RoundedCornerShape(12.dp))
                .background(Color.Black.copy(alpha = 0.6f))
                .padding(24.dp),
            contentAlignment = Alignment.Center
        ) {
            SubtitlePreviewText(
                text = AppStrings.t(lang, "settings.subtitle_preview"),
                profile = profile
            )
        }
    }
}

@Composable
internal fun AddonSettings(profile: UserProfile, lang: String, onManageAddons: () -> Unit, onUpdateProfile: (UserProfile) -> Unit) {
    SettingsSection(AppStrings.t(lang, "auto.add_ons_and_torrent"), AppStrings.t(lang, "auto.sources_and_stream_engine_preferences")) {
        SettingsActionTile(
            title = AppStrings.t(lang, "auto.manage_add_ons"),
            subtitle = AppStrings.t(lang, "auto.installed_add_ons_and_settings"),
            icon = FluxaIcons.Extension,
            onClick = onManageAddons
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.torrent_speed"),
            subtitle = AppStrings.t(lang, "settings.torrent_speed_desc"),
            options = torrentSpeedPresetOptions(lang),
            selected = profile.safeTorrentSpeedPreset,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(torrentSpeedPreset = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.torrent_wifi_only"),
            subtitle = AppStrings.t(lang, "settings.torrent_wifi_only_desc"),
            checked = profile.safeTorrentWifiOnly,
            onToggle = { onUpdateProfile(profile.copy(torrentWifiOnly = it)) }
        )
    }
}

@Composable
internal fun SystemSettings(profile: UserProfile, lang: String, onReboot: () -> Unit, onLogout: () -> Unit, onUpdateProfile: (UserProfile) -> Unit) {
    val scope = rememberCoroutineScope()
    var isCheckingUpdate by remember { mutableStateOf(false) }
    var updateStatus by remember { mutableStateOf<String?>(null) }
    SettingsSection(AppStrings.t(lang, "auto.system"), AppStrings.t(lang, "auto.app_and_device_actions")) {
        SettingsInfoTile(
            AppStrings.t(lang, "auto.version"),
            "${com.fluxa.app.BuildConfig.VERSION_NAME} (${com.fluxa.app.BuildConfig.VERSION_CODE})",
            FluxaIcons.Settings
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.automatic_updates"),
            subtitle = AppStrings.t(lang, "settings.automatic_updates_desc"),
            checked = profile.safeAutomaticUpdates,
            onToggle = { onUpdateProfile(profile.copy(automaticUpdates = it)) }
        )
        SettingsActionTile(
            title = AppStrings.t(lang, "settings.check_for_updates"),
            subtitle = updateStatus ?: AppStrings.t(lang, "settings.check_for_updates_desc"),
            icon = FluxaIcons.Settings,
            onClick = {
                if (!isCheckingUpdate) {
                    isCheckingUpdate = true
                    updateStatus = AppStrings.t(lang, "settings.checking_for_updates")
                    scope.launch {
                        val update = UpdateManager.checkUpdate()
                        isCheckingUpdate = false
                        updateStatus = if (update != null)
                            AppStrings.format(lang, "settings.update_available", update.versionName)
                        else
                            AppStrings.t(lang, "settings.up_to_date")
                    }
                }
            }
        )
        SettingsActionTile(
            title = AppStrings.t(lang, "auto.restart_app"),
            subtitle = AppStrings.t(lang, "auto.refresh_the_app_after_playback_or_network_ch"),
            icon = FluxaIcons.Settings,
            onClick = onReboot
        )
        SettingsActionTile(
            title = AppStrings.t(lang, "auto.log_out"),
            subtitle = AppStrings.t(lang, "auto.sign_out_from_the_active_profile_on_this_dev"),
            icon = FluxaIcons.Logout,
            accent = FluxaColors.errorRed,
            onClick = onLogout
        )
    }
}
