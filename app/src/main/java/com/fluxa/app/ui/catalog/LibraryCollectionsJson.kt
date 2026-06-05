package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.LibraryCatalogSource
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.LibraryUserCollectionFolder
import com.google.gson.GsonBuilder
import com.google.gson.JsonParser
import com.google.gson.annotations.SerializedName
import com.google.gson.reflect.TypeToken

private val collectionsGson = GsonBuilder()
    .setPrettyPrinting()
    .disableHtmlEscaping()
    .create()

private data class CollectionJson(
    val id: String? = null,
    val title: String? = null,
    val showAllTab: Boolean? = null,
    val viewMode: String? = null,
    val showOnHome: Boolean? = null,
    val pinToTop: Boolean? = null,
    val focusGlowEnabled: Boolean? = null,
    val folders: List<CollectionFolderJson>? = null
)

private data class CollectionFolderJson(
    val id: String? = null,
    val title: String? = null,
    val tileShape: String? = null,
    val shape: String? = null,
    val hideTitle: Boolean? = null,
    val focusGifEnabled: Boolean? = null,
    val catalogSources: List<LibraryCatalogSource>? = null,
    val coverEmoji: String? = null,
    @SerializedName(value = "coverImageUrl", alternate = ["coverUrl", "coverImage", "cover", "poster", "thumbnail", "thumb"])
    val coverImageUrl: String? = null,
    @SerializedName(value = "imageUrl", alternate = ["image", "image_url", "posterUrl", "poster_url"])
    val imageUrl: String? = null,
    val focusGifUrl: String? = null,
    val titleLogoUrl: String? = null,
    @SerializedName(value = "heroBackdropUrl", alternate = ["background", "backdrop", "backgroundUrl", "backdropUrl"])
    val heroBackdropUrl: String? = null,
    val catalogId: String? = null,
    val catalogTitle: String? = null,
    val genre: String? = null
)

internal fun importLibraryCollectionsJson(rawJson: String): List<LibraryUserCollection> {
    val element = JsonParser.parseString(rawJson)
    val collectionType = object : TypeToken<List<CollectionJson>>() {}.type
    val imported: List<CollectionJson> = if (element.isJsonArray) {
        collectionsGson.fromJson(element, collectionType)
    } else {
        listOf(collectionsGson.fromJson(element, CollectionJson::class.java))
    }
    return imported.mapIndexedNotNull { index, collection ->
        val title = collection.title?.trim().orEmpty()
        if (title.isBlank()) return@mapIndexedNotNull null
        LibraryUserCollection(
            id = collection.id?.takeIf { it.isNotBlank() } ?: "imported_${System.currentTimeMillis()}_$index",
            title = title,
            imageUrl = collection.folders.orEmpty().firstNotNullOfOrNull { it.normalizedCoverImageUrl() },
            showOnHome = collection.showOnHome ?: false,
            folders = collection.folders.orEmpty().mapIndexedNotNull { folderIndex, folder ->
                val folderTitle = folder.title?.trim().orEmpty()
                if (folderTitle.isBlank()) return@mapIndexedNotNull null
                val sources = folder.catalogSources.orEmpty().ifEmpty {
                    val catalogId = folder.catalogId?.takeIf { it.isNotBlank() } ?: return@ifEmpty emptyList()
                    listOf(LibraryCatalogSource(catalogId = catalogId, type = "movie"))
                }
                val primarySource = sources.firstOrNull()
                val coverImageUrl = folder.normalizedCoverImageUrl()
                val focusGifUrl = folder.focusGifUrl.cleanedUrl()
                val heroBackdropUrl = folder.heroBackdropUrl.cleanedUrl()
                LibraryUserCollectionFolder(
                    id = folder.id?.takeIf { it.isNotBlank() } ?: "folder_${System.currentTimeMillis()}_$folderIndex",
                    title = folderTitle,
                    imageUrl = coverImageUrl,
                    shape = normalizeImportedTileShape(folder.tileShape ?: folder.shape),
                    catalogId = primarySource?.catalogId ?: folder.catalogId,
                    catalogTitle = folder.catalogTitle ?: folderTitle,
                    genre = folder.genre,
                    hideTitle = folder.hideTitle ?: false,
                    focusGifEnabled = folder.focusGifEnabled ?: true,
                    catalogSources = sources.takeIf { it.isNotEmpty() },
                    coverEmoji = folder.coverEmoji,
                    coverImageUrl = coverImageUrl,
                    focusGifUrl = focusGifUrl,
                    titleLogoUrl = folder.titleLogoUrl,
                    heroBackdropUrl = heroBackdropUrl
                )
            },
            showAllTab = collection.showAllTab ?: true,
            viewMode = collection.viewMode ?: "FOLLOW_LAYOUT",
            pinToTop = collection.pinToTop ?: false,
            focusGlowEnabled = collection.focusGlowEnabled ?: true
        )
    }
}

