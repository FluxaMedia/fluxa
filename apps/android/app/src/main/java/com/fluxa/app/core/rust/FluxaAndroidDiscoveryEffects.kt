@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.core.rust

import com.fluxa.app.BuildConfig
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.LibraryRemoteSource
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.TmdbMeta
import com.fluxa.app.data.remote.TmdbService
import com.fluxa.app.data.remote.ExternalSyncApi
import com.fluxa.app.core.rust.effects.fetchAddonCatalogPage
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.HomeCatalogSource
import com.fluxa.app.domain.discovery.buildDiscoverCatalogOptions
import com.fluxa.app.domain.discovery.buildDiscoverContentTypes
import com.fluxa.app.ui.catalog.DiscoverGenreOption
import com.fluxa.app.data.repository.TraktIntegration
import com.google.gson.JsonElement
import android.net.Uri
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit

internal suspend fun FluxaAndroidHeadlessEnvironment.runSearch(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val profile = effect.payload.profile()
    return ok(
        effect,
        mapOf(
            "results" to addonRepository.searchRows(
                query = effect.payload.string("query"),
                language = effect.payload.string("language", profile?.safeLanguage ?: "en"),
                authKey = profile?.authKey.orEmpty(),
                localAddons = profile?.safeLocalAddons.orEmpty()
            ).flatMap { it.items }
        )
    )
}

internal suspend fun FluxaAndroidHeadlessEnvironment.runDiscover(effect: NativeHeadlessEffect): HeadlessEffectCompletion = coroutineScope {
    val payload = effect.payload
    val profile = payload.profile()
    val contentType = payload.string("contentType")
    val filters = payload.objectValue("filters")
    val genre = filters?.stringOrNull("genre")
    val selectedCatalogKey = filters?.stringOrNull("catalogKey")
    val addons = addonRepository.getUserAddons(profile?.authKey.orEmpty(), profile?.safeLocalAddons.orEmpty())
    val catalogOptions = buildDiscoverCatalogOptions(addons, contentType)
    val catalogs = selectedCatalogKey
        ?.let { key -> catalogOptions.filter { it.key == key } }
        ?.takeIf { it.isNotEmpty() }
        ?: catalogOptions.take(1)
    // Fan out catalog fetches concurrently (was a sequential flatMap). awaitAll preserves
    // catalog order in the merged result; the semaphore caps concurrent addon requests.
    val semaphore = Semaphore(8)
    val fetched = catalogs.flatMap { catalog ->
        val selectedTypes = if (catalog.type == "all") listOf("movie", "series") else listOf(catalog.type)
        selectedTypes.map { type -> catalog to type }
    }.map { (catalog, type) ->
        async {
            semaphore.withPermit {
                runCatching {
                    val items = addonRepository.getAddonCatalog(
                        transportUrl = catalog.transportUrl,
                        type = type,
                        id = catalog.id,
                        genre = genre
                    )
                    val source = HomeCatalogSource(
                        transportUrl = catalog.transportUrl,
                        catalogId = catalog.id,
                        type = type,
                        genre = genre
                    )
                    items to source
                }.getOrDefault(emptyList<Meta>() to HomeCatalogSource(catalog.transportUrl, catalog.id, type, genre))
            }
        }
    }.awaitAll()
    val results = fetched.flatMap { it.first }
    val resultSources = linkedMapOf<String, HomeCatalogSource>()
    fetched.forEach { (items, source) ->
        items.forEach { item ->
            resultSources["${item.type}:${item.id}"] = source
            resultSources.putIfAbsent(item.id, source)
        }
    }
    ok(effect, mapOf("results" to results, "resultSources" to resultSources))
}

internal suspend fun FluxaAndroidHeadlessEnvironment.readDiscoverCatalogFilters(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val profile = payload.profile()
    val addons = addonRepository.getUserAddons(profile?.authKey.orEmpty(), profile?.safeLocalAddons.orEmpty())
    val catalogOptions = buildDiscoverCatalogOptions(addons, payload.string("contentType"))
    val contentTypes = buildDiscoverContentTypes(addons)
    val selectedCatalog = catalogOptions.firstOrNull { it.key == payload.stringOrNull("selectedCatalogKey") }
    val selectedGenres = selectedCatalog?.genres.orEmpty()
        .distinct()
        .sortedBy { it.lowercase(java.util.Locale.ROOT) }
        .map { DiscoverGenreOption(it, it) }
    val genres = if (selectedCatalog == null || selectedGenres.isEmpty()) {
        emptyList()
    } else if (!selectedCatalog.requiresGenre) {
        listOf(DiscoverGenreOption(null, AppStrings.t(payload.string("language", profile?.safeLanguage ?: "en"), "auto.all"))) + selectedGenres
    } else {
        selectedGenres
    }
    return ok(effect, mapOf("catalogs" to catalogOptions, "genres" to genres, "contentTypes" to contentTypes))
}

