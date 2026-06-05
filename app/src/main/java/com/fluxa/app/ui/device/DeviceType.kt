package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

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
