package com.fluxa.app.data.repository

import com.fluxa.app.common.PlatformLog
import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.data.PlatformSecrets
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.remote.AnilistGraphQlRequest
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.SimklEpisode
import com.fluxa.app.data.remote.SimklItem
import com.fluxa.app.data.remote.ExternalSyncApi
import com.fluxa.app.data.remote.TraktEpisode
import com.fluxa.app.common.AppStrings
import com.fluxa.app.domain.ContentIdentity
import com.google.gson.JsonParser
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

private const val EPISODE_PROGRESS_UNIT_MS = 45 * 60_000L

data class AnilistLibrarySnapshot(
    val watchlist: List<Pair<Meta, Long>> = emptyList(),
    val watching: List<Meta> = emptyList(),
    val completed: List<Meta> = emptyList()
)

private const val ANILIST_LIST_QUERY = """
query (${'$'}userId: Int) {
  MediaListCollection(userId: ${'$'}userId, type: ANIME) {
    lists {
      entries {
        status
        progress
        updatedAt
        media {
          id
          title { english romaji native }
          coverImage { extraLarge large }
          bannerImage
          episodes
          seasonYear
          genres
        }
      }
    }
  }
}
"""

@Singleton
class ExternalLibraryClient @Inject constructor(
    private val externalSyncApi: ExternalSyncApi,
    private val traktSyncClient: TraktSyncClient,
    private val simklSyncCoordinator: SimklSyncCoordinator,
    private val gson: com.google.gson.Gson
) {
    private val traktKey = PlatformSecrets.traktClientId

    private fun unknownName(language: String?): String = AppStrings.t(language, "auto.unknown")

    suspend fun getTraktContinueWatching(profile: UserProfile, language: String = "en"): List<Meta> =
        withContext(Dispatchers.IO) { getTraktContinueWatchingItems(profile, language) }

    suspend fun getSimklContinueWatching(profile: UserProfile, language: String = "en"): List<Meta> =
        withContext(Dispatchers.IO) { getSimklContinueWatchingItems(profile, language) }

    suspend fun getAnilistContinueWatching(profile: UserProfile): List<Meta> =
        withContext(Dispatchers.IO) { getAnilistContinueWatchingItems(profile.anilistAccessToken) }

    suspend fun getTraktSyncSnapshot(profile: UserProfile, language: String = profile.safeLanguage): TraktSyncSnapshot = withContext(Dispatchers.IO) {
        val token = profile.traktAccessToken
        if (token.isNullOrBlank() || !TraktIntegration.hasClient(traktKey)) {
            return@withContext TraktSyncSnapshot(0, 0)
        }
        supervisorScope {
            val continueWatching = async { getTraktContinueWatchingItems(profile, language).size }
            val watchlist = async { traktSyncClient.getWatchlist(token).size }
            TraktSyncSnapshot(
                continueWatchingCount = continueWatching.await(),
                watchlistCount = watchlist.await()
            )
        }
    }

    suspend fun getSimklLibraryItems(token: String?, status: String): List<Meta> = withContext(Dispatchers.IO) {
        if (token.isNullOrBlank() || PlatformSecrets.simklClientId.isBlank()) return@withContext emptyList()
        val types = listOf("movies" to "movie", "shows" to "series", "anime" to "series")
        supervisorScope {
            types.map { (apiType, metaType) ->
                async {
                    runCatching {
                        val response = externalSyncApi.getSimklAllItems(
                            type = apiType,
                            status = status,
                            token = "Bearer $token",
                            apiKey = PlatformSecrets.simklClientId
                        )
                        val items = when (apiType) {
                            "movies" -> response.movies
                            "anime" -> response.anime
                            else -> response.shows
                        }
                        items.mapNotNull { it.toLibraryMeta(metaType, "Simkl") }
                    }.getOrDefault(emptyList())
                }
            }.awaitAll().flatten().let(::distinctByIdentityKey)
        }
    }

    suspend fun getSimklLibraryItems(profile: UserProfile, status: String): List<Meta> = withContext(Dispatchers.IO) {
        val snapshot = simklSyncCoordinator.snapshot(profile)
        val keys = when (status) {
            "watching" -> listOf("moviesWatching" to "movie", "showsWatching" to "series", "animeWatching" to "anime")
            "plantowatch" -> listOf("moviesPlanToWatch" to "movie", "showsPlanToWatch" to "series", "animePlanToWatch" to "anime")
            "completed" -> listOf("moviesCompleted" to "movie", "showsCompleted" to "series", "animeCompleted" to "anime")
            else -> emptyList()
        }
        keys.flatMap { (key, type) ->
            val response = snapshot.resources[key] ?: return@flatMap emptyList()
            val items = when (type) {
                "movie" -> response.movies
                "anime" -> response.anime
                else -> response.shows
            }
            items.mapNotNull { it.toLibraryMeta(if (type == "anime") "series" else type, "Simkl") }
        }.let(::distinctByIdentityKey)
    }

    suspend fun getSimklWatchedEpisodesWithTimestamps(token: String?): Map<String, Long> = withContext(Dispatchers.IO) {
        if (token.isNullOrBlank() || PlatformSecrets.simklClientId.isBlank()) return@withContext emptyMap()
        val statuses = listOf("watching", "completed")
        supervisorScope {
            statuses.flatMap { status ->
                listOf("shows", "anime").map { apiType ->
                    async {
                        runCatching {
                            val response = externalSyncApi.getSimklAllItems(
                                type = apiType,
                                status = status,
                                token = "Bearer $token",
                                apiKey = PlatformSecrets.simklClientId
                            )
                            val items = if (apiType == "anime") response.anime else response.shows
                            items.flatMap { it.watchedEpisodeTimestamps() }
                        }.getOrDefault(emptyList())
                    }
                }
            }.awaitAll().flatten().toMap()
        }
    }

    suspend fun getSimklWatchedEpisodesWithTimestamps(profile: UserProfile): Map<String, Long> = withContext(Dispatchers.IO) {
        val snapshot = simklSyncCoordinator.snapshot(profile)
        val shows = listOf("showsWatching", "showsCompleted").flatMap { snapshot.resources[it]?.shows.orEmpty() }
        val anime = listOf("animeWatching", "animeCompleted").flatMap { snapshot.resources[it]?.anime.orEmpty() }
        (shows + anime).flatMap { it.watchedEpisodeTimestamps() }.toMap()
    }

    private fun SimklItem.watchedEpisodeTimestamps(): List<Pair<String, Long>> {
        val seriesId = effectiveIds?.imdb ?: effectiveIds?.tmdb ?: effectiveIds?.slug?.let { "simkl:$it" } ?: effectiveIds?.simkl?.let { "simkl:$it" } ?: return emptyList()
        return seasons.orEmpty().flatMap { season ->
            val seasonNumber = season.number ?: return@flatMap emptyList()
            season.episodes.orEmpty().mapNotNull { episode ->
                val episodeNumber = episode.number ?: return@mapNotNull null
                val watchedAtMs = episode.watchedAt
                    ?.let { runCatching { java.time.Instant.parse(it).toEpochMilli() }.getOrNull() }
                    ?: return@mapNotNull null
                "$seriesId:$seasonNumber:$episodeNumber" to watchedAtMs
            }
        }
    }

    private fun distinctByIdentityKey(items: List<Meta>): List<Meta> {
        if (items.isEmpty()) return items
        val keys = ContentIdentity.traktKeysBatch(items)
        val seen = HashSet<String>(keys.size)
        return items.filterIndexed { index, _ -> seen.add(keys[index]) }
    }

    private fun coreInvokeArray(method: String, argsJson: com.google.gson.JsonElement): List<com.google.gson.JsonObject> =
        FluxaCoreUniFfi.coreInvokeValue(method, argsJson.toString())
            .takeUnless { it.isJsonNull }
            ?.asJsonArray
            ?.map { it.asJsonObject }
            .orEmpty()

    private fun com.google.gson.JsonObject.toTraktMeta(language: String): Meta? {
        val id = get("id")?.takeUnless { it.isJsonNull }?.asString?.takeIf(String::isNotBlank) ?: return null
        return Meta(
            id = id,
            name = get("name")?.takeUnless { it.isJsonNull }?.asString?.takeIf(String::isNotBlank) ?: unknownName(language),
            type = get("type")?.takeUnless { it.isJsonNull }?.asString ?: "series",
            poster = get("poster")?.takeUnless { it.isJsonNull }?.asString,
            background = get("background")?.takeUnless { it.isJsonNull }?.asString,
            logo = get("logo")?.takeUnless { it.isJsonNull }?.asString,
            resumeProgressPercent = get("resumeProgressPercent")?.takeUnless { it.isJsonNull }?.asFloat,
            lastVideoId = get("lastVideoId")?.takeUnless { it.isJsonNull }?.asString,
            lastEpisodeName = get("lastEpisodeName")?.takeUnless { it.isJsonNull }?.asString,
            lastWatchedAt = get("savedAt")?.takeUnless { it.isJsonNull }?.asString
                ?.takeIf(String::isNotBlank)
                ?.let { runCatching { java.time.Instant.parse(it).toEpochMilli() }.getOrNull() },
            reason = "Trakt.tv",
            continueWatchingPoster = get("poster")?.takeUnless { it.isJsonNull }?.asString,
            continueWatchingBackground = get("background")?.takeUnless { it.isJsonNull }?.asString,
        )
    }

    private suspend fun getTraktContinueWatchingItems(profile: UserProfile, language: String): List<Meta> {
        val token = profile.traktAccessToken
        if (token.isNullOrBlank() || !TraktIntegration.hasClient(traktKey)) return emptyList()
        return try {
            val auth = TraktIntegration.bearer(token)
            val libraryItems = coreInvokeArray(
                "traktPlaybackItemsToLibrary",
                gson.toJsonTree(externalSyncApi.getPlayback(auth, traktKey)),
            )
            val upNextItems = coreInvokeArray(
                "traktUpNextToItems",
                externalSyncApi.getUpNext(auth, traktKey),
            )
            coreInvokeArray(
                "traktPlaybackItemsDedup",
                gson.toJsonTree(libraryItems + upNextItems),
            ).mapNotNull { it.toTraktMeta(language) }
        } catch (e: Exception) {
            PlatformLog.w("ExternalLibraryClient", "Failed to load Trakt continue watching items", e)
            emptyList()
        }
    }

    private suspend fun getSimklContinueWatchingItems(profile: UserProfile, language: String): List<Meta> {
        return runCatching {
            val resources = simklSyncCoordinator.snapshot(profile).resources
            val playbackSessions = fetchSimklPlaybackSessions(profile)
            val consumedSessionIds = java.util.concurrent.ConcurrentHashMap.newKeySet<Int>()
            val movies = resources["moviesWatching"]?.movies.orEmpty().mapNotNull { item ->
                resumeMetaFromPlayback(item, "movie", playbackSessions, language, consumedSessionIds)
                    ?: item.toContinueMeta("movie")
            }
            val shows = resources["showsWatching"]?.shows.orEmpty().mapNotNull { item ->
                val meta = resumeMetaFromPlayback(item, "series", playbackSessions, language, consumedSessionIds)
                    ?: item.toContinueMeta("series")
                meta?.let { item to it }
            }
            val showsWithStills = supervisorScope {
                shows.map { (item, meta) -> async { attachSimklEpisodeDetails(item, meta) } }.awaitAll()
            }
            val unmatchedSessions = playbackSessions.filter { it.get("id")?.asInt !in consumedSessionIds }
            val standaloneSessionItems = supervisorScope {
                unmatchedSessions.map { session -> async {
                    runCatching { metaFromPlaybackSession(session, language) }.getOrNull()
                } }.awaitAll()
            }.filterNotNull()
            movies + showsWithStills + standaloneSessionItems
        }.getOrElse {
            PlatformLog.w("ExternalLibraryClient", "Failed to load Simkl continue watching items", it)
            emptyList()
        }
    }

    private suspend fun fetchSimklPlaybackSessions(profile: UserProfile): List<com.google.gson.JsonObject> {
        val token = profile.simklAccessToken?.takeIf(String::isNotBlank) ?: return emptyList()
        if (PlatformSecrets.simklClientId.isBlank()) return emptyList()
        return runCatching {
            externalSyncApi.getSimklPlaybackSessions("Bearer $token", PlatformSecrets.simklClientId)
        }.getOrDefault(emptyList())
    }

    private fun com.google.gson.JsonObject.playbackSource(): com.google.gson.JsonObject? =
        getAsJsonObject("show") ?: getAsJsonObject("movie") ?: getAsJsonObject("anime")

    private suspend fun resumeMetaFromPlayback(
        item: SimklItem,
        type: String,
        playbackSessions: List<com.google.gson.JsonObject>,
        language: String,
        consumedSessionIds: MutableSet<Int>
    ): Meta? {
        val id = item.effectiveIds?.imdb ?: item.effectiveIds?.tmdb ?: item.effectiveIds?.slug?.let { "simkl:$it" } ?: item.effectiveIds?.simkl?.let { "simkl:$it" } ?: return null
        val locator = if (type == "series") item.nextEpisodeLocator() else null
        val session = playbackSessions.firstOrNull { session ->
            val source = session.playbackSource()
            val simklId = source?.getAsJsonObject("ids")?.get("simkl")?.takeUnless { it.isJsonNull }?.asInt
            val imdbId = source?.getAsJsonObject("ids")?.get("imdb")?.takeUnless { it.isJsonNull }?.asString
            val matchesShow = (simklId != null && simklId == item.effectiveIds?.simkl) || (imdbId != null && imdbId == item.effectiveIds?.imdb)
            if (!matchesShow) return@firstOrNull false
            if (locator == null) return@firstOrNull true
            val episode = session.getAsJsonObject("episode") ?: return@firstOrNull false
            episode.get("season")?.asInt == locator.first && episode.get("number")?.asInt == locator.second
        } ?: return null
        session.get("id")?.asInt?.let { consumedSessionIds.add(it) }
        val entry = simklPlaybackItemToContinueMetaEntry(session) ?: return null
        return Meta(
            id = id,
            name = entry.get("name")?.takeUnless { it.isJsonNull }?.asString ?: item.effectiveTitle ?: unknownName(language),
            type = type,
            poster = item.simklPosterUrl(),
            releaseInfo = item.effectiveYear?.toString(),
            released = item.effectiveYear?.let { "$it-01-01" },
            resumeProgressPercent = entry.get("resumeProgressPercent")?.takeUnless { it.isJsonNull }?.asFloat,
            lastVideoId = entry.get("lastVideoId")?.takeUnless { it.isJsonNull }?.asString,
            lastEpisodeName = entry.get("lastEpisodeName")?.takeUnless { it.isJsonNull }?.asString,
            reason = "Simkl"
        )
    }

    private suspend fun metaFromPlaybackSession(session: com.google.gson.JsonObject, language: String): Meta? {
        val source = session.playbackSource() ?: return null
        val id = source.getAsJsonObject("ids")?.get("imdb")?.takeUnless { it.isJsonNull }?.asString
            ?: source.getAsJsonObject("ids")?.get("tmdb")?.takeUnless { it.isJsonNull }?.asString?.let { "tmdb:$it" }
            ?: source.getAsJsonObject("ids")?.get("simkl")?.takeUnless { it.isJsonNull }?.asInt?.let { "simkl:$it" }
            ?: return null
        val type = if (session.getAsJsonObject("movie") != null) "movie" else "series"
        val entry = simklPlaybackItemToContinueMetaEntry(session) ?: return null
        val posterKey = source.get("poster")?.takeUnless { it.isJsonNull }?.asString
        val meta = Meta(
            id = id,
            name = entry.get("name")?.takeUnless { it.isJsonNull }?.asString ?: unknownName(language),
            type = type,
            poster = posterKey?.let { "https://simkl.in/posters/${it}_m.jpg" },
            releaseInfo = source.get("year")?.takeUnless { it.isJsonNull }?.asString,
            resumeProgressPercent = entry.get("resumeProgressPercent")?.takeUnless { it.isJsonNull }?.asFloat,
            lastVideoId = entry.get("lastVideoId")?.takeUnless { it.isJsonNull }?.asString,
            lastEpisodeName = entry.get("lastEpisodeName")?.takeUnless { it.isJsonNull }?.asString,
            reason = "Simkl"
        )
        return if (type == "series") {
            val simklId = source.getAsJsonObject("ids")?.get("simkl")?.takeUnless { it.isJsonNull }?.asInt
            if (simklId != null) attachEpisodeDetailsBySimklId(simklId, meta) else meta
        } else {
            meta
        }
    }

    private fun simklPlaybackItemToContinueMetaEntry(session: com.google.gson.JsonObject): com.google.gson.JsonObject? {
        val request = com.google.gson.JsonObject().apply { add("item", session) }
        val result = runCatching { FluxaCoreUniFfi.coreInvokeValue("simklPlaybackItemToContinueMeta", request.toString()) }.getOrNull()
        return result?.takeUnless { it.isJsonNull }?.asJsonObject
    }

    private data class SimklEpisodeCacheEntry(val still: String?, val title: String?)

    private val simklEpisodeCache = java.util.concurrent.ConcurrentHashMap<String, SimklEpisodeCacheEntry>()

    private suspend fun attachSimklEpisodeDetails(item: SimklItem, meta: Meta): Meta {
        val simklId = item.effectiveIds?.simkl ?: return meta
        val locator = item.nextEpisodeLocator() ?: return meta
        return attachEpisodeDetailsBySimklId(simklId, locator, meta)
    }

    private suspend fun attachEpisodeDetailsBySimklId(simklId: Int, meta: Meta): Meta {
        val locator = meta.lastVideoId?.split(":")?.takeLast(2)?.let { (season, episode) ->
            season.toIntOrNull()?.let { s -> episode.toIntOrNull()?.let { e -> s to e } }
        } ?: return meta
        return attachEpisodeDetailsBySimklId(simklId, locator, meta)
    }

    private suspend fun attachEpisodeDetailsBySimklId(simklId: Int, locator: Pair<Int, Int>, meta: Meta): Meta {
        val cacheKey = "$simklId:${locator.first}:${locator.second}"
        val fetched = runCatching {
            externalSyncApi.getSimklTvEpisodes(simklId, PlatformSecrets.simklClientId)
                .firstOrNull { it.season == locator.first && it.episode == locator.second }
        }.getOrNull()
        if (fetched != null) {
            simklEpisodeCache[cacheKey] = SimklEpisodeCacheEntry(fetched.img, fetched.title)
        }
        val resolved = fetched?.let { SimklEpisodeCacheEntry(it.img, it.title) } ?: simklEpisodeCache[cacheKey] ?: return meta
        val stillUrl = resolved.still?.let { "https://simkl.in/episodes/${it}_w.webp" }
        return meta.copy(
            continueWatchingPoster = stillUrl ?: meta.continueWatchingPoster,
            continueWatchingBackground = stillUrl ?: meta.continueWatchingBackground,
            lastEpisodeName = resolved.title ?: meta.lastEpisodeName
        )
    }

    private fun SimklItem.nextEpisodeLocator(): Pair<Int, Int>? = nextToWatchLocator

    private fun SimklItem.simklPosterUrl(): String? = effectivePoster?.let { "https://simkl.in/posters/${it}_m.jpg" }

    private fun SimklItem.toLibraryMeta(type: String, source: String): Meta? {
        val id = effectiveIds?.imdb ?: effectiveIds?.tmdb ?: effectiveIds?.slug?.let { "simkl:$it" } ?: effectiveIds?.simkl?.let { "simkl:$it" } ?: return null
        return Meta(
            id = id,
            name = effectiveTitle ?: unknownName(null),
            type = type,
            poster = simklPosterUrl(),
            releaseInfo = effectiveYear?.toString(),
            released = effectiveYear?.let { "$it-01-01" },
            reason = source
        )
    }

    private fun SimklItem.toContinueMeta(type: String): Meta? {
        val id = effectiveIds?.imdb ?: effectiveIds?.tmdb ?: effectiveIds?.slug?.let { "simkl:$it" } ?: effectiveIds?.simkl?.let { "simkl:$it" } ?: return null
        val next = nextToWatchLocator
        return Meta(
            id = id,
            name = effectiveTitle ?: unknownName(null),
            type = type,
            poster = simklPosterUrl(),
            releaseInfo = effectiveYear?.toString(),
            released = effectiveYear?.let { "$it-01-01" },
            timeOffset = 0L,
            duration = 0L,
            lastVideoId = next?.let { "$id:${it.first}:${it.second}" },
            reason = "Simkl"
        )
    }

    private suspend fun anilistSyncValue(token: String?): com.google.gson.JsonObject? {
        if (token.isNullOrBlank()) return null
        val authorization = "Bearer $token"
        return try {
            val viewerResponse = externalSyncApi.anilistGraphQl(
                authorization,
                AnilistGraphQlRequest(query = "query { Viewer { id } }")
            )
            val viewerId = viewerResponse
                .getAsJsonObject("data")
                ?.getAsJsonObject("Viewer")
                ?.get("id")
                ?.takeUnless { it.isJsonNull }
                ?.asInt ?: return null

            val listResponse = externalSyncApi.anilistGraphQl(
                authorization,
                AnilistGraphQlRequest(
                    query = ANILIST_LIST_QUERY,
                    variables = mapOf("userId" to viewerId)
                )
            )
            val lists = listResponse
                .getAsJsonObject("data")
                ?.getAsJsonObject("MediaListCollection")
                ?.getAsJsonArray("lists")
                ?: com.google.gson.JsonArray()
            val entries = com.google.gson.JsonArray()
            lists.forEach { list ->
                list.asJsonObject.getAsJsonArray("entries")?.forEach(entries::add)
            }
            if (entries.size() == 0) return null

            val args = com.google.gson.JsonObject().apply {
                add("entries", entries)
                addProperty("nowMs", System.currentTimeMillis())
            }
            val envelope = JsonParser.parseString(
                FluxaCoreUniFfi.coreInvoke("anilistEntriesToSync", args.toString())
            ).asJsonObject
            if (envelope.get("ok")?.asBoolean != true) return null
            envelope.getAsJsonObject("value")
        } catch (e: Exception) {
            PlatformLog.w("ExternalLibraryClient", "Failed to load AniList sync data", e)
            null
        }
    }

    private suspend fun getAnilistContinueWatchingItems(token: String?): List<Meta> {
        val progress = anilistSyncValue(token)?.getAsJsonObject("progress") ?: return emptyList()
        return progress.entrySet().mapNotNull { (_, value) -> anilistProgressEntryToMeta(value.asJsonObject) }
    }

    suspend fun getAnilistWatchlistWithTimestamps(token: String?): List<Pair<Meta, Long>> {
        val watchlist = anilistSyncValue(token)?.getAsJsonArray("watchlist") ?: return emptyList()
        return watchlist.mapNotNull { entry -> anilistWatchlistEntryToMeta(entry.asJsonObject) }
    }

    suspend fun getAnilistLibrarySnapshot(token: String?): AnilistLibrarySnapshot {
        val value = anilistSyncValue(token) ?: return AnilistLibrarySnapshot()
        return AnilistLibrarySnapshot(
            watchlist = value.getAsJsonArray("watchlist")?.mapNotNull { anilistWatchlistEntryToMeta(it.asJsonObject) }.orEmpty(),
            watching = value.getAsJsonArray("watching")?.mapNotNull { anilistItemToMeta(it.asJsonObject) }.orEmpty(),
            completed = value.getAsJsonArray("completed")?.mapNotNull { anilistItemToMeta(it.asJsonObject) }.orEmpty()
        )
    }

    private fun anilistItemToMeta(item: com.google.gson.JsonObject): Meta? {
        val id = item.get("id")?.takeUnless { it.isJsonNull }?.asString ?: return null
        return Meta(
            id = id,
            name = item.get("name")?.takeUnless { it.isJsonNull }?.asString ?: id,
            type = item.get("type")?.takeUnless { it.isJsonNull }?.asString ?: "series",
            poster = item.get("poster")?.takeUnless { it.isJsonNull }?.asString,
            background = item.get("background")?.takeUnless { it.isJsonNull }?.asString,
            reason = "AniList"
        )
    }

    private fun anilistWatchlistEntryToMeta(item: com.google.gson.JsonObject): Pair<Meta, Long>? {
        val id = item.get("id")?.takeUnless { it.isJsonNull }?.asString ?: return null
        val updatedAtMs = item.get("updatedAtMs")?.takeUnless { it.isJsonNull }?.asLong ?: return null
        val meta = Meta(
            id = id,
            name = item.get("name")?.takeUnless { it.isJsonNull }?.asString ?: id,
            type = item.get("type")?.takeUnless { it.isJsonNull }?.asString ?: "series",
            poster = item.get("poster")?.takeUnless { it.isJsonNull }?.asString,
            background = item.get("background")?.takeUnless { it.isJsonNull }?.asString,
            reason = "AniList"
        )
        return meta to updatedAtMs
    }

    private fun anilistProgressEntryToMeta(entry: com.google.gson.JsonObject): Meta? {
        val meta = entry.getAsJsonObject("meta") ?: return null
        val id = meta.get("id")?.takeUnless { it.isJsonNull }?.asString ?: return null
        val savedAt = entry.get("savedAt")?.takeUnless { it.isJsonNull }?.asString
        val lastWatchedAt = savedAt?.let { runCatching { java.time.Instant.parse(it).toEpochMilli() }.getOrNull() }
        return Meta(
            id = id,
            name = meta.get("name")?.takeUnless { it.isJsonNull }?.asString ?: id,
            type = meta.get("type")?.takeUnless { it.isJsonNull }?.asString ?: "series",
            poster = meta.get("poster")?.takeUnless { it.isJsonNull }?.asString,
            background = meta.get("background")?.takeUnless { it.isJsonNull }?.asString,
            timeOffset = entry.get("timeOffset")?.takeUnless { it.isJsonNull }?.asLong,
            duration = entry.get("duration")?.takeUnless { it.isJsonNull }?.asLong,
            lastVideoId = entry.get("lastVideoId")?.takeUnless { it.isJsonNull }?.asString,
            lastEpisodeName = entry.get("lastEpisodeName")?.takeUnless { it.isJsonNull }?.asString,
            lastWatchedAt = lastWatchedAt,
            reason = "AniList"
        )
    }

}
