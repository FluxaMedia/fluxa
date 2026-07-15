package com.fluxa.app.shared.navigation

sealed interface FluxaRoute {
    data object Home : FluxaRoute
    data object Search : FluxaRoute
    data object Library : FluxaRoute
    data object Calendar : FluxaRoute
    data object Settings : FluxaRoute
    data class Discover(val type: String = "movie", val genre: String? = null) : FluxaRoute
    data class Detail(val id: String, val type: String) : FluxaRoute
}

interface FluxaNavigator {
    fun navigate(route: FluxaRoute, clearBackStack: Boolean = false)
    fun goBack()
}
