package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.data.local.UserProfile

internal fun LazyListScope.mobilePlaybackSettingsSection(
    profile: UserProfile,
    lang: String,
    onChoiceDialog: (MobileChoiceDialogState) -> Unit,
    onUpdateProfile: (UserProfile) -> Unit
) {
    item {
        MobileSettingsGroup(AppStrings.t(lang, "auto.playback_dbc1ddba")) {
            MobileChoiceRow(
                title = AppStrings.t(lang, "auto.default_player"),
                value = playerChoiceLabel(profile.safePreferredPlayer, lang),
                subtitle = AppStrings.t(lang, "settings.default_player_desc"),
                onClick = {
                    onChoiceDialog(
                        MobileChoiceDialogState(
                            title = AppStrings.t(lang, "auto.player"),
                            options = listOf(
                                ChoiceOption("exoplayer", AppStrings.t(lang, "player.exoplayer")),
                                ChoiceOption("mpv", AppStrings.t(lang, "player.mpv"))
                            ),
                            selected = profile.safePreferredPlayer,
                            onSelect = { onUpdateProfile(profile.copy(preferredPlayer = it)) }
                        )
                    )
                }
            )
            if (profile.safePreferredPlayer == "mpv") {
                MobileMpvOptionsRow(
                    lang = lang,
                    value = profile.safeMpvCustomOptions,
                    onValueChange = { onUpdateProfile(profile.copy(mpvCustomOptions = it.ifBlank { null })) }
                )
                MobileMpvScriptsDirRow(lang = lang)
            }
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.anime_use_mpv"),
                subtitle = AppStrings.t(lang, "settings.anime_use_mpv_desc"),
                checked = profile.safeAnimeUseMpv,
                onToggle = { onUpdateProfile(profile.sanitizedUpdate(animeUseMpv = !profile.safeAnimeUseMpv)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.anime_prefer_japanese_audio"),
                subtitle = AppStrings.t(lang, "settings.anime_prefer_japanese_audio_desc"),
                checked = profile.safeAnimePreferJapaneseAudio,
                onToggle = { onUpdateProfile(profile.copy(animePreferJapaneseAudio = !profile.safeAnimePreferJapaneseAudio)) }
            )
            MobileChoiceRow(
                AppStrings.t(lang, "auto.playback_speed"),
                mobilePlaybackSpeedLabel(profile.safePlaybackSpeed),
                subtitle = AppStrings.t(lang, "settings.playback_speed_desc")
            ) {
                onChoiceDialog(
                    MobileChoiceDialogState(
                        title = AppStrings.t(lang, "auto.playback_speed"),
                        options = listOf(
                            ChoiceOption("0.75", "0.75x"),
                            ChoiceOption("1.0", "1.00x"),
                            ChoiceOption("1.25", "1.25x"),
                            ChoiceOption("1.5", "1.50x")
                        ),
                        selected = profile.safePlaybackSpeed.toString(),
                        onSelect = { onUpdateProfile(profile.copy(playbackSpeed = it.toFloat())) }
                    )
                )
            }
            MobileChoiceRow(
                title = AppStrings.t(lang, "auto.forward_rewind"),
                value = settingsSecondsLabel(lang, profile.safeSeekForwardSeconds),
                subtitle = AppStrings.t(lang, "settings.forward_rewind_desc"),
                onClick = {
                    onChoiceDialog(
                        MobileChoiceDialogState(
                            title = AppStrings.t(lang, "auto.forward_rewind"),
                            options = listOf(
                                ChoiceOption("10", settingsSecondsLabel(lang, 10)),
                                ChoiceOption("15", settingsSecondsLabel(lang, 15)),
                                ChoiceOption("30", settingsSecondsLabel(lang, 30))
                            ),
                            selected = profile.safeSeekForwardSeconds.toString(),
                            onSelect = {
                                val seconds = it.toInt()
                                onUpdateProfile(profile.sanitizedUpdate(seekForwardSeconds = seconds, seekBackwardSeconds = seconds))
                            }
                        )
                    )
                }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.hold_to_speed"),
                subtitle = AppStrings.t(lang, "settings.hold_to_speed_desc"),
                checked = profile.safeHoldToSpeedEnabled,
                onToggle = { onUpdateProfile(profile.sanitizedUpdate(holdToSpeedEnabled = !profile.safeHoldToSpeedEnabled)) }
            )
            MobileChoiceRow(
                title = AppStrings.t(lang, "settings.hold_speed"),
                value = "${profile.safeHoldSpeed}x",
                subtitle = AppStrings.t(lang, "settings.hold_speed_desc"),
                onClick = {
                    onChoiceDialog(
                        MobileChoiceDialogState(
                            title = AppStrings.t(lang, "settings.hold_speed"),
                            options = listOf(
                                ChoiceOption("1.25", "1.25x"),
                                ChoiceOption("1.5", "1.50x"),
                                ChoiceOption("1.75", "1.75x"),
                                ChoiceOption("2.0", "2.00x"),
                                ChoiceOption("2.5", "2.50x"),
                                ChoiceOption("3.0", "3.00x")
                            ),
                            selected = profile.safeHoldSpeed.toString(),
                            onSelect = { onUpdateProfile(profile.sanitizedUpdate(holdSpeed = it.toFloat())) }
                        )
                    )
                }
            )
        }
    }
    item {
        MobileSettingsGroup(AppStrings.t(lang, "settings.stream_settings")) {
            MobileChoiceRow(
                title = AppStrings.t(lang, "settings.stream_source_selection"),
                value = streamSourceSelectionLabel(profile.safeStreamSourceSelectionMode, lang),
                subtitle = AppStrings.t(lang, "settings.stream_source_selection_desc"),
                onClick = {
                    onChoiceDialog(
                        MobileChoiceDialogState(
                            title = AppStrings.t(lang, "settings.stream_source_selection"),
                            options = streamSourceSelectionOptions(lang),
                            selected = profile.safeStreamSourceSelectionMode,
                            onSelect = { onUpdateProfile(profile.sanitizedUpdate(streamSourceSelectionMode = it)) }
                        )
                    )
                }
            )
            if (profile.safeStreamSourceSelectionMode == STREAM_SOURCE_MODE_REGEX) {
                MobileRegexPatternRow(
                    lang = lang,
                    value = profile.safeStreamSourceRegexPattern,
                    onValueChange = { onUpdateProfile(profile.sanitizedUpdate(streamSourceRegexPattern = it)) }
                )
            }
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.auto_play_next_episode"),
                subtitle = AppStrings.t(lang, "settings.auto_play_next_episode_desc"),
                checked = profile.safeAutoplayMode != "off",
                onToggle = {
                    val enabled = profile.safeAutoplayMode == "off"
                    onUpdateProfile(profile.sanitizedUpdate(autoplayMode = if (enabled) "next_episode" else "off", autoPlayNextEpisode = enabled))
                }
            )
            if (profile.safeAutoplayMode != "off") {
                MobileChoiceRow(
                    title = AppStrings.t(lang, "settings.autoplay_countdown"),
                    value = "${profile.safeAutoPlayCountdownSecs}s",
                    subtitle = AppStrings.t(lang, "settings.autoplay_countdown_desc")
                ) {
                    onChoiceDialog(
                        MobileChoiceDialogState(
                            title = AppStrings.t(lang, "settings.autoplay_countdown"),
                            options = listOf(
                                ChoiceOption("5", "5s"),
                                ChoiceOption("7", "7s"),
                                ChoiceOption("10", "10s"),
                                ChoiceOption("15", "15s")
                            ),
                            selected = profile.safeAutoPlayCountdownSecs.toString(),
                            onSelect = { onUpdateProfile(profile.copy(autoPlayCountdownSecs = it.toInt())) }
                        )
                    )
                }
            }
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.auto_retry_next_source"),
                subtitle = AppStrings.t(lang, "settings.auto_retry_next_source_desc"),
                checked = profile.safeAutoRetryNextSource,
                onToggle = { onUpdateProfile(profile.copy(autoRetryNextSource = !profile.safeAutoRetryNextSource)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.try_binge_group"),
                subtitle = AppStrings.t(lang, "settings.try_binge_group_desc"),
                checked = profile.safeTryBingeGroup,
                onToggle = { onUpdateProfile(profile.sanitizedUpdate(tryBingeGroup = !profile.safeTryBingeGroup)) }
            )
            MobilePercentSliderRow(
                title = AppStrings.t(lang, "settings.next_episode_threshold"),
                value = profile.safeNextEpisodeThresholdPercent,
                subtitle = AppStrings.t(lang, "settings.next_episode_threshold_desc"),
                onValueChange = { onUpdateProfile(profile.sanitizedUpdate(nextEpisodeThresholdPercent = it)) }
            )
            MobilePercentSliderRow(
                title = AppStrings.t(lang, "settings.watched_threshold"),
                value = profile.safeWatchedThresholdPercent,
                subtitle = AppStrings.t(lang, "settings.watched_threshold_desc"),
                onValueChange = { onUpdateProfile(profile.sanitizedUpdate(watchedThresholdPercent = it)) }
            )
        }
    }
    item {
        MobileSettingsGroup(AppStrings.t(lang, "settings.skip_segments")) {
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.use_introdb"),
                subtitle = AppStrings.t(lang, "settings.use_introdb_desc"),
                checked = profile.safeUseIntroDb,
                onToggle = { onUpdateProfile(profile.sanitizedUpdate(useIntroDb = !profile.safeUseIntroDb)) }
            )
            if (profile.safeUseIntroDb) {
                MobileIntroDbApiKeyField(
                    lang = lang,
                    value = profile.safeIntroDbApiKey,
                    onValueChange = { onUpdateProfile(profile.copy(introDbApiKey = it.trim())) }
                )
            }
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.use_aniskip"),
                subtitle = AppStrings.t(lang, "settings.use_aniskip_desc"),
                checked = profile.safeUseAniSkip,
                onToggle = { onUpdateProfile(profile.sanitizedUpdate(useAniSkip = !profile.safeUseAniSkip)) }
            )
            MobileToggleRow(
                title = AppStrings.t(lang, "settings.use_chapter_skip"),
                subtitle = AppStrings.t(lang, "settings.use_chapter_skip_desc"),
                checked = profile.safeUseChapterSkip,
                onToggle = { onUpdateProfile(profile.sanitizedUpdate(useChapterSkip = !profile.safeUseChapterSkip)) }
            )
            if (profile.safeUseIntroDb || profile.safeUseAniSkip) {
                MobileToggleRow(
                    title = AppStrings.t(lang, "settings.auto_skip"),
                    subtitle = AppStrings.t(lang, "settings.auto_skip_desc"),
                    checked = profile.safeAutoSkipIntro,
                    onToggle = { onUpdateProfile(profile.sanitizedUpdate(autoSkipIntro = !profile.safeAutoSkipIntro)) }
                )
            }
        }
    }
}

