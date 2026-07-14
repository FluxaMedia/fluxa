package com.fluxa.app.ui.routes

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import com.fluxa.app.data.local.OfflineDownloadManager
import com.fluxa.app.shared.feature.library.LibraryAction
import com.fluxa.app.shared.feature.library.LibraryScreen
import com.fluxa.app.shared.feature.library.LibraryStore
import com.fluxa.app.shared.platform.FluxaLibraryServices
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import kotlinx.coroutines.launch

@Composable
internal fun SharedTvLibraryRoute(
    platformServices: FluxaLibraryServices,
    navigator: AppNavigator,
    offlineDownloadManager: OfflineDownloadManager,
    language: String?
) {
    val scope = rememberCoroutineScope()
    val store = remember(platformServices.libraryDataSource) {
        LibraryStore(platformServices.libraryDataSource, scope)
    }
    val state by store.state.collectAsState()

    LaunchedEffect(store) {
        store.dispatch(LibraryAction.Refresh)
    }

    LibraryScreen(
        state = state,
        language = language,
        onItemSelected = { item ->
            navigator.navigateTo(
                Screen.Detail(
                    type = item.type,
                    id = item.id,
                    sourceAddonTransportUrl = item.source.addonTransportUrl,
                    sourceAddonCatalogType = item.source.catalogType
                )
            )
        },
        onAction = { action ->
            if (action is LibraryAction.DownloadOpened) {
                offlineDownloadManager.items.value.firstOrNull { it.id == action.id }?.let { item ->
                    if (item.isPlayable) {
                        navigator.navigateTo(
                            Screen.Player(
                                meta = offlineDownloadManager.asPlayableMeta(item),
                                videoId = item.videoId,
                                initialProgress = 0L,
                                streamIndex = 0,
                                initialStreams = listOf(offlineDownloadManager.asPlayableStream(item))
                            )
                        )
                    }
                }
            } else {
                scope.launch { store.dispatch(action) }
            }
        }
    )
}
