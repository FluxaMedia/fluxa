package com.fluxa.app.shared.feature.detail

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn

class DetailStore(
    private val request: DetailRequestUiModel,
    private val dataSource: DetailDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<DetailUiState> = dataSource.observeDetail(request.id, request.type)
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), DetailUiState(isLoading = true))

    private val _navigation = MutableSharedFlow<DetailNavigationEvent>(extraBufferCapacity = 1)
    val navigation: SharedFlow<DetailNavigationEvent> = _navigation

    suspend fun load() {
        dataSource.loadDetail(request)
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
                val targetVideoId = action.episodeId ?: content?.resumeVideoId
                val progress = resumeProgressFor(content, targetVideoId)
                _navigation.emit(DetailNavigationEvent.PlayStream(action.stream, action.episodeId, progress))
            }
            DetailAction.Play -> {
                val content = state.value.content
                val episodeId = content?.selectedEpisodeId
                val targetVideoId = episodeId ?: content?.resumeVideoId
                val progress = resumeProgressFor(content, targetVideoId)
                val isCs3 = content?.id?.startsWith("cs3:") == true || episodeId?.startsWith("cs3:") == true
                if (isCs3) {
                    val stream = content?.streams?.firstOrNull()
                    if (stream != null) {
                        _navigation.emit(DetailNavigationEvent.PlayStream(stream, episodeId, progress))
                    } else {
                        _navigation.emit(DetailNavigationEvent.SelectSources(episodeId, progress))
                    }
                } else {
                    _navigation.emit(DetailNavigationEvent.SelectSources(episodeId, progress))
                }
            }
            is DetailAction.RelatedItemSelected -> Unit
        }
    }

    private fun resumeProgressFor(content: DetailUiModel?, targetVideoId: String?): Long =
        if (targetVideoId != null && targetVideoId == content?.resumeVideoId) content.resumeProgress else 0L
}
