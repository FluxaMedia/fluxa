package com.fluxa.app.shared.feature.settings

import androidx.compose.runtime.Composable
import com.fluxa.app.common.AppStrings

@Composable
internal fun SettingsSubtitlesContent(model: SettingsSubtitlesUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val languageOptions = listOf("none", "original", "device_language", "en", "tr", "ja", "ko", "es", "fr", "de").map { SettingsChoiceOption(it, languageOptionLabel(it, lang)) }
    SettingsSectionHeader(AppStrings.t(lang, "settings.preferences"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "settings.preferred_audio_language"), model.preferredAudioLanguage, languageOptions) { onAction(SettingsAction.SubtitlesChanged(model.copy(preferredAudioLanguage = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "settings.secondary_audio_language"), model.secondaryAudioLanguage, languageOptions) { onAction(SettingsAction.SubtitlesChanged(model.copy(secondaryAudioLanguage = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "settings.preferred_subtitle_language"), model.preferredSubtitleLanguage, languageOptions) { onAction(SettingsAction.SubtitlesChanged(model.copy(preferredSubtitleLanguage = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "settings.secondary_subtitle_language"), model.secondarySubtitleLanguage, languageOptions) { onAction(SettingsAction.SubtitlesChanged(model.copy(secondarySubtitleLanguage = it))) }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.subtitle.customize"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.auto_enable_subtitles"), value = model.autoEnableSubtitles) { onAction(SettingsAction.SubtitlesChanged(model.copy(autoEnableSubtitles = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.subtitle_shadow"), value = model.subtitleShadow) { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleShadow = it))) }
        SettingsStepperRow(AppStrings.t(lang, "settings.subtitle_size"), model.subtitleSize.toInt(), step = 10, min = 50, max = 200, formatValue = { "$it%" }) { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleSize = it.toFloat()))) }
        SettingsColorOpacityRow(AppStrings.t(lang, "settings.subtitle_text"), model.subtitleColorArgb, model.subtitleTextOpacity, onColorChanged = { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleColorArgb = it))) }, onOpacityChanged = { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleTextOpacity = it))) })
        SettingsColorOpacityRow(AppStrings.t(lang, "settings.subtitle_background"), model.subtitleBackgroundColorArgb, model.subtitleBackgroundOpacity, onColorChanged = { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleBackgroundColorArgb = it))) }, onOpacityChanged = { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleBackgroundOpacity = it))) })
        SettingsColorOpacityRow(AppStrings.t(lang, "settings.subtitle_outline"), model.subtitleOutlineColorArgb, model.subtitleOutlineOpacity, onColorChanged = { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleOutlineColorArgb = it))) }, onOpacityChanged = { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleOutlineOpacity = it))) })
    }
}

internal fun languageOptionLabel(value: String, lang: String?): String = when (value) {
    "none" -> AppStrings.t(lang, "settings.none")
    "original" -> AppStrings.t(lang, "settings.original")
    "device_language" -> AppStrings.t(lang, "settings.device_language")
    "en" -> "English"
    "tr" -> "Türkçe"
    else -> value.uppercase()
}
