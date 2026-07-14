package com.fluxa.app.shared.platform

import com.fluxa.app.common.AppStrings
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
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import platform.Foundation.NSNotificationCenter

class AppleDiscoverDataSource : DiscoverDataSource {
    private val state = MutableStateFlow(DiscoverUiState())

    override fun observeDiscover(): Flow<DiscoverUiState> = state.asStateFlow()

    override suspend fun updateFilters(filters: DiscoverFiltersUiModel) {
        state.value = state.value.copy(filters = filters, isLoading = true)
        NSNotificationCenter.defaultCenter.postNotificationName(
            aName = "FluxaAppleDiscoverRequested",
            `object` = buildJsonObject {
                put("contentType", filters.contentType)
                filters.catalogKey?.let { put("catalogKey", it) }
                filters.genre?.let { put("genre", it) }
            }.toString(),
            userInfo = null
        )
    }

    override suspend fun loadMore() {
        NSNotificationCenter.defaultCenter.postNotificationName(
            aName = "FluxaAppleDiscoverLoadMoreRequested",
            `object` = buildJsonObject {
                put("contentType", state.value.filters.contentType)
                state.value.filters.catalogKey?.let { put("catalogKey", it) }
                state.value.filters.genre?.let { put("genre", it) }
            }.toString(),
            userInfo = null
        )
    }

    fun updateJson(discoverJson: String) {
        state.value = runCatching {
            val root = Json.parseToJsonElement(discoverJson).jsonObject
            DiscoverUiState(
                filters = DiscoverFiltersUiModel(
                    contentType = root.string("contentType") ?: "movie",
                    catalogKey = root.string("catalogKey"),
                    genre = root.string("genre")
                ),
                catalogOptions = root["catalogOptions"]?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toFilterOption() },
                genreOptions = root["genreOptions"]?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toFilterOption() },
                results = root["results"]?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toCatalogItem() },
                isLoading = root.boolean("isLoading")
            )
        }.getOrElse { DiscoverUiState() }
    }
}

private fun Map<String, JsonElement>.toFilterOption(): DiscoverFilterOptionUiModel? {
    val label = string("label")?.ifBlank { AppStrings.t(null, "auto.all") } ?: return null
    return DiscoverFilterOptionUiModel(id = string("id"), label = label)
}

private fun Map<String, JsonElement>.toCatalogItem(): CatalogItemUiModel? {
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
            artworkMemoryCacheKey = artworkUrl?.let { "apple-discover:$it" },
            artworkDiskCacheKey = artworkUrl?.let { "apple-discover:$it" },
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
