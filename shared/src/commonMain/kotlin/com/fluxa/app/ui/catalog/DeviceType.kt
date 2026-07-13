package com.fluxa.app.ui.catalog

import androidx.compose.foundation.clickable
import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.Modifier

enum class DeviceType { TV, Mobile }
val LocalDeviceType = compositionLocalOf { DeviceType.TV }

@Composable
fun Modifier.touchClickable(onClick: () -> Unit): Modifier {
    val deviceType = LocalDeviceType.current
    return if (deviceType == DeviceType.Mobile) {
        this.clickable(onClick = onClick)
    } else {
        this
    }
}
