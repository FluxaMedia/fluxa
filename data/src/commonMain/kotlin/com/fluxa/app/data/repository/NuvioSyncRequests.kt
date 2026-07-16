package com.fluxa.app.data.repository

import com.fluxa.app.data.local.LibraryRemoteSource
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.NuvioLibraryItem
import com.fluxa.app.data.remote.Video

object NuvioSyncRequests {
    fun collection(collection: LibraryUserCollection): Map<String, Any?> = mapOf(
        "id" to collection.id,
        "title" to collection.title,
        "backdropImageUrl" to collection.imageUrl,
        "showOnHome" to collection.showOnHome,
        "pinToTop" to collection.pinToTop,
        "viewMode" to collection.viewMode,
        "showAllTab" to collection.showAllTab,
        "focusGlowEnabled" to collection.focusGlowEnabled,
        "community" to collection.community,
        "folders" to collection.folders.orEmpty().map { folder ->
            val sources = folder.sources.orEmpty().ifEmpty {
                folder.catalogSources.orEmpty().map { source ->
                    LibraryRemoteSource("addon", addonId = source.addonId, catalogId = source.catalogId, type = source.type, genre = source.genre)
                }
            }
            mapOf(
                "id" to folder.id,
                "title" to folder.title,
                "coverImageUrl" to (folder.coverImageUrl ?: folder.imageUrl),
                "coverEmoji" to folder.coverEmoji,
                "focusGifUrl" to folder.focusGifUrl,
                "focusGifEnabled" to folder.focusGifEnabled,
                "titleLogoUrl" to folder.titleLogoUrl,
                "heroBackdropUrl" to folder.heroBackdropUrl,
                "heroVideoUrl" to folder.heroVideoUrl,
                "tileShape" to folder.shape,
                "hideTitle" to folder.hideTitle,
                "sources" to sources.map(::remoteSource)
            )
        }
    )

    fun libraryItem(meta: Meta, addedAt: Long): Map<String, Any?> = mapOf(
        "content_id" to meta.id,
        "content_type" to meta.type,
        "name" to meta.name,
        "poster" to meta.poster,
        "background" to meta.background,
        "description" to meta.description,
        "release_info" to meta.releaseInfo,
        "imdb_rating" to meta.imdbRating?.toDoubleOrNull(),
        "genres" to meta.genres,
        "poster_shape" to "POSTER",
        "added_at" to addedAt
    )

    fun libraryItem(item: NuvioLibraryItem): Map<String, Any?> = mapOf(
        "content_id" to item.contentId,
        "content_type" to item.contentType,
        "name" to item.name,
        "poster" to item.poster,
        "background" to item.background,
        "description" to item.description,
        "release_info" to item.releaseInfo,
        "imdb_rating" to item.imdbRating,
        "genres" to item.genres,
        "poster_shape" to (item.posterShape ?: "POSTER"),
        "addon_base_url" to item.addonBaseUrl,
        "added_at" to item.addedAt
    )

    fun watchedItems(meta: Meta, episodes: List<Video>, watchedAt: Long): List<Map<String, Any?>> {
        if (meta.type == "movie") {
            return listOf(mapOf("content_id" to meta.id, "content_type" to "movie", "title" to meta.name, "watched_at" to watchedAt))
        }
        return episodes.mapNotNull { episode ->
            val season = episode.season ?: return@mapNotNull null
            val number = episode.number ?: return@mapNotNull null
            mapOf(
                "content_id" to meta.id,
                "content_type" to meta.type,
                "title" to meta.name,
                "season" to season,
                "episode" to number,
                "watched_at" to watchedAt
            )
        }
    }

    fun playbackProgress(meta: Meta, videoId: String?, position: Long, duration: Long, watchedAt: Long): Map<String, Any?> {
        val parts = videoId?.split(':')?.takeIf { it.size == 3 }
        val season = parts?.getOrNull(1)?.toIntOrNull()
        val episode = parts?.getOrNull(2)?.toIntOrNull()
        val progressKey = if (season != null && episode != null) "${meta.id}_s${season}e$episode" else meta.id
        return mapOf(
            "content_id" to meta.id,
            "content_type" to meta.type,
            "video_id" to (videoId ?: meta.id),
            "position" to position,
            "duration" to duration,
            "last_watched" to watchedAt,
            "season" to season,
            "episode" to episode,
            "progress_key" to progressKey
        )
    }

    private fun remoteSource(source: LibraryRemoteSource): Map<String, Any?> = mapOf(
        "provider" to source.provider,
        "title" to source.title,
        "mediaType" to source.mediaType,
        "traktListId" to source.traktListId,
        "tmdbSourceType" to source.tmdbSourceType,
        "tmdbId" to source.tmdbId,
        "sortBy" to source.sortBy,
        "sortHow" to source.sortHow,
        "filters" to source.filters,
        "addonId" to source.addonId,
        "catalogId" to source.catalogId,
        "type" to source.type,
        "genre" to source.genre
    )
}
