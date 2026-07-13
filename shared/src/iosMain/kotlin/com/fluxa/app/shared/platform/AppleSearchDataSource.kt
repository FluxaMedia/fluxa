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
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import platform.Foundation.NSNotificationCenter

class AppleSearchDataSource : SearchDataSource {
    private val state = MutableStateFlow(SearchUiState())

    override fun observeSearch(): Flow<SearchUiState> = state.asStateFlow()

    override suspend fun search(query: String) {
        state.value = state.value.copy(query = query, isLoading = query.isNotBlank())
        NSNotificationCenter.defaultCenter.postNotificationName(
            aName = "FluxaAppleSearchRequested",
            `object` = query,
            userInfo = null
        )
    }

    override suspend fun clearHistory() = Unit

    fun updateJson(searchJson: String) {
        state.value = runCatching {
            val root = Json.parseToJsonElement(searchJson).jsonObject
            SearchUiState(
                query = root.string("query").orEmpty(),
                results = root["results"]?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toSearchItem() },
                isLoading = root.boolean("isLoading")
            )
        }.getOrElse { SearchUiState() }
    }
}

private fun Map<String, JsonElement>.toSearchItem(): CatalogItemUiModel? {
    val id = string("id") ?: return null
    val title = string("title").orEmpty()
    val artworkUrl = string("artworkUrl")
    return CatalogItemUiModel(
        id = id,
        type = string("type").orEmpty(),
        source = CatalogSourceUiModel(
            addonTransportUrl = string("addonTransportUrl"),
            catalogType = string("catalogType")
        ),
        card = CatalogCardUiModel(
            title = title,
            subtitle = string("subtitle").orEmpty(),
            showTitleBar = true,
            artworkUrl = artworkUrl,
            artworkMemoryCacheKey = artworkUrl?.let { "apple-search:$it" },
            artworkDiskCacheKey = artworkUrl?.let { "apple-search:$it" },
            requestWidthPx = 264,
            requestHeightPx = 396,
            logoUrl = string("logoUrl"),
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

private fun Map<String, JsonElement>.string(key: String): String? =
    get(key)?.jsonPrimitive?.contentOrNull

private fun Map<String, JsonElement>.boolean(key: String): Boolean =
    string(key)?.toBooleanStrictOrNull() ?: false
