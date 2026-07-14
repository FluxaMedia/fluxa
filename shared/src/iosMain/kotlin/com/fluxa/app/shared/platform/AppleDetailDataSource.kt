package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.detail.DetailRequestUiModel
import com.fluxa.app.shared.feature.detail.DetailUiModel
import com.fluxa.app.shared.feature.detail.DetailUiState
import com.fluxa.app.ui.catalog.CatalogCardUiModel
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

data class AppleDetailRequestSnapshot(
    val id: String,
    val type: String,
    val addonTransportUrl: String? = null,
    val catalogType: String? = null,
    val title: String? = null
)

data class AppleDetailSnapshot(
    val id: String,
    val type: String,
    val title: String,
    val description: String = "",
    val posterUrl: String? = null,
    val backgroundUrl: String? = null,
    val logoUrl: String? = null,
    val releaseLabel: String = "",
    val ratingLabel: String = "",
    val isInWatchlist: Boolean = false,
    val isLoading: Boolean = false,
    val errorKey: String? = null
)

class AppleDetailDataSource : DetailDataSource {
    private val state = MutableStateFlow(DetailUiState())
    private var onLoadRequested: (AppleDetailRequestSnapshot) -> Unit = {}
    private var onWatchlistRequested: (AppleDetailRequestSnapshot) -> Unit = {}

    override fun observeDetail(id: String, type: String): Flow<DetailUiState> = state.asStateFlow()

    override suspend fun loadDetail(request: DetailRequestUiModel) {
        onLoadRequested(request.toAppleSnapshot())
    }

    override suspend fun toggleWatchlist(id: String, type: String) {
        onWatchlistRequested(AppleDetailRequestSnapshot(id, type, title = state.value.content?.title))
    }

    override suspend fun selectSeason(season: Int) {
        Unit
    }

    override suspend fun selectEpisode(episodeId: String) {
        Unit
    }

    override suspend fun selectAddonFilter(addonName: String?) {
        Unit
    }

    override suspend fun downloadEpisode(episodeId: String) {
        Unit
    }

    override suspend fun downloadSeason(season: Int) {
        Unit
    }

    fun setHandlers(
        load: (AppleDetailRequestSnapshot) -> Unit,
        watchlist: (AppleDetailRequestSnapshot) -> Unit
    ) {
        onLoadRequested = load
        onWatchlistRequested = watchlist
    }

    fun update(snapshot: AppleDetailSnapshot) {
        state.value = DetailUiState(
            content = DetailUiModel(snapshot.id, snapshot.type, snapshot.title, snapshot.description, snapshot.posterUrl, snapshot.backgroundUrl, snapshot.logoUrl, snapshot.releaseLabel, snapshot.ratingLabel, null, isInWatchlist = snapshot.isInWatchlist, relatedItems = emptyList()),
            isLoading = snapshot.isLoading,
            errorKey = snapshot.errorKey
        )
    }
}

private fun DetailRequestUiModel.toAppleSnapshot() = AppleDetailRequestSnapshot(id, type, source.addonTransportUrl, source.catalogType)
