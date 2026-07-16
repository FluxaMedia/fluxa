package com.fluxa.app.shared.navigation

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class FluxaNavigationStore(
    initialRoute: FluxaRoute = FluxaRoute.Home
) : FluxaNavigator {
    private val _backStack = MutableStateFlow(listOf(initialRoute))
    val backStack: StateFlow<List<FluxaRoute>> = _backStack.asStateFlow()
    val currentRoute: FluxaRoute get() = _backStack.value.last()

    override fun navigate(route: FluxaRoute, clearBackStack: Boolean) {
        _backStack.value = when {
            clearBackStack -> listOf(route)
            _backStack.value.lastOrNull() == route -> _backStack.value
            else -> _backStack.value + route
        }
    }

    override fun goBack() {
        if (_backStack.value.size > 1) {
            _backStack.value = _backStack.value.dropLast(1)
        }
    }
}
