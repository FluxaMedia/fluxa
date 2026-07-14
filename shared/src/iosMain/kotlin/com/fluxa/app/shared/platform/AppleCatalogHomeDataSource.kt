package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.ui.catalog.CatalogCardUiModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import androidx.compose.ui.unit.dp

data class AppleCatalogHomeSnapshot(
    val rows: List<AppleCatalogRowSnapshot> = emptyList(),
    val isLoading: Boolean = false
)

data class AppleCatalogRowSnapshot(
    val id: String,
    val title: String,
    val items: List<AppleCatalogItemSnapshot>,
    val canLoadMore: Boolean = false
)

data class AppleCatalogItemSnapshot(
    val id: String,
    val type: String,
    val title: String,
    val subtitle: String = "",
    val artworkUrl: String? = null,
    val logoUrl: String? = null,
    val addonTransportUrl: String? = null,
    val catalogType: String? = null,
    val progress: Float? = null,
    val topTenRank: Int? = null
)

class AppleCatalogHomeDataSource : CatalogHomeDataSource {
    private val state = MutableStateFlow(CatalogHomeUiState())
    private var onRefreshRequested: () -> Unit = {}

    override fun observeHome(): Flow<CatalogHomeUiState> = state.asStateFlow()

    override suspend fun refresh() {
        onRefreshRequested()
    }

    override suspend fun loadMore(rowId: String) = Unit

    override suspend fun setFilter(filter: String) {
        state.value = state.value.copy(activeFilter = filter)
    }

    fun setOnRefreshRequested(handler: () -> Unit) {
        onRefreshRequested = handler
    }

    fun update(snapshot: AppleCatalogHomeSnapshot) {
        state.value = CatalogHomeUiState(
            rows = snapshot.rows.map { row ->
                CatalogRowUiModel(
                    id = row.id,
                    title = row.title,
                    canLoadMore = row.canLoadMore,
                    items = row.items.map { item -> item.toCatalogItemUiModel() }
                )
            },
            isLoading = snapshot.isLoading
        )
    }
}

private fun AppleCatalogItemSnapshot.toCatalogItemUiModel(): CatalogItemUiModel {
    return CatalogItemUiModel(
        id = id,
        type = type,
        source = CatalogSourceUiModel(
            addonTransportUrl = addonTransportUrl,
            catalogType = catalogType
        ),
        card = CatalogCardUiModel(
            title = title,
            subtitle = subtitle,
            showTitleBar = true,
            artworkUrl = artworkUrl,
            artworkMemoryCacheKey = artworkUrl?.let { "apple-catalog:$it" },
            artworkDiskCacheKey = artworkUrl?.let { "apple-catalog:$it" },
            requestWidthPx = 264,
            requestHeightPx = 396,
            logoUrl = logoUrl,
            logoMemoryCacheKey = null,
            showLogo = false,
            allowCoverFallback = true,
            coverFallbackText = title,
            coverFallbackIsEmoji = false,
            width = 132.dp,
            imageHeight = 198.dp,
            outerWidth = 132.dp,
            cardBackgroundIsSurfaceCard = true,
            progress = progress?.coerceIn(0f, 1f) ?: 0f,
            showProgressBar = progress != null,
            showUpNextBadge = false,
            upNextLabel = "",
            topTenRank = topTenRank,
            rankNumberBoxWidth = 0.dp,
            rankOffsetX = 0.dp,
            rankOffsetY = 0.dp,
            rankFontSizeRatio = 1f,
            loadArtwork = true
        )
    )
}
