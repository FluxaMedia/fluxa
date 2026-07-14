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
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import androidx.compose.ui.unit.dp
import platform.Foundation.NSNotificationCenter

class AppleCatalogHomeDataSource : CatalogHomeDataSource {
    private val state = MutableStateFlow(CatalogHomeUiState())

    override fun observeHome(): Flow<CatalogHomeUiState> = state.asStateFlow()

    override suspend fun refresh() {
        NSNotificationCenter.defaultCenter.postNotificationName(
            aName = "FluxaAppleCatalogRefreshRequested",
            `object` = null,
            userInfo = null
        )
    }

    override suspend fun loadMore(rowId: String) = Unit

    override suspend fun setFilter(filter: String) {
        state.value = state.value.copy(activeFilter = filter)
        NSNotificationCenter.defaultCenter.postNotificationName(
            aName = "FluxaAppleCatalogFilterChanged",
            `object` = filter,
            userInfo = null
        )
    }

    fun update(home: CatalogHomeUiState) {
        state.value = home
    }

    fun updateJson(homeJson: String) {
        val home = runCatching {
            val root = Json.parseToJsonElement(homeJson).jsonObject
            CatalogHomeUiState(
                rows = root["rows"]?.jsonArray.orEmpty().mapNotNull { rowElement ->
                    val row = rowElement.jsonObject
                    val id = row.string("id") ?: return@mapNotNull null
                    CatalogRowUiModel(
                        id = id,
                        title = row.string("title").orEmpty(),
                        canLoadMore = row.boolean("canLoadMore"),
                        items = row["items"]?.jsonArray.orEmpty().mapNotNull { itemElement ->
                            itemElement.jsonObject.toCatalogItemUiModel()
                        }
                    )
                },
                isLoading = root.boolean("isLoading")
            )
        }.getOrElse { CatalogHomeUiState() }
        update(home)
    }
}

private fun Map<String, kotlinx.serialization.json.JsonElement>.toCatalogItemUiModel(): CatalogItemUiModel? {
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
            artworkMemoryCacheKey = artworkUrl?.let { "apple-catalog:$it" },
            artworkDiskCacheKey = artworkUrl?.let { "apple-catalog:$it" },
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
            progress = number("progress")?.toFloat()?.coerceIn(0f, 1f) ?: 0f,
            showProgressBar = number("progress") != null,
            showUpNextBadge = false,
            upNextLabel = "",
            topTenRank = number("topTenRank")?.toInt(),
            rankNumberBoxWidth = 0.dp,
            rankOffsetX = 0.dp,
            rankOffsetY = 0.dp,
            rankFontSizeRatio = 1f,
            loadArtwork = true
        )
    )
}

private fun Map<String, kotlinx.serialization.json.JsonElement>.string(key: String): String? =
    get(key)?.jsonPrimitive?.contentOrNull

private fun Map<String, kotlinx.serialization.json.JsonElement>.number(key: String): Double? =
    string(key)?.toDoubleOrNull()

private fun Map<String, kotlinx.serialization.json.JsonElement>.boolean(key: String): Boolean =
    string(key)?.toBooleanStrictOrNull() ?: false
