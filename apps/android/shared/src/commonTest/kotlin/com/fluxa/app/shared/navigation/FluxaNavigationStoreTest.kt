package com.fluxa.app.shared.navigation

import kotlin.test.Test
import kotlin.test.assertEquals

class FluxaNavigationStoreTest {
    @Test
    fun navigationMaintainsPortableBackStack() {
        val store = FluxaNavigationStore()

        store.navigate(FluxaRoute.Detail("item", "movie"))
        store.navigate(FluxaRoute.Player("item", "movie"))
        store.goBack()

        assertEquals(FluxaRoute.Detail("item", "movie"), store.currentRoute)
    }

    @Test
    fun clearBackStackReplacesHistory() {
        val store = FluxaNavigationStore(FluxaRoute.Search)

        store.navigate(FluxaRoute.Library, clearBackStack = true)
        store.goBack()

        assertEquals(listOf(FluxaRoute.Library), store.backStack.value)
    }
}
