package com.fluxa.app.ui.routes

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.viewmodel.compose.viewModel
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video
import com.fluxa.app.shared.feature.detail.DetailNavigationEvent
import com.fluxa.app.shared.feature.detail.DetailNavigationLogic
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import com.fluxa.app.ui.asNavigationMeta
import com.fluxa.app.ui.toMeta
import androidx.hilt.navigation.compose.hiltViewModel
import com.fluxa.app.ui.catalog.DetailScreen
import com.fluxa.app.ui.catalog.DetailViewModel
import com.fluxa.app.ui.catalog.toDetailStreamUiModel

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

    fun effectiveResumeVideoId(): String? = screen.lastVideoId ?: detailViewModel.uiState.value.savedPlayback?.lastVideoId
    fun effectiveResumeProgress(): Long = screen.initialProgress ?: detailViewModel.uiState.value.savedPlayback?.timeOffset ?: 0L

    fun navigate(event: DetailNavigationEvent, meta: Meta, streamsForIndex: List<Stream>, episode: Video?) {
        when (event) {
            is DetailNavigationEvent.PlayStream -> {
                val index = streamsForIndex.indexOfFirst { it.playableUrl == event.stream.playableUrl }.coerceAtLeast(0)
                navigator.navigateTo(
                    Screen.Player(
                        meta,
                        videoId = event.episodeId,
                        initialProgress = event.resumeProgress,
                        streamIndex = index,
                        initialStreams = streamsForIndex,
                        lastStreamUrl = event.stream.playableUrl,
                        lastStreamTitle = event.stream.title
                    )
                )
            }
            is DetailNavigationEvent.SelectSources -> {
                navigator.navigateTo(
                    Screen.Sources(
                        meta = meta,
                        video = episode,
                        videoId = event.episodeId,
                        initialProgress = event.resumeProgress,
                        lastStreamIndex = screen.lastStreamIndex,
                        lastStreamUrl = screen.lastStreamUrl,
                        lastStreamTitle = screen.lastStreamTitle
                    )
                )
            }
        }
    }

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
            val meta = snap.detail?.toMeta() ?: Meta(screen.id, "", screen.type, "", "")
            val streamUiModel = stream.toDetailStreamUiModel()
            if (streamUiModel != null) {
                val event = DetailNavigationLogic.forStream(
                    contentResumeVideoId = effectiveResumeVideoId(),
                    contentResumeProgress = effectiveResumeProgress(),
                    stream = streamUiModel,
                    episodeId = episode?.id
                )
                navigate(event, meta, snap.filteredStreams, episode)
            }
        },
        { selectedEpisode ->
            val snap = detailViewModel.uiState.value
            snap.detail?.let { meta ->
                val isSeries = meta.type == "series"
                val episodeId = if (isSeries) selectedEpisode?.id else null
                val firstStream = snap.streams.firstOrNull()?.toDetailStreamUiModel()
                val event = DetailNavigationLogic.forPlay(
                    contentId = meta.id,
                    contentResumeVideoId = if (isSeries) effectiveResumeVideoId() else null,
                    contentResumeProgress = effectiveResumeProgress(),
                    episodeId = episodeId,
                    firstStreamIfCs3 = firstStream
                )
                navigate(event, meta.asNavigationMeta(), snap.filteredStreams, selectedEpisode)
            }
        },
        { episode ->
            val snap = detailViewModel.uiState.value
            snap.detail?.let { meta ->
                val firstStream = snap.streams.firstOrNull()?.toDetailStreamUiModel()
                val event = DetailNavigationLogic.forPlay(
                    contentId = meta.id,
                    contentResumeVideoId = effectiveResumeVideoId(),
                    contentResumeProgress = effectiveResumeProgress(),
                    episodeId = episode.id,
                    firstStreamIfCs3 = firstStream
                )
                navigate(event, meta.asNavigationMeta(), snap.filteredStreams, episode)
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
