package com.fluxa.app.shared.feature.detail

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

class DetailStore(
    private val request: DetailRequestUiModel,
    private val dataSource: DetailDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<DetailUiState> = dataSource.observeDetail(request.id, request.type)
        .map { state ->
            if (state.content != null || request.initialContent == null) state
            else state.copy(content = request.initialContent)
        }
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), DetailUiState(isLoading = true))

    private val _navigation = MutableSharedFlow<DetailNavigationEvent>(extraBufferCapacity = 1)
    val navigation: SharedFlow<DetailNavigationEvent> = _navigation

    suspend fun load() {
        dataSource.loadDetail(request)
    }

    suspend fun loadSources(episodeId: String?) {
        dataSource.loadSources(request.id, request.type, episodeId)
    }

    suspend fun dispatch(action: DetailAction) {
        when (action) {
            DetailAction.ToggleWatchlist -> dataSource.toggleWatchlist(request.id, request.type)
            is DetailAction.SeasonSelected -> dataSource.selectSeason(action.season)
            is DetailAction.EpisodeSelected -> dataSource.selectEpisode(action.episodeId)
            is DetailAction.AddonFilterSelected -> dataSource.selectAddonFilter(action.addonName)
            is DetailAction.DownloadEpisode -> dataSource.downloadEpisode(action.episodeId)
            is DetailAction.DownloadSeason -> dataSource.downloadSeason(action.season)
            is DetailAction.StreamSelected -> {
                val content = state.value.content
                _navigation.emit(
                    DetailNavigationLogic.forStream(
                        contentResumeVideoId = content?.resumeVideoId,
                        contentResumeProgress = content?.resumeProgress ?: 0L,
                        stream = action.stream,
                        episodeId = action.episodeId
                    )
                )
            }
            is DetailAction.Play -> {
                val content = state.value.content
                val episodeId = content?.selectedEpisodeId
                _navigation.emit(
                    DetailNavigationLogic.forPlay(
                        contentId = content?.id,
                        contentResumeVideoId = content?.resumeVideoId,
                        contentResumeProgress = content?.resumeProgress ?: 0L,
                        episodeId = episodeId,
                        firstStreamIfCs3 = content?.streams?.firstOrNull(),
                        fromStart = action.fromStart
                    )
                )
            }
            is DetailAction.RelatedItemSelected -> Unit
        }
    }
}
