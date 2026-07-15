package com.fluxa.app.ui

import com.fluxa.app.shared.navigation.FluxaNavigator
import com.fluxa.app.shared.navigation.FluxaRoute

class AndroidFluxaNavigator(
    private val navigator: AppNavigator
) : FluxaNavigator {
    override fun navigate(route: FluxaRoute, clearBackStack: Boolean) {
        navigator.navigateTo(route.toAndroidScreen(), clearBackStack)
    }

    override fun goBack() {
        navigator.navigateBack()
    }
}

fun FluxaRoute.toAndroidScreen(): Screen = when (this) {
    FluxaRoute.Home -> Screen.Home
    FluxaRoute.Search -> Screen.Search
    FluxaRoute.Library -> Screen.Watchlist
    FluxaRoute.Calendar -> Screen.Calendar
    FluxaRoute.Settings -> Screen.Settings()
    is FluxaRoute.Discover -> Screen.Explore(initialType = type, initialGenre = genre)
    is FluxaRoute.Detail -> Screen.Detail(type = type, id = id)
}
