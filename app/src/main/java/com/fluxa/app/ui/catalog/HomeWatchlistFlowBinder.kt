package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch

internal class HomeWatchlistFlowBinder(
    private val watchlistManager: WatchlistManager,
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
            watchlistManager.getWatchlistFlow().collectLatest { list ->
                setWatchlist(list)
                refreshDynamicRows()
            }
        }
        scope.launch {
            watchlistManager.getContinueWatchingFlow().collectLatest { list ->
                setLocalContinueWatching(list)
                refreshDynamicRows()
                prefetchContinueWatchingArtwork(list)
            }
        }
        scope.launch {
            watchlistManager.getExternalContinueWatchingFlow().collectLatest { list ->
                setExternalContinueWatching(list)
                refreshDynamicRows()
                prefetchContinueWatchingArtwork(list)
            }
        }
        scope.launch {
            watchlistManager.getLikedFlow().collectLatest { list ->
                setLikedItems(list)
            }
        }
    }
}
