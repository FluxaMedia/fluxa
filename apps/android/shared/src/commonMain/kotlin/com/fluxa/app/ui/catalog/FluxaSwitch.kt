package com.fluxa.app.ui.catalog

import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance

@Composable
fun FluxaSwitch(
    checked: Boolean,
    onCheckedChange: ((Boolean) -> Unit)?,
    modifier: Modifier = Modifier,
    accentColor: Color = LocalAccentColor.current,
) {
    val checkedThumbColor = if (accentColor.luminance() > 0.5f) Color.Black else Color.White
    Switch(
        checked = checked,
        onCheckedChange = onCheckedChange,
        modifier = modifier,
        colors = SwitchDefaults.colors(
            checkedThumbColor = checkedThumbColor,
            checkedTrackColor = accentColor,
            checkedBorderColor = Color.Transparent,
            uncheckedThumbColor = Color.White.copy(alpha = 0.8f),
            uncheckedTrackColor = Color.White.copy(alpha = 0.18f),
            uncheckedBorderColor = Color.Transparent,
        ),
    )
}