@Composable
private fun MobileIntroDbApiKeyField(
    lang: String,
    value: String,
    onValueChange: (String) -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    Column(modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp)) {
        Text(
            text = AppStrings.t(lang, "settings.introdb_api_key"),
            color = colors.text.copy(alpha = 0.9f),
            fontWeight = androidx.compose.ui.text.font.FontWeight.Bold
        )
        Text(
            text = AppStrings.t(lang, "settings.introdb_api_key_desc"),
            color = colors.text.copy(alpha = 0.56f),
            fontSize = 12.sp,
            modifier = Modifier.padding(bottom = 6.dp)
        )
        OutlinedTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = Modifier.fillMaxWidth(),
            singleLine = true,
            textStyle = androidx.compose.ui.text.TextStyle(fontSize = 13.sp, color = colors.text),
            placeholder = { Text("idb_...", fontSize = 13.sp, color = colors.text.copy(alpha = 0.32f)) },
            colors = OutlinedTextFieldDefaults.colors(
                focusedBorderColor = colors.accent,
                unfocusedBorderColor = colors.text.copy(alpha = 0.16f),
                focusedTextColor = colors.text,
                unfocusedTextColor = colors.text,
                cursorColor = colors.accent,
                focusedContainerColor = Color.Transparent,
                unfocusedContainerColor = Color.Transparent
            )
        )
    }
}

