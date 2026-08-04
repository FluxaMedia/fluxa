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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.DragHandle
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
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
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.FluxaDimensions
import kotlinx.coroutines.launch

data class SettingsChoiceOption(val value: String, val label: String)

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
private fun SettingsIconChip(content: @Composable () -> Unit) {
    Box(
        modifier = Modifier
            .size(40.dp)
            .clip(RoundedCornerShape(FluxaDimensions.CornerPresets.soft))
            .background(LocalSettingsAccentColor.current.copy(alpha = FluxaDimensions.Alpha.subtleBorder)),
        contentAlignment = Alignment.Center,
    ) {
        content()
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
                currentLabel,
                color = Color.White.copy(alpha = FluxaDimensions.Alpha.valueText),
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.widthIn(max = 140.dp)
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

@Composable
private fun SettingsStepperDialog(
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
private fun SettingsIconButton(
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

@Composable
fun SettingsActionRow(
    label: String,
    value: String? = null,
    destructive: Boolean = false,
    icon: (@Composable () -> Unit)? = null,
    onClick: () -> Unit
) {
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
            .clickable(onClick = onClick)
            .padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            if (icon != null) {
                Box(modifier = Modifier.size(22.dp), contentAlignment = Alignment.Center) { icon() }
                Spacer(Modifier.width(14.dp))
            }
            Text(label, color = if (destructive) FluxaColors.errorRed else Color.White, style = MaterialTheme.typography.bodyMedium)
        }
        if (value != null) {
            Text(
                value,
                color = Color.White.copy(alpha = FluxaDimensions.Alpha.secondaryText),
                fontSize = 13.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.widthIn(max = 140.dp)
            )
        }
    }
}

@Composable
fun SettingsConnectionRow(
    label: String,
    connected: Boolean,
    connectedLabel: String,
    icon: (@Composable () -> Unit)? = null,
    hasSyncFailure: Boolean = false,
    syncFailedLabel: String? = null,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth().settingsRowDivider().settingsFocusRing().clickable(onClick = onClick).padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Box(
                modifier = Modifier
                    .size(32.dp)
                    .clip(CircleShape)
                    .background(Color.White.copy(alpha = FluxaDimensions.Alpha.subtleBorder)),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = label.take(1).uppercase(),
                    color = Color.White.copy(alpha = FluxaDimensions.Alpha.secondaryText),
                    style = MaterialTheme.typography.bodySmall,
                    fontWeight = FontWeight.SemiBold
                )
            }
            Spacer(Modifier.width(14.dp))
            Text(label, color = Color.White, style = MaterialTheme.typography.bodyMedium)
        }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            val statusText = if (connected && hasSyncFailure) syncFailedLabel.orEmpty() else if (connected) connectedLabel else null
            if (statusText != null) {
                Text(
                    statusText,
                    color = if (hasSyncFailure) FluxaColors.errorRed else Color.White.copy(alpha = FluxaDimensions.Alpha.valueText),
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.widthIn(max = 140.dp)
                )
            }
            Icon(
                imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = Color.White.copy(alpha = FluxaDimensions.Alpha.placeholderText),
                modifier = Modifier.size(18.dp)
            )
        }
    }
}

@Composable
fun SettingsSecretFieldRow(
    label: String,
    value: String,
    placeholder: String? = null,
    onValueChanged: (String) -> Unit
) {
    var showDialog by remember { mutableStateOf(false) }
    Row(
        modifier = Modifier
            .fillMaxWidth()
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
                if (value.isBlank()) placeholder.orEmpty() else "••••••••",
                color = Color.White.copy(alpha = FluxaDimensions.Alpha.valueText),
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.widthIn(max = 140.dp)
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
        SettingsTextEditDialog(
            title = label,
            value = value,
            placeholder = placeholder,
            isSecret = true,
            onValueChanged = onValueChanged,
            onDismiss = { showDialog = false }
        )
    }
}

