package com.fluxa.app.data.repository

import com.fluxa.app.data.local.LibraryCatalogSource
import com.fluxa.app.data.local.LibraryRemoteSource
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.LibraryUserCollectionFolder
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.NuvioAddon
import com.fluxa.app.data.remote.NuvioCollectionRow
import com.fluxa.app.data.remote.NuvioLibraryItem
import com.fluxa.app.data.remote.NuvioWatchProgress
import com.fluxa.app.data.remote.NuvioWatchedItem

data class NuvioAddonState(val installedUrls: List<String>, val disabledUrls: List<String>)

object NuvioImportPolicy {
    fun addonState(addons: List<NuvioAddon>): NuvioAddonState {
        val ordered = addons.sortedBy(NuvioAddon::sortOrder)
        return NuvioAddonState(
            installedUrls = ordered.map(NuvioAddon::url).distinct(),
            disabledUrls = ordered.filterNot(NuvioAddon::enabled).map(NuvioAddon::url).distinct()
        )
    }

    fun libraryMetas(items: List<NuvioLibraryItem>): Map<String, Meta> = items.associate { item ->
        item.contentId to Meta(
            id = item.contentId,
            name = item.name,
            type = item.contentType,
            poster = item.poster,
            background = item.background,
            description = item.description,
            releaseInfo = item.releaseInfo,
            imdbRating = item.imdbRating?.toString(),
            genres = item.genres
        )
    }

    fun progressVideoId(entry: NuvioWatchProgress): String =
        if (entry.season != null && entry.episode != null) {
            "${entry.contentId}:${entry.season}:${entry.episode}"
        } else {
            entry.videoId.ifBlank { entry.contentId }
        }

    fun latestProgressByContent(entries: List<NuvioWatchProgress>): List<NuvioWatchProgress> = entries
        .groupBy(NuvioWatchProgress::contentId)
        .values
        .mapNotNull { values -> values.maxWithOrNull(compareBy(NuvioWatchProgress::lastWatched, NuvioWatchProgress::position)) }
        .sortedByDescending(NuvioWatchProgress::lastWatched)

    fun watchedVideoIds(
        items: List<NuvioWatchedItem>,
        progressVideoIds: Set<String>
    ): Map<String, Set<String>> {
        val watched = mutableMapOf<String, MutableSet<String>>()
        items.forEach { item ->
            val videoId = if (item.season != null && item.episode != null) {
                "${item.contentId}:${item.season}:${item.episode}"
            } else {
                item.contentId
            }
            if (videoId !in progressVideoIds) watched.getOrPut(item.contentId, ::mutableSetOf).add(videoId)
        }
        return watched
    }

    fun collections(rows: List<NuvioCollectionRow>, nextId: () -> String): List<LibraryUserCollection> =
        rows.flatMap { it.collectionsJson.orEmpty() }.map { collection ->
            LibraryUserCollection(
                id = collection.id ?: nextId(),
                title = collection.title.orEmpty(),
                imageUrl = collection.backdropImageUrl,
                showOnHome = collection.showOnHome ?: true,
                pinToTop = collection.pinToTop,
                viewMode = collection.viewMode,
                showAllTab = collection.showAllTab,
                focusGlowEnabled = collection.focusGlowEnabled ?: true,
                community = collection.community,
                folders = collection.folders?.map { folder ->
                    LibraryUserCollectionFolder(
                        id = folder.id ?: nextId(),
                        title = folder.title.orEmpty(),
                        coverImageUrl = folder.coverImageUrl,
                        coverEmoji = folder.coverEmoji,
                        focusGifUrl = folder.focusGifUrl,
                        focusGifEnabled = folder.focusGifEnabled ?: true,
                        titleLogoUrl = folder.titleLogoUrl,
                        heroBackdropUrl = folder.heroBackdropUrl,
                        heroVideoUrl = folder.heroVideoUrl,
                        shape = folderShape(folder.tileShape),
                        hideTitle = folder.hideTitle,
                        catalogSources = folder.catalogSources?.map { source ->
                            LibraryCatalogSource(source.addonId, source.catalogId.orEmpty(), source.type ?: "movie", source.genre)
                        }?.filter { it.catalogId.isNotBlank() },
                        sources = folder.catalogSources?.map { source ->
                            LibraryRemoteSource(
                                provider = source.provider ?: "addon",
                                title = source.title,
                                mediaType = source.mediaType,
                                traktListId = source.traktListId,
                                tmdbSourceType = source.tmdbSourceType,
                                tmdbId = source.tmdbId,
                                sortBy = source.sortBy,
                                sortHow = source.sortHow,
                                filters = source.filters,
                                addonId = source.addonId,
                                catalogId = source.catalogId,
                                type = source.type,
                                genre = source.genre
                            )
                        }
                    )
                }
            )
        }

    private fun folderShape(value: String?): String = when (value?.trim()?.lowercase()) {
        "landscape", "wide" -> "wide"
        "square" -> "square"
        else -> "poster"
    }
}
