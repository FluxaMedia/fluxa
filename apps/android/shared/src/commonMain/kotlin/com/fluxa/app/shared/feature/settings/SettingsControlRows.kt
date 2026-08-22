package com.fluxa.app.shared.feature.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.DragHandle
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.FluxaDimensions
import kotlinx.coroutines.launch

@Composable
internal fun SettingsStepperDialog(
    title: String,
    value: Int,
    step: Int,
    min: Int,
    max: Int,
    formatValue: (Int) -> String,
    onValueChanged: (Int) -> Unit,
    onDismiss: () -> Unit
) {
    val haptics = LocalHapticFeedback.current
    androidx.compose.ui.window.Dialog(onDismissRequest = onDismiss) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(20.dp))
                .background(FluxaColors.surfaceRaised)
                .padding(vertical = 20.dp, horizontal = 20.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text(title, style = MaterialTheme.typography.titleMedium, color = Color.White)
            Spacer(Modifier.height(20.dp))
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                SettingsIconButton(Icons.Filled.Remove) {
                    haptics.performHapticFeedback(HapticFeedbackType.LongPress)
                    onValueChanged((value - step).coerceIn(min, max))
                }
                Text(formatValue(value), color = Color.White, style = MaterialTheme.typography.bodyLarge, modifier = Modifier.width(64.dp), textAlign = androidx.compose.ui.text.style.TextAlign.Center)
                SettingsIconButton(Icons.Filled.Add) {
                    haptics.performHapticFeedback(HapticFeedbackType.LongPress)
                    onValueChanged((value + step).coerceIn(min, max))
                }
            }
        }
    }
}

@Composable
internal fun SettingsIconButton(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    enabled: Boolean = true,
    onClick: () -> Unit
) {
    var focused by remember { mutableStateOf(false) }
    Box(
        modifier = Modifier
            .size(48.dp)
            .clip(CircleShape)
            .onFocusChanged { focused = it.isFocused }
            .clickable(enabled = enabled, onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Box(
            modifier = Modifier
                .size(28.dp)
                .clip(CircleShape)
                .background(
                    when {
                        focused -> LocalSettingsAccentColor.current
                        enabled -> Color.White.copy(alpha = 0.1f)
                        else -> Color.White.copy(alpha = 0.04f)
                    }
                ),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = if (focused) Color.White else Color.White.copy(alpha = if (enabled) 0.85f else 0.25f),
                modifier = Modifier.size(16.dp)
            )
        }
    }
}

@Composable
fun SettingsPercentSliderRow(label: String, value: Float, onValueChanged: (Float) -> Unit) {
    var dragValue by remember(value) { mutableFloatStateOf(value) }
    Column(modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp)) {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(label, color = Color.White, style = MaterialTheme.typography.bodyMedium)
            Text("${dragValue.toInt()}%", color = Color.White.copy(alpha = FluxaDimensions.Alpha.valueText), style = MaterialTheme.typography.bodySmall)
        }
        Slider(
            value = dragValue,
            onValueChange = { dragValue = it },
            onValueChangeFinished = { onValueChanged(dragValue) },
            valueRange = 0f..100f,
            colors = SliderDefaults.colors(
                thumbColor = LocalSettingsAccentColor.current,
                activeTrackColor = LocalSettingsAccentColor.current,
                inactiveTrackColor = Color.White.copy(alpha = FluxaDimensions.Alpha.trackInactive)
            )
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
    var dragOpacity by remember(opacity) { mutableFloatStateOf(opacity) }
    Column(modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp)) {
        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
            Text(label, color = Color.White, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(1f))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                SETTINGS_COLOR_SWATCHES.forEach { swatch ->
                    var swatchFocused by remember { mutableStateOf(false) }
                    val selected = swatch == colorArgb
                    Box(
                        modifier = Modifier
                            .size(22.dp)
                            .clip(CircleShape)
                            .onFocusChanged { swatchFocused = it.isFocused }
                            .background(Color(swatch.toInt()))
                            .then(
                                if (selected || swatchFocused) Modifier.border(2.dp, Color.White, CircleShape) else Modifier
                            )
                            .clickable { onColorChanged(swatch) },
                        contentAlignment = Alignment.Center
                    ) {
                        if (selected) {
                            val isLight = Color(swatch.toInt()).luminance() > 0.5f
                            Icon(
                                Icons.Filled.Check,
                                contentDescription = null,
                                tint = if (isLight) Color.Black else Color.White,
                                modifier = Modifier.size(12.dp)
                            )
                        }
                    }
                }
            }
        }
        Slider(
            value = dragOpacity,
            onValueChange = { dragOpacity = it },
            onValueChangeFinished = { onOpacityChanged(dragOpacity) },
            valueRange = 0f..1f,
            colors = SliderDefaults.colors(
                thumbColor = LocalSettingsAccentColor.current,
                activeTrackColor = LocalSettingsAccentColor.current,
                inactiveTrackColor = Color.White.copy(alpha = FluxaDimensions.Alpha.trackInactive)
            )
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
    val density = androidx.compose.ui.platform.LocalDensity.current
    val rowHeightPx = with(density) { 52.dp.toPx() }
    val dragOffset = remember { androidx.compose.animation.core.Animatable(0f) }
    val scope = androidx.compose.runtime.rememberCoroutineScope()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 8.dp)
            .offset { androidx.compose.ui.unit.IntOffset(0, dragOffset.value.toInt()) },
        verticalAlignment = Alignment.CenterVertically
    ) {
        var checkboxFocused by remember { mutableStateOf(false) }
        val checkboxShape = RoundedCornerShape(6.dp)
        Box(
            modifier = Modifier
                .size(22.dp)
                .clip(checkboxShape)
                .onFocusChanged { checkboxFocused = it.isFocused }
                .background(if (selected) LocalSettingsAccentColor.current else Color.White.copy(alpha = FluxaDimensions.Alpha.subtleBorder))
                .then(
                    if (checkboxFocused) Modifier.border(2.dp, Color.White, checkboxShape) else Modifier
                )
                .clickable(onClick = onToggle),
            contentAlignment = Alignment.Center
        ) {
            if (selected) Icon(Icons.Filled.Check, contentDescription = null, tint = Color.White, modifier = Modifier.size(14.dp))
        }
        Spacer(Modifier.width(12.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(label, color = Color.White, fontSize = 14.sp)
            if (subtitle != null) Text(subtitle, color = Color.White.copy(alpha = FluxaDimensions.Alpha.faintText), fontSize = 11.sp)
        }
        if (selected) {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .pointerInput(canMoveUp, canMoveDown) {
                        detectDragGestures(
                            onDragEnd = { scope.launch { dragOffset.animateTo(0f) } },
                            onDragCancel = { scope.launch { dragOffset.animateTo(0f) } }
                        ) { change, dragAmount ->
                            change.consume()
                            scope.launch {
                                val next = dragOffset.value + dragAmount.y
                                when {
                                    next > rowHeightPx / 2 && canMoveDown -> {
                                        onMoveDown()
                                        dragOffset.snapTo(next - rowHeightPx)
                                    }
                                    next < -rowHeightPx / 2 && canMoveUp -> {
                                        onMoveUp()
                                        dragOffset.snapTo(next + rowHeightPx)
                                    }
                                    else -> dragOffset.snapTo(next)
                                }
                            }
                        }
                    },
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    Icons.Filled.DragHandle,
                    contentDescription = null,
                    tint = Color.White.copy(alpha = FluxaDimensions.Alpha.faintText),
                    modifier = Modifier.size(20.dp)
                )
            }
        }
    }
}
