@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.core.rust

import com.fluxa.app.BuildConfig
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.LibraryRemoteSource
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.distinctByTypeAndId
import com.fluxa.app.data.remote.TmdbMeta
import com.fluxa.app.data.remote.TmdbService
import com.fluxa.app.data.remote.ExternalSyncApi
import com.fluxa.app.core.rust.effects.fetchAddonCatalogPage
import com.fluxa.app.ui.catalog.HomeCatalogSource
import com.fluxa.app.data.repository.TraktIntegration
import com.google.gson.JsonElement
import com.google.gson.reflect.TypeToken
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope

internal suspend fun FluxaAndroidHeadlessEnvironment.runSearch(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val profile = effect.payload.profile()
    val query = effect.payload.string("query").trim()
    val rows = addonRepository.searchRows(
        query = query,
        language = effect.payload.string("language", profile?.safeLanguage ?: "en"),
        authKey = profile?.authKey.orEmpty(),
        localAddons = profile?.safeLocalAddons.orEmpty()
    )
    val sources = rows.map { row ->
        val items = row.items.map { meta ->
            gson.toJsonTree(meta).asJsonObject.apply {
                row.sourceAddonTransportUrl?.let { addProperty("sourceAddonTransportUrl", it) }
                row.sourceAddonCatalogType?.let { addProperty("sourceAddonCatalogType", it) }
            }
        }
        mapOf(
            "id" to row.id,
            "name" to row.title,
            "semanticName" to row.title,
            "type" to row.type,
            "items" to items
        )
    }
    val merged = FluxaCoreUniFfi.coreInvokeValue("mergeSearchSources", gson.toJson(sources)).asJsonObject
    val results = merged.get("results")
    val categories = merged.get("categories")
    val grouping = FluxaCoreUniFfi.coreInvokeValue(
        "searchResultGrouping",
        gson.toJson(mapOf("query" to query, "results" to results))
    )
    return ok(
        effect,
        mapOf(
            "results" to results,
            "categories" to categories,
            "grouping" to grouping
        )
    )
}

internal suspend fun FluxaAndroidHeadlessEnvironment.runDiscover(effect: NativeHeadlessEffect): HeadlessEffectCompletion = coroutineScope {
    val payload = effect.payload
    val contentType = payload.string("contentType")
    val requestPlan = FluxaCoreUniFfi.coreInvokeValue(
        "discoverSourceRequests",
        gson.toJson(mapOf("contentType" to contentType, "filters" to payload.objectValue("filters")))
    ).asJsonArray
    if (requestPlan.isEmpty) return@coroutineScope error(effect, "discover_plan_has_no_requests")
    // Core owns request-type expansion, field aliases, and provenance for all hosts.
    val fetched = requestPlan.map { requestElement ->
        val request = requestElement.asJsonObject
        val transportUrl = request.get("transportUrl")?.asString.orEmpty()
        val type = request.get("type")?.asString.orEmpty()
        val catalogId = request.get("catalogId")?.asString.orEmpty()
        val extra = request.getAsJsonObject("extra")
        val genre = request.get("genre")?.takeUnless { it.isJsonNull }?.asString
        val search = extra?.get("search")?.takeUnless { it.isJsonNull }?.asString.orEmpty().trim()
        async {
            runCatching {
                val items = addonRepository.getAddonCatalog(
                    transportUrl = transportUrl,
                    type = type,
                    id = catalogId,
                    skip = extra?.get("skip")?.takeUnless { it.isJsonNull }?.asInt ?: 0,
                    genre = genre,
                    search = search.takeIf { it.isNotBlank() }
                )
                val source = HomeCatalogSource(
                    transportUrl = transportUrl,
                    catalogId = catalogId,
                    type = type,
                    genre = genre
                )
                items to source
            }
                .getOrDefault(emptyList<Meta>() to HomeCatalogSource(transportUrl, catalogId, type, genre))
        }
    }.awaitAll()
    val merged = FluxaCoreNative.mergeDiscoverSources(fetched.map { (items, source) ->
        mapOf(
            "transportUrl" to source.transportUrl,
            "catalogId" to source.catalogId,
            "type" to source.type,
            "genre" to source.genre,
            "items" to items
        )
    }).asJsonObject
    val results: List<Meta> = gson.fromJson(
        merged.get("results"),
        object : TypeToken<List<Meta>>() {}.type
    ) ?: emptyList()
    val resultSources: Map<String, HomeCatalogSource> = gson.fromJson(
        merged.get("resultSources"),
        object : TypeToken<Map<String, HomeCatalogSource>>() {}.type
    ) ?: emptyMap()
    return@coroutineScope ok(effect, mapOf("results" to results, "resultSources" to resultSources))
}

internal suspend fun FluxaAndroidHeadlessEnvironment.readDiscoverCatalogFilters(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val profile = payload.profile()
    val addons = configuredStreamAddons(profile)
    return ok(effect, mapOf("addons" to addons))
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
    }.awaitAll().flatten().distinctByTypeAndId()
}

internal suspend fun FluxaAndroidHeadlessEnvironment.fetchTraktCollectionSource(source: LibraryRemoteSource, skip: Int): List<Meta> {
    if (!TraktIntegration.hasClient(BuildConfig.TRAKT_CLIENT_ID)) return emptyList()
    val listId = source.traktListId ?: return emptyList()
    val isSeries = FluxaCoreNative.isSeriesContentType(source.mediaType)
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
    val mediaType = if (FluxaCoreNative.isSeriesContentType(source.mediaType)) "tv" else "movie"
    val sourceType = source.tmdbSourceType.orEmpty().uppercase()
    val filtersJson = source.filters?.let { gson.toJson(it) }
    val url = FluxaCoreNative.tmdbCollectionSourceUrl(
        sourceType = sourceType,
        sourceId = sourceId,
        mediaType = source.mediaType,
        skip = skip,
        sortBy = source.sortBy,
        filtersJson = filtersJson,
        apiKey = apiKey,
        language = profile?.safeLanguage ?: "en",
    ) ?: return emptyList()
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
        val tmdbNumId = FluxaCoreNative.tmdbNumericId(seriesId)
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
    val streamJson = gson.toJson(effect.payload["stream"] ?: emptyMap<String, Any?>())
    val result = FluxaCoreUniFfi.coreInvokeValue("streamSubtitlesResult", streamJson)
    return ok(effect, result)
}

private fun TmdbMeta.toCollectionMeta(defaultMediaType: String): Meta? {
    val type = FluxaCoreNative.tmdbItemContentType(media_type, first_air_date != null, defaultMediaType)
    val title = if (FluxaCoreNative.isSeriesContentType(type)) name else title
    return title?.let {
        Meta(
            id = "tmdb:$id",
            name = it,
            type = type,
            poster = FluxaCoreNative.tmdbImageUrl(posterPath, "w500"),
            background = FluxaCoreNative.tmdbImageUrl(backdropPath, "w1280"),
            description = overview,
            releaseInfo = (if (FluxaCoreNative.isSeriesContentType(type)) first_air_date else release_date)?.take(4),
            originalName = original_name
        )
    }
}
