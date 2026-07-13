package com.fluxa.app.shared.feature.catalog

import com.fluxa.app.ui.catalog.CatalogCardUiModel
import kotlinx.coroutines.flow.Flow

data class CatalogItemUiModel(
    val id: String,
    val type: String,
    val card: CatalogCardUiModel,
    val source: CatalogSourceUiModel = CatalogSourceUiModel(),
    val resume: CatalogResumeUiModel? = null,
    val backdropUrl: String? = null
)

data class CatalogSourceUiModel(
    val addonTransportUrl: String? = null,
    val catalogType: String? = null
)

data class CatalogResumeUiModel(
    val positionMs: Long,
    val durationMs: Long?,
    val videoId: String?,
    val streamUrl: String?,
    val streamTitle: String?
)

data class CatalogRowUiModel(
    val id: String,
    val title: String,
    val items: List<CatalogItemUiModel>,
    val canLoadMore: Boolean = false
)

data class CatalogHomeUiState(
    val rows: List<CatalogRowUiModel> = emptyList(),
    val isLoading: Boolean = false
)

sealed interface CatalogAction {
    data object Refresh : CatalogAction
    data class LoadMore(val rowId: String) : CatalogAction
    data class ItemSelected(val item: CatalogItemUiModel) : CatalogAction
    data class PlayRequested(val item: CatalogItemUiModel) : CatalogAction
    data class ResumeRequested(val item: CatalogItemUiModel) : CatalogAction
}

interface CatalogHomeDataSource {
    fun observeHome(): Flow<CatalogHomeUiState>
    suspend fun refresh()
    suspend fun loadMore(rowId: String)
}