internal suspend fun FluxaAndroidHeadlessEnvironment.fetchCatalogPage(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val remoteSources = payload.remoteSources()
    if (remoteSources.isNotEmpty()) {
        return ok(
            effect,
            mapOf(
                "items" to fetchRemoteCollectionSources(
                    sources = remoteSources,
                    skip = payload.number("skip")?.toInt() ?: 0,
                    profile = payload.profile()
                )
            )
        )
    }
    return fetchAddonCatalogPage(effect, addonRepository)
}

internal suspend fun FluxaAndroidHeadlessEnvironment.fetchRemoteCollectionSources(
    sources: List<LibraryRemoteSource>,
    skip: Int,
    profile: UserProfile?
): List<Meta> = coroutineScope {
    sources.map { source ->
        async {
            when (source.provider.trim().lowercase()) {
                "trakt" -> fetchTraktCollectionSource(source, skip)
                "tmdb" -> fetchTmdbCollectionSource(source, skip, profile)
                else -> emptyList()
            }
        }
    }.awaitAll().flatten().distinctBy { "${it.type}:${it.id}" }
}

internal suspend fun FluxaAndroidHeadlessEnvironment.fetchTraktCollectionSource(source: LibraryRemoteSource, skip: Int): List<Meta> {
    if (!TraktIntegration.hasClient(BuildConfig.TRAKT_CLIENT_ID)) return emptyList()
    val listId = source.traktListId ?: return emptyList()
    val isSeries = source.mediaType.equals("series", ignoreCase = true) || source.mediaType.equals("show", ignoreCase = true) || source.mediaType.equals("tv", ignoreCase = true)
    return ExternalSyncApi.create().getListItems(
        listId = listId,
        type = if (isSeries) "show" else "movie",
        apiKey = BuildConfig.TRAKT_CLIENT_ID,
        page = (skip / 50) + 1,
        sortBy = source.sortBy,
        sortHow = source.sortHow
    ).mapNotNull { item ->
        val summary = (if (isSeries) item.show else item.movie) ?: return@mapNotNull null
        val id = summary.ids.imdb ?: summary.ids.tmdb?.let { "tmdb:$it" } ?: return@mapNotNull null
        Meta(
            id = id,
            name = summary.title ?: return@mapNotNull null,
            type = if (isSeries) "series" else "movie",
            poster = null,
            releaseInfo = summary.year?.toString(),
            runtime = summary.runtime?.toString()
        )
    }
}

