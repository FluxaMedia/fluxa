package com.fluxa.app.data.remote

import com.google.gson.annotations.SerializedName

data class NuvioCredentials(val email: String, val password: String)
data class NuvioRefreshRequest(@SerializedName("refresh_token") val refreshToken: String)

data class NuvioUser(val id: String, val email: String)

data class NuvioSession(
    @SerializedName("access_token") val accessToken: String,
    @SerializedName("refresh_token") val refreshToken: String,
    @SerializedName("expires_in") val expiresIn: Long?,
    val user: NuvioUser?
)

data class NuvioProfile(
    val id: String,
    @SerializedName("user_id") val userId: String? = null,
    @SerializedName("profile_index") val profileIndex: Int,
    val name: String?,
    @SerializedName("avatar_color_hex") val avatarColorHex: String?,
    @SerializedName("avatar_id") val avatarId: String?,
    @SerializedName("avatar_url") val avatarUrl: String?
)

data class NuvioAddon(
    val id: String? = null,
    @SerializedName("user_id") val userId: String? = null,
    @SerializedName("profile_id") val profileId: Int? = null,
    val url: String,
    val name: String?,
    val enabled: Boolean,
    @SerializedName("sort_order") val sortOrder: Int
)

data class NuvioLibraryItem(
    @SerializedName("content_id") val contentId: String,
    @SerializedName("content_type") val contentType: String,
    val name: String,
    val poster: String?,
    val background: String?,
    val description: String?,
    @SerializedName("release_info") val releaseInfo: String?,
    @SerializedName("imdb_rating") val imdbRating: Double?,
    val genres: List<String>?
)

data class NuvioWatchProgress(
    @SerializedName("content_id") val contentId: String,
    @SerializedName("content_type") val contentType: String,
    @SerializedName("video_id") val videoId: String,
    val season: Int?,
    val episode: Int?,
    val position: Long,
    val duration: Long,
    @SerializedName("last_watched") val lastWatched: Long
)

data class NuvioWatchedItem(
    @SerializedName("content_id") val contentId: String,
    @SerializedName("content_type") val contentType: String,
    val season: Int?,
    val episode: Int?
)

data class NuvioCollectionFolderSource(
    val provider: String? = null,
    @SerializedName("addonId") val addonId: String?,
    @SerializedName("catalogId") val catalogId: String?,
    val type: String?,
    val genre: String? = null
)

data class NuvioCollectionFolder(
    val id: String?,
    val title: String?,
    @SerializedName("coverImageUrl") val coverImageUrl: String?,
    @SerializedName("coverEmoji") val coverEmoji: String?,
    @SerializedName("focusGifUrl") val focusGifUrl: String?,
    @SerializedName("focusGifEnabled") val focusGifEnabled: Boolean? = null,
    @SerializedName("titleLogoUrl") val titleLogoUrl: String?,
    @SerializedName("heroBackdropUrl") val heroBackdropUrl: String? = null,
    @SerializedName("tileShape") val tileShape: String?,
    @SerializedName("hideTitle") val hideTitle: Boolean?,
    @SerializedName(value = "catalogSources", alternate = ["sources"]) val catalogSources: List<NuvioCollectionFolderSource>?
)

data class NuvioCollection(
    val id: String?,
    val title: String?,
    @SerializedName("backdropImageUrl") val backdropImageUrl: String?,
    @SerializedName("pinToTop") val pinToTop: Boolean?,
    @SerializedName("showOnHome") val showOnHome: Boolean? = null,
    @SerializedName("viewMode") val viewMode: String?,
    @SerializedName("showAllTab") val showAllTab: Boolean?,
    @SerializedName("focusGlowEnabled") val focusGlowEnabled: Boolean? = null,
    val folders: List<NuvioCollectionFolder>?
)

data class NuvioCollectionRow(
    @SerializedName("collections_json") val collectionsJson: List<NuvioCollection>?
)

data class NuvioProfileSettingsRow(
    @SerializedName("settings_json") val settingsJson: Map<String, Any?>?
)

data class NuvioAvatar(
    val id: String,
    @SerializedName("storage_path") val storagePath: String?
)
