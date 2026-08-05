package com.fluxa.app.data.remote

import com.google.gson.JsonElement
import com.google.gson.annotations.SerializedName
import retrofit2.Response
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.Header
import retrofit2.http.POST
import retrofit2.http.Path
import retrofit2.http.Query
import retrofit2.http.Url

interface TmdbService {
    @GET("3/{type}/{id}/recommendations")
    suspend fun getRecommendations(@Path("type") type: String, @Path("id") id: String, @Query("language") lang: String = "en", @Query("api_key") apiKey: String = ""): TmdbCatalogResponse

    @GET("3/{type}/{id}/similar")
    suspend fun getSimilar(@Path("type") type: String, @Path("id") id: String, @Query("language") lang: String = "en", @Query("api_key") apiKey: String = ""): TmdbCatalogResponse

    @GET("3/find/{id}")
    suspend fun findById(@Path("id") id: String, @Query("external_source") source: String = "imdb_id", @Query("language") lang: String = "en", @Query("api_key") apiKey: String = ""): TmdbFindResponse

    @GET("3/{type}/{id}/videos")
    suspend fun getVideos(@Path("type") type: String, @Path("id") id: String, @Query("language") lang: String = "en", @Query("api_key") apiKey: String = ""): TmdbVideosResponse

    @GET("3/{type}/{id}")
    suspend fun getDetail(@Path("type") type: String, @Path("id") id: String, @Query("language") lang: String = "en", @Query("api_key") apiKey: String = ""): TmdbDetailResponse

    @GET("3/{type}/{id}/credits")
    suspend fun getCredits(@Path("type") type: String, @Path("id") id: String, @Query("language") lang: String = "en", @Query("api_key") apiKey: String = ""): TmdbCreditsResponse

    @GET("3/{type}/{id}/images")
    suspend fun getImages(@Path("type") type: String, @Path("id") id: String, @Query("include_image_language") langs: String = "en,null", @Query("api_key") apiKey: String = ""): TmdbImagesResponse

    @GET("3/tv/{id}/season/{season}")
    suspend fun getSeasonDetail(@Path("id") id: String, @Path("season") season: Int, @Query("language") lang: String = "en", @Query("api_key") apiKey: String = ""): TmdbSeasonDetailResponse

    @GET("3/movie/{id}/release_dates")
    suspend fun getMovieReleaseDates(@Path("id") id: String, @Query("api_key") apiKey: String = ""): TmdbReleaseDatesResponse

    @GET("3/tv/{id}/content_ratings")
    suspend fun getTvContentRatings(@Path("id") id: String, @Query("api_key") apiKey: String = ""): TmdbContentRatingsResponse

    @GET("3/collection/{id}")
    suspend fun getCollection(@Path("id") id: Int, @Query("language") lang: String = "en", @Query("api_key") apiKey: String = ""): TmdbCollectionResponse

    @GET
    suspend fun getCollectionSource(@Url url: String): JsonElement

    companion object {
        private var instance: TmdbService? = null
        fun create(): TmdbService {
            if (instance == null) {
                instance = Retrofit.Builder()
                    .baseUrl("https://api.themoviedb.org/")
                    .callFactory { request -> StremioService.sharedClient.newCall(request) }
                    .addConverterFactory(GsonConverterFactory.create())
                    .build()
                    .create(TmdbService::class.java)
            }
            return instance!!
        }
    }
}

interface OpenSubtitlesService {
    @GET("3/subtitles")
    suspend fun searchSubtitles(@Query("movie_hash") hash: String): OpenSubtitlesResponse

    companion object {
        fun create(): OpenSubtitlesService {
            return Retrofit.Builder()
                .baseUrl("https://api.opensubtitles.com/")
                .callFactory { request -> StremioService.sharedClient.newCall(request) }
                .addConverterFactory(GsonConverterFactory.create())
                .build()
                .create(OpenSubtitlesService::class.java)
        }
    }
}

data class OpenSubtitlesResponse(val data: List<SubtitleData>)

data class TmdbFindResponse(
    @SerializedName("movie_results") val movieResults: List<TmdbMeta>,
    @SerializedName("tv_results") val tvResults: List<TmdbMeta>
)

