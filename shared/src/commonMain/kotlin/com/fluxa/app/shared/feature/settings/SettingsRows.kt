package com.fluxa.app.shared.feature.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.FluxaDimensions

data class SettingsChoiceOption(val value: String, val label: String, val description: String? = null)

val LocalSettingsHighlightLabel = compositionLocalOf<String?> { null }
val LocalSettingsAccentColor = compositionLocalOf { FluxaColors.accent }
private val LocalSettingsGroupRowCounter = compositionLocalOf<IntArray?> { null }

fun Modifier.settingsHighlight(highlighted: Boolean): Modifier = composed {
    if (highlighted) {
        clip(RoundedCornerShape(FluxaDimensions.CornerPresets.highlight)).background(LocalSettingsAccentColor.current.copy(alpha = FluxaDimensions.Alpha.mediumBorder))
    } else {
        this
    }
}

fun Modifier.settingsFocusRing(shape: Shape = RoundedCornerShape(FluxaDimensions.CornerPresets.highlight)): Modifier = composed {
    var focused by remember { mutableStateOf(false) }
    this
        .clip(shape)
        .onFocusChanged { focused = it.isFocused }
        .background(if (focused) LocalSettingsAccentColor.current.copy(alpha = FluxaDimensions.Alpha.mediumBorder) else Color.Transparent)
        .then(if (focused) Modifier.border(1.dp, LocalSettingsAccentColor.current.copy(alpha = FluxaDimensions.Alpha.secondaryText), shape) else Modifier)
}

fun Modifier.settingsRowDivider(): Modifier = composed {
    val counter = LocalSettingsGroupRowCounter.current
    val isFirstInGroup = counter != null && counter[0] == 0
    counter?.let { it[0] = it[0] + 1 }
    if (isFirstInGroup) {
        this
    } else {
        drawBehind {
            drawLine(
                color = Color.White.copy(alpha = FluxaDimensions.Alpha.subtleBorder),
                start = Offset(0f, 0f),
                end = Offset(size.width, 0f),
                strokeWidth = 1.dp.toPx()
            )
        }
    }
}

@Composable
fun SettingsSectionHeader(title: String) {
    Text(
        text = title.uppercase(),
        color = Color.White.copy(alpha = FluxaDimensions.Alpha.mutedLabel),
        fontWeight = FontWeight.Medium,
        fontSize = 12.sp,
        letterSpacing = 0.8.sp,
        modifier = Modifier.padding(start = 4.dp)
    )
}

@Composable
fun SettingsGroupCard(content: @Composable androidx.compose.foundation.layout.ColumnScope.() -> Unit) {
    val counter = remember { IntArray(1) }
    counter[0] = 0
    CompositionLocalProvider(LocalSettingsGroupRowCounter provides counter) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(16.dp))
                .background(FluxaColors.surfaceCard)
                .padding(horizontal = 16.dp),
            content = content,
        )
    }
}

@Composable
fun SettingsToggleRow(label: String, description: String? = null, value: Boolean, onValueChanged: (Boolean) -> Unit) {
    val haptics = LocalHapticFeedback.current
    val highlighted = LocalSettingsHighlightLabel.current == label
    val bringIntoViewRequester = remember { BringIntoViewRequester() }
    LaunchedEffect(highlighted) {
        if (highlighted) bringIntoViewRequester.bringIntoView()
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .bringIntoViewRequester(bringIntoViewRequester)
            .settingsHighlight(highlighted)
            .settingsRowDivider()
            .clickable {
                haptics.performHapticFeedback(HapticFeedbackType.LongPress)
                onValueChanged(!value)
            }
            .padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(label, color = Color.White, style = MaterialTheme.typography.bodyMedium)
            if (description != null) {
                Text(description, color = Color.White.copy(alpha = FluxaDimensions.Alpha.secondaryText), style = MaterialTheme.typography.bodySmall, modifier = Modifier.padding(top = 2.dp))
            }
        }
        Spacer(Modifier.width(12.dp))
        val accentColor = LocalSettingsAccentColor.current
        val checkedThumbColor = if (accentColor.luminance() > 0.5f) Color.Black else Color.White
        Switch(
            checked = value,
            onCheckedChange = null,
            modifier = Modifier.scale(0.82f),
            colors = SwitchDefaults.colors(
                checkedThumbColor = checkedThumbColor,
                checkedTrackColor = accentColor,
                checkedBorderColor = Color.Transparent,
                uncheckedThumbColor = Color.White.copy(alpha = 0.8f),
                uncheckedTrackColor = Color.White.copy(alpha = FluxaDimensions.Alpha.trackInactive),
                uncheckedBorderColor = Color.Transparent
            )
        )
    }
}

