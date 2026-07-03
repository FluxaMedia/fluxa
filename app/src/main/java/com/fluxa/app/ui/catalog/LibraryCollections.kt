@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.remote.Meta

internal fun filterWatchlistLibraryItems(items: List<Meta>, selectedType: String): List<Meta> {
    return when (selectedType) {
        "movie" -> items.filter { it.type == "movie" }
        "series" -> items.filter { it.type == "series" }
        "anime" -> items.filter { meta ->
            meta.type == "anime" ||
                meta.genres.orEmpty().any { it.contains("anime", ignoreCase = true) } ||
                meta.originalLanguage == "ja"
        }
        else -> items
    }
}

internal fun buildLibraryCollections(
    lang: String,
    favorites: List<Meta>,
    traktCollection: List<Meta>,
    planned: List<Meta>,
    completed: List<Meta>,
    malWatching: List<Meta>,
    malPlanned: List<Meta>,
    malCompleted: List<Meta>,
    simklWatching: List<Meta>,
    simklPlanned: List<Meta>,
    simklCompleted: List<Meta>,
    userCollections: List<LibraryUserCollection> = emptyList()
): List<LibraryCollectionUi> {
    fun subtitle(size: Int) = AppStrings.format(lang, "format.items_count", size)
    return buildList {
        if (traktCollection.isNotEmpty()) add(LibraryCollectionUi("Trakt Collection", subtitle(traktCollection.size), traktCollection))
        userCollections.forEach { collection ->
            val folders = collection.folders.orEmpty()
            add(
                LibraryCollectionUi(
                    title = collection.title,
                    subtitle = if (folders.isNotEmpty()) subtitle(folders.size) else subtitle(collection.itemIds.orEmpty().size),
                    items = folders.map { folder ->
                        Meta(
                            id = folder.id,
                            name = folder.title,
                            type = "catalog_folder",
                            poster = folder.effectiveImageUrl(),
                            background = folder.heroBackdropUrl ?: folder.effectiveImageUrl(),
                            logo = folder.titleLogoUrl,
                            releaseInfo = folder.catalogTitle,
                            reason = collection.effectiveFolderShape(folder),
                            focusGifUrl = folder.focusGifUrl.takeIf { folder.focusGifEnabled != false },
                            coverEmoji = folder.coverEmoji,
                            hideTitle = folder.hideTitle,
                            focusGlowEnabled = collection.focusGlowEnabled
                        )
                    },
                    userCollectionId = collection.id
                )
            )
        }
    }
}

internal fun librarySectionTitle(lang: String, section: String): String = when (section) {
    "planned" -> AppStrings.t(lang, "auto.planned")
    "completed" -> AppStrings.t(lang, "auto.completed")
    "downloads" -> AppStrings.t(lang, "auto.downloads")
    "favorites" -> AppStrings.t(lang, "auto.favorites")
    else -> AppStrings.t(lang, "auto.continue_watching")
}
