@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.Image
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.annotation.DrawableRes
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.graphics.drawscope.Stroke
import coil3.compose.AsyncImage
import java.util.Locale

@Composable
internal fun MobileChoiceRow(title: String, value: String, subtitle: String? = null, onClick: () -> Unit) {
    val colors = LocalMobileSettingsPalette.current
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onClick() }
            .padding(horizontal = 16.dp, vertical = colors.rowVerticalPadding),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                color = colors.text.copy(alpha = 0.9f),
                fontSize = 15.sp,
                fontWeight = FontWeight.SemiBold
            )
            subtitle?.let {
                Text(
                    text = it,
                    color = colors.text.copy(alpha = 0.44f),
                    fontSize = 12.sp,
                    lineHeight = 16.sp
                )
            }
        }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(value, color = colors.text.copy(alpha = 0.52f), fontSize = 13.sp, fontWeight = FontWeight.Medium)
            Icon(FluxaIcons.ChevronRight, null, tint = colors.text.copy(alpha = 0.32f), modifier = Modifier.size(18.dp))
        }
    }
}

@Composable
internal fun MobileToggleRow(
    title: String,
    checked: Boolean,
    onToggle: () -> Unit,
    subtitle: String? = null
) {
    val colors = LocalMobileSettingsPalette.current
    var localChecked by remember { mutableStateOf(checked) }
    LaunchedEffect(checked) { localChecked = checked }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable {
                localChecked = !localChecked
                onToggle()
            }
            .padding(horizontal = 16.dp, vertical = colors.rowVerticalPadding),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                color = colors.text.copy(alpha = 0.9f),
                fontSize = 15.sp,
                fontWeight = FontWeight.SemiBold
            )
            subtitle?.let {
                Text(
                    text = it,
                    color = colors.text.copy(alpha = 0.44f),
                    fontSize = 12.sp,
                    lineHeight = 16.sp
                )
            }
        }
        Switch(
            checked = localChecked,
            onCheckedChange = { localChecked = it; onToggle() },
            colors = SwitchDefaults.colors(
                checkedThumbColor = colors.onAccent,
                checkedTrackColor = colors.accent,
                uncheckedThumbColor = colors.text.copy(alpha = 0.72f),
                uncheckedTrackColor = colors.text.copy(alpha = 0.16f)
            )
        )
    }
}

@Composable
internal fun MobileExpandableHeaderRow(
    title: String,
    count: Int,
    subtitle: String? = null,
    expanded: Boolean,
    onClick: () -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onClick() }
            .padding(horizontal = 12.dp, vertical = colors.rowVerticalPadding),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(title, color = colors.text.copy(alpha = 0.9f), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            Text(
                subtitle ?: count.toString(),
                color = colors.text.copy(alpha = 0.42f),
                fontSize = 10.sp,
                fontWeight = FontWeight.Normal,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis
            )
        }
        Icon(
            if (expanded) FluxaIcons.KeyboardArrowUp else FluxaIcons.KeyboardArrowDown,
            null,
            tint = colors.text.copy(alpha = 0.48f),
            modifier = Modifier.size(20.dp)
        )
    }
}

internal fun settingsIsEnglish(lang: String?): Boolean {
    return lang?.substringBefore('-')?.substringBefore('_')?.equals("en", ignoreCase = true) == true
}

@Composable
internal fun MobileOrderedToggleRow(
    title: String,
    checked: Boolean,
    onToggle: () -> Unit,
    canMoveUp: Boolean,
    canMoveDown: Boolean,
    onMoveUp: () -> Unit,
    onMoveDown: () -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    val display = remember(title) { metadataFeedDisplay(title) }
    var localChecked by remember { mutableStateOf(checked) }
    LaunchedEffect(checked) { localChecked = checked }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = 8.dp, end = 8.dp, top = colors.rowVerticalPadding + 2.dp, bottom = colors.rowVerticalPadding + 2.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(3.dp)
    ) {
        Column(
            modifier = Modifier
                .weight(1f)
                .clickable { localChecked = !localChecked; onToggle() }
                .padding(start = 0.dp, top = 4.dp, end = 2.dp, bottom = 4.dp)
        ) {
            Text(
                text = display.title,
                color = colors.text,
                fontSize = 13.sp,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis
            )
            Text(
                text = display.provider,
                color = colors.text.copy(alpha = 0.42f),
                fontSize = 10.sp,
                fontWeight = FontWeight.Normal,
                maxLines = 1,
                overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis
            )
        }
        Row(
            modifier = Modifier.width(38.dp),
            horizontalArrangement = Arrangement.spacedBy(0.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                FluxaIcons.KeyboardArrowUp,
                null,
                tint = colors.text.copy(alpha = if (canMoveUp) 0.82f else 0.22f),
                modifier = Modifier
                    .size(19.dp)
                    .clip(CircleShape)
                    .clickable(enabled = canMoveUp) { onMoveUp() }
                    .padding(1.dp)
            )
            Icon(
                FluxaIcons.KeyboardArrowDown,
                null,
                tint = colors.text.copy(alpha = if (canMoveDown) 0.82f else 0.22f),
                modifier = Modifier
                    .size(19.dp)
                    .clip(CircleShape)
                    .clickable(enabled = canMoveDown) { onMoveDown() }
                    .padding(1.dp)
            )
        }
        Switch(
            checked = localChecked,
            onCheckedChange = { localChecked = it; onToggle() },
            colors = SwitchDefaults.colors(
                checkedThumbColor = colors.onAccent,
                checkedTrackColor = colors.accent,
                uncheckedThumbColor = colors.text.copy(alpha = 0.72f),
                uncheckedTrackColor = colors.text.copy(alpha = 0.16f)
            )
        )
    }
}

