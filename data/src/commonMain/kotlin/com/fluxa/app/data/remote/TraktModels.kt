package com.fluxa.app.data.remote

import kotlinx.serialization.Serializable

@Serializable
data class TraktEpisode(
    val season: Int,
    val number: Int,
    val title: String?,
    val translations: List<TraktTranslation>?,
    val ids: TraktIds? = null
)

@Serializable
data class TraktTranslation(val title: String?, val overview: String?, val language: String?)

@Serializable
data class TraktSearchResult(val show: TraktShow?)

@Serializable
data class TraktShow(val ids: TraktIds)

@Serializable
data class TraktIds(
    val trakt: Int? = null,
    val imdb: String? = null,
    val tmdb: Int? = null,
    val slug: String? = null,
    val tvdb: Int? = null
)

@Serializable
data class TraktSyncItem(
    val rank: Int? = null,
    val id: Long? = null,
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null,
    val seasons: List<TraktWatchedSeason>? = null
)

@Serializable
data class TraktWatchedSeason(
    val number: Int? = null,
    val episodes: List<TraktWatchedEpisode>? = null
)

@Serializable
data class TraktWatchedEpisode(
    val number: Int? = null,
    val runtime: Int? = null,
    val last_watched_at: String? = null,
    val plays: Int? = null
) {
    val lastWatchedAt: String? get() = last_watched_at
}

@Serializable
data class TraktListItem(val movie: TraktSummary? = null, val show: TraktSummary? = null)

@Serializable
data class TraktPlaybackItem(
    val id: Long? = null,
    val progress: Float? = null,
    val paused_at: String? = null,
    val type: String? = null,
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null,
    val episode: TraktEpisode? = null
)

@Serializable
data class TraktSummary(val title: String?, val year: Int?, val ids: TraktIds, val runtime: Int? = null)

@Serializable
data class TraktHistoryItem(
    val id: Long?,
    val watched_at: String?,
    val action: String?,
    val type: String?,
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null,
    val episode: TraktEpisode? = null
)

@Serializable
data class TraktScrobbleRequest(
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null,
    val episode: TraktScrobbleEpisode? = null,
    val progress: Float
)

@Serializable
data class TraktScrobbleEpisode(
    val ids: TraktIds? = null,
    val season: Int? = null,
    val number: Int? = null,
    val title: String? = null
)

@Serializable
data class TraktScrobbleResponse(
    val action: String? = null,
    val progress: Float? = null,
    val id: Long? = null,
    val watched_at: String? = null,
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null,
    val episode: TraktEpisode? = null
)

@Serializable
data class TraktHistorySyncRequest(
    val movies: List<TraktHistoryMovie>? = null,
    val shows: List<TraktHistoryShow>? = null
)

@Serializable
data class TraktHistoryMovie(val ids: TraktIds)

@Serializable
data class TraktHistoryShow(val ids: TraktIds, val seasons: List<TraktHistorySeason>? = null)

@Serializable
data class TraktHistorySeason(val number: Int, val episodes: List<TraktHistoryEpisode>? = null)

@Serializable
data class TraktHistoryEpisode(val number: Int)

@Serializable
data class TraktTrendingItem(
    val watchers: Int? = null,
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null
)

@Serializable
data class TraktAnticipatedItem(
    val list_count: Int? = null,
    val movie: TraktSummary? = null,
    val show: TraktSummary? = null
)

@Serializable
data class TraktTokenResponse(
    val access_token: String,
    val token_type: String,
    val expires_in: Long,
    val refresh_token: String,
    val scope: String,
    val created_at: Long
) {
    val accessToken: String get() = access_token
    val tokenType: String get() = token_type
    val expiresIn: Long get() = expires_in
    val refreshToken: String get() = refresh_token
    val createdAt: Long get() = created_at
}

@Serializable
data class TraktTokenRequest(
    val code: String,
    val client_id: String,
    val client_secret: String,
    val redirect_uri: String,
    val grant_type: String = "authorization_code"
)

@Serializable
data class TraktRefreshTokenRequest(
    val refresh_token: String,
    val client_id: String,
    val client_secret: String,
    val redirect_uri: String,
    val grant_type: String = "refresh_token"
)

@Serializable
data class TraktDeviceCodeRequest(val client_id: String)

@Serializable
data class TraktDeviceCodeResponse(
    val device_code: String,
    val user_code: String,
    val verification_url: String,
    val expires_in: Int,
    val interval: Int
) {
    val deviceCode: String get() = device_code
    val userCode: String get() = user_code
    val verificationUrl: String get() = verification_url
    val expiresIn: Int get() = expires_in
}

@Serializable
data class TraktDeviceTokenRequest(val code: String, val client_id: String, val client_secret: String)

@Serializable
data class ExternalOAuthTokenResponse(
    val access_token: String,
    val refresh_token: String? = null,
    val token_type: String? = null,
    val expires_in: Long? = null
) {
    val accessToken: String get() = access_token
    val refreshToken: String? get() = refresh_token
    val tokenType: String? get() = token_type
    val expiresIn: Long? get() = expires_in
}

@Serializable
data class AnilistTokenRequest(
    val grant_type: String = "authorization_code",
    val client_id: String,
    val client_secret: String,
    val redirect_uri: String,
    val code: String
)

data class AnilistGraphQlRequest(
    val query: String,
    val variables: Map<String, @JvmSuppressWildcards Any?> = emptyMap()
)

@Serializable
data class MalMainPicture(val medium: String? = null, val large: String? = null)

@Serializable
data class MalAnimeListResponse(val data: List<MalAnimeListEntry> = emptyList())

@Serializable
data class MalAnimeListEntry(val node: MalAnimeNode, val list_status: MalListStatus? = null) {
    val listStatus: MalListStatus? get() = list_status
}

@Serializable
data class MalAnimeNode(
    val id: Int,
    val title: String,
    val main_picture: MalMainPicture? = null,
    val num_episodes: Int? = null
) {
    val mainPicture: MalMainPicture? get() = main_picture
    val numEpisodes: Int? get() = num_episodes
}

@Serializable
data class MalListStatus(
    val status: String? = null,
    val num_episodes_watched: Int? = null,
    val updated_at: String? = null
) {
    val numEpisodesWatched: Int? get() = num_episodes_watched
    val updatedAt: String? get() = updated_at
}

@Serializable
data class SimklAllItemsResponse(
    val movies: List<SimklItem> = emptyList(),
    val shows: List<SimklItem> = emptyList(),
    val anime: List<SimklItem> = emptyList()
)

@Serializable
data class SimklItem(
    val title: String? = null,
    val year: Int? = null,
    val ids: SimklIds? = null,
    val status: String? = null,
    val last_watched_at: String? = null,
    val seasons: List<SimklSeason>? = null
) {
    val lastWatchedAt: String? get() = last_watched_at
}

@Serializable
data class SimklIds(
    val imdb: String? = null,
    val tmdb: String? = null,
    val slug: String? = null,
    val simkl: Int? = null
)

@Serializable
data class SimklSeason(val number: Int? = null, val episodes: List<SimklEpisode>? = null)

@Serializable
data class SimklEpisode(val number: Int? = null, val watched_at: String? = null) {
    val watchedAt: String? get() = watched_at
}

@Serializable
data class SimklSearchResult(val type: String? = null, val ids: SimklIds? = null)

@Serializable
data class SimklEpisodeInfo(
    val season: Int? = null,
    val episode: Int? = null,
    val date: String? = null,
    val title: String? = null
)
