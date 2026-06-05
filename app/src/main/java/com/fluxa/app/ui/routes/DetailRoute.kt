package com.fluxa.app.ui.routes

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.viewmodel.compose.viewModel
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import com.fluxa.app.ui.asNavigationMeta
import com.fluxa.app.ui.toMeta
import androidx.hilt.navigation.compose.hiltViewModel
import com.fluxa.app.ui.catalog.DetailScreen
import com.fluxa.app.ui.catalog.DetailViewModel

@Composable
internal fun DetailRoute(
    screen: Screen.Detail,
    activeProfile: UserProfile?,
    navigator: AppNavigator,
    onBack: () -> Unit
) {
    val context = LocalContext.current
    val detailViewModel: DetailViewModel = hiltViewModel(
        key = "DetailViewModel_${screen.id}"
    )
    DetailScreen(
        screen.type,
        screen.id,
        activeProfile,
        screen.initialProgress,
        screen.lastVideoId,
        screen.lastStreamIndex,
        screen.autoPlay,
        screen.targetSeason,
        screen.targetEpisode,
        onBack,
        { stream, episode ->
            val snap = detailViewModel.uiState.value
            val savedPlayback = snap.savedPlayback
            val effectiveLastVideoId = screen.lastVideoId ?: savedPlayback?.lastVideoId
            val effectiveInitialProgress = screen.initialProgress ?: savedPlayback?.timeOffset
            val index = snap.streams.indexOf(stream).coerceAtLeast(0)
            val targetVideoId = episode?.id ?: effectiveLastVideoId
            val resumeProgress = if (targetVideoId != null && targetVideoId == effectiveLastVideoId) {
                effectiveInitialProgress ?: 0L
            } else {
                0L
            }
            navigator.navigateTo(
                Screen.Player(
                    snap.detail?.toMeta() ?: Meta(screen.id, "", screen.type, "", ""),
                    videoId = targetVideoId,
                    initialProgress = resumeProgress,
                    streamIndex = index,
                    initialStreams = snap.filteredStreams,
                    lastStreamUrl = stream.playableUrl,
                    lastStreamTitle = stream.title
                )
            )
        },
        { selectedEpisode ->
            detailViewModel.uiState.value.detail?.let { meta ->
                val savedPlayback = detailViewModel.uiState.value.savedPlayback
                val effectiveLastVideoId = screen.lastVideoId ?: savedPlayback?.lastVideoId
                val effectiveInitialProgress = screen.initialProgress ?: savedPlayback?.timeOffset
                val targetVideoId = if (meta.type == "series") selectedEpisode?.id ?: effectiveLastVideoId else null
                val resumeProgress = if (selectedEpisode?.id != null && selectedEpisode.id == effectiveLastVideoId) {
                    effectiveInitialProgress ?: 0L
                } else {
                    0L
                }
                if (meta.id.startsWith("cs3:") || targetVideoId?.startsWith("cs3:") == true) {
                    navigator.navigateTo(
                        Screen.Player(
                            meta = meta.asNavigationMeta(),
                            videoId = targetVideoId,
                            initialProgress = resumeProgress
                        )
                    )
                } else {
                    navigator.navigateTo(
                        Screen.Sources(
                            meta = meta.asNavigationMeta(),
                            video = selectedEpisode,
                            videoId = targetVideoId,
                            initialProgress = resumeProgress,
                            lastStreamIndex = screen.lastStreamIndex,
                            lastStreamUrl = screen.lastStreamUrl,
                            lastStreamTitle = screen.lastStreamTitle
                        )
                    )
                }
            }
        },
        { episode ->
            detailViewModel.uiState.value.detail?.let {
                val savedPlayback = detailViewModel.uiState.value.savedPlayback
                val effectiveLastVideoId = screen.lastVideoId ?: savedPlayback?.lastVideoId
                val effectiveInitialProgress = screen.initialProgress ?: savedPlayback?.timeOffset
                val resumeProgress = if (episode.id == effectiveLastVideoId) effectiveInitialProgress ?: 0L else 0L
                if (it.id.startsWith("cs3:") || episode.id.startsWith("cs3:")) {
                    navigator.navigateTo(
                        Screen.Player(
                            meta = it.asNavigationMeta(),
                            videoId = episode.id,
                            initialProgress = resumeProgress
                        )
                    )
                } else {
                    navigator.navigateTo(
                        Screen.Sources(
                            meta = it.asNavigationMeta(),
                            video = episode,
                            videoId = episode.id,
                            initialProgress = resumeProgress,
                            lastStreamIndex = screen.lastStreamIndex,
                            lastStreamUrl = screen.lastStreamUrl,
                            lastStreamTitle = screen.lastStreamTitle
                        )
                    )
                }
            }
        },
        { episode ->
            detailViewModel.downloadEpisodes(listOf(episode), context)
        },
        { episodes ->
            detailViewModel.downloadEpisodes(episodes, context)
        },
        detailViewModel,
        screen.lastStreamUrl,
        screen.lastStreamTitle,
        screen.sourceAddonTransportUrl,
        screen.sourceAddonCatalogType,
        screen.initialMeta
    )
}
