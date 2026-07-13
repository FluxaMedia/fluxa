package com.fluxa.app.shared.di

import org.koin.core.context.startKoin

fun initSharedKoin() {
    startKoin {
        modules(sharedModule)
    }
}
