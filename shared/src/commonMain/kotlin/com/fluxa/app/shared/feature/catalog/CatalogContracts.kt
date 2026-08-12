package com.fluxa.app.shared.feature.catalog

import androidx.compose.runtime.Immutable
import com.fluxa.app.shared.feature.player.TrailerCue
import com.fluxa.app.ui.catalog.CatalogCardUiModel
import kotlinx.coroutines.flow.Flow

@Immutable
data class CatalogItemUiModel(
    val id: String,
    val type: String,
    val card: CatalogCardUiModel,
    val source: CatalogSourceUiModel = CatalogSourceUiModel(),
    val lazyKey: String = catalogLazyKey(id, type, source),
    val resume: CatalogResumeUiModel? = null,
    /** Raw source-owned artwork. Card artwork may be a landscape progress image. */
    val posterUrl: String? = null,
    val backdropUrl: String? = null,
    val description: String? = null,
    val releaseLabel: String? = null,
    val ratingLabel: String? = null,
    val ageRating: String? = null,
    val genres: List<String> = emptyList(),
    val seasonsCount: Int? = null,
    val runtimeLabel: String? = null
)

fun CatalogItemUiModel.stableLazyKey(): String = lazyKey

private fun catalogLazyKey(id: String, type: String, source: CatalogSourceUiModel): String = buildString {
    append(source.providerId ?: source.addonTransportUrl ?: "catalog")
    append(':')
    append(source.providerAccountId.orEmpty())
    append(':')
    append(source.catalogType ?: type)
    append(':')
    append(type)
    append(':')
    append(id)
}

@Immutable
data class CatalogSourceUiModel(
    val addonTransportUrl: String? = null,
    val catalogType: String? = null,
    val providerId: String? = null,
    val providerAccountId: String? = null,
    val strictProviderData: Boolean = false
)

@Immutable
data class CatalogResumeUiModel(
    val positionMs: Long,
    val durationMs: Long?,
    val videoId: String?,
    val streamUrl: String?,
    val streamTitle: String?,
    val progressPercent: Float? = null
)

@Immutable
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

@Immutable
data class CatalogBillboardUiModel(
    val item: CatalogItemUiModel,
    val logoUrl: String? = null,
    val trailerUrl: String? = null,
    val trailerSubtitleCues: List<TrailerCue> = emptyList()
)

@Immutable
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
    suspend fun heroPageChanged(item: CatalogItemUiModel) = Unit
}
