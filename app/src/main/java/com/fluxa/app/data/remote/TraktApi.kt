package com.fluxa.app.data.remote

import com.google.gson.annotations.SerializedName
import okhttp3.OkHttpClient
import retrofit2.Response
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory
import retrofit2.http.Body
import retrofit2.http.DELETE
import retrofit2.http.Field
import retrofit2.http.FormUrlEncoded
import retrofit2.http.GET
import retrofit2.http.Header
import retrofit2.http.Headers
import retrofit2.http.POST
import retrofit2.http.Path
import retrofit2.http.Query

interface TraktApi {
    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @GET("search/tmdb/{id}?type=show")
    suspend fun getShowByTmdbId(
        @Path("id") tmdbId: Int,
        @Header("trakt-api-key") apiKey: String
    ): List<TraktSearchResult>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @GET("shows/{id}/seasons/{season}")
    suspend fun getSeasonEpisodes(
        @Path("id") traktOrImdbId: String, 
        @Path("season") seasonNumber: Int,
        @Header("trakt-api-key") apiKey: String,
        @Query("translations") translations: String? = null
    ): List<TraktEpisode>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @GET("sync/watchlist")
    suspend fun getWatchlist(
        @Header("Authorization") token: String,
        @Header("trakt-api-key") apiKey: String,
        @Query("page") page: Int = 1,
        @Query("limit") limit: Int = 1000
    ): Response<List<TraktSyncItem>>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @GET("sync/playback")
    suspend fun getPlayback(
        @Header("Authorization") token: String,
        @Header("trakt-api-key") apiKey: String
    ): List<TraktPlaybackItem>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @DELETE("sync/playback/{id}")
    suspend fun deletePlayback(
        @Path("id") id: Long,
        @Header("Authorization") token: String,
        @Header("trakt-api-key") apiKey: String
    ): Response<Unit>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @GET("sync/watched/movies")
    suspend fun getWatchedMovies(
        @Header("Authorization") token: String,
        @Header("trakt-api-key") apiKey: String,
        @Query("page") page: Int = 1,
        @Query("limit") limit: Int = 1000,
        @Query("extended") extended: String? = null
    ): Response<List<TraktSyncItem>>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @GET("sync/watched/shows")
    suspend fun getWatchedShows(
        @Header("Authorization") token: String,
        @Header("trakt-api-key") apiKey: String,
        @Query("page") page: Int = 1,
        @Query("limit") limit: Int = 1000,
        @Query("extended") extended: String? = null
    ): Response<List<TraktSyncItem>>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @GET("sync/collection/movies")
    suspend fun getMovieCollection(
        @Header("Authorization") token: String,
        @Header("trakt-api-key") apiKey: String,
        @Query("page") page: Int = 1,
        @Query("limit") limit: Int = 1000
    ): Response<List<TraktSyncItem>>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @GET("sync/collection/shows")
    suspend fun getShowCollection(
        @Header("Authorization") token: String,
        @Header("trakt-api-key") apiKey: String,
        @Query("page") page: Int = 1,
        @Query("limit") limit: Int = 1000
    ): Response<List<TraktSyncItem>>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @GET("sync/watchlist/{type}")
    suspend fun getWatchlistByType(
        @Path("type") type: String,
        @Header("Authorization") token: String,
        @Header("trakt-api-key") apiKey: String,
        @Query("page") page: Int = 1,
        @Query("limit") limit: Int = 1000
    ): Response<List<TraktSyncItem>>

    @POST("https://api.trakt.tv/oauth/token")
    suspend fun exchangeCode(@Body request: TraktTokenRequest): TraktTokenResponse

    @POST("https://api.trakt.tv/oauth/token")
    suspend fun refreshToken(@Body request: TraktRefreshTokenRequest): TraktTokenResponse

    @Headers("Content-Type: application/json")
    @POST("oauth/device/code")
    suspend fun createDeviceCode(@Body request: TraktDeviceCodeRequest): TraktDeviceCodeResponse

    @Headers("Content-Type: application/json")
    @POST("oauth/device/token")
    suspend fun exchangeDeviceCode(@Body request: TraktDeviceTokenRequest): Response<TraktTokenResponse>