@Composable
fun SettingsChoiceRow(
    label: String,
    value: String,
    options: List<SettingsChoiceOption>,
    onValueChanged: (String) -> Unit
) {
    var showDialog by remember { mutableStateOf(false) }
    val currentLabel = options.firstOrNull { it.value == value }?.label ?: value
    val highlighted = LocalSettingsHighlightLabel.current == label
    val bringIntoViewRequester = remember { BringIntoViewRequester() }
    LaunchedEffect(highlighted) {
        if (highlighted) bringIntoViewRequester.bringIntoView()
    }
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .bringIntoViewRequester(bringIntoViewRequester)
            .settingsHighlight(highlighted)
            .settingsRowDivider()
            .settingsFocusRing()
            .clickable { showDialog = true }
            .padding(vertical = 10.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(label, color = Color.White, style = MaterialTheme.typography.bodyMedium)
            Icon(
                imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = Color.White.copy(alpha = FluxaDimensions.Alpha.placeholderText),
                modifier = Modifier.size(18.dp)
            )
        }
        Text(
            currentLabel,
            color = Color.White.copy(alpha = FluxaDimensions.Alpha.valueText),
            style = MaterialTheme.typography.bodySmall,
            modifier = Modifier.padding(top = 4.dp)
        )
    }
    if (showDialog) {
        SettingsChoiceDialog(
            title = label,
            options = options,
            selected = value,
            onSelected = {
                onValueChanged(it)
                showDialog = false
            },
            onDismiss = { showDialog = false }
        )
    }
}

@Composable
fun SettingsInlineChoiceCards(
    options: List<SettingsChoiceOption>,
    selected: String,
    onSelected: (String) -> Unit
) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        options.forEach { option ->
            val isSelected = option.value == selected
            val shape = RoundedCornerShape(16.dp)
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(shape)
                    .background(FluxaColors.surfaceCard)
                    .then(
                        if (isSelected) Modifier.border(1.dp, Color.White.copy(alpha = FluxaDimensions.Alpha.valueText), shape) else Modifier
                    )
                    .clickable { onSelected(option.value) }
                    .padding(horizontal = 16.dp, vertical = 12.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(option.label, color = Color.White, style = MaterialTheme.typography.bodyMedium)
                    if (option.description != null) {
                        Text(option.description, color = Color.White.copy(alpha = FluxaDimensions.Alpha.mutedLabel), style = MaterialTheme.typography.bodySmall, modifier = Modifier.padding(top = 2.dp))
                    }
                }
                if (isSelected) {
                    Icon(Icons.Filled.Check, contentDescription = null, tint = Color.White, modifier = Modifier.size(18.dp))
                }
            }
        }
    }
}

@Composable
fun SettingsChoiceDialog(
    title: String,
    options: List<SettingsChoiceOption>,
    selected: String,
    onSelected: (String) -> Unit,
    onDismiss: () -> Unit
) {
    androidx.compose.ui.window.Dialog(onDismissRequest = onDismiss) {
        val shape = RoundedCornerShape(20.dp)
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .clip(shape)
                .background(FluxaColors.surfaceRaised)
                .padding(vertical = 8.dp)
        ) {
            Text(
                title,
                style = MaterialTheme.typography.titleMedium,
                color = Color.White,
                modifier = Modifier.padding(horizontal = 20.dp, vertical = 14.dp)
            )
            androidx.compose.foundation.lazy.LazyColumn(modifier = Modifier.heightIn(max = 360.dp)) {
                items(options.size) { index ->
                    val option = options[index]
                    val isSelected = option.value == selected
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .settingsFocusRing(shape = RoundedCornerShape(0.dp))
                            .clickable { onSelected(option.value) }
                            .padding(horizontal = 20.dp, vertical = 14.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            option.label,
                            color = if (isSelected) LocalSettingsAccentColor.current else Color.White.copy(alpha = 0.75f),
                            fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Normal
                        )
                        if (isSelected) {
                            Icon(Icons.Filled.Check, contentDescription = null, tint = LocalSettingsAccentColor.current, modifier = Modifier.size(18.dp))
                        }
                    }
                }
            }
            Spacer(Modifier.height(4.dp))
        }
    }
}

@Composable
fun SettingsStepperRow(
    label: String,
    value: Int,
    step: Int = 1,
    min: Int = 0,
    max: Int = Int.MAX_VALUE,
    formatValue: (Int) -> String = { it.toString() },
    onValueChanged: (Int) -> Unit
) {
    var showDialog by remember { mutableStateOf(false) }
    val highlighted = LocalSettingsHighlightLabel.current == label
    val bringIntoViewRequester = remember { BringIntoViewRequester() }
    LaunchedEffect(highlighted) {
        if (highlighted) bringIntoViewRequester.bringIntoView()
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .bringIntoViewRequester(bringIntoViewRequester)
            .settingsHighlight(highlighted)
            .settingsRowDivider()
            .settingsFocusRing()
            .clickable { showDialog = true }
            .padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(label, color = Color.White, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(1f))
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                formatValue(value),
                color = Color.White.copy(alpha = FluxaDimensions.Alpha.valueText),
                style = MaterialTheme.typography.bodySmall,
            )
            Spacer(Modifier.width(4.dp))
            Icon(
                imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = Color.White.copy(alpha = FluxaDimensions.Alpha.placeholderText),
                modifier = Modifier.size(18.dp)
            )
        }
    }
    if (showDialog) {
        SettingsStepperDialog(
            title = label,
            value = value,
            step = step,
            min = min,
            max = max,
            formatValue = formatValue,
            onValueChanged = onValueChanged,
            onDismiss = { showDialog = false }
        )
    }
}
