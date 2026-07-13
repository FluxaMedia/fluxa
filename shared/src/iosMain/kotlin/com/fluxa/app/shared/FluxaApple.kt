package com.fluxa.app.shared

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.window.ComposeUIViewController
import com.fluxa.app.common.AppStrings
import platform.UIKit.UIViewController

object FluxaApple {
    fun rootViewController(): UIViewController = ComposeUIViewController {
        FluxaApp()
    }
}

@Composable
private fun FluxaApp() {
    MaterialTheme {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center
        ) {
            Text(AppStrings.t(null, "nav.home"))
        }
    }
}
