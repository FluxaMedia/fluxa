package com.fluxa.app.data.remote

import kotlinx.serialization.Serializable

@Serializable
data class DatastoreRequest(val authKey: String, val collection: String)

@Serializable
data class DatastoreResponse(val result: List<LibraryItem>)

@Serializable
data class DatastorePutRequest(val authKey: String, val collection: String, val items: List<LibraryItem>)

@Serializable
data class LibraryItem(
    val _id: String,
    val name: String,
    val type: String,
    val poster: String?,
    val background: String? = null,
    val logo: String? = null,
    val state: LibraryItemState? = null,
    val lastWatched: String? = null
) {
    val id: String get() = _id
}

@Serializable
data class LibraryItemState(
    val lastWatched: String? = null,
    val timeOffset: Long? = 0,
    val duration: Long? = 0,
    val videoId: String? = null,
    val timesWatched: Int? = 0,
    val flaggedWatched: Int? = 0
)

@Serializable
data class CatalogResponse(val metas: List<Meta>? = null)

data class StreamResponse(val streams: List<Stream>? = null)

data class SubtitleResponse(val subtitles: List<SubtitleData>? = null)
