package com.fluxa.app.shared.feature.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.ui.catalog.FluxaColors

data class SettingsChoiceOption(val value: String, val label: String)

@Composable
fun SettingsSectionHeader(title: String) {
    Text(
        text = title,
        color = Color.White.copy(alpha = 0.5f),
        fontWeight = FontWeight.Bold,
        fontSize = 12.sp,
        modifier = Modifier.padding(bottom = 4.dp, top = 12.dp)
    )
}

@Composable
fun SettingsToggleRow(label: String, description: String? = null, value: Boolean, onValueChanged: (Boolean) -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(label, color = Color.White)
            if (description != null) {
                Text(description, color = Color.White.copy(alpha = 0.5f), fontSize = 12.sp)
            }
        }
        Switch(checked = value, onCheckedChange = onValueChanged)
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
    Row(
        modifier = Modifier.fillMaxWidth().clickable { showDialog = true }.padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(label, color = Color.White, modifier = Modifier.weight(1f))
        Text(currentLabel, color = Color.White.copy(alpha = 0.55f))
    }
    if (showDialog) {
        SettingsChoiceDialog(
            title = label,
            options = options,
            selected = value,
            onSelected = { onValueChanged(it); showDialog = false },
            onDismiss = { showDialog = false }
        )
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
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(title, color = Color.White, fontWeight = FontWeight.Bold) },
        text = {
            LazyColumn {
                items(options.size) { index ->
                    val option = options[index]
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickable { onSelected(option.value) }
                            .padding(vertical = 10.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        RadioButton(selected = option.value == selected, onClick = { onSelected(option.value) })
                        Spacer(Modifier.width(8.dp))
                        Text(option.label, color = Color.White)
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = onDismiss) { Text("OK", color = Color.White) }
        },
        containerColor = Color(0xFF1A1D26)
    )
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
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(label, color = Color.White, modifier = Modifier.weight(1f))
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            SettingsStepperButton("−") { onValueChanged((value - step).coerceIn(min, max)) }
            Text(formatValue(value), color = Color.White, modifier = Modifier.width(48.dp), fontSize = 14.sp)
            SettingsStepperButton("+") { onValueChanged((value + step).coerceIn(min, max)) }
        }
    }
}

@Composable
private fun SettingsStepperButton(symbol: String, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .size(28.dp)
            .clip(CircleShape)
            .background(Color.White.copy(alpha = 0.1f))
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Text(symbol, color = Color.White, fontWeight = FontWeight.Bold)
    }
}

@Composable
fun SettingsPercentSliderRow(label: String, value: Float, onValueChanged: (Float) -> Unit) {
    Column(modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp)) {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(label, color = Color.White)
            Text("${value.toInt()}%", color = Color.White.copy(alpha = 0.55f))
        }
        Slider(
            value = value,
            onValueChange = onValueChanged,
            valueRange = 0f..100f,
            colors = SliderDefaults.colors(thumbColor = Color.White, activeTrackColor = Color.White)
        )
    }
}

val SETTINGS_COLOR_SWATCHES: List<Long> = listOf(
    0xFFFFFFFFL, 0xFF000000L, 0xFFEF5350L, 0xFF42A5F5L, 0xFFFFEE58L, 0xFF66BB6AL
)

@Composable
fun SettingsColorOpacityRow(
    label: String,
    colorArgb: Long,
    opacity: Float,
    onColorChanged: (Long) -> Unit,
    onOpacityChanged: (Float) -> Unit
) {
    Column(modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp)) {
        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
            Text(label, color = Color.White, modifier = Modifier.weight(1f))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                SETTINGS_COLOR_SWATCHES.forEach { swatch ->
                    Box(
                        modifier = Modifier
                            .size(22.dp)
                            .clip(CircleShape)
                            .background(Color(swatch.toInt()))
                            .then(
                                if (swatch == colorArgb) Modifier.padding(1.dp) else Modifier
                            )
                            .clickable { onColorChanged(swatch) }
                    )
                }
            }
        }
        Slider(
            value = opacity,
            onValueChange = onOpacityChanged,
            valueRange = 0f..1f,
            colors = SliderDefaults.colors(thumbColor = Color.White, activeTrackColor = Color.White)
        )
    }
}

@Composable
fun SettingsOrderedToggleRow(
    label: String,
    subtitle: String? = null,
    selected: Boolean,
    canMoveUp: Boolean,
    canMoveDown: Boolean,
    onToggle: () -> Unit,
    onMoveUp: () -> Unit,
    onMoveDown: () -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .size(22.dp)
                .clip(RoundedCornerShape(6.dp))
                .background(if (selected) Color.White.copy(alpha = 0.9f) else Color.White.copy(alpha = 0.08f))
                .clickable(onClick = onToggle),
            contentAlignment = Alignment.Center
        ) {
            if (selected) Text("✓", color = Color.Black, fontSize = 12.sp)
        }
        Spacer(Modifier.width(12.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(label, color = Color.White, fontSize = 14.sp)
            if (subtitle != null) Text(subtitle, color = Color.White.copy(alpha = 0.4f), fontSize = 11.sp)
        }
        if (selected) {
            SettingsStepperButton("↑".takeIf { canMoveUp } ?: "") { if (canMoveUp) onMoveUp() }
            Spacer(Modifier.width(4.dp))
            SettingsStepperButton("↓".takeIf { canMoveDown } ?: "") { if (canMoveDown) onMoveDown() }
        }
    }
}

@Composable
fun SettingsActionRow(
    label: String,
    value: String? = null,
    destructive: Boolean = false,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth().clickable(onClick = onClick).padding(vertical = 12.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(label, color = if (destructive) FluxaColors.errorRed else Color.White, fontWeight = FontWeight.Medium)
        if (value != null) {
            Text(value, color = Color.White.copy(alpha = 0.5f), fontSize = 13.sp)
        }
    }
}

@Composable
fun SettingsInfoRow(label: String, value: String) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(label, color = Color.White.copy(alpha = 0.6f))
        Text(value, color = Color.White.copy(alpha = 0.85f))
    }
}

@Composable
fun SettingsNavRow(label: String, onClick: () -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().clickable(onClick = onClick).padding(vertical = 14.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(label, color = Color.White, fontWeight = FontWeight.Medium)
        Text("›", color = Color.White.copy(alpha = 0.4f), fontSize = 18.sp)
    }
}

@Composable
fun SettingsTextFieldRow(
    label: String,
    value: String,
    onValueChanged: (String) -> Unit
) {
    androidx.compose.material3.OutlinedTextField(
        value = value,
        onValueChange = onValueChanged,
        label = { Text(label) },
        modifier = Modifier.fillMaxWidth().padding(vertical = 6.dp),
        colors = androidx.compose.material3.OutlinedTextFieldDefaults.colors(
            focusedTextColor = Color.White,
            unfocusedTextColor = Color.White,
            focusedBorderColor = Color.White.copy(alpha = 0.4f),
            unfocusedBorderColor = Color.White.copy(alpha = 0.15f),
            focusedLabelColor = Color.White.copy(alpha = 0.7f),
            unfocusedLabelColor = Color.White.copy(alpha = 0.4f),
            cursorColor = Color.White
        )
    )
}
