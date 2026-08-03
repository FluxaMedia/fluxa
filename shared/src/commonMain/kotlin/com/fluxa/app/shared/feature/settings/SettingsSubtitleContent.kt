package com.fluxa.app.shared.feature.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
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
    SettingsSubtitlePreview(model, lang)
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

@Composable
private fun SettingsSubtitlePreview(model: SettingsSubtitlesUiModel, lang: String?) {
    val fontSize = (15.sp.value * (model.subtitleSize / 100f)).sp
    val fillColor = Color(model.subtitleColorArgb.toInt()).copy(alpha = model.subtitleTextOpacity)
    val outlineColor = Color(model.subtitleOutlineColorArgb.toInt()).copy(alpha = model.subtitleOutlineOpacity)
    val backgroundColor = Color(model.subtitleBackgroundColorArgb.toInt()).copy(alpha = model.subtitleBackgroundOpacity)
    val shadow = if (model.subtitleShadow) {
        Shadow(color = Color.Black.copy(alpha = 0.8f), offset = androidx.compose.ui.geometry.Offset(0f, 2f), blurRadius = 6f)
    } else {
        null
    }
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 4.dp, bottom = 4.dp)
            .aspectRatio(16f / 9f)
            .clip(RoundedCornerShape(14.dp))
            .background(Brush.verticalGradient(listOf(Color(0xFF1A1A1A), Color(0xFF0A0A0A)))),
        contentAlignment = Alignment.BottomCenter
    ) {
        Box(
            modifier = Modifier
                .padding(bottom = 16.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(backgroundColor)
                .padding(horizontal = 8.dp, vertical = 4.dp)
        ) {
            val text = AppStrings.t(lang, "settings.subtitle_preview_text")
            val baseStyle = TextStyle(
                fontSize = fontSize,
                fontWeight = FontWeight.Medium,
                textAlign = TextAlign.Center,
                shadow = shadow
            )
            if (model.subtitleOutlineOpacity > 0f) {
                androidx.compose.material3.Text(
                    text = text,
                    style = baseStyle.copy(
                        color = outlineColor,
                        drawStyle = Stroke(width = fontSize.value / 5f)
                    )
                )
            }
            androidx.compose.material3.Text(text = text, style = baseStyle.copy(color = fillColor))
        }
    }
}
