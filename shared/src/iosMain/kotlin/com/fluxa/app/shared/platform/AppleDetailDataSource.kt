package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.detail.DetailUiModel
import com.fluxa.app.shared.feature.detail.DetailUiState
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

class AppleDetailDataSource : DetailDataSource {
    private val state = MutableStateFlow(DetailUiState())

    override fun observeDetail(id: String, type: String): Flow<DetailUiState> = state.asStateFlow()

    override suspend fun loadDetail(id: String, type: String) = Unit

    override suspend fun toggleWatchlist(id: String, type: String) = Unit

    fun updateJson(detailJson: String) {
        state.value = runCatching {
            val root = Json.parseToJsonElement(detailJson).jsonObject
            DetailUiState(
                content = root.toDetailUiModel(),
                isLoading = root.boolean("isLoading"),
                errorKey = root.string("errorKey")
            )
        }.getOrElse { DetailUiState() }
    }
}

private fun Map<String, JsonElement>.toDetailUiModel(): DetailUiModel? {
    val id = string("id") ?: return null
    val type = string("type") ?: return null
    val title = string("title") ?: return null
    return DetailUiModel(
        id = id,
        type = type,
        title = title,
        description = string("description").orEmpty(),
        posterUrl = string("posterUrl"),
        backgroundUrl = string("backgroundUrl"),
        logoUrl = string("logoUrl"),
        releaseLabel = string("releaseLabel").orEmpty(),
        ratingLabel = string("ratingLabel").orEmpty(),
        isInWatchlist = boolean("isInWatchlist"),
        relatedItems = get("relatedItems")?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toCatalogItem() }
    )
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
            artworkMemoryCacheKey = artworkUrl?.let { "apple-detail:$it" },
            artworkDiskCacheKey = artworkUrl?.let { "apple-detail:$it" },
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
