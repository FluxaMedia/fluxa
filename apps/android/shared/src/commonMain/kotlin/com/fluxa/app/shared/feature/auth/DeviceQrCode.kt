package com.fluxa.app.shared.feature.auth

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

@Composable
expect fun DeviceQrCode(payload: String, modifier: Modifier = Modifier)
