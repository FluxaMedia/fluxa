package com.fluxa.app.shared.platform

import androidx.compose.ui.unit.dp
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.library.LibraryCollectionUiModel
import com.fluxa.app.shared.feature.library.LibraryDataSource
import com.fluxa.app.shared.feature.library.LibraryDownloadEpisodeUiModel
import com.fluxa.app.shared.feature.library.LibraryDownloadGroupUiModel
import com.fluxa.app.shared.feature.library.LibraryUiState
import com.fluxa.app.ui.catalog.CatalogCardUiModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.int
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import platform.Foundation.NSNotificationCenter

class AppleLibraryDataSource : LibraryDataSource {
    private val state = MutableStateFlow(LibraryUiState())

    override fun observeLibrary(): Flow<LibraryUiState> = state.asStateFlow()

    override suspend fun refresh() {
        state.value = state.value.copy(isLoading = true)
        NSNotificationCenter.defaultCenter.postNotificationName(
            aName = "FluxaAppleLibraryRequested",
            `object` = null,
            userInfo = null
        )
    }

    override suspend fun createCollection(title: String) {
        postLibraryAction("FluxaAppleLibraryCollectionCreateRequested") { put("title", title) }
    }

    override suspend fun renameCollection(id: String, title: String) {
        postLibraryAction("FluxaAppleLibraryCollectionRenameRequested") {
            put("id", id)
            put("title", title)
        }
    }

    override suspend fun deleteCollection(id: String) {
        postLibraryAction("FluxaAppleLibraryCollectionDeleteRequested") { put("id", id) }
    }

    override suspend fun cancelDownload(id: String) {
        postLibraryAction("FluxaAppleLibraryDownloadCancelRequested") { put("id", id) }
    }

    private fun postLibraryAction(name: String, extra: kotlinx.serialization.json.JsonObjectBuilder.() -> Unit) {
        NSNotificationCenter.defaultCenter.postNotificationName(
            aName = name,
            `object` = buildJsonObject(extra).toString(),
            userInfo = null
        )
    }

    fun updateJson(libraryJson: String) {
        state.value = runCatching {
            val root = Json.parseToJsonElement(libraryJson).jsonObject
            LibraryUiState(
                isLoading = root.boolean("isLoading"),
                planned = root["planned"]?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toLibraryItem() },
                completed = root["completed"]?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toLibraryItem() },
                favorites = root["favorites"]?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toLibraryItem() },
                collections = root["collections"]?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toCollection() },
                downloadGroups = root["downloadGroups"]?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toDownloadGroup() }
            )
        }.getOrElse { LibraryUiState() }
    }
}

private fun Map<String, JsonElement>.toCollection(): LibraryCollectionUiModel? {
    val title = string("title") ?: return null
    return LibraryCollectionUiModel(
        id = string("id"),
        title = title,
        subtitle = string("subtitle").orEmpty(),
        items = get("items")?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toLibraryItem() },
        locked = boolean("locked")
    )
}

private fun Map<String, JsonElement>.toDownloadGroup(): LibraryDownloadGroupUiModel? {
    val key = string("key") ?: return null
    return LibraryDownloadGroupUiModel(
        key = key,
        title = string("title").orEmpty(),
        posterUrl = string("posterUrl"),
        episodes = get("episodes")?.jsonArray.orEmpty().mapNotNull { it.jsonObject.toDownloadEpisode() },
        totalSizeLabel = string("totalSizeLabel").orEmpty()
    )
}

private fun Map<String, JsonElement>.toDownloadEpisode(): LibraryDownloadEpisodeUiModel? {
    val id = string("id") ?: return null
    return LibraryDownloadEpisodeUiModel(
        id = id,
        title = string("title").orEmpty(),
        statusLabel = string("statusLabel").orEmpty(),
        sizeLabel = string("sizeLabel").orEmpty(),
        progressPercent = get("progressPercent")?.jsonPrimitive?.int ?: 0,
        isDownloaded = boolean("isDownloaded"),
        isPlayable = boolean("isPlayable")
    )
}

private fun Map<String, JsonElement>.toLibraryItem(): CatalogItemUiModel? {
    val id = string("id") ?: return null
    val title = string("name").orEmpty()
    val artworkUrl = string("poster")
    return CatalogItemUiModel(
        id = id,
        type = string("type").orEmpty(),
        source = CatalogSourceUiModel(
            addonTransportUrl = string("addonTransportUrl"),
            catalogType = string("catalogType")
        ),
        card = CatalogCardUiModel(
            title = title,
            subtitle = string("releaseInfo").orEmpty(),
            showTitleBar = true,
            artworkUrl = artworkUrl,
            artworkMemoryCacheKey = artworkUrl?.let { "apple-library:$it" },
            artworkDiskCacheKey = artworkUrl?.let { "apple-library:$it" },
            requestWidthPx = 264,
            requestHeightPx = 396,
            logoUrl = string("logo"),
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
