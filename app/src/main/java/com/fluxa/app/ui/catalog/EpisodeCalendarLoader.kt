package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.StremioRepository
import com.google.gson.Gson
import com.google.gson.JsonObject
import com.google.gson.reflect.TypeToken
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import java.util.Locale

internal data class EpisodeCalendarLoadResult(
    val items: List<CalendarUpcomingItem>,
    val localItems: List<Meta>,
    val externalItems: List<Meta>
)

internal class EpisodeCalendarLoader(
    private val repository: StremioRepository,
    private val watchlistManager: WatchlistManager
) {
    private val gson = Gson()

    suspend fun loadMonth(
        profile: UserProfile?,
        year: Int,
        month: Int,
        plannedItems: List<Meta> = emptyList()
    ): EpisodeCalendarLoadResult = withContext(Dispatchers.IO) {
        profile?.id?.let(watchlistManager::setActiveProfile)
        val monthPrefix = "%04d-%02d".format(Locale.US, year, month)
        val localLibraryItems = runCatching { watchlistManager.getWatchlistSnapshot() }.getOrDefault(emptyList())
        val localContinueWatching = runCatching { watchlistManager.getContinueWatchingSnapshot() }.getOrDefault(emptyList())
        val persistedExternalItems = runCatching { watchlistManager.getExternalContinueWatchingSnapshot() }.getOrDefault(emptyList())
        val refreshedExternalItems = fetchExternalContinueWatching(profile)
        val localItems = calendarCandidatePlan(listOf(localLibraryItems, localContinueWatching))
        val externalItems = calendarCandidatePlan(listOf(persistedExternalItems, refreshedExternalItems))
        val candidates = calendarCandidatePlan(listOf(localItems, externalItems, plannedItems))
        val candidateSemaphore = Semaphore(MAX_CONCURRENT_CANDIDATES)
        val rawItems = coroutineScope {
            candidates.map { meta ->
                async {
                    candidateSemaphore.withPermit {
                        runCatching { loadItemsForMeta(meta, monthPrefix, profile) }.getOrDefault(emptyList())
                    }
                }
            }.awaitAll()
        }
            .flatten()
        val items = calendarContentPlan(rawItems, monthPrefix)

        EpisodeCalendarLoadResult(items, localItems, externalItems)
    }

    private suspend fun fetchExternalContinueWatching(profile: UserProfile?): List<Meta> {
        if (profile == null) return emptyList()
        return withTimeoutOrNull(8_000L) {
            repository.getExternalContinueWatching(profile, profile.safeLanguage)
        }.orEmpty()
    }

    private suspend fun loadItemsForMeta(
        meta: Meta,
        monthPrefix: String,
        profile: UserProfile?
    ): List<CalendarUpcomingItem> = withContext(Dispatchers.IO) {
        val immediateRows = calendarReleaseRows(meta, null, emptyList(), monthPrefix, profile)
        if (immediateRows.isNotEmpty()) return@withContext immediateRows

        val language = profile?.safeLanguage ?: "en"
        val hasConfiguredAddons = !profile?.authKey.isNullOrBlank() || profile?.safeLocalAddons.orEmpty().isNotEmpty()
        val configuredDetail = runCatching {
            repository.getMetaDetail(
                type = "series",
                id = meta.id,
                language = language,
                authKey = profile?.authKey.orEmpty(),
                localAddons = profile?.safeLocalAddons.orEmpty(),
                useConfiguredAddons = hasConfiguredAddons
            )
        }.getOrNull() ?: runCatching {
            repository.getMetaDetail(type = "series", id = meta.id, language = language)
        }.getOrNull()
        val detail = configuredDetail ?: return@withContext emptyList()

        val seasonNumbers = calendarSeasonCandidates(meta, detail).distinct()
        val seasonSemaphore = Semaphore(MAX_CONCURRENT_SEASONS)
        val seasonVideos = coroutineScope {
            seasonNumbers.map { season ->
                async {
                    seasonSemaphore.withPermit {
                        runCatching {
                            repository.getTvSeason(
                                id = meta.id,
                                seasonNumber = season,
                                language = language,
                                authKey = profile?.authKey.orEmpty(),
                                localAddons = profile?.safeLocalAddons.orEmpty(),
                                useConfiguredAddons = true
                            )
                        }.getOrDefault(emptyList())
                    }
                }
            }.awaitAll()
        }.flatten()
        calendarReleaseRows(meta, detail, detail.videos.orEmpty() + seasonVideos, monthPrefix, profile)
    }

    private fun calendarCandidatePlan(groups: List<List<Meta>>): List<Meta> {
        val request = JsonObject().apply { add("groups", gson.toJsonTree(groups)) }
        return gson.fromJson(
            FluxaCoreUniFfi.coreInvokeValue("calendarCandidatePlan", request.toString()),
            object : TypeToken<List<Meta>>() {}.type
        )
    }

    private fun calendarReleaseRows(
        meta: Meta,
        detail: MetaDetail?,
        videos: List<Video>,
        monthPrefix: String,
        profile: UserProfile?
    ): List<CalendarUpcomingItem> {
        val request = JsonObject().apply {
            add("meta", gson.toJsonTree(meta))
            detail?.let { add("detail", gson.toJsonTree(it)) }
            add("videos", gson.toJsonTree(videos))
            addProperty("monthPrefix", monthPrefix)
            addProperty("movieLabel", AppStrings.t(profile?.safeLanguage, "auto.movie"))
        }
        return gson.fromJson(
            FluxaCoreUniFfi.coreInvokeValue("calendarReleaseRows", request.toString()),
            object : TypeToken<List<CalendarUpcomingItem>>() {}.type
        )
    }

    private fun calendarContentPlan(items: List<CalendarUpcomingItem>, monthPrefix: String): List<CalendarUpcomingItem> {
        val request = JsonObject().apply {
            add("items", gson.toJsonTree(items))
            addProperty("monthPrefix", monthPrefix)
        }
        return gson.fromJson(
            FluxaCoreUniFfi.coreInvokeValue("calendarContentPlan", request.toString()),
            object : TypeToken<List<CalendarUpcomingItem>>() {}.type
        )
    }

    private fun calendarSeasonCandidates(meta: Meta, detail: MetaDetail): List<Int> {
        val request = JsonObject().apply {
            addProperty("seasonsCount", detail.seasonsCount ?: meta.seasonsCount ?: 1)
            meta.lastVideoId?.let { addProperty("lastVideoId", it) }
        }
        val value = FluxaCoreUniFfi.coreInvokeValue("calendarSeasonCandidates", request.toString())
        return gson.fromJson(value, object : TypeToken<List<Int>>() {}.type)
    }

    private companion object {
        const val MAX_CONCURRENT_CANDIDATES = 6
        const val MAX_CONCURRENT_SEASONS = 3
    }
}
