package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.WatchlistStore
import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch

internal class HomeWatchlistFlowBinder(
    private val watchlistStore: WatchlistStore,
    private val scope: CoroutineScope,
    private val setWatchlist: (List<Meta>) -> Unit,
    private val setLocalContinueWatching: (List<Meta>) -> Unit,
    private val setExternalContinueWatching: (List<Meta>) -> Unit,
    private val setLikedItems: (List<Meta>) -> Unit,
    private val refreshDynamicRows: () -> Unit,
    private val prefetchContinueWatchingArtwork: (List<Meta>) -> Unit
) {
    fun bind() {
        scope.launch {
            watchlistStore.observeWatchlist().collectLatest { list ->
                setWatchlist(list)
                refreshDynamicRows()
            }
        }
        scope.launch {
            watchlistStore.observeContinueWatching().collectLatest { list ->
                setLocalContinueWatching(list)
                refreshDynamicRows()
                prefetchContinueWatchingArtwork(list)
            }
        }
        scope.launch {
            watchlistStore.observeExternalContinueWatching().collectLatest { list ->
                setExternalContinueWatching(list)
                refreshDynamicRows()
                prefetchContinueWatchingArtwork(list)
            }
        }
        scope.launch {
            watchlistStore.observeLiked().collectLatest { list ->
                setLikedItems(list)
            }
        }
    }
}
