package com.fluxa.app.data.remote

import com.google.gson.annotations.SerializedName

data class LoginRequest(val email: String, val password: String)
data class AuthRequest(val authKey: String)
data class AuthResponse(val result: AuthResult)
data class AuthResponseWrapper(val user: AuthUser)
data class AuthResult(val user: AuthUser)
data class AuthUser(val id: String, val email: String, val authKey: String)
data class AddonCollectionResponse(val result: AddonCollectionResult)
data class AddonCollectionResult(val addons: List<AddonDescriptor>)

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

data class AddonDescriptor(val manifest: AddonManifest, val transportUrl: String)
data class AddonManifest(val id: String, val name: String, val description: String? = null, val version: String? = null, val resources: List<Any>?, val types: List<String>?, val catalogs: List<AddonCatalog>?, val idPrefixes: List<String>? = null, val logo: String? = null, val background: String? = null, val configurable: Boolean? = null)
data class AddonCatalog(val type: String? = null, val id: String? = null, val name: String? = null, val extra: List<CatalogExtra>? = null, val extraSupported: List<String>? = null, val genres: List<String>? = null)
data class CatalogExtra(val name: String? = null, val options: List<String>? = null, val isRequired: Boolean? = null, val optionsLimit: Int? = null)
data class CatalogResponse(val metas: List<Meta>? = null)
data class MetaDetailResponse(val meta: MetaDetail? = null)
data class StreamResponse(val streams: List<Stream>? = null)

data class SubtitleResponse(val subtitles: List<SubtitleData>? = null) // Stremio format
