package com.fluxa.app.data.repository

import android.util.Log
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.models.NativeAddonFetchResult
import okhttp3.Request
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.AuthRequest
import com.fluxa.app.data.remote.CastMember
import com.fluxa.app.data.remote.CastMemberDeserializer
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.MetaDetailResponse
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.StreamResponse
import com.fluxa.app.data.remote.StremioService
import com.fluxa.app.data.remote.SubtitleData
import com.fluxa.app.data.remote.SubtitleResponse
import com.fluxa.app.domain.discovery.StremioAddonUrls
import com.fluxa.app.data.remote.Video
import com.google.gson.JsonParseException
import com.google.gson.GsonBuilder
import com.google.gson.reflect.TypeToken
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import okhttp3.OkHttpClient
import java.io.IOException
import java.lang.reflect.Type
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.CopyOnWriteArrayList

import javax.inject.Inject
import javax.inject.Named

class StremioAddonResourceClient @Inject constructor(
    private val authService: StremioService,
    private val cache: RepositoryMemoryCache,
    private val persistentCache: AddonPersistentCache,
    private val addonManifestClient: StremioAddonManifestClient,
    @param:Named("AddonResourceClient") private val httpClient: OkHttpClient
) {
    private val stremioGson = GsonBuilder()
        .registerTypeAdapter(CastMember::class.java, CastMemberDeserializer())
        .create()
    private val streamListType = object : TypeToken<List<Stream>>() {}.type
    private val metaListType = object : TypeToken<List<Meta>>() {}.type
    private val subtitleListType = object : TypeToken<List<SubtitleData>>() {}.type
    private val userAddonsLocks = ConcurrentHashMap<String, Mutex>()

    suspend fun getUserAddons(
        authKey: String,
        localAddons: List<String>? = emptyList(),
        forceRefresh: Boolean = false
    ): List<AddonDescriptor> = withContext(Dispatchers.IO) {
        val normalizedLocalAddons = localAddons.orEmpty()
            .map(addonManifestClient::normalizeAddonTransportUrl)
            .filter { it.isNotBlank() }
            .distinctBy(StremioAddonUrls::identity)
        val cacheKey = "addons_v9_${authKey}_${normalizedLocalAddons.joinToString("|")}"
        if (!forceRefresh) {
            cache.get<List<AddonDescriptor>>(cacheKey)?.let { return@withContext it }
            persistentCache.getUserAddons(cacheKey).takeIf { it.isNotEmpty() }?.let {
                cache.put(cacheKey, it)
                return@withContext it
            }
        }
        userAddonsLocks.getOrPut(cacheKey) { Mutex() }.withLock {
            if (!forceRefresh) {
                cache.get<List<AddonDescriptor>>(cacheKey)?.let { return@withLock it }
            }
            val allAddons = CopyOnWriteArrayList<AddonDescriptor>()
            fun addAddon(addon: AddonDescriptor) {
                val normalized = addonManifestClient.normalizeAddonTransportUrl(addon.transportUrl)
                if (allAddons.none { StremioAddonUrls.identity(it.transportUrl) == StremioAddonUrls.identity(normalized) }) {
                    allAddons.add(addon.copy(transportUrl = normalized))
                }
            }
            if (authKey.isNotEmpty()) {
                try {
                    val response = authService.getAddons(AuthRequest(authKey))
                    response.body()?.result?.addons.orEmpty().map { addon ->
                        async {
                            val liveManifest = withTimeoutOrNull(3000) {
                                addonManifestClient.getAddonManifest(addon.transportUrl, forceRefresh)
                            }
                            addAddon(with(addonManifestClient) { addon.mergeLiveManifest(liveManifest) })
                        }
                    }.awaitAll()
                } catch (e: Exception) {
                    Log.w("StremioRepository", "Failed to load user addons", e)
                }
            }
            normalizedLocalAddons.map { url ->
                async {
                    try {
                        withTimeoutOrNull(3000) {
                            addonManifestClient.getAddonManifest(url, forceRefresh)
                        }?.let(::addAddon)
                    } catch (e: Exception) {
                        Log.w("StremioRepository", "Failed to load local addon manifest: $url", e)
                    }
                }
            }.awaitAll()
            allAddons.toList().ifEmpty {
                persistentCache.getUserAddons(cacheKey)
            }.also {
                cache.put(cacheKey, it)
                persistentCache.putUserAddons(cacheKey, it)
            }
        }
    }

    suspend fun getSubtitlesFromAddon(
        baseUrl: String,
        type: String,
        id: String,
        extra: String = ""
    ): List<SubtitleData> = resultOrEmpty(
        operation = "subtitles",
        result = getSubtitlesFromAddonResult(baseUrl, type, id, extra)
    )

    suspend fun getSubtitlesFromAddonResult(
        baseUrl: String,
        type: String,
        id: String,
        extra: String = ""
    ): AddonResourceResult<List<SubtitleData>> = withContext(Dispatchers.IO) {
        val extraArgs = FluxaCoreNative.parseExtraArgs(extra)
        val parsed = fetchAddonResourcePayload(
            transportUrl = baseUrl,
            resource = "subtitles",
            type = type,
            id = id,
            extraArgs = extraArgs,
            extraRaw = extra,
            fallbackUrl = addonManifestClient.buildAddonResourceUrl(baseUrl, "subtitles", type, id)
        )
        val success = parsed as? AddonResourceResult.Success ?: return@withContext parsed.toTypedEmpty()
        decodeResourceList(
            result = success,
            type = subtitleListType,
            transformJson = { valueJson, url -> FluxaCoreNative.normalizeAddonSubtitles(valueJson, url) }
        )
    }

    suspend fun getStreamsFromAddon(
        addonTransportUrl: String,
        addonName: String,
        type: String,
        id: String
    ): List<Stream> = resultOrEmpty(
        operation = "stream",
        result = getStreamsFromAddonResult(addonTransportUrl, addonName, type, id)
    )

    suspend fun getStreamsFromAddonResult(
        addonTransportUrl: String,
        addonName: String,
        type: String,
        id: String
    ): AddonResourceResult<List<Stream>> = withContext(Dispatchers.IO) {
        val parsed = fetchAddonResourcePayload(
            transportUrl = addonTransportUrl,
            resource = "stream",
            type = type,
            id = id,
            fallbackUrl = addonManifestClient.buildAddonResourceUrl(addonTransportUrl, "stream", type, id)
        )
        val success = parsed as? AddonResourceResult.Success ?: return@withContext parsed.toTypedEmpty()
        decodeResourceList(
            result = success,
            type = streamListType,
            transformJson = { valueJson, _ -> FluxaCoreNative.addonStreamsWithProvider(valueJson, addonName) }
        )
    }

    suspend fun getMetaDetailFromAddon(
        addonTransportUrl: String,
        type: String,
        id: String
    ): MetaDetail? = when (val result = getMetaDetailFromAddonResult(addonTransportUrl, type, id)) {
        is AddonResourceResult.Success -> result.value
        is AddonResourceResult.Empty,
        is AddonResourceResult.AddonUnsupported -> null
        is AddonResourceResult.NetworkError -> {
            Log.w("StremioAddonResourceClient", "Meta request failed: ${result.url} status=${result.statusCode}", result.cause)
            null
        }
        is AddonResourceResult.ParseError -> {
            Log.w("StremioAddonResourceClient", "Meta parse failed: ${result.url}", result.cause)
            null
        }
    }

    suspend fun getMetaDetailFromAddonResult(
        addonTransportUrl: String,
        type: String,
        id: String
    ): AddonResourceResult<MetaDetail> = withContext(Dispatchers.IO) {
        val parsed = fetchAddonResourcePayload(
            transportUrl = addonTransportUrl,
            resource = "meta",
            type = type,
            id = id,
            fallbackUrl = addonManifestClient.buildAddonResourceUrl(addonTransportUrl, "meta", type, id)
        )
        val success = parsed as? AddonResourceResult.Success ?: return@withContext parsed.toTypedEmpty()
        decodeMetaDetail(success)
    }

    suspend fun getAddonCatalog(
        transportUrl: String,
        type: String,
        id: String,
        skip: Int = 0,
        genre: String? = null,
        search: String? = null
    ): List<Meta> = resultOrEmpty(
        operation = "catalog",
        result = getAddonCatalogResult(transportUrl, type, id, skip, genre, search)
    )

    suspend fun getAddonCatalogResult(
        transportUrl: String,
        type: String,
        id: String,
        skip: Int = 0,
        genre: String? = null,
        search: String? = null
    ): AddonResourceResult<List<Meta>> = withContext(Dispatchers.IO) {
        val extraArgs = mapOf(
            "search" to search?.takeIf { it.isNotBlank() },
            "genre" to genre?.takeIf { it.isNotBlank() },
            "skip" to skip.takeIf { it > 0 }?.toString()
        )
        val cacheKey = "catalog_v1_${transportUrl}_${type}_${id}_${skip}_${genre.orEmpty()}_${search.orEmpty()}"
        cache.get<List<Meta>>(cacheKey)?.let { return@withContext AddonResourceResult.Success(it, cacheKey) }
        val parsed = fetchAddonResourcePayload(
            transportUrl = transportUrl,
            resource = "catalog",
            type = type,
            id = id,
            extraArgs = extraArgs,
            fallbackUrl = addonManifestClient.buildAddonResourceUrl(transportUrl, "catalog", type, id, extraArgs)
        )
        val success = parsed as? AddonResourceResult.Success ?: return@withContext parsed.toTypedEmpty()
        val result = decodeResourceList<Meta>(result = success, type = metaListType)
        if (result is AddonResourceResult.Success) {
            cache.put(cacheKey, result.value)
        }
        result
    }

    private fun <T> decodeResourceList(
        result: AddonResourceResult.Success<String>,
        type: Type,
        transformJson: (String, String) -> String = { valueJson, _ -> valueJson }
    ): AddonResourceResult<List<T>> {
        try {
            val items = stremioGson.fromJson<List<T>>(transformJson(result.value, result.url), type).orEmpty()
            return if (items.isEmpty()) {
                AddonResourceResult.Empty(result.url)
            } else {
                AddonResourceResult.Success(items, result.url)
            }
        } catch (e: JsonParseException) {
            return AddonResourceResult.ParseError(result.url, e)
        } catch (e: IOException) {
            return AddonResourceResult.NetworkError(result.url, e)
        } catch (e: Exception) {
            return AddonResourceResult.NetworkError(result.url, e)
        }
    }

    private fun decodeMetaDetail(result: AddonResourceResult.Success<String>): AddonResourceResult<MetaDetail> {
        return try {
            decodeMetaDetailPayload(result.value)
                ?.applyAppExtras()
                ?.applyLinks()
                ?.let { AddonResourceResult.Success(it, result.url) }
                ?: AddonResourceResult.Empty(result.url)
        } catch (e: JsonParseException) {
            AddonResourceResult.ParseError(result.url, e)
        } catch (e: IOException) {
            AddonResourceResult.NetworkError(result.url, e)
        } catch (e: Exception) {
            AddonResourceResult.NetworkError(result.url, e)
        }
    }

    private fun decodeMetaDetailPayload(json: String): MetaDetail? {
        return stremioGson.fromJson(json, MetaDetailResponse::class.java)?.meta
            ?: stremioGson.fromJson(json, MetaDetail::class.java)
    }

    private fun MetaDetail.applyAppExtras(): MetaDetail {
        val extras = appExtras ?: return this
        var updated = this
        val extrasSeasonPosters = extras.seasonPosters
            ?.takeIf { it.isNotEmpty() }
            ?.let { posters -> mapAppExtraSeasonPosters(posters) }
        if (extrasSeasonPosters != null && seasonPosters == null) {
            updated = updated.copy(seasonPosters = extrasSeasonPosters)
        }
        val cert = extras.certificationLocal?.takeIf { it.isNotBlank() }
            ?: extras.certification?.takeIf { it.isNotBlank() }
        if (!cert.isNullOrBlank() && ageRating.isNullOrBlank()) {
            updated = updated.copy(ageRating = cert)
        }
        extras.cast?.takeIf { it.isNotEmpty() }?.let { extrasCast ->
            if (cast.isNullOrEmpty()) {
                updated = updated.copy(cast = extrasCast)
            }
        }
        return updated
    }

    private fun MetaDetail.mapAppExtraSeasonPosters(posters: List<String?>): Map<String, String> =
        mapAppExtraSeasonPosters(videos.orEmpty(), seasonsCount, posters)

    private fun MetaDetail.applyLinks(): MetaDetail {
        val linkList = links?.takeIf { it.isNotEmpty() } ?: return this
        var updated = this
        if (imdbRating.isNullOrBlank()) {
            linkList.firstOrNull { it.category.equals("imdb", ignoreCase = true) }
                ?.name?.trim()?.takeIf { it.isNotBlank() && it.toDoubleOrNull().let { v -> v != null && v > 0.0 } }
                ?.let { updated = updated.copy(imdbRating = it) }
        }
        if (genres.isNullOrEmpty()) {
            val genreNames = linkList
                .filter { it.category.equals("Genres", ignoreCase = true) }
                .mapNotNull { it.name.trim().takeIf { n -> n.isNotBlank() } }
            if (genreNames.isNotEmpty()) updated = updated.copy(genres = genreNames)
        }
        return updated
    }

    private fun <T> resultOrEmpty(operation: String, result: AddonResourceResult<List<T>>): List<T> {
        return when (result) {
            is AddonResourceResult.Success -> result.value
            is AddonResourceResult.Empty,
            is AddonResourceResult.AddonUnsupported -> emptyList()
            is AddonResourceResult.NetworkError -> {
                Log.w("StremioAddonResourceClient", "$operation request failed: ${result.url} status=${result.statusCode}", result.cause)
                emptyList()
            }
            is AddonResourceResult.ParseError -> {
                Log.w("StremioAddonResourceClient", "$operation parse failed: ${result.url}", result.cause)
                emptyList()
            }
        }
    }

    private fun fetchAddonBodyResult(url: String): NativeAddonFetchResult {
        return try {
            val httpRequest = Request.Builder().url(url).build()
            val httpResponse = httpClient.newCall(httpRequest).execute()
            val status = httpResponse.code
            val body = httpResponse.body.string().also { httpResponse.close() }
            NativeAddonFetchResult(url = url, statusCode = status, body = body)
        } catch (e: Exception) {
            Log.w("StremioAddonResourceClient", "Addon resource HTTP failed: $url", e)
            NativeAddonFetchResult(url = url, error = e.message)
        }
    }

    private fun fetchAddonResourcePayload(
        transportUrl: String,
        resource: String,
        type: String,
        id: String,
        extraArgs: Map<String, String?> = emptyMap(),
        extraRaw: String = "",
        fallbackUrl: String
    ): AddonResourceResult<String> {
        val candidates = FluxaCoreNative.addonResourceRequestPlan(
            transportUrl = transportUrl,
            resource = resource,
            type = type,
            id = id,
            extraArgs = extraArgs,
            extraRaw = extraRaw
        ).urls.ifEmpty { listOf(fallbackUrl) }
        var lastEmpty: AddonResourceResult.Empty? = null
        for (url in candidates) {
            when (val parsed = fetchAndParseAddonResource(resource, url)) {
                is AddonResourceResult.Success -> return parsed
                is AddonResourceResult.Empty -> lastEmpty = parsed
                is AddonResourceResult.NetworkError -> return parsed
                is AddonResourceResult.ParseError -> return parsed
                is AddonResourceResult.AddonUnsupported -> return parsed
            }
        }
        return lastEmpty ?: AddonResourceResult.Empty(fallbackUrl)
    }

    private fun fetchAndParseAddonResource(resource: String, url: String): AddonResourceResult<String> {
        val response = fetchAddonBodyResult(url)
        response.error?.let { error ->
            return AddonResourceResult.NetworkError(url, IOException(error), response.statusCode)
        }
        val parsed = FluxaCoreNative.parseAddonResourceResult(
            resource = resource,
            url = url,
            statusCode = response.statusCode ?: 0,
            body = response.body
        )
        return when (parsed.kind) {
            "success" -> parsed.valueJson
                ?.let { AddonResourceResult.Success(it, parsed.url.ifBlank { url }) }
                ?: AddonResourceResult.Empty(parsed.url.ifBlank { url })
            "empty" -> AddonResourceResult.Empty(parsed.url.ifBlank { url })
            "network_error" -> AddonResourceResult.NetworkError(
                parsed.url.ifBlank { url },
                statusCode = parsed.statusCode
            )
            "parse_error" -> AddonResourceResult.ParseError(
                parsed.url.ifBlank { url },
                JsonParseException(parsed.error ?: "Invalid addon resource response")
            )
            else -> AddonResourceResult.ParseError(
                parsed.url.ifBlank { url },
                JsonParseException("Unknown addon resource result: ${parsed.kind}")
            )
        }
    }

    @Suppress("UNCHECKED_CAST")
    private fun <T> AddonResourceResult<String>.toTypedEmpty(): AddonResourceResult<T> {
        return when (this) {
            is AddonResourceResult.Success -> AddonResourceResult.Empty(url)
            is AddonResourceResult.Empty -> this
            is AddonResourceResult.NetworkError -> this
            is AddonResourceResult.ParseError -> this
            is AddonResourceResult.AddonUnsupported -> this
        }
    }
}

internal fun mapAppExtraSeasonPosters(
    videos: List<Video>,
    seasonsCount: Int?,
    posters: List<String?>
): Map<String, String> {
    val positiveSeasons = videos
        .mapNotNull { it.season }
        .filter { it > 0 }
        .distinct()
        .sorted()
        .ifEmpty {
            val count = seasonsCount ?: 0
            if (count > 0) (1..count).toList() else emptyList()
        }
    val seasonKeys = when {
        posters.size == positiveSeasons.size + 1 -> listOf(0) + positiveSeasons
        posters.size == positiveSeasons.size -> positiveSeasons
        seasonsCount != null && posters.size == seasonsCount + 1 -> listOf(0) + (1..seasonsCount)
        seasonsCount != null && posters.size == seasonsCount -> (1..seasonsCount).toList()
        else -> posters.indices.map { it + 1 }
    }
    return posters.mapIndexedNotNull { index, url ->
        val posterUrl = url?.takeIf { it.isNotBlank() } ?: return@mapIndexedNotNull null
        val season = seasonKeys.getOrNull(index) ?: return@mapIndexedNotNull null
        season.toString() to posterUrl
    }.toMap()
}
