package com.fluxa.app.shared.feature.detail

import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import kotlinx.coroutines.flow.Flow

data class DetailUiModel(
    val id: String,
    val type: String,
    val title: String,
    val description: String,
    val posterUrl: String?,
    val backgroundUrl: String?,
    val logoUrl: String?,
    val releaseLabel: String,
    val ratingLabel: String,
    val isInWatchlist: Boolean,
    val relatedItems: List<CatalogItemUiModel>
)

data class DetailUiState(
    val content: DetailUiModel? = null,
    val isLoading: Boolean = false,
    val errorKey: String? = null
)

sealed interface DetailAction {
    data object Play : DetailAction
    data object ToggleWatchlist : DetailAction
    data class RelatedItemSelected(val item: CatalogItemUiModel) : DetailAction
}

interface DetailDataSource {
    fun observeDetail(id: String, type: String): Flow<DetailUiState>
    suspend fun loadDetail(id: String, type: String)
    suspend fun toggleWatchlist(id: String, type: String)
}
