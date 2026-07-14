package com.fluxa.app.shared.platform

import androidx.compose.ui.unit.dp
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.search.SearchDataSource
import com.fluxa.app.shared.feature.search.SearchUiState
import com.fluxa.app.ui.catalog.CatalogCardUiModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.serialization.json.Json
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.decodeFromString
import platform.Foundation.NSUserDefaults

data class AppleSearchSnapshot(
    val query: String,
    val results: List<AppleCatalogItemSnapshot> = emptyList(),
    val isLoading: Boolean = false
)

class AppleSearchDataSource : SearchDataSource {
    private val defaults = NSUserDefaults.standardUserDefaults
    private val state = MutableStateFlow(SearchUiState(recentItems = readHistory()))
    private var onSearchRequested: (String) -> Unit = {}

    override fun observeSearch(): Flow<SearchUiState> = state.asStateFlow()

    override suspend fun search(query: String) {
        state.value = state.value.copy(query = query, isLoading = query.isNotBlank())
        onSearchRequested(query)
    }

    override suspend fun recordSelection(item: CatalogItemUiModel) {
        val current = state.value
        val history = (listOf(item) + current.recentItems)
            .distinctBy { "${it.type}:${it.id}" }
            .take(MaxRecentItems)
        state.value = current.copy(
            recentItems = history
        )
        saveHistory(history)
    }

    override suspend fun clearHistory() {
        state.value = state.value.copy(recentItems = emptyList())
        defaults.removeObjectForKey(HistoryKey)
    }

    fun setOnSearchRequested(handler: (String) -> Unit) {
        onSearchRequested = handler
    }

    fun update(snapshot: AppleSearchSnapshot) {
        state.value = SearchUiState(
            query = snapshot.query,
            results = snapshot.results.map { it.toSearchItem() },
            recentItems = state.value.recentItems,
            isLoading = snapshot.isLoading
        )
    }

    private companion object {
        const val MaxRecentItems = 10
        const val HistoryKey = "fluxa.apple.searchHistory"
    }

    private fun readHistory(): List<CatalogItemUiModel> {
        val json = defaults.stringForKey(HistoryKey) ?: return emptyList()
        return runCatching {
            Json.decodeFromString<List<AppleSearchHistoryItem>>(json).map { it.toCatalogItem() }
        }.getOrDefault(emptyList())
    }

    private fun saveHistory(items: List<CatalogItemUiModel>) {
        val value = items.map(AppleSearchHistoryItem::from)
        defaults.setObject(Json.encodeToString(value), HistoryKey)
    }
}

private fun AppleCatalogItemSnapshot.toSearchItem(): CatalogItemUiModel {
    return appleSearchCatalogItem(
        id = id,
        type = type,
        title = title,
        subtitle = subtitle,
        artworkUrl = artworkUrl,
        logoUrl = logoUrl,
        addonTransportUrl = addonTransportUrl,
        catalogType = catalogType
    )
}

private fun appleSearchCatalogItem(
    id: String,
    type: String,
    title: String,
    subtitle: String,
    artworkUrl: String?,
    logoUrl: String?,
    addonTransportUrl: String?,
    catalogType: String?
): CatalogItemUiModel {
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
            artworkMemoryCacheKey = artworkUrl?.let { "apple-search:$it" },
            artworkDiskCacheKey = artworkUrl?.let { "apple-search:$it" },
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

@Serializable
private data class AppleSearchHistoryItem(
    val id: String,
    val type: String,
    val title: String,
    val subtitle: String,
    val artworkUrl: String?,
    val logoUrl: String?,
    val addonTransportUrl: String?,
    val catalogType: String?
) {
    fun toCatalogItem(): CatalogItemUiModel = appleSearchCatalogItem(
        id = id,
        type = type,
        title = title,
        subtitle = subtitle,
        artworkUrl = artworkUrl,
        logoUrl = logoUrl,
        addonTransportUrl = addonTransportUrl,
        catalogType = catalogType
    )

    companion object {
        fun from(item: CatalogItemUiModel): AppleSearchHistoryItem = AppleSearchHistoryItem(
            id = item.id,
            type = item.type,
            title = item.card.title,
            subtitle = item.card.subtitle,
            artworkUrl = item.card.artworkUrl,
            logoUrl = item.card.logoUrl,
            addonTransportUrl = item.source.addonTransportUrl,
            catalogType = item.source.catalogType
        )
    }
}
