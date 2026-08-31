package com.fluxa.app.shared.feature.auth

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

@Composable
actual fun DeviceQrCode(payload: String, modifier: Modifier) = Text(payload, modifier = modifier)
