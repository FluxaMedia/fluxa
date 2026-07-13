package com.fluxa.app.shared

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import com.fluxa.app.shared.di.GreetingViewModel
import org.koin.compose.viewmodel.koinViewModel

@Composable
fun CmpSmokeTest() {
    val viewModel = koinViewModel<GreetingViewModel>()
    Column {
        Text(viewModel.greeting)
    }
}
