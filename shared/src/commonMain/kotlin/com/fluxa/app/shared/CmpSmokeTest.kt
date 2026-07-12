package com.fluxa.app.shared

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable

@Composable
fun CmpSmokeTest() {
    Column {
        Text("Fluxa on ${platformName()}")
    }
}
