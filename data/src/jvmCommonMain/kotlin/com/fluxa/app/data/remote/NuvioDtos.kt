package com.fluxa.app.data.remote

import com.google.gson.annotations.SerializedName

data class NuvioRefreshRequestDto(@SerializedName("refresh_token") val refreshToken: String)

fun NuvioRefreshRequest.toDto(): NuvioRefreshRequestDto = NuvioRefreshRequestDto(refreshToken)

data class NuvioSessionDto(
    @SerializedName("access_token") val accessToken: String,
    @SerializedName("refresh_token") val refreshToken: String,
    @SerializedName("expires_in") val expiresIn: Long?,
    val user: NuvioUser?
) {
    fun toDomain(): NuvioSession = NuvioSession(
        accessToken = accessToken,
        refreshToken = refreshToken,
        expiresIn = expiresIn,
        user = user
    )
}

data class NuvioProfileDto(
    val id: String,
    @SerializedName("user_id") val userId: String? = null,
    @SerializedName("profile_index") val profileIndex: Int,
    val name: String?,
    @SerializedName("avatar_color_hex") val avatarColorHex: String?,
    @SerializedName("avatar_id") val avatarId: String?,
    @SerializedName("avatar_url") val avatarUrl: String?,
    @SerializedName("uses_primary_addons") val usesPrimaryAddons: Boolean = false,
    @SerializedName("uses_primary_plugins") val usesPrimaryPlugins: Boolean = false,
    @SerializedName("pin_enabled") val pinEnabled: Boolean = false,
    @SerializedName("pin_locked_until") val pinLockedUntil: String? = null,
    @SerializedName("updated_at") val updatedAt: String? = null
) {
    fun toDomain(): NuvioProfile = NuvioProfile(id, userId, profileIndex, name, avatarColorHex, avatarId, avatarUrl, pinEnabled, pinLockedUntil, updatedAt)
}

data class NuvioPinVerifyResultDto(
    val unlocked: Boolean = false,
    @SerializedName("retry_after_seconds") val retryAfterSeconds: Int = 0,
    val message: String? = null,
) {
    fun toDomain() = NuvioPinVerifyResult(unlocked, retryAfterSeconds, message)
}

data class NuvioProfileLockDto(
    @SerializedName("profile_index") val profileIndex: Int,
    @SerializedName("pin_enabled") val pinEnabled: Boolean = false,
    @SerializedName("pin_locked_until") val pinLockedUntil: String? = null,
)

data class NuvioAddonDto(
    val id: String? = null,
    @SerializedName("user_id") val userId: String? = null,
    @SerializedName("profile_id") val profileId: Int? = null,
    val url: String,
    val name: String?,
    val enabled: Boolean,
    @SerializedName("sort_order") val sortOrder: Int
) {
    fun toDomain(): NuvioAddon = NuvioAddon(id, userId, profileId, url, name, enabled, sortOrder)
}

data class NuvioLibraryItemDto(
    @SerializedName("content_id") val contentId: String,
    @SerializedName("content_type") val contentType: String,
    val name: String,
    val poster: String?,
    val background: String?,
    val description: String?,
    @SerializedName("release_info") val releaseInfo: String?,
    @SerializedName("imdb_rating") val imdbRating: Double?,
    val genres: List<String>?,
    @SerializedName("poster_shape") val posterShape: String? = null,
    @SerializedName("addon_base_url") val addonBaseUrl: String? = null,
    @SerializedName("added_at") val addedAt: Long? = null
) {
    fun toDomain(): NuvioLibraryItem = NuvioLibraryItem(
        contentId, contentType, name, poster, background, description, releaseInfo, imdbRating,
        genres, posterShape, addonBaseUrl, addedAt
    )
}

data class NuvioLibraryDeltaEventDto(
    @SerializedName("event_id") val eventId: Long,
    val operation: String,
    @SerializedName("content_id") val contentId: String,
    @SerializedName("content_type") val contentType: String,
    val name: String? = null,
    val poster: String? = null,
    val background: String? = null,
    val description: String? = null,
    @SerializedName("release_info") val releaseInfo: String? = null,
    @SerializedName("imdb_rating") val imdbRating: Double? = null,
    val genres: List<String>? = null,
    @SerializedName("poster_shape") val posterShape: String? = null,
    @SerializedName("addon_base_url") val addonBaseUrl: String? = null,
    @SerializedName("added_at") val addedAt: Long? = null
)

/**
 * A Nuvio account stores plugin repositories separately from Stremio-style
 * addons. Earlier server revisions used different names for the repository
 * URL, so accept all published spellings while keeping one local contract.
 */
data class NuvioPluginDto(
    @com.google.gson.annotations.SerializedName(
        value = "url",
        alternate = ["manifest_url", "manifestUrl", "repository_url", "repositoryUrl", "plugin_url", "pluginUrl"]
    )
    val manifestUrl: String? = null,
    val enabled: Boolean = true,
    @com.google.gson.annotations.SerializedName(value = "sort_order", alternate = ["sortOrder"])
    val sortOrder: Int = 0
)

data class NuvioWatchProgressDto(
    @SerializedName("content_id") val contentId: String,
    @SerializedName("content_type") val contentType: String,
    @SerializedName("video_id") val videoId: String,
    val season: Int?,
    val episode: Int?,
    val position: Long,
    val duration: Long,
    @SerializedName("last_watched") val lastWatched: Long,
    @SerializedName("progress_key") val progressKey: String? = null
) {
    fun toDomain(): NuvioWatchProgress = NuvioWatchProgress(
        contentId, contentType, videoId, season, episode, position, duration, lastWatched, progressKey
    )
}

