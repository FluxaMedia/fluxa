package com.fluxa.app.data.remote

import com.google.gson.annotations.SerializedName

data class TraktEpisode(
    val season: Int,
    val number: Int,
    val title: String?,
    val translations: List<TraktTranslation>?,
    val ids: TraktIds? = null
)

data class TraktTranslation(
    val title: String?,
    val overview: String?,
    val language: String?
)

data class TraktSearchResult(
    val show: TraktShow?
)

data class TraktShow(
    val ids: TraktIds
)

data class TraktIds(
    val trakt: Int? = null,
    val imdb: String? = null,
    val tmdb: Int? = null,
    val slug: String? = null,
    val tvdb: Int? = null
)

data class TraktSyncItem(
    val rank: Int? = null,
    val id: Long? = null,
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null,
    val seasons: List<TraktWatchedSeason>? = null
)

data class TraktWatchedSeason(
    val number: Int? = null,
    val episodes: List<TraktWatchedEpisode>? = null
)

data class TraktWatchedEpisode(
    val number: Int? = null,
    val runtime: Int? = null,
    @SerializedName("last_watched_at") val lastWatchedAt: String? = null,
    val plays: Int? = null
)

data class TraktPlaybackItem(
    val id: Long? = null,
    val progress: Float? = null,
    val paused_at: String? = null,
    val type: String? = null,
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null,
    val episode: TraktEpisode? = null
)

data class TraktSummary(
    val title: String?,
    val year: Int?,
    val ids: TraktIds,
    val runtime: Int? = null
)

data class TraktHistoryItem(
    val id: Long?,
    val watched_at: String?,
    val action: String?,
    val type: String?,
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null,
    val episode: TraktEpisode? = null
)

data class TraktScrobbleRequest(
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null,
    val episode: TraktScrobbleEpisode? = null,
    val progress: Float // 0 to 100
)

data class TraktScrobbleEpisode(
    val ids: TraktIds? = null,
    val season: Int? = null,
    val number: Int? = null,
    val title: String? = null
)

data class TraktScrobbleResponse(
    val action: String? = null,
    val progress: Float? = null,
    val id: Long? = null,
    val watched_at: String? = null,
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null,
    val episode: TraktEpisode? = null
)

data class TraktHistorySyncRequest(
    val movies: List<TraktHistoryMovie>? = null,
    val shows: List<TraktHistoryShow>? = null
)

data class TraktHistoryMovie(
    val ids: TraktIds
)

data class TraktHistoryShow(
    val ids: TraktIds,
    val seasons: List<TraktHistorySeason>? = null
)

data class TraktHistorySeason(
    val number: Int,
    val episodes: List<TraktHistoryEpisode>? = null
)

data class TraktHistoryEpisode(
    val number: Int
)

data class TraktTrendingItem(
    val watchers: Int? = null,
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null
)

data class TraktAnticipatedItem(
    val list_count: Int? = null,
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null
)

data class TraktTokenResponse(
    @SerializedName("access_token") val accessToken: String,
    @SerializedName("token_type") val tokenType: String,
    @SerializedName("expires_in") val expiresIn: Long,
    @SerializedName("refresh_token") val refreshToken: String,
    @SerializedName("scope") val scope: String,
    @SerializedName("created_at") val createdAt: Long
)

data class TraktTokenRequest(
    val code: String,
    val client_id: String,
    val client_secret: String,
    val redirect_uri: String,
    val grant_type: String = "authorization_code"
)

data class TraktRefreshTokenRequest(
    val refresh_token: String,
    val client_id: String,
    val client_secret: String,
    val redirect_uri: String,
    val grant_type: String = "refresh_token"
)

data class TraktDeviceCodeRequest(
    val client_id: String
)

data class TraktDeviceCodeResponse(
    @SerializedName("device_code") val deviceCode: String,
    @SerializedName("user_code") val userCode: String,
    @SerializedName("verification_url") val verificationUrl: String,
    @SerializedName("expires_in") val expiresIn: Int,
    val interval: Int
)

data class TraktDeviceTokenRequest(
    val code: String,
    val client_id: String,
    val client_secret: String
)

data class ExternalOAuthTokenResponse(
    @SerializedName("access_token") val accessToken: String,
    @SerializedName("refresh_token") val refreshToken: String? = null,
    @SerializedName("token_type") val tokenType: String? = null,
    @SerializedName("expires_in") val expiresIn: Long? = null
)

data class MalAnimeListResponse(
    val data: List<MalAnimeListEntry> = emptyList()
)

data class MalAnimeListEntry(
    val node: MalAnimeNode,
    @SerializedName("list_status") val listStatus: MalListStatus? = null
)

data class MalAnimeNode(
    val id: Int,
    val title: String,
    @SerializedName("main_picture") val mainPicture: MalMainPicture? = null,
    @SerializedName("num_episodes") val numEpisodes: Int? = null
)

data class MalMainPicture(
    val medium: String? = null,
    val large: String? = null
)

data class MalListStatus(
    val status: String? = null,
    @SerializedName("num_episodes_watched") val numEpisodesWatched: Int? = null,
    @SerializedName("updated_at") val updatedAt: String? = null
)

data class SimklAllItemsResponse(
    val movies: List<SimklItem> = emptyList(),
    val shows: List<SimklItem> = emptyList(),
    val anime: List<SimklItem> = emptyList()
)

data class SimklItem(
    val title: String? = null,
    val year: Int? = null,
    val ids: SimklIds? = null,
    val status: String? = null,
    @SerializedName("last_watched_at") val lastWatchedAt: String? = null,
    val seasons: List<SimklSeason>? = null
)

data class SimklIds(
    val imdb: String? = null,
    val tmdb: String? = null,
    val slug: String? = null,
    val simkl: Int? = null
)

data class SimklSeason(
    val number: Int? = null,
    val episodes: List<SimklEpisode>? = null
)

data class SimklEpisode(
    val number: Int? = null,
    @SerializedName("watched_at") val watchedAt: String? = null
)

data class SimklSearchResult(
    val type: String? = null,
    val ids: SimklIds? = null
)

data class SimklEpisodeInfo(
    val season: Int? = null,
    val episode: Int? = null,
    val date: String? = null,
    val title: String? = null
)
