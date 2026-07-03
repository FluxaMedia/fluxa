@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.ui.res.painterResource
import com.fluxa.app.R
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.TextButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.runtime.*
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.ClickableSurfaceDefaults
import androidx.tv.material3.Surface
import coil3.compose.AsyncImage
import java.util.Locale

@Composable
internal fun SettingsSection(title: String, subtitle: String, content: @Composable ColumnScope.() -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
        Text(title, color = Color.White, fontSize = 26.sp, fontWeight = FontWeight.Black)
        Text(subtitle, color = Color.White.copy(alpha = 0.58f), fontSize = 14.sp)
        Column(verticalArrangement = Arrangement.spacedBy(12.dp), content = content)
    }
}

@Composable
internal fun SettingsInfoTile(title: String, value: String, icon: ImageVector) {
    SettingsPanel {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
            Icon(icon, null, tint = Color.White.copy(alpha = 0.72f))
            Column {
                Text(title, color = Color.White, fontWeight = FontWeight.Bold)
                Text(value, color = Color.White.copy(alpha = 0.58f), fontSize = 13.sp)
            }
        }
    }
}

@Composable
internal fun SettingsToggleTile(title: String, subtitle: String, checked: Boolean, onToggle: (Boolean) -> Unit) {
    var focused by remember { mutableStateOf(false) }
    var localChecked by remember { mutableStateOf(checked) }
    LaunchedEffect(checked) { localChecked = checked }
    Surface(
        onClick = { localChecked = !localChecked; onToggle(localChecked) },
        modifier = Modifier.fillMaxWidth().height(86.dp).onFocusChanged { focused = it.isFocused },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(20.dp)),
        colors = ClickableSurfaceDefaults.colors(
            containerColor = if (focused) Color.White.copy(alpha = 0.12f) else Color.White.copy(alpha = 0.05f),
            contentColor = Color.White
        )
    ) {
        Row(
            modifier = Modifier.fillMaxSize().padding(horizontal = 18.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(title, color = Color.White, fontWeight = FontWeight.Black, fontSize = 16.sp)
                Text(subtitle, color = Color.White.copy(alpha = 0.56f), fontSize = 13.sp, lineHeight = 18.sp)
            }
            Switch(
                checked = localChecked,
                onCheckedChange = { localChecked = it; onToggle(it) },
                colors = SwitchDefaults.colors(
                    checkedThumbColor = Color.Black,
                    checkedTrackColor = Color.White,
                    uncheckedThumbColor = Color(0xFFD6D6D6),
                    uncheckedTrackColor = Color(0xFF3A3A3A)
                )
            )
        }
    }
}

@Composable
internal fun SettingsOrderedToggleTile(
    title: String,
    checked: Boolean,
    onToggle: (Boolean) -> Unit,
    canMoveUp: Boolean,
    canMoveDown: Boolean,
    onMoveUp: () -> Unit,
    onMoveDown: () -> Unit
) {
    var focused by remember { mutableStateOf(false) }
    var localChecked by remember { mutableStateOf(checked) }
    LaunchedEffect(checked) { localChecked = checked }
    Surface(
        onClick = { localChecked = !localChecked; onToggle(localChecked) },
        modifier = Modifier.fillMaxWidth().height(72.dp).onFocusChanged { focused = it.isFocused },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(20.dp)),
        colors = ClickableSurfaceDefaults.colors(
            containerColor = if (focused) Color.White.copy(alpha = 0.12f) else Color.White.copy(alpha = 0.05f),
            contentColor = Color.White
        )
    ) {
        Row(
            modifier = Modifier.fillMaxSize().padding(horizontal = 18.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text(title, color = Color.White, fontWeight = FontWeight.Black, fontSize = 16.sp, modifier = Modifier.weight(1f))
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                IconButton(onClick = onMoveUp, enabled = canMoveUp) {
                    Icon(FluxaIcons.ArrowUp, null, tint = if (canMoveUp) Color.White.copy(alpha = 0.72f) else Color.White.copy(alpha = 0.2f))
                }
                IconButton(onClick = onMoveDown, enabled = canMoveDown) {
                    Icon(FluxaIcons.ArrowDown, null, tint = if (canMoveDown) Color.White.copy(alpha = 0.72f) else Color.White.copy(alpha = 0.2f))
                }
                Switch(
                    checked = localChecked,
                    onCheckedChange = { localChecked = it; onToggle(it) },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = Color.Black,
                        checkedTrackColor = Color.White,
                        uncheckedThumbColor = Color(0xFFD6D6D6),
                        uncheckedTrackColor = Color(0xFF3A3A3A)
                    )
                )
            }
        }
    }
}