@Composable
private fun SettingsTextEditDialog(
    title: String,
    value: String,
    placeholder: String? = null,
    isSecret: Boolean = false,
    onValueChanged: (String) -> Unit,
    onDismiss: () -> Unit
) {
    var text by remember { mutableStateOf(value) }
    var revealed by remember { mutableStateOf(false) }
    androidx.compose.ui.window.Dialog(onDismissRequest = { onValueChanged(text); onDismiss() }) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(20.dp))
                .background(FluxaColors.surfaceRaised)
                .padding(vertical = 20.dp, horizontal = 20.dp)
        ) {
            Text(title, style = MaterialTheme.typography.titleMedium, color = Color.White)
            Spacer(Modifier.height(16.dp))
            androidx.compose.material3.OutlinedTextField(
                value = text,
                onValueChange = { text = it },
                placeholder = placeholder?.let { { Text(it, color = Color.White.copy(alpha = FluxaDimensions.Alpha.placeholderText)) } },
                singleLine = true,
                visualTransformation = if (isSecret && !revealed) PasswordVisualTransformation() else VisualTransformation.None,
                trailingIcon = if (isSecret) {
                    { IconButtonToggle(revealed) { revealed = !revealed } }
                } else null,
                modifier = Modifier.fillMaxWidth(),
                colors = androidx.compose.material3.OutlinedTextFieldDefaults.colors(
                    focusedTextColor = Color.White,
                    unfocusedTextColor = Color.White,
                    focusedBorderColor = LocalSettingsAccentColor.current,
                    unfocusedBorderColor = Color.White.copy(alpha = FluxaDimensions.Alpha.borderFaint),
                    cursorColor = LocalSettingsAccentColor.current
                )
            )
        }
    }
}

@Composable
private fun IconButtonToggle(revealed: Boolean, onClick: () -> Unit) {
    Box(
        modifier = Modifier.size(24.dp).settingsFocusRing(shape = CircleShape).clickable(onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Icon(
            imageVector = if (revealed) Icons.Filled.VisibilityOff else Icons.Filled.Visibility,
            contentDescription = null,
            tint = Color.White.copy(alpha = FluxaDimensions.Alpha.iconMuted),
            modifier = Modifier.size(20.dp)
        )
    }
}

@Composable
fun SettingsInfoRow(label: String, value: String) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(label, color = Color.White.copy(alpha = FluxaDimensions.Alpha.iconMuted), style = MaterialTheme.typography.bodyMedium)
        Text(value, color = Color.White.copy(alpha = 0.85f), style = MaterialTheme.typography.bodySmall)
    }
}

@Composable
fun SettingsNavRow(
    label: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector? = null,
    description: String? = null,
    value: String? = null,
    onClick: () -> Unit
) {
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
            .clickable(onClick = onClick)
            .padding(vertical = if (description != null) 10.dp else 12.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.weight(1f)) {
            if (icon != null) {
                SettingsIconChip {
                    androidx.compose.material3.Icon(
                        imageVector = icon,
                        contentDescription = null,
                        tint = LocalSettingsAccentColor.current,
                        modifier = Modifier.size(20.dp)
                    )
                }
                Spacer(Modifier.width(14.dp))
            }
            Column {
                Text(label, color = Color.White, style = MaterialTheme.typography.bodyMedium)
                if (description != null) {
                    Text(description, color = Color.White.copy(alpha = FluxaDimensions.Alpha.mutedLabel), style = MaterialTheme.typography.bodySmall, modifier = Modifier.padding(top = 2.dp))
                }
            }
        }
        Row(verticalAlignment = Alignment.CenterVertically) {
            if (value != null) {
                Text(
                    value,
                    color = Color.White.copy(alpha = FluxaDimensions.Alpha.secondaryText),
                    fontSize = 13.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.widthIn(max = 140.dp)
                )
                Spacer(Modifier.width(8.dp))
            }
            Icon(
                imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = Color.White.copy(alpha = FluxaDimensions.Alpha.placeholderText),
                modifier = Modifier.size(20.dp)
            )
        }
    }
}

@Composable
fun SettingsTextFieldRow(
    label: String,
    value: String,
    onValueChanged: (String) -> Unit
) {
    var showDialog by remember { mutableStateOf(false) }
    Row(
        modifier = Modifier
            .fillMaxWidth()
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
                value,
                color = Color.White.copy(alpha = FluxaDimensions.Alpha.valueText),
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.widthIn(max = 140.dp)
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
        SettingsTextEditDialog(
            title = label,
            value = value,
            onValueChanged = onValueChanged,
            onDismiss = { showDialog = false }
        )
    }
}