    @FormUrlEncoded
    @POST("https://myanimelist.net/v1/oauth2/token")
    suspend fun exchangeMalCode(
        @Field("client_id") clientId: String,
        @Field("client_secret") clientSecret: String?,
        @Field("grant_type") grantType: String,
        @Field("code") code: String,
        @Field("redirect_uri") redirectUri: String,
        @Field("code_verifier") codeVerifier: String
    ): ExternalOAuthTokenResponse

    @FormUrlEncoded
    @POST("https://api.simkl.com/oauth/token")
    suspend fun exchangeSimklCode(
        @Field("client_id") clientId: String,
        @Field("client_secret") clientSecret: String,
        @Field("grant_type") grantType: String,
        @Field("code") code: String,
        @Field("redirect_uri") redirectUri: String
    ): ExternalOAuthTokenResponse

    @GET("https://api.myanimelist.net/v2/users/@me/animelist")
    suspend fun getMalAnimeList(
        @Header("Authorization") token: String,
        @Query("status") status: String = "watching",
        @Query("fields") fields: String = "list_status,num_episodes,main_picture",
        @Query("limit") limit: Int = 1000
    ): MalAnimeListResponse

    @GET("https://api.simkl.com/sync/all-items/{type}/{status}")
    suspend fun getSimklAllItems(
        @Path("type") type: String,
        @Path("status") status: String,
        @Header("Authorization") token: String,
        @Header("simkl-api-key") apiKey: String,
        @Query("extended") extended: String = "full",
        @Query("episode_watched_at") episodeWatchedAt: String = "yes"
    ): SimklAllItemsResponse

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @POST("scrobble/start")
    suspend fun scrobbleStart(@Header("Authorization") token: String, @Header("trakt-api-key") apiKey: String, @Body request: TraktScrobbleRequest): Response<TraktScrobbleResponse>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @POST("scrobble/pause")
    suspend fun scrobblePause(@Header("Authorization") token: String, @Header("trakt-api-key") apiKey: String, @Body request: TraktScrobbleRequest): Response<TraktScrobbleResponse>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @POST("scrobble/stop")
    suspend fun scrobbleStop(@Header("Authorization") token: String, @Header("trakt-api-key") apiKey: String, @Body request: TraktScrobbleRequest): Response<TraktScrobbleResponse>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @POST("sync/history")
    suspend fun addToHistory(@Header("Authorization") token: String, @Header("trakt-api-key") apiKey: String, @Body request: TraktHistorySyncRequest): Response<Unit>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @POST("sync/history/remove")
    suspend fun removeFromHistory(@Header("Authorization") token: String, @Header("trakt-api-key") apiKey: String, @Body request: TraktHistorySyncRequest): Response<Unit>
    
    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @GET("movies/trending?limit=15")
    suspend fun getTrendingMovies(@Header("trakt-api-key") apiKey: String): List<TraktTrendingItem>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @GET("shows/trending?limit=15")
    suspend fun getTrendingShows(@Header("trakt-api-key") apiKey: String): List<TraktTrendingItem>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @GET("movies/anticipated?limit=10")
    suspend fun getAnticipatedMovies(@Header("trakt-api-key") apiKey: String): List<TraktAnticipatedItem>

    @Headers("Content-Type: application/json", "trakt-api-version: 2")
    @GET("shows/anticipated?limit=10")
    suspend fun getAnticipatedShows(@Header("trakt-api-key") apiKey: String): List<TraktAnticipatedItem>
    
    companion object {
        private const val BASE_URL = "https://api.trakt.tv/"
        private var instance: TraktApi? = null
        fun create(): TraktApi {
            if (instance == null) {
                val logging = okhttp3.logging.HttpLoggingInterceptor().apply { level = okhttp3.logging.HttpLoggingInterceptor.Level.NONE }
                val client = OkHttpClient.Builder()
                    .addInterceptor(logging)
                    .connectTimeout(15, java.util.concurrent.TimeUnit.SECONDS)
                    .readTimeout(15, java.util.concurrent.TimeUnit.SECONDS)
                    .build()
                instance = Retrofit.Builder()
                    .baseUrl(BASE_URL)
                    .callFactory { request -> client.newCall(request) }
                    .addConverterFactory(GsonConverterFactory.create())
                    .build()
                    .create(TraktApi::class.java)
            }
            return instance!!
        }
    }
}