@Composable
internal fun SettingsChoiceTile(
    title: String,
    subtitle: String,
    options: List<ChoiceOption>,
    selected: String,
    onSelect: (String) -> Unit
) {
    var expanded by remember { mutableStateOf(false) }
    val selectedLabel = options.firstOrNull { it.value == selected }?.label ?: selected
    SettingsPanel {
        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            Column(modifier = Modifier.weight(1f)) {
                Text(title, color = Color.White, fontWeight = FontWeight.Black, fontSize = 16.sp)
                if (subtitle.isNotBlank()) {
                    Text(subtitle, color = Color.White.copy(alpha = 0.56f), fontSize = 13.sp, lineHeight = 18.sp)
                }
            }
            Spacer(Modifier.width(16.dp))
            SettingsDropdownButton(selectedLabel) { expanded = true }
        }
    }
    if (expanded) {
        SettingsOptionDialog(
            title = title,
            options = options,
            selected = selected,
            onSelect = { onSelect(it); expanded = false },
            onDismiss = { expanded = false }
        )
    }
}

@Composable
private fun SettingsDropdownButton(label: String, onClick: () -> Unit) {
    var focused by remember { mutableStateOf(false) }
    Surface(
        onClick = onClick,
        modifier = Modifier.height(40.dp).widthIn(min = 150.dp, max = 260.dp).onFocusChanged { focused = it.isFocused },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(10.dp)),
        colors = ClickableSurfaceDefaults.colors(
            containerColor = if (focused) Color.White.copy(alpha = 0.18f) else Color.White.copy(alpha = 0.07f),
            contentColor = Color.White
        )
    ) {
        Row(
            modifier = Modifier.fillMaxHeight().padding(horizontal = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Text(
                label,
                color = Color.White,
                fontSize = 13.sp,
                fontWeight = FontWeight.Bold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            Icon(FluxaIcons.ChevronDown, null, tint = Color.White.copy(alpha = 0.6f), modifier = Modifier.size(18.dp))
        }
    }
}

@Composable
private fun SettingsOptionDialog(
    title: String,
    options: List<ChoiceOption>,
    selected: String,
    onSelect: (String) -> Unit,
    onDismiss: () -> Unit
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        confirmButton = {},
        title = { Text(title, fontWeight = FontWeight.Black) },
        text = {
            LazyColumn(verticalArrangement = Arrangement.spacedBy(4.dp), modifier = Modifier.heightIn(max = 360.dp)) {
                items(options, key = { it.value }) { option ->
                    SettingsOptionRow(option.label, option.value == selected) { onSelect(option.value) }
                }
            }
        }
    )
}

@Composable
private fun SettingsOptionRow(label: String, selected: Boolean, onClick: () -> Unit) {
    var focused by remember { mutableStateOf(false) }
    Surface(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth().height(48.dp).onFocusChanged { focused = it.isFocused },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(10.dp)),
        colors = ClickableSurfaceDefaults.colors(
            containerColor = when {
                focused -> Color.White.copy(alpha = 0.16f)
                selected -> Color.White.copy(alpha = 0.08f)
                else -> Color.Transparent
            },
            contentColor = Color.White
        )
    ) {
        Row(
            modifier = Modifier.fillMaxSize().padding(horizontal = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text(
                label,
                color = Color.White,
                fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
                fontSize = 14.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f)
            )
            if (selected) Icon(FluxaIcons.Check, null, tint = Color.White, modifier = Modifier.size(18.dp))
        }
    }
}

@Composable
internal fun SettingsTextFieldTile(
    title: String,
    subtitle: String,
    value: String,
    placeholder: String,
    onValueChange: (String) -> Unit
) {
    SettingsPanel {
        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Text(title, color = Color.White, fontWeight = FontWeight.Black, fontSize = 16.sp)
            Text(subtitle, color = Color.White.copy(alpha = 0.56f), fontSize = 13.sp, lineHeight = 18.sp)
            OutlinedTextField(
                value = value,
                onValueChange = onValueChange,
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
                placeholder = { Text(placeholder) },
                colors = OutlinedTextFieldDefaults.colors(
                    focusedBorderColor = Color.White,
                    unfocusedBorderColor = Color.White.copy(alpha = 0.18f),
                    focusedTextColor = Color.White,
                    unfocusedTextColor = Color.White,
                    cursorColor = Color.White,
                    focusedPlaceholderColor = Color.White.copy(alpha = 0.4f),
                    unfocusedPlaceholderColor = Color.White.copy(alpha = 0.32f),
                    focusedContainerColor = Color.Transparent,
                    unfocusedContainerColor = Color.Transparent
                )
            )
        }
    }
}

