package com.fluxa.app.data.remote

import com.google.gson.annotations.SerializedName

data class DatastoreRequest(val authKey: String, val collection: String)
data class DatastoreResponse(val result: List<LibraryItem>)
data class DatastorePutRequest(val authKey: String, val collection: String, val items: List<LibraryItem>)

data class LibraryItem(
    @SerializedName("_id") val id: String,
    val name: String,
    val type: String,
    val poster: String?,
    val background: String? = null,
    val logo: String? = null,
    val state: LibraryItemState? = null,
    val lastWatched: String? = null
)

data class LibraryItemState(
    val lastWatched: String? = null,
    val timeOffset: Long? = 0,
    val duration: Long? = 0,
    val videoId: String? = null,
    val timesWatched: Int? = 0,
    val flaggedWatched: Int? = 0
)

data class CatalogResponse(val metas: List<Meta>? = null)
data class StreamResponse(val streams: List<Stream>? = null)

data class SubtitleResponse(val subtitles: List<SubtitleData>? = null) // Stremio format
