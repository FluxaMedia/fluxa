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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
internal fun PlaybackSettings(
    profile: UserProfile,
    lang: String,
    onOpenSubtitles: () -> Unit,
    onOpenAdvanced: () -> Unit,
    onUpdateProfile: (UserProfile) -> Unit
) {
    SettingsSection(AppStrings.t(lang, "auto.playback"), AppStrings.t(lang, "auto.shape_how_playback_behaves")) {
        SettingsActionTile(
            title = AppStrings.t(lang, "auto.subtitles"),
            subtitle = AppStrings.t(lang, "settings.subtitles_settings_desc"),
            icon = FluxaIcons.Subtitles,
            onClick = onOpenSubtitles
        )
        SettingsActionTile(
            title = AppStrings.t(lang, "settings.advanced_settings"),
            subtitle = AppStrings.t(lang, "settings.advanced_settings_desc"),
            icon = FluxaIcons.Speed,
            onClick = onOpenAdvanced
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.skip_forward"),
            subtitle = AppStrings.t(lang, "auto.how_far_right_arrow_should_jump"),
            options = listOf(ChoiceOption("10", "10s"), ChoiceOption("15", "15s"), ChoiceOption("30", "30s")),
            selected = profile.safeSeekForwardSeconds.toString(),
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(seekForwardSeconds = it.toInt())) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.skip_back"),
            subtitle = AppStrings.t(lang, "auto.how_far_left_arrow_should_rewind"),
            options = listOf(ChoiceOption("10", "10s"), ChoiceOption("15", "15s"), ChoiceOption("30", "30s")),
            selected = profile.safeSeekBackwardSeconds.toString(),
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(seekBackwardSeconds = it.toInt())) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.use_introdb"),
            subtitle = AppStrings.t(lang, "settings.use_introdb_desc"),
            checked = profile.safeUseIntroDb,
            onToggle = { onUpdateProfile(profile.sanitizedUpdate(useIntroDb = it)) }
        )
        if (profile.safeUseIntroDb) {
            SettingsTextFieldTile(
                title = AppStrings.t(lang, "settings.introdb_api_key"),
                subtitle = AppStrings.t(lang, "settings.introdb_api_key_desc"),
                value = profile.safeIntroDbApiKey,
                placeholder = "",
                onValueChange = { onUpdateProfile(profile.copy(introDbApiKey = it.trim())) }
            )
        }
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.use_aniskip"),
            subtitle = AppStrings.t(lang, "settings.use_aniskip_desc"),
            checked = profile.safeUseAniSkip,
            onToggle = { onUpdateProfile(profile.sanitizedUpdate(useAniSkip = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.use_chapter_skip"),
            subtitle = AppStrings.t(lang, "settings.use_chapter_skip_desc"),
            checked = profile.safeUseChapterSkip,
            onToggle = { onUpdateProfile(profile.sanitizedUpdate(useChapterSkip = it)) }
        )
        if (profile.safeUseIntroDb || profile.safeUseAniSkip) {
            SettingsToggleTile(
                title = AppStrings.t(lang, "auto.auto_skip_intros"),
                subtitle = AppStrings.t(lang, "auto.prefer_skipping_intros_where_supported"),
                checked = profile.safeAutoSkipIntro,
                onToggle = { onUpdateProfile(profile.sanitizedUpdate(autoSkipIntro = it)) }
            )
        }
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.stream_source_selection"),
            subtitle = AppStrings.t(lang, "settings.stream_source_selection_desc"),
            options = streamSourceSelectionOptions(lang),
            selected = profile.safeStreamSourceSelectionMode,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(streamSourceSelectionMode = it)) }
        )
        if (profile.safeStreamSourceSelectionMode == STREAM_SOURCE_MODE_REGEX) {
            SettingsTextFieldTile(
                title = AppStrings.t(lang, "settings.regex_pattern"),
                subtitle = AppStrings.t(lang, "settings.regex_pattern_desc"),
                value = profile.safeStreamSourceRegexPattern,
                placeholder = AppStrings.t(lang, "settings.regex_pattern_placeholder"),
                onValueChange = { onUpdateProfile(profile.sanitizedUpdate(streamSourceRegexPattern = it)) }
            )
        }
        SettingsToggleTile(
            title = AppStrings.t(lang, "auto.auto_play_next_episode"),
            subtitle = AppStrings.t(lang, "auto.queue_the_next_episode_automatically"),
            checked = profile.safeAutoPlayNextEpisode,
            onToggle = { onUpdateProfile(profile.sanitizedUpdate(autoPlayNextEpisode = it)) }
        )
        if (profile.safeAutoPlayNextEpisode) {
            SettingsChoiceTile(
                title = AppStrings.t(lang, "settings.autoplay_countdown"),
                subtitle = AppStrings.t(lang, "settings.autoplay_countdown_desc"),
                options = listOf(
                    ChoiceOption("5", "5s"),
                    ChoiceOption("7", "7s"),
                    ChoiceOption("10", "10s"),
                    ChoiceOption("15", "15s")
                ),
                selected = profile.safeAutoPlayCountdownSecs.toString(),
                onSelect = { onUpdateProfile(profile.copy(autoPlayCountdownSecs = it.toInt())) }
            )
        }
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.auto_retry_next_source"),
            subtitle = AppStrings.t(lang, "settings.auto_retry_next_source_desc"),
            checked = profile.safeAutoRetryNextSource,
            onToggle = { onUpdateProfile(profile.copy(autoRetryNextSource = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.try_binge_group"),
            subtitle = AppStrings.t(lang, "settings.try_binge_group_desc"),
            checked = profile.safeTryBingeGroup,
            onToggle = { onUpdateProfile(profile.sanitizedUpdate(tryBingeGroup = it)) }
        )
        SettingsToggleTile(
            title = AppStrings.t(lang, "settings.anime_prefer_japanese_audio"),
            subtitle = AppStrings.t(lang, "settings.anime_prefer_japanese_audio_desc"),
            checked = profile.safeAnimePreferJapaneseAudio,
            onToggle = { onUpdateProfile(profile.copy(animePreferJapaneseAudio = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "auto.preferred_player"),
            subtitle = AppStrings.t(lang, "auto.choose_between_internal_or_external_playback"),
            options = listOf(
                ChoiceOption("exoplayer", "ExoPlayer"),
                ChoiceOption("mpv", "MPV")
            ),
            selected = profile.safePreferredPlayer,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(preferredPlayer = it)) }
        )
        SettingsChoiceTile(
            title = AppStrings.t(lang, "settings.dv_fallback"),
            subtitle = AppStrings.t(lang, "settings.dv_fallback_desc"),
            options = dolbyVisionFallbackOptions(lang),
            selected = profile.safeDolbyVisionFallbackMode,
            onSelect = { onUpdateProfile(profile.sanitizedUpdate(dolbyVisionFallbackMode = it)) }
        )
        if (profile.safeDolbyVisionFallbackMode == "auto") {
            SettingsChoiceTile(
                title = AppStrings.t(lang, "settings.dv_rpu_mode"),
                subtitle = AppStrings.t(lang, "settings.dv_rpu_mode_desc"),
                options = dvRpuModeOptions(lang),
                selected = profile.safeDvRpuMode.toString(),
                onSelect = { onUpdateProfile(profile.sanitizedUpdate(dvRpuMode = it.toIntOrNull() ?: 2)) }
            )
            SettingsToggleTile(
                title = AppStrings.t(lang, "settings.dv_zero_level5"),
                subtitle = AppStrings.t(lang, "settings.dv_zero_level5_desc"),
                checked = profile.safeDvZeroLevel5,
                onToggle = { onUpdateProfile(profile.sanitizedUpdate(dvZeroLevel5 = it)) }
            )
            SettingsChoiceTile(
                title = AppStrings.t(lang, "settings.dv_hdr10plus_mode"),
                subtitle = AppStrings.t(lang, "settings.dv_hdr10plus_mode_desc"),
                options = dvHdr10PlusModeOptions(lang),
                selected = profile.safeDvHdr10PlusMode,
                onSelect = { onUpdateProfile(profile.sanitizedUpdate(dvHdr10PlusMode = it)) }
            )
        }
    }
}