@Composable
private fun MobileMpvOptionsRow(
    lang: String,
    value: String,
    onValueChange: (String) -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    Column(modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp)) {
        Text(
            text = AppStrings.t(lang, "settings.mpv_custom_options"),
            color = colors.text.copy(alpha = 0.9f),
            fontWeight = androidx.compose.ui.text.font.FontWeight.Bold
        )
        Text(
            text = AppStrings.t(lang, "settings.mpv_custom_options_desc"),
            color = colors.text.copy(alpha = 0.56f),
            fontSize = 12.sp,
            modifier = Modifier.padding(bottom = 6.dp)
        )
        OutlinedTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = Modifier.fillMaxWidth(),
            minLines = 4,
            maxLines = 12,
            textStyle = androidx.compose.ui.text.TextStyle(
                fontFamily = FontFamily.Monospace,
                fontSize = 13.sp,
                color = colors.text
            ),
            placeholder = {
                Text(
                    "# one option per line\nsub-scale=1.2\nvolume-max=200",
                    fontFamily = FontFamily.Monospace,
                    fontSize = 13.sp,
                    color = colors.text.copy(alpha = 0.32f)
                )
            },
            colors = OutlinedTextFieldDefaults.colors(
                focusedBorderColor = colors.accent,
                unfocusedBorderColor = colors.text.copy(alpha = 0.16f),
                focusedTextColor = colors.text,
                unfocusedTextColor = colors.text,
                cursorColor = colors.accent,
                focusedContainerColor = Color.Transparent,
                unfocusedContainerColor = Color.Transparent
            )
        )
    }
}

