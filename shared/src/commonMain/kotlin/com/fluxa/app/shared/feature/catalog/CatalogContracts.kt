package com.fluxa.app.shared.feature.catalog

import com.fluxa.app.player.TrailerCue
import com.fluxa.app.ui.catalog.CatalogCardUiModel
import kotlinx.coroutines.flow.Flow

data class CatalogItemUiModel(
    val id: String,
    val type: String,
    val card: CatalogCardUiModel,
    val source: CatalogSourceUiModel = CatalogSourceUiModel(),
    val resume: CatalogResumeUiModel? = null,
    val backdropUrl: String? = null,
    val description: String? = null,
    val ageRating: String? = null,
    val seasonsCount: Int? = null,
    val runtimeLabel: String? = null
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
    val canLoadMore: Boolean = false,
    val categoryType: String = "movie",
    val cardLayout: String = "poster",
    val artworkPreference: String? = null,
    val isActionRow: Boolean = false,
    val topTenEnabled: Boolean = false
)

data class CatalogBillboardUiModel(
    val item: CatalogItemUiModel,
    val logoUrl: String? = null,
    val trailerUrl: String? = null,
    val trailerSubtitleCues: List<TrailerCue> = emptyList()
)

data class CatalogHomeUiState(
    val rows: List<CatalogRowUiModel> = emptyList(),
    val isLoading: Boolean = false,
    val billboard: CatalogBillboardUiModel? = null,
    val heroItems: List<CatalogItemUiModel> = emptyList(),
    val showHeroSection: Boolean = true,
    val activeFilter: String = "all"
)

sealed interface CatalogAction {
    data object Refresh : CatalogAction
    data class LoadMore(val rowId: String) : CatalogAction
    data class ItemSelected(val item: CatalogItemUiModel) : CatalogAction
    data class PlayRequested(val item: CatalogItemUiModel) : CatalogAction
    data class ResumeRequested(val item: CatalogItemUiModel) : CatalogAction
    data class ItemFocused(val item: CatalogItemUiModel, val rowId: String) : CatalogAction
    data class HeroPageChanged(val item: CatalogItemUiModel) : CatalogAction
    data class FilterChanged(val filter: String) : CatalogAction
    data class MarkWatchedRequested(val item: CatalogItemUiModel) : CatalogAction
    data class DropRequested(val item: CatalogItemUiModel) : CatalogAction
    data class AddToLibraryRequested(val item: CatalogItemUiModel) : CatalogAction
}

interface CatalogHomeDataSource {
    fun observeHome(): Flow<CatalogHomeUiState>
    fun initialHomeState(): CatalogHomeUiState? = null
    suspend fun refresh()
    suspend fun loadMore(rowId: String)
    suspend fun setFilter(filter: String)
}
