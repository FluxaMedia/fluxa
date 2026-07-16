package com.fluxa.app.data.local

import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class InMemoryWatchlistStoreTest {
    @Test
    fun togglesWatchlistItems() = runTest {
        val store = InMemoryWatchlistStore()
        val item = Meta(id = "id", name = "Title", type = "movie")

        store.toggleWatchlist(item)
        assertTrue(store.isInWatchlist(item.id))
        store.toggleWatchlist(item)
        assertFalse(store.isInWatchlist(item.id))
    }
}
