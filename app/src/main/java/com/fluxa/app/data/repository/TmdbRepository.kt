package com.fluxa.app.data.repository

import com.fluxa.app.common.Constants
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.CastMember
import com.fluxa.app.data.remote.DetailTrailer
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.MetaRating
import com.fluxa.app.data.remote.TmdbService
import com.fluxa.app.data.remote.TmdbVideo
import com.fluxa.app.data.remote.Video
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class TmdbRepository @Inject constructor(
    private val tmdbService: TmdbService
) {
    suspend fun getRecommendations(type: String, id: String, language: String = "en"): List<Meta> = withContext(Dispatchers.IO) {
        tmdbService.getRecommendations(type, id, language).results.map { it.toMeta(type) }
    }

    suspend fun getSimilar(type: String, id: String, language: String = "en"): List<Meta> = withContext(Dispatchers.IO) {
        tmdbService.getSimilar(type, id, language).results.map { it.toMeta(type) }
    }

    suspend fun findTmdbId(type: String, imdbId: String): String? = withContext(Dispatchers.IO) {
        val response = tmdbService.findById(imdbId)
        val result = if (type == "movie") response.movieResults.firstOrNull() else response.tvResults.firstOrNull()
        result?.id?.toString()
    }

    suspend fun getTrailers(type: String, id: String, language: String = "en", apiKey: String): List<DetailTrailer> = withContext(Dispatchers.IO) {
        if (apiKey.isBlank()) return@withContext emptyList()
        try {
            val tmdbType = if (type == "series") "tv" else type
            val tmdbId = resolveTmdbId(type, id, language, apiKey) ?: return@withContext emptyList()
            tmdbService.getVideos(tmdbType, tmdbId, language, apiKey).results
                .mapNotNull { it.toTrailer() }
        } catch (e: Exception) {
            emptyList()
        }
    }

    suspend fun enrichDetail(
        detail: MetaDetail,
        apiKey: String,
        profile: UserProfile,
        language: String
    ): MetaDetail = withContext(Dispatchers.IO) {
        if (apiKey.isBlank()) return@withContext detail
        try {
            val tmdbId = resolveTmdbId(detail.type, detail.id, language, apiKey) ?: return@withContext detail
            val tmdbType = if (detail.type == "series") "tv" else "movie"
            coroutineScope {
                val needsDetail = profile.safeTmdbBasicInfoEnabled ||
                    profile.safeTmdbDetailsEnabled ||
                    profile.safeTmdbProductionsEnabled ||
                    profile.safeTmdbNetworksEnabled ||
                    profile.safeTmdbCollectionInfoEnabled ||
                    profile.safeTmdbRatingsEnabled ||
                    profile.safeTmdbCastImagesEnabled

                val detailDeferred = if (needsDetail) {
                    async { runCatching { tmdbService.getDetail(tmdbType, tmdbId, language, apiKey) }.getOrNull() }
                } else null

                val creditsDeferred = if (profile.safeTmdbCastImagesEnabled) {
                    async { runCatching { tmdbService.getCredits(tmdbType, tmdbId, language, apiKey) }.getOrNull() }
                } else null

                val imagesDeferred = if (profile.safeTmdbLogosBackdropsEnabled) {
                    async { runCatching { tmdbService.getImages(tmdbType, tmdbId, apiKey = apiKey) }.getOrNull() }
                } else null

                val certDeferred = if (profile.safeTmdbRatingsEnabled) {
                    if (tmdbType == "movie") async {
                        runCatching {
                            val results = tmdbService.getMovieReleaseDates(tmdbId, apiKey).results ?: return@runCatching null
                            results.firstOrNull { it.country == "US" }?.releaseDates?.mapNotNull { it.certification }?.firstOrNull { it.isNotBlank() }
                                ?: results.firstOrNull { it.country == "GB" }?.releaseDates?.mapNotNull { it.certification }?.firstOrNull { it.isNotBlank() }
                        }.getOrNull()
                    } else async {
                        runCatching {
                            val results = tmdbService.getTvContentRatings(tmdbId, apiKey).results ?: return@runCatching null
                            results.firstOrNull { it.country == "US" }?.rating?.takeIf { it.isNotBlank() }
                                ?: results.firstOrNull { it.country == "GB" }?.rating?.takeIf { it.isNotBlank() }
                        }.getOrNull()
                    }
                } else null

                val tmdbDetail = detailDeferred?.await()
                val credits = creditsDeferred?.await()
                val images = imagesDeferred?.await()
                val certification = certDeferred?.await()

                // Collection fetch deferred (needs collectionRef from tmdbDetail, so sequential)
                val collectionRef = tmdbDetail?.belongsToCollection
                val collection = if (profile.safeTmdbCollectionInfoEnabled && collectionRef != null) {
                    runCatching { tmdbService.getCollection(collectionRef.id, language, apiKey) }.getOrNull()
                } else null

                // Season posters from tmdbDetail.seasons (TV only)
                val seasonPosters: Map<String, String>? = if (tmdbType == "tv") {
                    tmdbDetail?.seasons
                        ?.filter { it.seasonNumber > 0 && !it.posterPath.isNullOrBlank() }
                        ?.associate { it.seasonNumber.toString() to tmdbImageUrl(it.posterPath!!, "w342") }
                        ?.takeIf { it.isNotEmpty() }
                } else null

                var result = detail

                if (profile.safeTmdbBasicInfoEnabled && tmdbDetail != null) {
                    if (!tmdbDetail.overview.isNullOrBlank())
                        result = result.copy(description = tmdbDetail.overview)
                    if (!tmdbDetail.genres.isNullOrEmpty())
                        result = result.copy(genres = tmdbDetail.genres.map { it.name })
                    if ((tmdbDetail.voteAverage ?: 0.0) > 0)
                        result = result.copy(imdbRating = String.format("%.1f", tmdbDetail.voteAverage))
                }

                if (profile.safeTmdbDetailsEnabled && tmdbDetail != null) {
                    val runtimeMin = tmdbDetail.runtime ?: tmdbDetail.episodeRunTime?.firstOrNull()
                    if (runtimeMin != null && runtimeMin > 0)
                        result = result.copy(runtime = "$runtimeMin min")
                    if (!tmdbDetail.status.isNullOrBlank())
                        result = result.copy(status = tmdbDetail.status)
                    tmdbDetail.originCountry?.firstOrNull()?.takeIf { it.isNotBlank() }
                        ?.let { result = result.copy(country = it) }
                    if (!tmdbDetail.originalLanguage.isNullOrBlank())
                        result = result.copy(originalLanguage = tmdbDetail.originalLanguage)
                }

                if (profile.safeTmdbProductionsEnabled && tmdbDetail != null) {
                    if (!tmdbDetail.productionCompanies.isNullOrEmpty())
                        result = result.copy(productionCompanies = tmdbDetail.productionCompanies.map { it.name })
                }

                if (profile.safeTmdbNetworksEnabled && tmdbDetail != null && tmdbType == "tv") {
                    if (!tmdbDetail.networks.isNullOrEmpty())
                        result = result.copy(networks = tmdbDetail.networks.map { it.name })
                }

                if (profile.safeTmdbCastImagesEnabled && credits != null) {
                    val castList = credits.cast?.sortedBy { it.order }?.take(20)?.map { member ->
                        CastMember(
                            name = member.name,
                            character = member.character,
                            profilePath = member.profilePath?.let { tmdbImageUrl(it, "w185") }
                        )
                    }
                    if (!castList.isNullOrEmpty() && result.cast.isNullOrEmpty())
                        result = result.copy(cast = castList)
                    val directors = credits.crew
                        ?.filter { it.job.equals("Director", ignoreCase = true) }
                        ?.map { it.name }
                        ?.takeIf { it.isNotEmpty() }
                    val creators = tmdbDetail?.createdBy?.map { it.name }?.takeIf { it.isNotEmpty() }
                    (directors ?: creators)?.let { result = result.copy(director = it) }
                }

                if (profile.safeTmdbLogosBackdropsEnabled && images != null) {
                    val logo = images.logos
                        ?.filter { it.language == "en" || it.language.isNullOrBlank() }
                        ?.maxByOrNull { it.voteAverage ?: 0.0 }
                        ?: images.logos?.maxByOrNull { it.voteAverage ?: 0.0 }
                    logo?.let { result = result.copy(logo = tmdbImageUrl(it.filePath, "w500")) }
                    images.backdrops?.maxByOrNull { it.voteAverage ?: 0.0 }
                        ?.let { result = result.copy(background = tmdbImageUrl(it.filePath, "w1280")) }
                }

                if (profile.safeTmdbRatingsEnabled) {
                    if (!certification.isNullOrBlank())
                        result = result.copy(ageRating = certification)
                    if ((tmdbDetail?.voteAverage ?: 0.0) > 0) {
                        val tmdbRating = MetaRating("TMDB", String.format("%.1f", tmdbDetail!!.voteAverage))
                        val existing = result.ratings?.filter { it.source != "TMDB" }.orEmpty()
                        result = result.copy(ratings = existing + tmdbRating)
                    }
                }

                if (profile.safeTmdbCollectionInfoEnabled && collection != null) {
                    val parts = collection.parts?.sortedBy { it.release_date }?.map { it.toMeta("movie") }
                    result = result.copy(
                        collectionName = collection.name,
                        collectionId = collection.id,
                        collectionParts = parts?.takeIf { it.isNotEmpty() }
                    )
                }

                if (seasonPosters != null) result = result.copy(seasonPosters = seasonPosters)

                result
            }
        } catch (e: Exception) {
            detail
        }
    }

    suspend fun enrichSeasonEpisodes(
        tmdbId: String,
        seasonNumber: Int,
        episodes: List<Video>,
        apiKey: String,
        language: String
    ): List<Video> = withContext(Dispatchers.IO) {
        if (apiKey.isBlank()) return@withContext episodes
        try {
            val seasonDetail = tmdbService.getSeasonDetail(tmdbId, seasonNumber, language, apiKey)
            val tmdbEpisodes = seasonDetail.episodes ?: return@withContext episodes
            episodes.map { video ->
                val episodeNum = video.number ?: return@map video
                val tmdb = tmdbEpisodes.firstOrNull { it.episodeNumber == episodeNum } ?: return@map video
                video.copy(
                    thumbnail = video.thumbnail ?: tmdb.stillPath?.let { tmdbImageUrl(it, "w300") },
                    overview = video.overview ?: tmdb.overview?.takeIf { it.isNotBlank() },
                    episodeRuntime = video.episodeRuntime ?: tmdb.runtime
                )
            }
        } catch (e: Exception) {
            episodes
        }
    }

    private suspend fun resolveTmdbId(type: String, id: String, language: String, apiKey: String): String? {
        val baseId = if (id.startsWith("tmdb:", ignoreCase = true)) {
            id.removePrefix("tmdb:").substringBefore(":")
        } else {
            id.substringBefore(":")
        }
        if (baseId.toIntOrNull() != null) return baseId
        val imdbId = id.substringBefore(":").takeIf { it.startsWith("tt") } ?: return null
        val response = tmdbService.findById(imdbId, lang = language, apiKey = apiKey)
        return if (type == "movie") response.movieResults.firstOrNull()?.id?.toString() else response.tvResults.firstOrNull()?.id?.toString()
    }

    private fun TmdbVideo.toTrailer(): DetailTrailer? {
        if (!site.equals("YouTube", ignoreCase = true)) return null
        val videoKey = key?.takeIf { it.isNotBlank() } ?: return null
        val videoType = type?.takeIf { it.isNotBlank() } ?: return null
        if (!videoType.equals("Trailer", ignoreCase = true) && !videoType.equals("Teaser", ignoreCase = true) && !videoType.equals("Clip", ignoreCase = true)) return null
        return DetailTrailer(
            id = id ?: videoKey,
            title = name?.takeIf { it.isNotBlank() } ?: videoType,
            type = videoType,
            url = "https://www.youtube.com/watch?v=$videoKey",
            thumbnail = "https://i.ytimg.com/vi/$videoKey/hqdefault.jpg",
            source = "tmdb"
        )
    }

    private fun com.fluxa.app.data.remote.TmdbMeta.toMeta(type: String): Meta {
        return Meta(
            id = "tmdb:$id",
            name = title ?: name ?: "",
            type = type,
            poster = null,
            releaseInfo = release_date ?: first_air_date
        )
    }

    private fun tmdbImageUrl(path: String, size: String): String =
        "${Constants.Images.TMDB_BASE_URL}$size$path"
}