data class TmdbCatalogResponse(val results: List<TmdbMeta>)
data class TmdbVideosResponse(val results: List<TmdbVideo> = emptyList())
data class TmdbVideo(
    val id: String?,
    val key: String?,
    val name: String?,
    val site: String?,
    val type: String?,
    val official: Boolean? = null,
    @SerializedName("published_at") val publishedAt: String? = null
)
data class TmdbMeta(
    val id: Int,
    val title: String?,
    val name: String?,
    val original_name: String? = null,
    val release_date: String?,
    val first_air_date: String?,
    val media_type: String?,
    @SerializedName("poster_path") val posterPath: String? = null,
    @SerializedName("backdrop_path") val backdropPath: String? = null,
    val overview: String? = null
)

// Full detail response (movie or TV)
data class TmdbDetailResponse(
    val id: Int?,
    val overview: String?,
    val genres: List<TmdbGenre>?,
    val runtime: Int?,
    val status: String?,
    @SerializedName("origin_country") val originCountry: List<String>?,
    @SerializedName("original_language") val originalLanguage: String?,
    @SerializedName("vote_average") val voteAverage: Double?,
    @SerializedName("vote_count") val voteCount: Int?,
    @SerializedName("production_companies") val productionCompanies: List<TmdbCompany>?,
    @SerializedName("belongs_to_collection") val belongsToCollection: TmdbCollectionRef?,
    // TV-specific
    @SerializedName("episode_run_time") val episodeRunTime: List<Int>?,
    val networks: List<TmdbNetwork>?,
    @SerializedName("number_of_seasons") val numberOfSeasons: Int?,
    @SerializedName("created_by") val createdBy: List<TmdbCreator>?,
    val seasons: List<TmdbSeasonInfo>?
)

data class TmdbGenre(val id: Int, val name: String)

data class TmdbCompany(
    val id: Int,
    val name: String,
    @SerializedName("logo_path") val logoPath: String?
)

data class TmdbNetwork(
    val id: Int,
    val name: String,
    @SerializedName("logo_path") val logoPath: String?
)

data class TmdbCollectionRef(
    val id: Int,
    val name: String?,
    @SerializedName("poster_path") val posterPath: String?,
    @SerializedName("backdrop_path") val backdropPath: String?
)

data class TmdbCreator(
    val id: Int,
    val name: String,
    @SerializedName("profile_path") val profilePath: String?
)

data class TmdbSeasonInfo(
    val id: Int,
    @SerializedName("season_number") val seasonNumber: Int,
    @SerializedName("poster_path") val posterPath: String?,
    val name: String?,
    @SerializedName("episode_count") val episodeCount: Int?
)

// Credits
data class TmdbCreditsResponse(
    val cast: List<TmdbCastMember>?,
    val crew: List<TmdbCrewMember>?
)

data class TmdbCastMember(
    val id: Int,
    val name: String,
    val character: String?,
    @SerializedName("profile_path") val profilePath: String?,
    val order: Int?
)

data class TmdbCrewMember(
    val id: Int,
    val name: String,
    val job: String?,
    val department: String?,
    @SerializedName("profile_path") val profilePath: String?
)

// Images
data class TmdbImagesResponse(
    val logos: List<TmdbImage>?,
    val backdrops: List<TmdbImage>?,
    val posters: List<TmdbImage>?
)

data class TmdbImage(
    @SerializedName("file_path") val filePath: String,
    @SerializedName("iso_639_1") val language: String?,
    @SerializedName("vote_average") val voteAverage: Double?
)

// TV Season with episodes
data class TmdbSeasonDetailResponse(
    val id: Int?,
    @SerializedName("season_number") val seasonNumber: Int?,
    @SerializedName("poster_path") val posterPath: String?,
    val episodes: List<TmdbEpisodeDetail>?
)

data class TmdbEpisodeDetail(
    val id: Int,
    @SerializedName("episode_number") val episodeNumber: Int,
    @SerializedName("season_number") val seasonNumber: Int,
    val name: String?,
    val overview: String?,
    @SerializedName("still_path") val stillPath: String?,
    val runtime: Int?,
    @SerializedName("vote_average") val voteAverage: Double?,
    @SerializedName("air_date") val airDate: String?
)