@Composable
private fun MobileMpvScriptsDirRow(lang: String) {
    val colors = LocalMobileSettingsPalette.current
    val context = LocalContext.current
    val clipboard = remember(context) {
        context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    }
    val scriptsDir = remember(context) {
        context.filesDir.resolve("mpv/scripts").absolutePath
    }
    MobileActionRow(
        title = AppStrings.t(lang, "settings.mpv_scripts_dir"),
        value = scriptsDir,
        onClick = {
            clipboard.setPrimaryClip(
                ClipData.newPlainText(AppStrings.t(lang, "settings.mpv_scripts_dir"), scriptsDir)
            )
        }
    )
}

@Composable
private fun MobileRegexPatternRow(
    lang: String,
    value: String,
    onValueChange: (String) -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    Column(modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp)) {
        Text(
            text = AppStrings.t(lang, "settings.regex_pattern"),
            color = colors.text.copy(alpha = 0.9f),
            fontWeight = androidx.compose.ui.text.font.FontWeight.Bold
        )
        OutlinedTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = Modifier.fillMaxWidth(),
            singleLine = true,
            placeholder = { Text(AppStrings.t(lang, "settings.regex_pattern_placeholder")) },
            colors = OutlinedTextFieldDefaults.colors(
                focusedBorderColor = colors.accent,
                unfocusedBorderColor = colors.text.copy(alpha = 0.16f),
                focusedTextColor = colors.text,
                unfocusedTextColor = colors.text,
                cursorColor = colors.accent,
                focusedPlaceholderColor = colors.text.copy(alpha = 0.36f),
                unfocusedPlaceholderColor = colors.text.copy(alpha = 0.28f),
                focusedContainerColor = Color.Transparent,
                unfocusedContainerColor = Color.Transparent
            )
        )
    }
}