private fun Modifier.dpadAdjustable(onStepLeft: () -> Unit, onStepRight: () -> Unit): Modifier =
    onPreviewKeyEvent { event ->
        if (event.type != KeyEventType.KeyDown) return@onPreviewKeyEvent false
        when (event.key) {
            Key.DirectionLeft -> { onStepLeft(); true }
            Key.DirectionRight -> { onStepRight(); true }
            else -> false
        }
    }

@Composable
internal fun SettingsSliderTile(
    title: String,
    subtitle: String,
    value: Float,
    valueRange: ClosedFloatingPointRange<Float> = 50f..99f,
    onValueChange: (Float) -> Unit
) {
    val safeValue = value.coerceIn(valueRange.start, valueRange.endInclusive)
    SettingsPanel {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(title, color = Color.White, fontWeight = FontWeight.Black, fontSize = 16.sp)
            Text("${safeValue.toInt()}%", color = Color.White.copy(alpha = 0.7f), fontSize = 14.sp, fontWeight = FontWeight.Bold)
        }
        Text(subtitle, color = Color.White.copy(alpha = 0.56f), fontSize = 13.sp, lineHeight = 18.sp)
        val step = (valueRange.endInclusive - valueRange.start) / 100f
        Slider(
            value = safeValue,
            onValueChange = onValueChange,
            valueRange = valueRange,
            modifier = Modifier.dpadAdjustable(
                onStepLeft = { onValueChange((safeValue - step).coerceIn(valueRange.start, valueRange.endInclusive)) },
                onStepRight = { onValueChange((safeValue + step).coerceIn(valueRange.start, valueRange.endInclusive)) }
            ),
            colors = androidx.compose.material3.SliderDefaults.colors(
                thumbColor = Color.White,
                activeTrackColor = Color.White,
                inactiveTrackColor = Color.White.copy(alpha = 0.18f)
            )
        )
    }
}

@Composable
internal fun SettingsSecondsSliderTile(
    title: String,
    subtitle: String,
    value: Int,
    valueRange: IntRange = 0..10,
    zeroLabel: String? = null,
    onValueChange: (Int) -> Unit
) {
    val safeValue = value.coerceIn(valueRange.first, valueRange.last)
    SettingsPanel {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(title, color = Color.White, fontWeight = FontWeight.Black, fontSize = 16.sp)
            Text(
                if (safeValue == 0 && zeroLabel != null) zeroLabel else "${safeValue}s",
                color = Color.White.copy(alpha = 0.7f),
                fontSize = 14.sp,
                fontWeight = FontWeight.Bold
            )
        }
        Text(subtitle, color = Color.White.copy(alpha = 0.56f), fontSize = 13.sp, lineHeight = 18.sp)
        Slider(
            value = safeValue.toFloat(),
            onValueChange = { onValueChange(it.toInt()) },
            valueRange = valueRange.first.toFloat()..valueRange.last.toFloat(),
            steps = (valueRange.last - valueRange.first - 1).coerceAtLeast(0),
            modifier = Modifier.dpadAdjustable(
                onStepLeft = { onValueChange((safeValue - 1).coerceIn(valueRange.first, valueRange.last)) },
                onStepRight = { onValueChange((safeValue + 1).coerceIn(valueRange.first, valueRange.last)) }
            ),
            colors = androidx.compose.material3.SliderDefaults.colors(
                thumbColor = Color.White,
                activeTrackColor = Color.White,
                inactiveTrackColor = Color.White.copy(alpha = 0.18f)
            )
        )
    }
}

@Composable
internal fun SettingsAccentColorTile(profile: UserProfile, onUpdateProfile: (UserProfile) -> Unit) {
    SettingsPanel {
        Row(horizontalArrangement = Arrangement.spacedBy(14.dp)) {
            listOf(
                Color.White,
                Color(0xFF3F7CFF),
                Color(0xFF35C2A0),
                Color(0xFFFF9D42),
                Color(0xFFFF5D5D),
                Color(0xFFFF4DA0)
            ).forEach { color ->
                val colorArgb = color.toArgb()
                val selected = profile.safeAccentColorArgb == colorArgb
                Box(
                    modifier = Modifier
                        .size(if (selected) 36.dp else 28.dp)
                        .clip(CircleShape)
                        .background(color)
                        .border(
                            width = if (selected) 3.dp else 0.dp,
                            color = if (color == Color.White) Color(0xFF8F5CFF) else Color.White.copy(alpha = 0.88f),
                            shape = CircleShape
                        )
                        .clickable { onUpdateProfile(profile.copy(accentColorArgb = colorArgb)) }
                )
            }
        }
    }
}

