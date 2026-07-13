package com.fluxa.app.shared.di

import androidx.lifecycle.ViewModel
import com.fluxa.app.shared.platformName
import org.koin.core.module.dsl.viewModel
import org.koin.dsl.module

class GreetingViewModel : ViewModel() {
    val greeting: String = "Fluxa on ${platformName()}"
}

val sharedModule = module {
    viewModel { GreetingViewModel() }
}
