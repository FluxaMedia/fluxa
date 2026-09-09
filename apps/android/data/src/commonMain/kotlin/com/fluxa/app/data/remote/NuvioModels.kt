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
    val avatarUrl: String?,
    val pinEnabled: Boolean = false,
    val pinLockedUntil: String? = null,
    val updatedAt: String? = null,
)

@Serializable
data class NuvioPinVerifyResult(
    val unlocked: Boolean = false,
    val retryAfterSeconds: Int = 0,
    val message: String? = null,
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
