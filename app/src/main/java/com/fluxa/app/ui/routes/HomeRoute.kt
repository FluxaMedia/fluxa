package com.fluxa.app.ui.routes

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import com.fluxa.app.ui.catalog.HomeScreen
import com.fluxa.app.ui.catalog.HomeViewModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

@Composable
internal fun HomeRoute(
    activeProfile: UserProfile?,
    navigator: AppNavigator,
    homeViewModel: HomeViewModel,
    previewPlayer: ExoPlayer,
    coroutineScope: CoroutineScope
) {
    HomeScreen(
        activeProfile,
        onMovieClick = { meta, addonUrl, catalogType ->
            if (meta.type == "catalog_folder") {
                navigator.navigateTo(Screen.CategoryResults(meta.id, meta.name))
            } else {
                navigator.navigateTo(meta.detailScreen(addonUrl, catalogType))
            }
        },
        onPlayDirect = { item ->
            val canResumeDirect = item.lastVideoId != null || (item.timeOffset ?: 0L) > 0L
            if (canResumeDirect) {
                navigator.navigateTo(item.resumePlayerScreen(returnToSourcesOnError = true))
            } else if (item.type == "movie") {
                coroutineScope.launch {
                    val target = homeViewModel.prepareDirectPlayback(item)
                    if (target != null) {
                        val streamIndex = when {
                            !item.lastStreamUrl.isNullOrBlank() -> target.streams.indexOfFirst { stream -> stream.playableUrl == item.lastStreamUrl }
                            !item.lastStreamTitle.isNullOrBlank() -> target.streams.indexOfFirst { stream -> stream.title == item.lastStreamTitle }
                            item.lastStreamIndex != null && item.lastStreamIndex in target.streams.indices -> item.lastStreamIndex!!
                            else -> 0
                        }.coerceAtLeast(0)
                        navigator.navigateTo(
                            Screen.Player(
                                meta = target.meta,
                                videoId = item.lastVideoId ?: target.videoId,
                                initialProgress = 0L,
                                streamIndex = streamIndex,
                                initialStreams = target.streams,
                                lastStreamUrl = target.streams.getOrNull(streamIndex)?.playableUrl,
                                lastStreamTitle = target.streams.getOrNull(streamIndex)?.title,
                                returnToSourcesOnError = false
                            )
                        )
                    } else if (item.id.startsWith("cs3:")) {
                        // CS3 content: Player handles stream loading via the plugin directly.
                        navigator.navigateTo(
                            Screen.Player(
                                meta = item,
                                videoId = item.lastVideoId,
                                initialProgress = 0L
                            )
                        )
                    } else {
                        navigator.navigateTo(Screen.Sources(item))
                    }
                }
            } else {
                navigator.navigateTo(Screen.Detail(item.type, item.id))
            }
        },
        onWatchlistClick = { navigator.navigateTo(Screen.Watchlist) },
        onProfileClick = { navigator.navigateTo(Screen.Settings()) },
        onSearchClick = { navigator.navigateTo(Screen.Search) },
        onExploreClick = { type, genre -> navigator.navigateTo(Screen.Explore(initialType = type, initialGenre = genre)) },
        onCategoryClick = { category -> navigator.navigateTo(Screen.CategoryResults(category.id, category.name)) },
        viewModel = homeViewModel,
        sharedPlayer = previewPlayer,
        activePreviewUrl = homeViewModel.previewUrl.collectAsState().value
    )
}
