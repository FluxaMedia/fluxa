package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.discover.DiscoverDataSource
import com.fluxa.app.shared.feature.discover.DiscoverFilterOptionUiModel
import com.fluxa.app.shared.feature.discover.DiscoverFiltersUiModel
import com.fluxa.app.shared.feature.discover.DiscoverUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.ui.catalog.CatalogCardUiModel
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

data class AppleDiscoverRequestSnapshot(
    val contentType: String,
    val catalogKey: String? = null,
    val genre: String? = null
)

data class AppleDiscoverFilterOptionSnapshot(
    val id: String? = null,
    val label: String
)

data class AppleDiscoverSnapshot(
    val request: AppleDiscoverRequestSnapshot,
    val catalogOptions: List<AppleDiscoverFilterOptionSnapshot> = emptyList(),
    val genreOptions: List<AppleDiscoverFilterOptionSnapshot> = emptyList(),
    val results: List<AppleCatalogItemSnapshot> = emptyList(),
    val isLoading: Boolean = false
)

class AppleDiscoverDataSource : DiscoverDataSource {
    private val state = MutableStateFlow(DiscoverUiState())
    private var onDiscoverRequested: (AppleDiscoverRequestSnapshot) -> Unit = {}

    override fun observeDiscover(): Flow<DiscoverUiState> = state.asStateFlow()

    override suspend fun updateFilters(filters: DiscoverFiltersUiModel) {
        state.value = state.value.copy(filters = filters, isLoading = true)
        onDiscoverRequested(filters.toAppleRequest())
    }

    override suspend fun loadMore() {
        onDiscoverRequested(state.value.filters.toAppleRequest())
    }

    fun setOnDiscoverRequested(handler: (AppleDiscoverRequestSnapshot) -> Unit) {
        onDiscoverRequested = handler
    }

    fun update(snapshot: AppleDiscoverSnapshot) {
        state.value = DiscoverUiState(
            filters = DiscoverFiltersUiModel(snapshot.request.contentType, snapshot.request.catalogKey, snapshot.request.genre),
            catalogOptions = snapshot.catalogOptions.map { DiscoverFilterOptionUiModel(it.id, it.label) },
            genreOptions = snapshot.genreOptions.map { DiscoverFilterOptionUiModel(it.id, it.label) },
            results = snapshot.results.map { it.toAppleDiscoverCatalogItem() },
            isLoading = snapshot.isLoading
        )
    }
}

private fun DiscoverFiltersUiModel.toAppleRequest() = AppleDiscoverRequestSnapshot(contentType, catalogKey, genre)

private fun AppleCatalogItemSnapshot.toAppleDiscoverCatalogItem(): CatalogItemUiModel {
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
            artworkMemoryCacheKey = artworkUrl?.let { "apple-discover:$it" },
            artworkDiskCacheKey = artworkUrl?.let { "apple-discover:$it" },
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
            progress = 0f,
            showProgressBar = false,
            showUpNextBadge = false,
            upNextLabel = "",
            topTenRank = null,
            rankNumberBoxWidth = 0.dp,
            rankOffsetX = 0.dp,
            rankOffsetY = 0.dp,
            rankFontSizeRatio = 1f,
            loadArtwork = true
        )
    )
}
