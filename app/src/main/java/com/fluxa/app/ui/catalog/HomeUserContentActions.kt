package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

internal class HomeUserContentActions(
    private val watchlistManager: WatchlistManager,
    private val scope: CoroutineScope,
    private val setActiveProfile: (UserProfile) -> Unit,
    private val refreshDynamicRows: () -> Unit
) {
    fun toggleWatchlist(meta: Meta) {
        scope.launch {
            watchlistManager.toggleWatchlist(meta)
        }
    }

    fun setFeedback(movie: Meta, isLike: Boolean) {
        scope.launch {
            watchlistManager.setFeedback(movie.id, isLike, movie)
        }
    }

    fun applyUpdatedProfile(profile: UserProfile) {
        setActiveProfile(profile)
        refreshDynamicRows()
    }
}
