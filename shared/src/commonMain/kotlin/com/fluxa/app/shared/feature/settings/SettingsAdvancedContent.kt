package com.fluxa.app.shared.feature.settings

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
internal fun SettingsAdvancedContent(model: SettingsAdvancedUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val bufferCacheOptions = listOf("100", "500", "1000", "2000").map { SettingsChoiceOption(it, "$it MB") }
    val bufferSecondOptions = listOf("0", "15", "30", "60", "120").map { SettingsChoiceOption(it, "${it}s") }
    val audioDecoderOptions = listOf(
        SettingsChoiceOption("hw_prefer", AppStrings.t(lang, "settings.audio_decoder_hw_prefer")),
        SettingsChoiceOption("hw_only", AppStrings.t(lang, "settings.audio_decoder_hw_only")),
        SettingsChoiceOption("sw_only", AppStrings.t(lang, "settings.audio_decoder_sw_only"))
    )
    SettingsSectionHeader(AppStrings.t(lang, "settings.advanced"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "settings.buffer_cache"), model.playerBufferCacheMb.toString(), bufferCacheOptions) { onAction(SettingsAction.AdvancedChanged(model.copy(playerBufferCacheMb = it.toIntOrNull() ?: model.playerBufferCacheMb))) }
        SettingsChoiceRow(AppStrings.t(lang, "settings.forward_buffer"), model.playerForwardBufferSeconds.toString(), bufferSecondOptions) { onAction(SettingsAction.AdvancedChanged(model.copy(playerForwardBufferSeconds = it.toIntOrNull() ?: model.playerForwardBufferSeconds))) }
        SettingsChoiceRow(AppStrings.t(lang, "settings.back_buffer"), model.playerBackBufferSeconds.toString(), bufferSecondOptions) { onAction(SettingsAction.AdvancedChanged(model.copy(playerBackBufferSeconds = it.toIntOrNull() ?: model.playerBackBufferSeconds))) }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.decoder"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "settings.audio_decoder_mode"), model.audioDecoderMode, audioDecoderOptions) { onAction(SettingsAction.AdvancedChanged(model.copy(audioDecoderMode = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tunneled_playback"), value = model.tunneledPlayback) { onAction(SettingsAction.AdvancedChanged(model.copy(tunneledPlayback = it))) }
    }
    Spacer(Modifier.height(20.dp))
    var confirmingReset by remember { mutableStateOf(false) }
    SettingsGroupCard { SettingsActionRow(AppStrings.t(lang, "settings.reset_to_defaults"), destructive = true) { confirmingReset = true } }
    if (confirmingReset) AdvancedSettingsResetDialog(lang, onDismiss = { confirmingReset = false }) {
        confirmingReset = false
        onAction(SettingsAction.AdvancedChanged(SettingsAdvancedUiModel()))
    }
}

@Composable
private fun AdvancedSettingsResetDialog(lang: String?, onDismiss: () -> Unit, onConfirm: () -> Unit) {
    AlertDialog(onDismissRequest = onDismiss, title = { Text(AppStrings.t(lang, "settings.reset_to_defaults")) }, text = { Text(AppStrings.t(lang, "settings.reset_to_defaults_confirm")) }, confirmButton = { TextButton(onClick = onConfirm) { Text(AppStrings.t(lang, "settings.reset_to_defaults"), color = FluxaColors.errorRed) } }, dismissButton = { TextButton(onClick = onDismiss) { Text(AppStrings.t(lang, "common.cancel")) } })
}