internal data class MetadataFeedDisplay(
    val title: String,
    val provider: String
)

internal fun metadataFeedDisplay(label: String): MetadataFeedDisplay {
    val parts = label.split(" - ").map { it.trim() }.filter { it.isNotEmpty() }
    if (parts.size >= 2) {
        return MetadataFeedDisplay(
            title = parts.drop(1).joinToString(" - "),
            provider = parts.first()
        )
    }
    return MetadataFeedDisplay(title = label, provider = "Metadata")
}

@Composable
internal fun MobileProviderToggleRow(
    title: String,
    allLabel: String,
    checked: Boolean,
    onToggle: () -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    var localChecked by remember { mutableStateOf(checked) }
    LaunchedEffect(checked) { localChecked = checked }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 8.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                color = colors.text,
                fontSize = 13.sp,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis
            )
            Text(
                text = allLabel,
                color = colors.text.copy(alpha = 0.42f),
                fontSize = 10.sp,
                fontWeight = FontWeight.Normal
            )
        }
        Switch(
            checked = localChecked,
            onCheckedChange = { localChecked = it; onToggle() },
            colors = SwitchDefaults.colors(
                checkedThumbColor = colors.onAccent,
                checkedTrackColor = colors.accent,
                uncheckedThumbColor = colors.text.copy(alpha = 0.72f),
                uncheckedTrackColor = colors.text.copy(alpha = 0.16f)
            )
        )
    }
}

@Composable
internal fun MobileStepperRow(title: String, value: String, subtitle: String? = null, onDecrease: () -> Unit, onIncrease: () -> Unit) {
    val colors = LocalMobileSettingsPalette.current
    Row(
        modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = colors.rowVerticalPadding),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                color = colors.text.copy(alpha = 0.9f),
                fontSize = 15.sp,
                fontWeight = FontWeight.SemiBold
            )
            subtitle?.let {
                Text(
                    text = it,
                    color = colors.text.copy(alpha = 0.44f),
                    fontSize = 12.sp,
                    lineHeight = 16.sp
                )
            }
        }
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Icon(
                FluxaIcons.Remove,
                null,
                tint = colors.text.copy(alpha = 0.82f),
                modifier = Modifier
                    .size(30.dp)
                    .clip(CircleShape)
                    .clickable { onDecrease() }
                    .padding(5.dp)
            )
            Text(value, color = colors.text.copy(alpha = 0.78f), fontSize = 15.sp, fontWeight = FontWeight.Medium)
            Icon(
                FluxaIcons.Add,
                null,
                tint = colors.text.copy(alpha = 0.82f),
                modifier = Modifier
                    .size(30.dp)
                    .clip(CircleShape)
                    .clickable { onIncrease() }
                    .padding(5.dp)
            )
        }
    }
}

@Composable
internal fun MobilePercentSliderRow(
    title: String,
    value: Float,
    subtitle: String? = null,
    valueRange: ClosedFloatingPointRange<Float> = 50f..99f,
    steps: Int = 48,
    onValueChange: (Float) -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    val safeValue = value.coerceIn(valueRange.start, valueRange.endInclusive)
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = colors.rowVerticalPadding),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = title,
                    color = colors.text.copy(alpha = 0.9f),
                    fontSize = 15.sp,
                    fontWeight = FontWeight.SemiBold
                )
                subtitle?.let {
                    Text(
                        text = it,
                        color = colors.text.copy(alpha = 0.44f),
                        fontSize = 12.sp,
                        lineHeight = 16.sp
                    )
                }
            }
            Text(
                text = "${safeValue.toInt()}%",
                color = colors.text.copy(alpha = 0.64f),
                fontSize = 13.sp,
                fontWeight = FontWeight.Medium
            )
        }
        Slider(
            value = safeValue,
            onValueChange = { onValueChange(it.toInt().toFloat()) },
            valueRange = valueRange,
            steps = steps,
            colors = SliderDefaults.colors(
                thumbColor = colors.accent,
                activeTrackColor = colors.accent,
                inactiveTrackColor = colors.accent.copy(alpha = 0.22f),
                activeTickColor = Color.Transparent,
                inactiveTickColor = Color.Transparent
            )
        )
    }
}

@Composable
internal fun MobileColorOpacityRow(
    title: String,
    subtitle: String,
    opacityTitle: String,
    selectedColor: Int,
    opacity: Float,
    lang: String,
    onColorClick: () -> Unit,
    onOpacity: (Float) -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    Column(modifier = Modifier.fillMaxWidth()) {
        MobileChoiceRow(
            title = title,
            value = mobileColorLabel(selectedColor, lang),
            subtitle = subtitle.ifBlank { null },
            onClick = onColorClick
        )
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 8.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Text(opacityTitle, color = colors.text.copy(alpha = 0.62f), fontSize = 13.sp, fontWeight = FontWeight.Medium)
                Text(mobileOpacityLabel(opacity), color = colors.text.copy(alpha = 0.52f), fontSize = 13.sp, fontWeight = FontWeight.Medium)
            }
            Slider(
                value = opacity.coerceIn(0f, 1f),
                onValueChange = { onOpacity((it * 10f).toInt() / 10f) },
                valueRange = 0f..1f,
                steps = 9,
                colors = SliderDefaults.colors(
                    thumbColor = colors.accent,
                    activeTrackColor = colors.accent,
                    inactiveTrackColor = colors.accent.copy(alpha = 0.22f),
                    activeTickColor = Color.Transparent,
                    inactiveTickColor = Color.Transparent
                )
            )
        }
    }
}
