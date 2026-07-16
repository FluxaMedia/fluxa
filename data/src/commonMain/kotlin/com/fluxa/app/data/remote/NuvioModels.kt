package com.fluxa.app.data.remote

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class NuvioCredentials(val email: String, val password: String)

@Serializable
data class NuvioRefreshRequest(@SerialName("refresh_token") val refreshToken: String)

@Serializable
data class NuvioHealth(val status: String? = null)

@Serializable
data class NuvioUser(val id: String, val email: String)

@Serializable
data class NuvioSession(
    val accessToken: String,
    val refreshToken: String,
    val expiresIn: Long?,
    val user: NuvioUser?
)

@Serializable
data class NuvioProfile(
    val id: String,
    val userId: String? = null,
    val profileIndex: Int,
    val name: String?,
    val avatarColorHex: String?,
    val avatarId: String?,
    val avatarUrl: String?
)

@Serializable
data class NuvioAddon(
    val id: String? = null,
    val userId: String? = null,
    val profileId: Int? = null,
    val url: String,
    val name: String?,
    val enabled: Boolean,
    val sortOrder: Int
)

@Serializable
data class NuvioAvatar(val id: String, val storagePath: String?)

@Serializable
data class NuvioLibraryItem(
    val contentId: String,
    val contentType: String,
    val name: String,
    val poster: String?,
    val background: String?,
    val description: String?,
    val releaseInfo: String?,
    val imdbRating: Double?,
    val genres: List<String>?,
    val posterShape: String? = null,
    val addonBaseUrl: String? = null,
    val addedAt: Long? = null
)

@Serializable
data class NuvioWatchProgress(
    val contentId: String,
    val contentType: String,
    val videoId: String,
    val season: Int?,
    val episode: Int?,
    val position: Long,
    val duration: Long,
    val lastWatched: Long,
    val progressKey: String? = null
)

@Serializable
data class NuvioWatchedItem(
    val contentId: String,
    val contentType: String,
    val title: String? = null,
    val season: Int?,
    val episode: Int?,
    val watchedAt: Long? = null
)

data class NuvioCollectionFolderSource(
    val provider: String? = null,
    val addonId: String?,
    val catalogId: String?,
    val type: String?,
    val genre: String? = null,
    val title: String? = null,
    val mediaType: String? = null,
    val traktListId: Long? = null,
    val tmdbSourceType: String? = null,
    val tmdbId: Long? = null,
    val sortBy: String? = null,
    val sortHow: String? = null,
    val filters: Map<String, Any?>? = null
)

data class NuvioCollectionFolder(
    val id: String?,
    val title: String?,
    val coverImageUrl: String?,
    val coverEmoji: String?,
    val focusGifUrl: String?,
    val focusGifEnabled: Boolean? = null,
    val titleLogoUrl: String?,
    val heroBackdropUrl: String? = null,
    val heroVideoUrl: String? = null,
    val tileShape: String?,
    val hideTitle: Boolean?,
    val catalogSources: List<NuvioCollectionFolderSource>?
)

data class NuvioCollection(
    val id: String?,
    val title: String?,
    val backdropImageUrl: String?,
    val pinToTop: Boolean?,
    val showOnHome: Boolean? = null,
    val viewMode: String?,
    val showAllTab: Boolean?,
    val focusGlowEnabled: Boolean? = null,
    val folders: List<NuvioCollectionFolder>?,
    val community: Map<String, Any?>? = null
)

data class NuvioCollectionRow(val collectionsJson: List<NuvioCollection>?)

data class NuvioProfileSettingsRow(val settingsJson: Map<String, Any?>?)