data class NuvioWatchProgressDeltaEventDto(
    @SerializedName("event_id") val eventId: Long,
    val operation: String,
    @SerializedName("progress_key") val progressKey: String? = null,
    @SerializedName("content_id") val contentId: String,
    @SerializedName("content_type") val contentType: String,
    @SerializedName("video_id") val videoId: String? = null,
    val season: Int? = null,
    val episode: Int? = null,
    val position: Long? = null,
    val duration: Long? = null,
    @SerializedName("last_watched") val lastWatched: Long? = null
)

data class NuvioWatchedItemDto(
    @SerializedName("content_id") val contentId: String,
    @SerializedName("content_type") val contentType: String,
    val title: String? = null,
    val season: Int?,
    val episode: Int?,
    @SerializedName("watched_at") val watchedAt: Long? = null
) {
    fun toDomain(): NuvioWatchedItem = NuvioWatchedItem(contentId, contentType, title, season, episode, watchedAt)
}

data class NuvioWatchedItemDeltaEventDto(
    @SerializedName("event_id") val eventId: Long,
    val operation: String,
    @SerializedName("content_id") val contentId: String,
    @SerializedName("content_type") val contentType: String,
    val title: String? = null,
    val season: Int? = null,
    val episode: Int? = null,
    @SerializedName("watched_at") val watchedAt: Long? = null
)

data class NuvioCollectionFolderSourceDto(
    val provider: String? = null,
    @SerializedName("addonId") val addonId: String?,
    @SerializedName("catalogId") val catalogId: String?,
    val type: String?,
    val genre: String? = null,
    val title: String? = null,
    @SerializedName("mediaType") val mediaType: String? = null,
    @SerializedName("traktListId") val traktListId: Long? = null,
    @SerializedName("tmdbSourceType") val tmdbSourceType: String? = null,
    @SerializedName("tmdbId") val tmdbId: Long? = null,
    @SerializedName("sortBy") val sortBy: String? = null,
    @SerializedName("sortHow") val sortHow: String? = null,
    val filters: Map<String, Any?>? = null
) {
    fun toDomain(): NuvioCollectionFolderSource = NuvioCollectionFolderSource(
        provider, addonId, catalogId, type, genre, title, mediaType, traktListId,
        tmdbSourceType, tmdbId, sortBy, sortHow, filters
    )
}

data class NuvioCollectionFolderDto(
    val id: String?,
    val title: String?,
    @SerializedName("coverImageUrl") val coverImageUrl: String?,
    @SerializedName("coverEmoji") val coverEmoji: String?,
    @SerializedName("focusGifUrl") val focusGifUrl: String?,
    @SerializedName("focusGifEnabled") val focusGifEnabled: Boolean? = null,
    @SerializedName("titleLogoUrl") val titleLogoUrl: String?,
    @SerializedName("heroBackdropUrl") val heroBackdropUrl: String? = null,
    @SerializedName("heroVideoUrl") val heroVideoUrl: String? = null,
    @SerializedName("tileShape") val tileShape: String?,
    @SerializedName("hideTitle") val hideTitle: Boolean?,
    @SerializedName(value = "catalogSources", alternate = ["sources"]) val catalogSources: List<NuvioCollectionFolderSourceDto>?
) {
    fun toDomain(): NuvioCollectionFolder = NuvioCollectionFolder(
        id, title, coverImageUrl, coverEmoji, focusGifUrl, focusGifEnabled, titleLogoUrl,
        heroBackdropUrl, heroVideoUrl, tileShape, hideTitle, catalogSources?.map { it.toDomain() }
    )
}

data class NuvioCollectionDto(
    val id: String?,
    val title: String?,
    @SerializedName("backdropImageUrl") val backdropImageUrl: String?,
    @SerializedName("pinToTop") val pinToTop: Boolean?,
    @SerializedName("showOnHome") val showOnHome: Boolean? = null,
    @SerializedName("viewMode") val viewMode: String?,
    @SerializedName("showAllTab") val showAllTab: Boolean?,
    @SerializedName("focusGlowEnabled") val focusGlowEnabled: Boolean? = null,
    val folders: List<NuvioCollectionFolderDto>?,
    val community: Map<String, Any?>? = null
) {
    fun toDomain(): NuvioCollection = NuvioCollection(
        id, title, backdropImageUrl, pinToTop, showOnHome, viewMode, showAllTab,
        focusGlowEnabled, folders?.map { it.toDomain() }, community
    )
}

data class NuvioCollectionRowDto(
    @SerializedName("collections_json") val collectionsJson: List<NuvioCollectionDto>?
) {
    fun toDomain(): NuvioCollectionRow = NuvioCollectionRow(collectionsJson?.map { it.toDomain() })
}

data class NuvioProfileSettingsRowDto(
    @SerializedName("settings_json") val settingsJson: Map<String, Any?>?
) {
    fun toDomain(): NuvioProfileSettingsRow = NuvioProfileSettingsRow(settingsJson)
}

data class NuvioAvatarDto(
    val id: String,
    @SerializedName("storage_path") val storagePath: String?
) {
    fun toDomain(): NuvioAvatar = NuvioAvatar(id, storagePath)
}
