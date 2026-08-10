package com.fluxa.app.shared.feature.settings

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.LocalDeviceType

@Composable
internal fun SettingsPlaybackCoreContent(
    model: SettingsPlaybackUiModel,
    subtitles: SettingsSubtitlesUiModel,
    lang: String?,
    onAction: (SettingsAction) -> Unit,
    onNavigate: (SettingsCategory) -> Unit
) {
    val platformInAppOptions = model.inAppPlayerOptions.ifEmpty {
        when (LocalDeviceType.current) {
            DeviceType.Desktop -> listOf(
                SettingsChoiceOption("mpv", "libmpv"),
                SettingsChoiceOption("vlc", "libVLC"),
            )
            else -> listOf(
                SettingsChoiceOption("internal", "ExoPlayer"),
                SettingsChoiceOption("mpv", "libmpv"),
            )
        }
    }
    val playerOptions = platformInAppOptions +
        SettingsChoiceOption("external", AppStrings.t(lang, "settings.external_player"))
    val playbackSpeedOptions = listOf("0.75", "1.0", "1.25", "1.5").map { SettingsChoiceOption(it, "${it}x") }
    val seekOptions = listOf("10", "15", "30").map { SettingsChoiceOption(it, "${it}s") }
    val holdSpeedOptions = listOf("1.25", "1.5", "1.75", "2.0", "2.5", "3.0").map { SettingsChoiceOption(it, "${it}x") }

    SettingsSectionHeader(AppStrings.t(lang, "settings.section_playback_general"))
    SettingsGroupCard {
        SettingsNavRow(
            AppStrings.t(lang, "auto.subtitles"),
            value = languageOptionLabel(subtitles.preferredSubtitleLanguage, lang)
        ) { onNavigate(SettingsCategory.Subtitles) }
        SettingsNavRow(AppStrings.t(lang, "settings.advanced_settings")) { onNavigate(SettingsCategory.Advanced) }
    }

    SettingsSectionHeader(AppStrings.t(lang, "auto.playback"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "auto.player"), model.preferredPlayer, playerOptions) { onAction(SettingsAction.PlaybackChanged(model.copy(preferredPlayer = it))) }
        if (model.preferredPlayer == "mpv") {
            SettingsTextFieldRow(AppStrings.t(lang, "settings.mpv_custom_options"), model.mpvCustomOptions) { onAction(SettingsAction.PlaybackChanged(model.copy(mpvCustomOptions = it))) }
            SettingsChoiceRow(
                AppStrings.t(lang, "settings.anime_upscaling"),
                model.animeUpscalingMode,
                listOf(
                    SettingsChoiceOption("auto", AppStrings.t(lang, "settings.auto")),
                    SettingsChoiceOption("off", AppStrings.t(lang, "settings.off")),
                ),
            ) { onAction(SettingsAction.PlaybackChanged(model.copy(animeUpscalingMode = it))) }
            SettingsChoiceRow(
                AppStrings.t(lang, "settings.anime_upscaling_quality"),
                model.animeUpscalingQuality,
                listOf(
                    SettingsChoiceOption("anime4k_s", AppStrings.t(lang, "settings.anime4k_s")),
                    SettingsChoiceOption("anime4k_m", AppStrings.t(lang, "settings.anime4k_m")),
                    SettingsChoiceOption("anime4k_l", AppStrings.t(lang, "settings.anime4k_l")),
                ),
            ) { onAction(SettingsAction.PlaybackChanged(model.copy(animeUpscalingQuality = it))) }
        }
        if (model.preferredPlayer == "external") {
            val automaticLabelKey = if (LocalDeviceType.current == DeviceType.Desktop) {
                "settings.external_player_automatic"
            } else {
                "settings.external_player_ask_every_time"
            }
            val externalOptions = listOf(
                SettingsChoiceOption("", AppStrings.t(lang, automaticLabelKey))
            ) + model.externalPlayerOptions
            SettingsChoiceRow(
                AppStrings.t(lang, "settings.external_player_app"),
                model.externalPlayerTarget,
                externalOptions,
            ) {
                onAction(SettingsAction.PlaybackChanged(model.copy(externalPlayerTarget = it)))
            }
            if (LocalDeviceType.current == DeviceType.Desktop) {
                SettingsTextFieldRow(
                    AppStrings.t(lang, "settings.external_player_custom_command"),
                    model.externalPlayerTarget.takeIf { current ->
                        current.isNotBlank() && model.externalPlayerOptions.none { it.value == current }
                    }.orEmpty(),
                ) { custom ->
                    onAction(SettingsAction.PlaybackChanged(model.copy(externalPlayerTarget = custom)))
                }
            }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.anime_use_mpv"), value = model.animeUseMpv) { onAction(SettingsAction.PlaybackChanged(model.copy(animeUseMpv = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.anime_prefer_japanese_audio"), value = model.animePreferJapaneseAudio) {
            onAction(SettingsAction.PlaybackChanged(model.copy(animePreferJapaneseAudio = it)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "auto.playback_speed"), formatFloat(model.playbackSpeed), playbackSpeedOptions) {
            onAction(SettingsAction.PlaybackChanged(model.copy(playbackSpeed = it.toFloatOrNull() ?: model.playbackSpeed)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "auto.forward_rewind"), model.seekForwardSeconds.toString(), seekOptions) {
            val seconds = it.toIntOrNull() ?: model.seekForwardSeconds
            onAction(SettingsAction.PlaybackChanged(model.copy(seekForwardSeconds = seconds, seekBackwardSeconds = seconds)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.hold_to_speed"), value = model.holdToSpeedEnabled) { onAction(SettingsAction.PlaybackChanged(model.copy(holdToSpeedEnabled = it))) }
        if (model.holdToSpeedEnabled) {
            SettingsChoiceRow(AppStrings.t(lang, "settings.hold_speed"), formatFloat(model.holdSpeed), holdSpeedOptions) {
                onAction(SettingsAction.PlaybackChanged(model.copy(holdSpeed = it.toFloatOrNull() ?: model.holdSpeed)))
            }
        }
    }

    SettingsGroupCard {
        SettingsNavRow(AppStrings.t(lang, "settings.stream_settings")) { onNavigate(SettingsCategory.PlaybackStream) }
    }
}

@Composable
internal fun SettingsPlaybackStreamContent(
    model: SettingsPlaybackUiModel,
    lang: String?,
    onAction: (SettingsAction) -> Unit,
    onNavigate: (SettingsCategory) -> Unit
) {
    val streamSourceModeOptions = listOf(
        SettingsChoiceOption("manual", AppStrings.t(lang, "settings.stream_source_manual")),
        SettingsChoiceOption("first", AppStrings.t(lang, "settings.stream_source_first")),
        SettingsChoiceOption("regex", AppStrings.t(lang, "settings.stream_source_regex"))
    )
    val autoplayCountdownOptions = listOf("5", "7", "10", "15").map { SettingsChoiceOption(it, "${it}s") }

    SettingsSectionHeader(AppStrings.t(lang, "settings.stream_settings"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "settings.stream_source_selection"), model.streamSourceSelectionMode, streamSourceModeOptions) {
            onAction(SettingsAction.PlaybackChanged(model.copy(streamSourceSelectionMode = it)))
        }
        if (model.streamSourceSelectionMode == "regex") {
            SettingsTextFieldRow(AppStrings.t(lang, "settings.regex_pattern"), model.streamSourceRegexPattern) {
                onAction(SettingsAction.PlaybackChanged(model.copy(streamSourceRegexPattern = it)))
            }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.auto_play_next_episode"), value = model.autoplayMode == "next_episode") {
            onAction(SettingsAction.PlaybackChanged(model.copy(autoplayMode = if (it) "next_episode" else "off", autoPlayNextEpisode = it)))
        }
        if (model.autoplayMode == "next_episode") {
            SettingsChoiceRow(AppStrings.t(lang, "settings.autoplay_countdown"), model.autoPlayCountdownSecs.toString(), autoplayCountdownOptions) {
                onAction(SettingsAction.PlaybackChanged(model.copy(autoPlayCountdownSecs = it.toIntOrNull() ?: model.autoPlayCountdownSecs)))
            }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.auto_retry_next_source"), value = model.autoRetryNextSource) { onAction(SettingsAction.PlaybackChanged(model.copy(autoRetryNextSource = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.try_binge_group"), value = model.tryBingeGroup) { onAction(SettingsAction.PlaybackChanged(model.copy(tryBingeGroup = it))) }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.progress_thresholds"))
    SettingsGroupCard {
        SettingsPercentSliderRow(AppStrings.t(lang, "settings.next_episode_threshold"), model.nextEpisodeThresholdPercent) {
            onAction(SettingsAction.PlaybackChanged(model.copy(nextEpisodeThresholdPercent = it)))
        }
        SettingsPercentSliderRow(AppStrings.t(lang, "settings.watched_threshold"), model.watchedThresholdPercent) {
            onAction(SettingsAction.PlaybackChanged(model.copy(watchedThresholdPercent = it)))
        }
    }

    SettingsGroupCard {
        SettingsNavRow(AppStrings.t(lang, "settings.skip_segments")) { onNavigate(SettingsCategory.PlaybackSkip) }
    }
}

@Composable
internal fun SettingsPlaybackSkipContent(model: SettingsPlaybackUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    SettingsSectionHeader(AppStrings.t(lang, "settings.skip_segments"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.use_skip_segments"), value = model.useSkipSegments) { onAction(SettingsAction.PlaybackChanged(model.copy(useSkipSegments = it))) }
        if (model.useSkipSegments) {
            SettingsTextFieldRow(AppStrings.t(lang, "settings.introdb_api_key"), model.introDbApiKey) { onAction(SettingsAction.PlaybackChanged(model.copy(introDbApiKey = it))) }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.use_chapter_skip"), value = model.useChapterSkip) { onAction(SettingsAction.PlaybackChanged(model.copy(useChapterSkip = it))) }
        if (model.useSkipSegments) {
            SettingsToggleRow(AppStrings.t(lang, "settings.auto_skip"), value = model.autoSkipIntro) { onAction(SettingsAction.PlaybackChanged(model.copy(autoSkipIntro = it))) }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.content_warnings_enabled"), value = model.contentWarningsEnabled) { onAction(SettingsAction.PlaybackChanged(model.copy(contentWarningsEnabled = it))) }
    }

    var confirmingReset by remember { mutableStateOf(false) }
    SettingsGroupCard {
        SettingsActionRow(AppStrings.t(lang, "settings.reset_to_defaults"), destructive = true) {
            confirmingReset = true
        }
    }
    if (confirmingReset) {
        SettingsResetConfirmDialog(lang, onDismiss = { confirmingReset = false }) {
            confirmingReset = false
            onAction(SettingsAction.PlaybackChanged(SettingsPlaybackUiModel()))
        }
    }
}

internal fun formatFloat(value: Float): String {
    val rounded = (value * 100).toInt() / 100f
    return if (rounded == rounded.toInt().toFloat()) rounded.toInt().toString() + ".0" else rounded.toString()
}

@Composable
internal fun SettingsResetConfirmDialog(lang: String?, onDismiss: () -> Unit, onConfirm: () -> Unit) {
    androidx.compose.material3.AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(AppStrings.t(lang, "settings.reset_to_defaults")) },
        text = { Text(AppStrings.t(lang, "settings.reset_to_defaults_confirm")) },
        confirmButton = {
            TextButton(onClick = onConfirm) {
                Text(AppStrings.t(lang, "settings.reset_to_defaults"), color = FluxaColors.errorRed)
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(AppStrings.t(lang, "common.cancel"))
            }
        }
    )
}

@Composable
internal fun SettingsContentCategoryContent(model: SettingsContentUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.show_hero_section"), description = AppStrings.t(lang, "settings.show_hero_section_desc"), value = model.showHeroSection) {
            onAction(SettingsAction.ShowHeroSectionChanged(it))
        }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.hero_catalogs"))
    SettingsGroupCard {
        model.heroFeeds.forEach { feed ->
            SettingsOrderedToggleRow(
                label = feed.label,
                subtitle = feed.providerLabel,
                selected = feed.selected,
                canMoveUp = feed.canMoveUp,
                canMoveDown = feed.canMoveDown,
                onToggle = { onAction(SettingsAction.HeroFeedToggled(feed.key)) },
                onMoveUp = { onAction(SettingsAction.HeroFeedMoved(feed.key, -1)) },
                onMoveDown = { onAction(SettingsAction.HeroFeedMoved(feed.key, 1)) }
            )
        }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.home_catalogs"))
    SettingsGroupCard {
        model.homeFeeds.forEach { feed ->
            SettingsOrderedToggleRow(
                label = feed.label,
                subtitle = feed.providerLabel,
                selected = feed.selected,
                canMoveUp = feed.canMoveUp,
                canMoveDown = feed.canMoveDown,
                onToggle = { onAction(SettingsAction.HomeFeedToggled(feed.key)) },
                onMoveUp = { onAction(SettingsAction.HomeFeedMoved(feed.key, -1)) },
                onMoveDown = { onAction(SettingsAction.HomeFeedMoved(feed.key, 1)) }
            )
        }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.top_10_catalogs"))
    SettingsGroupCard {
        model.topTenFeeds.forEach { feed ->
            SettingsToggleRow(feed.label, value = feed.selected) { onAction(SettingsAction.TopTenFeedToggled(feed.key)) }
        }
    }
}

@Composable
internal fun SettingsDownloadsContent(model: SettingsDownloadsUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val streamSourceModeOptions = listOf(
        SettingsChoiceOption("manual", AppStrings.t(lang, "settings.stream_source_manual"), AppStrings.t(lang, "settings.stream_source_manual_desc")),
        SettingsChoiceOption("first", AppStrings.t(lang, "settings.stream_source_first"), AppStrings.t(lang, "settings.stream_source_first_desc")),
        SettingsChoiceOption("regex", AppStrings.t(lang, "settings.stream_source_regex"), AppStrings.t(lang, "settings.stream_source_regex_desc"))
    )
    val downloadSubtitleOptions = listOf(
        SettingsChoiceOption("preferred", AppStrings.t(lang, "settings.download_subtitle_preferred")),
        SettingsChoiceOption("off", AppStrings.t(lang, "settings.download_subtitle_off")),
        SettingsChoiceOption("tr", AppStrings.t(lang, "language.turkish")),
        SettingsChoiceOption("en", AppStrings.t(lang, "language.english"))
    )
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "settings.download_source_selection"), model.downloadSourceSelectionMode, streamSourceModeOptions) {
            onAction(SettingsAction.DownloadsChanged(model.copy(downloadSourceSelectionMode = it)))
        }
        if (model.downloadSourceSelectionMode == "regex") {
            SettingsTextFieldRow(AppStrings.t(lang, "settings.regex_pattern"), model.downloadSourceRegexPattern) {
                onAction(SettingsAction.DownloadsChanged(model.copy(downloadSourceRegexPattern = it)))
            }
        }
        SettingsChoiceRow(AppStrings.t(lang, "settings.download_subtitle"), model.downloadSubtitleLanguage, downloadSubtitleOptions) {
            onAction(SettingsAction.DownloadsChanged(model.copy(downloadSubtitleLanguage = it)))
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.source_modes"))
    SettingsInlineChoiceCards(
        options = streamSourceModeOptions,
        selected = model.downloadSourceSelectionMode
    ) { onAction(SettingsAction.DownloadsChanged(model.copy(downloadSourceSelectionMode = it))) }
}

@Composable
internal fun SettingsDeveloperContent(model: SettingsDeveloperUiModel, lang: String?) {
    SettingsSectionHeader(AppStrings.t(lang, "settings.last_media_probe"))
    SettingsGroupCard {
    if (model.lastProbeUpdatedAt == null) {
        Text(AppStrings.t(lang, "settings.no_media_probe"), color = Color.White.copy(alpha = 0.5f))
    } else {
        SettingsInfoRow(AppStrings.t(lang, "settings.last_media_probe_updated"), model.lastProbeUpdatedAt)
        SettingsInfoRow(AppStrings.t(lang, "settings.last_media_probe_title"), model.lastProbeTitle.orEmpty())
        SettingsInfoRow(AppStrings.t(lang, "settings.last_media_probe_url"), model.lastProbeUrl.orEmpty())
    }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.media_file_data"))
    SettingsGroupCard {
        Text(model.technicalInfo, color = Color.White.copy(alpha = 0.7f), fontSize = 12.sp, modifier = Modifier.padding(vertical = 10.dp))
    }
}
