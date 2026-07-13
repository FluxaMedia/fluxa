package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.detail.DetailUiModel
import com.fluxa.app.shared.feature.detail.DetailUiState as SharedDetailUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class AndroidDetailDataSource(
    private val detailViewModel: DetailViewModel,
    private val activeProfile: () -> UserProfile?
) : DetailDataSource {
    override fun observeDetail(id: String, type: String): Flow<SharedDetailUiState> {
        return detailViewModel.uiState.map { state ->
            SharedDetailUiState(
                content = state.detail?.let { detail ->
                    DetailUiModel(
                        id = detail.id,
                        type = detail.type,
                        title = detail.name,
                        description = detail.description.orEmpty(),
                        posterUrl = detail.poster,
                        backgroundUrl = detail.background,
                        logoUrl = detail.logo,
                        releaseLabel = detail.releaseInfo.orEmpty(),
                        ratingLabel = detail.imdbRating.orEmpty(),
                        isInWatchlist = state.isInWatchlist,
                        relatedItems = state.similarItems.toCatalogItems(activeProfile())
                    )
                },
                isLoading = state.isLoading
            )
        }
    }

    override suspend fun loadDetail(id: String, type: String) {
        detailViewModel.loadDetail(type = type, id = id, profile = activeProfile())
    }

    override suspend fun toggleWatchlist(id: String, type: String) {
        detailViewModel.toggleWatchlist()
    }
}