internal suspend fun FluxaAndroidHeadlessEnvironment.fetchTmdbCollectionSource(source: LibraryRemoteSource, skip: Int, profile: UserProfile?): List<Meta> {
    val apiKey = profile?.safeTmdbApiKey.orEmpty()
    val sourceId = source.tmdbId ?: return emptyList()
    if (apiKey.isBlank()) return emptyList()
    val mediaType = if (source.mediaType.equals("series", true) || source.mediaType.equals("tv", true) || source.mediaType.equals("show", true)) "tv" else "movie"
    val sourceType = source.tmdbSourceType.orEmpty().uppercase()
    val path = when (sourceType) {
        "LIST" -> "list/$sourceId"
        "COLLECTION" -> "collection/$sourceId"
        "PERSON", "DIRECTOR" -> "person/$sourceId/combined_credits"
        "COMPANY" -> "discover/$mediaType"
        "NETWORK" -> "discover/tv"
        else -> "discover/$mediaType"
    }
    val url = Uri.parse("https://api.themoviedb.org/3/$path").buildUpon()
        .appendQueryParameter("api_key", apiKey)
        .appendQueryParameter("language", profile?.safeLanguage ?: "en")
        .apply {
            if (sourceType !in setOf("COLLECTION", "PERSON", "DIRECTOR")) {
                appendQueryParameter("page", (skip / 20 + 1).toString())
            }
            when (sourceType) {
                "COMPANY" -> appendQueryParameter("with_companies", sourceId.toString())
                "NETWORK" -> appendQueryParameter("with_networks", sourceId.toString())
            }
            if (sourceType !in setOf("LIST", "COLLECTION", "PERSON", "DIRECTOR")) {
                appendQueryParameter("sort_by", source.sortBy ?: "popularity.desc")
            }
            val filters = source.filters.orEmpty()
            mapOf(
                "year" to if (mediaType == "tv") "first_air_date_year" else "year",
                "withGenres" to "with_genres",
                "watchRegion" to "watch_region",
                "voteCountGte" to "vote_count.gte",
                "withKeywords" to "with_keywords",
                "withNetworks" to "with_networks",
                "withCompanies" to "with_companies",
                "releaseDateGte" to if (mediaType == "tv") "first_air_date.gte" else "primary_release_date.gte",
                "releaseDateLte" to if (mediaType == "tv") "first_air_date.lte" else "primary_release_date.lte",
                "voteAverageGte" to "vote_average.gte",
                "voteAverageLte" to "vote_average.lte",
                "withOriginCountry" to "with_origin_country",
                "withWatchProviders" to "with_watch_providers",
                "withOriginalLanguage" to "with_original_language"
            ).forEach { (input, output) -> filters[input]?.let { appendQueryParameter(output, it.toString()) } }
        }
        .build()
        .toString()
    val root = TmdbService.create().getCollectionSource(url)
    val items = root.asJsonObjectOrNull()?.let { objectNode ->
        when {
            sourceType == "DIRECTOR" -> objectNode.getAsJsonArrayOrNull("crew")?.filter { it.asJsonObjectOrNull()?.get("job")?.asString == "Director" && it.asJsonObjectOrNull()?.get("media_type")?.asString == mediaType }
            sourceType == "PERSON" -> objectNode.getAsJsonArrayOrNull("cast")?.filter { it.asJsonObjectOrNull()?.get("media_type")?.asString == mediaType }
            else -> listOf("results", "parts", "items", "cast", "crew").firstNotNullOfOrNull { key -> objectNode.getAsJsonArrayOrNull(key) }
        }
    } ?: emptyList<JsonElement>()
    return items.mapNotNull { item ->
        runCatching { gson.fromJson(item, TmdbMeta::class.java) }.getOrNull()?.toCollectionMeta(mediaType)
    }
}

internal suspend fun FluxaAndroidHeadlessEnvironment.fetchSeasonEpisodes(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val profile = payload.profile()
    val language = payload.string("language", profile?.safeLanguage ?: "en")
    val seriesId = payload.string("seriesId")
    val seasonNumber = payload.number("season")?.toInt() ?: 1
    val episodes = repository.getTvSeason(
        id = seriesId,
        seasonNumber = seasonNumber,
        language = language,
        authKey = profile?.authKey.orEmpty(),
        localAddons = profile?.safeLocalAddons.orEmpty(),
        useConfiguredAddons = true
    )
    val enriched = if (profile?.safeTmdbApiKey?.isNotBlank() == true && profile.safeTmdbEpisodeImagesEnabled) {
        val tmdbNumId = when {
            seriesId.startsWith("tmdb:", ignoreCase = true) ->
                seriesId.removePrefix("tmdb:").substringBefore(":").takeIf { it.toIntOrNull() != null }
            seriesId.substringBefore(":").toIntOrNull() != null ->
                seriesId.substringBefore(":")
            else -> null
        }
        if (tmdbNumId != null) {
            repository.enrichSeasonEpisodesWithTmdb(tmdbNumId, seasonNumber, episodes, profile.safeTmdbApiKey, language)
        } else {
            episodes
        }
    } else {
        episodes
    }
    return ok(effect, mapOf("episodes" to enriched))
}

internal fun FluxaAndroidHeadlessEnvironment.fetchSubtitles(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val stream = effect.payload.objectValue("stream")?.let { gson.fromJson(gson.toJsonTree(it), Stream::class.java) }
    return ok(effect, mapOf("subtitles" to stream?.subtitles.orEmpty()))
}

private fun TmdbMeta.toCollectionMeta(defaultMediaType: String): Meta? {
    val type = if (media_type == "tv" || defaultMediaType == "tv") "series" else "movie"
    val title = if (type == "series") name else title
    return title?.let {
        Meta(
            id = "tmdb:$id",
            name = it,
            type = type,
            poster = posterPath?.let { path -> "https://image.tmdb.org/t/p/w500$path" },
            background = backdropPath?.let { path -> "https://image.tmdb.org/t/p/w1280$path" },
            description = overview,
            releaseInfo = (if (type == "series") first_air_date else release_date)?.take(4),
            originalName = original_name
        )
    }
}
