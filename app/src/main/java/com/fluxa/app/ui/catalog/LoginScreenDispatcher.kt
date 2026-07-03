package com.fluxa.app.ui.catalog

import android.content.Context
import androidx.compose.runtime.Composable
import com.fluxa.app.data.local.UserProfile

@Composable
fun LoginScreen(
    context: Context,
    onLoginSuccess: (UserProfile) -> Unit,
    onCancel: () -> Unit,
    startOnNuvio: Boolean = false
) {
    if (LocalDeviceType.current == DeviceType.Mobile) {
        MobileLoginScreen(context = context, onLoginSuccess = onLoginSuccess, onCancel = onCancel, startOnNuvio = startOnNuvio)
    } else {
        TvLoginScreen(context = context, onLoginSuccess = onLoginSuccess, onCancel = onCancel)
    }
}