internal fun exportLibraryCollectionsJson(collections: List<LibraryUserCollection>): String {
    val exportRows = collections.map { collection ->
        CollectionJson(
            id = collection.id,
            title = collection.title,
            showAllTab = collection.showAllTab ?: true,
            viewMode = collection.viewMode ?: "FOLLOW_LAYOUT",
            showOnHome = collection.showOnHome ?: false,
            pinToTop = collection.pinToTop ?: collection.showOnHome ?: false,
            focusGlowEnabled = collection.focusGlowEnabled ?: true,
            folders = collection.folders.orEmpty().map { folder ->
                CollectionFolderJson(
                    id = folder.id,
                    title = folder.title,
                    tileShape = exportTileShape(folder.shape),
                    hideTitle = folder.hideTitle ?: false,
                    focusGifEnabled = folder.focusGifEnabled ?: true,
                    catalogSources = folder.catalogSources.orEmpty().ifEmpty {
                        folder.catalogId?.let { listOf(LibraryCatalogSource(catalogId = it, type = "movie")) }.orEmpty()
                    },
                    coverEmoji = folder.coverEmoji,
                    coverImageUrl = folder.coverImageUrl ?: folder.imageUrl,
                    focusGifUrl = folder.focusGifUrl,
                    titleLogoUrl = folder.titleLogoUrl,
                    heroBackdropUrl = folder.heroBackdropUrl
                )
            }
        )
    }
    return collectionsGson.toJson(exportRows)
}

internal fun LibraryUserCollectionFolder.effectiveCatalogId(): String? {
    return catalogSources.orEmpty().firstOrNull()?.catalogId ?: catalogId
}

internal fun LibraryUserCollectionFolder.effectiveCatalogType(): String? {
    return catalogSources.orEmpty().firstOrNull()?.type
}

internal fun LibraryUserCollectionFolder.effectiveImageUrl(): String? {
    return coverImageUrl.cleanedArtworkUrl() ?: imageUrl.cleanedArtworkUrl()
}

internal fun LibraryUserCollectionFolder.effectiveShape(): String {
    return shape ?: normalizeImportedTileShape(null)
}

internal fun LibraryUserCollection.effectiveFolderShape(folder: LibraryUserCollectionFolder): String? {
    return folder.effectiveShape()
}

private fun normalizeImportedTileShape(value: String?): String {
    return when (value?.trim()?.uppercase()) {
        "LANDSCAPE", "WIDE" -> "wide"
        "SQUARE" -> "square"
        else -> "poster"
    }
}

private fun exportTileShape(value: String?): String {
    return when (value?.trim()?.lowercase()) {
        "wide", "landscape" -> "LANDSCAPE"
        "square" -> "SQUARE"
        else -> "POSTER"
    }
}

private fun CollectionFolderJson.normalizedCoverImageUrl(): String? {
    return coverImageUrl.cleanedArtworkUrl() ?: imageUrl.cleanedArtworkUrl()
}

private fun String?.cleanedUrl(): String? {
    return this?.trim()?.takeIf { it.isNotBlank() }
}

private fun String?.cleanedArtworkUrl(): String? {
    val raw = cleanedUrl()
        ?.trim('"', '\'')
        ?.takeIf { it.isNotBlank() }
        ?: return null
    val withScheme = when {
        raw.startsWith("//") -> "https:$raw"
        else -> raw
    }
    val githubBlob = Regex("""^https://github\.com/([^/]+)/([^/]+)/blob/([^/]+)/(.+)$""")
    val normalized = githubBlob.matchEntire(withScheme)?.let { match ->
        val (owner, repo, branch, path) = match.destructured
        "https://raw.githubusercontent.com/$owner/$repo/$branch/$path"
    } ?: withScheme
    return normalized.replace(" ", "%20")
}
