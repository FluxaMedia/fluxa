package com.fluxa.app.shared.feature.localmedia

import kotlinx.coroutines.flow.StateFlow
import kotlinx.serialization.Serializable

@Serializable
enum class LocalMediaKind { Movies, TvShows, Anime }

@Serializable
enum class LocalMediaSourceType { LocalFolder, Smb, WebDav }

@Serializable
data class LocalMediaSourceConfig(
    val id: String,
    val kind: LocalMediaKind,
    val sourceType: LocalMediaSourceType,
    val location: String,
    val displayName: String,
    val username: String? = null,
    val password: String? = null,
    val enabled: Boolean = true,
)

data class LocalMediaSourceInput(
    val kind: LocalMediaKind,
    val sourceType: LocalMediaSourceType,
    val location: String,
    val displayName: String = "",
    val username: String = "",
    val password: String = "",
)

data class LocalMediaPickedFolder(
    val location: String,
    val displayName: String = "",
)

data class LocalMediaSourceUiModel(
    val id: String,
    val kind: LocalMediaKind,
    val sourceType: LocalMediaSourceType,
    val displayName: String,
    val location: String,
    val enabled: Boolean,
)

@Serializable
data class LocalMediaIndexedFile(
    val id: String,
    val sourceId: String,
    val locator: String,
    val displayName: String,
    val sizeBytes: Long = 0,
    val modifiedAtMs: Long = 0,
    val signature: String,
    val parsedTitle: String,
    val parsedYear: Int? = null,
    val season: Int? = null,
    val episode: Int? = null,
    val absoluteEpisode: Int? = null,
    val contentId: String? = null,
    val contentType: String? = null,
    val videoId: String? = null,
    val metadataAddonUrl: String? = null,
    val matchConfidence: Float = 0f,
)

@Serializable
data class LocalMediaCatalogEntry(
    val contentId: String,
    val contentType: String,
    val kind: LocalMediaKind,
    val title: String,
    val year: Int? = null,
    val posterUrl: String? = null,
    val backdropUrl: String? = null,
    val logoUrl: String? = null,
    val description: String? = null,
    val releaseLabel: String? = null,
    val ratingLabel: String? = null,
    val ageRating: String? = null,
    val genres: List<String> = emptyList(),
    val seasonsCount: Int? = null,
    val runtimeLabel: String? = null,
    val metadataAddonUrl: String? = null,
    val fileCount: Int = 1,
)

data class LocalMediaPlaybackStream(
    val fileId: String,
    val title: String,
    val playableUrl: String,
    val sizeBytes: Long,
    val sourceLabel: String,
)

data class LocalMediaLibrarySnapshot(
    val sources: List<LocalMediaSourceUiModel> = emptyList(),
    val movies: List<LocalMediaCatalogEntry> = emptyList(),
    val tvShows: List<LocalMediaCatalogEntry> = emptyList(),
    val anime: List<LocalMediaCatalogEntry> = emptyList(),
    val indexedFileCount: Int = 0,
    val unmatchedFileCount: Int = 0,
    val isScanning: Boolean = false,
    val lastScanAtMs: Long = 0L,
    val error: String? = null,
) {
    val itemCount: Int get() = movies.size + tvShows.size + anime.size
}

interface LocalMediaLibraryService {
    val state: StateFlow<LocalMediaLibrarySnapshot>
    suspend fun addSource(input: LocalMediaSourceInput)
    suspend fun removeSource(sourceId: String)
    suspend fun scan(forceMetadata: Boolean = false)
    fun playbackStreams(contentId: String, contentType: String, videoId: String?): List<LocalMediaPlaybackStream>
    fun close() = Unit
}

object EmptyLocalMediaLibraryService : LocalMediaLibraryService {
    private val empty = kotlinx.coroutines.flow.MutableStateFlow(LocalMediaLibrarySnapshot())
    override val state: StateFlow<LocalMediaLibrarySnapshot> = empty
    override suspend fun addSource(input: LocalMediaSourceInput) = Unit
    override suspend fun removeSource(sourceId: String) = Unit
    override suspend fun scan(forceMetadata: Boolean) = Unit
    override fun playbackStreams(contentId: String, contentType: String, videoId: String?) = emptyList<LocalMediaPlaybackStream>()
}