@Composable
internal fun SettingsColorOpacityTile(
    title: String,
    subtitle: String,
    opacityTitle: String,
    colorOptions: List<ChoiceOption>,
    selectedColor: Int,
    opacity: Float,
    onColorSelect: (Int) -> Unit,
    onOpacity: (Float) -> Unit
) {
    var expanded by remember { mutableStateOf(false) }
    val selectedColorLabel = colorOptions.firstOrNull { it.value.toIntOrNull() == selectedColor }?.label ?: ""
    SettingsPanel {
        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            Column(modifier = Modifier.weight(1f)) {
                Text(title, color = Color.White, fontWeight = FontWeight.Black, fontSize = 16.sp)
                Text(subtitle, color = Color.White.copy(alpha = 0.56f), fontSize = 13.sp, lineHeight = 18.sp)
            }
            Spacer(Modifier.width(16.dp))
            SettingsDropdownButton(selectedColorLabel) { expanded = true }
        }
        Text(opacityTitle, color = Color.White.copy(alpha = 0.56f), fontSize = 13.sp)
        Slider(
            value = opacity,
            onValueChange = onOpacity,
            valueRange = 0f..1f,
            modifier = Modifier.dpadAdjustable(
                onStepLeft = { onOpacity((opacity - 0.05f).coerceIn(0f, 1f)) },
                onStepRight = { onOpacity((opacity + 0.05f).coerceIn(0f, 1f)) }
            ),
            colors = androidx.compose.material3.SliderDefaults.colors(
                thumbColor = Color.White,
                activeTrackColor = Color.White,
                inactiveTrackColor = Color.White.copy(alpha = 0.18f)
            )
        )
    }
    if (expanded) {
        SettingsOptionDialog(
            title = title,
            options = colorOptions,
            selected = selectedColor.toString(),
            onSelect = { value -> value.toIntOrNull()?.let(onColorSelect); expanded = false },
            onDismiss = { expanded = false }
        )
    }
}

@Composable
internal fun SettingsActionTile(
    title: String,
    subtitle: String,
    icon: ImageVector,
    accent: Color = Color.White,
    onClick: () -> Unit
) {
    var focused by remember { mutableStateOf(false) }
    Surface(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth().height(86.dp).onFocusChanged { focused = it.isFocused },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(20.dp)),
        colors = ClickableSurfaceDefaults.colors(
            containerColor = if (focused) accent.copy(alpha = 0.18f) else Color.White.copy(alpha = 0.05f),
            contentColor = accent
        )
    ) {
        Row(
            modifier = Modifier.fillMaxSize().padding(horizontal = 18.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            Icon(icon, null, tint = accent)
            Column {
                Text(title, color = accent, fontWeight = FontWeight.Black, fontSize = 16.sp)
                Text(subtitle, color = Color.White.copy(alpha = 0.56f), fontSize = 13.sp)
            }
        }
    }
}

@Composable
internal fun SettingsPanel(content: @Composable ColumnScope.() -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(20.dp))
            .background(Color.White.copy(alpha = 0.05f))
            .border(1.dp, Color.White.copy(alpha = 0.05f), RoundedCornerShape(20.dp))
            .padding(18.dp)
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(8.dp), content = content)
    }
}

@Composable
internal fun SettingsConnectionTile(
    title: String,
    value: String,
    @DrawableRes iconRes: Int,
    onClick: (() -> Unit)? = null
) {
    val clickableModifier = if (onClick != null) {
        Modifier.clickable { onClick() }
    } else {
        Modifier
    }

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(20.dp))
            .background(Color.White.copy(alpha = 0.05f))
            .border(1.dp, Color.White.copy(alpha = 0.05f), RoundedCornerShape(20.dp))
            .then(clickableModifier)
            .padding(horizontal = 18.dp, vertical = 16.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Box(
                    modifier = Modifier
                        .size(30.dp)
                        .clip(RoundedCornerShape(9.dp))
                        .background(Color.White.copy(alpha = 0.06f)),
                    contentAlignment = Alignment.Center
                ) {
                    Image(
                        painter = painterResource(id = iconRes),
                        contentDescription = title,
                        modifier = Modifier.size(21.dp)
                    )
                }

                Text(
                    text = title,
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 16.sp
                )
            }

            Text(
                text = value,
                color = Color.White.copy(alpha = 0.62f),
                fontSize = 13.sp,
                fontWeight = FontWeight.Medium
            )
        }
    }
}

@Composable
internal fun SettingsPill(text: String, selected: Boolean, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(999.dp))
            .background(if (selected) Color.White else Color.White.copy(alpha = 0.06f))
            .clickable { onClick() }
            .padding(horizontal = 14.dp, vertical = 9.dp)
    ) {
        Text(text, color = if (selected) Color.Black else Color.White, fontWeight = FontWeight.Bold, fontSize = 13.sp)
    }
}