// Release dates (movie age rating)
data class TmdbReleaseDatesResponse(val results: List<TmdbReleaseDateCountry>?)

data class TmdbReleaseDateCountry(
    @SerializedName("iso_3166_1") val country: String,
    @SerializedName("release_dates") val releaseDates: List<TmdbReleaseDate>?
)

data class TmdbReleaseDate(
    val certification: String?,
    @SerializedName("release_date") val releaseDate: String?,
    val type: Int?
)

// TV content ratings
data class TmdbContentRatingsResponse(val results: List<TmdbContentRating>?)

data class TmdbContentRating(
    @SerializedName("iso_3166_1") val country: String,
    val rating: String?
)

// Collection
data class TmdbCollectionResponse(
    val id: Int?,
    val name: String?,
    val overview: String?,
    @SerializedName("poster_path") val posterPath: String?,
    @SerializedName("backdrop_path") val backdropPath: String?,
    val parts: List<TmdbMeta>?
)

interface ImdbApiService {
    @GET("titles/{titleId}/parentsGuide")
    suspend fun getParentsGuide(@Path("titleId") titleId: String): ParentsGuideResponse
}

data class ParentsGuideResponse(val parentsGuide: List<ParentsGuideCategory> = emptyList())
data class ParentsGuideCategory(
    val category: String = "",
    val severityBreakdowns: List<ParentsGuideSeverity> = emptyList(),
    val reviews: List<ParentsGuideReview> = emptyList()
) {
    val dominantSeverity: String?
        get() = severityBreakdowns.maxByOrNull { it.voteCount }?.severityLevel
}
data class ParentsGuideSeverity(val severityLevel: String = "", val voteCount: Int = 0)
data class ParentsGuideReview(val text: String = "", val isSpoiler: Boolean = false)

interface IntroDbService {
    @GET("segments")
    suspend fun getIntro(
        @Query("imdb_id") imdbId: String,
        @Query("season") season: Int,
        @Query("episode") episode: Int
    ): Response<JsonElement>

    @POST("submit")
    suspend fun submitSegment(
        @Header("X-API-Key") apiKey: String,
        @Body request: IntroDbSubmissionRequest
    ): Response<IntroDbSubmissionResponse>

    companion object {
        fun create(): IntroDbService {
            return Retrofit.Builder()
                .baseUrl("https://api.introdb.app/")
                .callFactory { request -> StremioService.sharedClient.newCall(request) }
                .addConverterFactory(GsonConverterFactory.create())
                .build()
                .create(IntroDbService::class.java)
        }
    }
}

data class IntroDbSubmissionRequest(
    @SerializedName("segment_type") val segmentType: String,
    @SerializedName("imdb_id") val imdbId: String,
    val season: Int,
    val episode: Int,
    @SerializedName("start_sec") val startSec: Double,
    @SerializedName("end_sec") val endSec: Double
)

data class IntroDbSubmissionResponse(
    val id: String? = null,
    val status: String? = null,
    val weight: Double? = null
)

interface AniSkipService {
    @GET("skip-times/{malId}/{episode}")
    suspend fun getSkipTimes(
        @Path("malId") malId: Int,
        @Path("episode") episode: Int,
        @Query("types") types: String = "op,ed,recap"
    ): Response<AniSkipResponse>

    companion object {
        fun create(): AniSkipService {
            return Retrofit.Builder()
                .baseUrl("https://api.aniskip.com/v2/")
                .callFactory { request -> StremioService.sharedClient.newCall(request) }
                .addConverterFactory(GsonConverterFactory.create())
                .build()
                .create(AniSkipService::class.java)
        }
    }
}

data class AniSkipResponse(
    @SerializedName("results") val results: List<AniSkipResult>?
)

data class AniSkipResult(
    @SerializedName("skip_type") val skipType: String,
    @SerializedName("interval") val interval: AniSkipInterval,
    @SerializedName("episode_length") val episodeLength: Double
)

data class AniSkipInterval(
    @SerializedName("start_time") val startTime: Double,
    @SerializedName("end_time") val endTime: Double
)
