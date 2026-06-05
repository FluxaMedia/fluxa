package com.fluxa.app.ui.catalog

import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.StremioRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
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
    suspend fun loadMonth(
        profile: UserProfile?,
        year: Int,
        month: Int,
        plannedItems: List<Meta> = emptyList()
    ): EpisodeCalendarLoadResult = withContext(Dispatchers.IO) {
        profile?.id?.let(watchlistManager::setActiveProfile)
        val monthPrefix = "%04d-%02d".format(Locale.US, year, month)
        val localItems = runCatching { watchlistManager.getWatchlistSnapshot() }.getOrDefault(emptyList())
        val externalItems = fetchExternalContinueWatching(profile)
        val candidates = (localItems + externalItems + plannedItems)
            .filter { it.id.isNotBlank() && it.type != "catalog_folder" }
            .distinctBy { "${it.type}:${it.id}" }
        val items = coroutineScope {
            candidates.map { meta ->
                async { loadItemsForMeta(meta, monthPrefix, profile) }
            }.awaitAll()
        }
            .flatten()
            .distinctBy { "${it.dateIso}:${it.meta.id}:${it.subtitle.orEmpty()}" }
            .sortedWith(compareBy<CalendarUpcomingItem> { it.dateIso }.thenBy { it.title })

        EpisodeCalendarLoadResult(items, localItems, externalItems)
    }

    private suspend fun fetchExternalContinueWatching(profile: UserProfile?): List<Meta> {
        if (profile == null || !profile.hasExternalContinueProvider()) return emptyList()
        return withTimeoutOrNull(8_000L) {
            repository.getExternalContinueWatching(profile, profile.safeLanguage)
        }.orEmpty()
    }

    private suspend fun loadItemsForMeta(
        meta: Meta,
        monthPrefix: String,
        profile: UserProfile?
    ): List<CalendarUpcomingItem> = withContext(Dispatchers.IO) {
        val date = ReleaseDateUtils.isoDate(meta.released)
        if (meta.type == "movie") {
            return@withContext if (date != null && date.startsWith(monthPrefix)) {
                listOf(CalendarUpcomingItem(date, meta, meta.name, AppStrings.t(profile?.safeLanguage, "auto.movie"), meta.poster?.takeIf(::isUsableCalendarArtwork)))
            } else {
                emptyList()
            }
        }
        if (meta.type != "series") return@withContext emptyList()

        val language = profile?.safeLanguage ?: "en"
        val configuredDetail = if (!profile?.authKey.isNullOrBlank() || profile?.safeLocalAddons.orEmpty().isNotEmpty()) {
            repository.getMetaDetail(
                type = "series",
                id = meta.id,
                language = language,
                authKey = profile?.authKey.orEmpty(),
                localAddons = profile?.safeLocalAddons.orEmpty(),
                useConfiguredAddons = true
            )
        } else {
            null
        }
        val details = listOfNotNull(configuredDetail)
        if (details.isEmpty()) return@withContext emptyList()
        val seasonArtworkDetail = details.first()

        val directItems = details.flatMap { detail ->
            detail.videos.orEmpty().mapNotNull { video ->
                calendarItemFromVideo(meta, detail, video, monthPrefix, profile)
            }
        }

        val seasonNumbers = details.flatMap { calendarSeasonCandidates(meta, it) }.distinct()
        val seasonItems = coroutineScope {
            seasonNumbers.map { season ->
                async {
                    repository.getTvSeason(
                        id = meta.id,
                        seasonNumber = season,
                        language = language,
                        authKey = profile?.authKey.orEmpty(),
                        localAddons = profile?.safeLocalAddons.orEmpty(),
                        useConfiguredAddons = true
                    ).mapNotNull { episode ->
                        calendarItemFromVideo(meta, seasonArtworkDetail, episode, monthPrefix, profile)
                    }
                }
            }.awaitAll()
        }.flatten()

        (directItems + seasonItems)
            .distinctBy { "${it.dateIso}:${it.meta.id}:${it.subtitle.orEmpty()}" }
    }

    private fun calendarItemFromVideo(
        meta: Meta,
        detail: MetaDetail,
        video: Video,
        monthPrefix: String,
        profile: UserProfile?
    ): CalendarUpcomingItem? {
        val episodeDate = ReleaseDateUtils.isoDate(video.released)
        if (episodeDate == null || !episodeDate.startsWith(monthPrefix)) return null
        val seasonEpisode = listOfNotNull(
            video.season?.let { "S$it" },
            video.number?.let { "E$it" }
        ).joinToString(":")
        val fallbackPoster = listOf(
            meta.poster,
            detail.poster,
            meta.continueWatchingPoster,
            meta.background,
            meta.continueWatchingBackground,
            detail.background
        ).firstOrNull(::isUsableCalendarArtwork)
        return CalendarUpcomingItem(
            dateIso = episodeDate,
            meta = meta,
            title = meta.name,
            subtitle = listOfNotNull(seasonEpisode.takeIf { it.isNotBlank() }, video.name?.takeIf { it.isNotBlank() }).joinToString(" "),
            poster = fallbackPoster,
            episodePoster = fallbackPoster,
            seasonNumber = video.season,
            episodeNumber = video.number,
            episodeTitle = video.name?.takeIf { it.isNotBlank() }
        )
    }

    private fun calendarSeasonCandidates(meta: Meta, detail: MetaDetail): List<Int> {
        val seasonsCount = detail.seasonsCount ?: meta.seasonsCount ?: 1
        val watchedSeason = meta.lastVideoId
            ?.split(":")
            ?.getOrNull(1)
            ?.toIntOrNull()
        val focused = listOfNotNull(
            watchedSeason,
            watchedSeason?.plus(1),
            seasonsCount
        ).filter { it > 0 && it <= seasonsCount }
        val full = if (seasonsCount <= 8) (1..seasonsCount).toList() else focused
        return (focused + full).distinct().take(12)
    }

    private fun UserProfile.hasExternalContinueProvider(): Boolean {
        return !traktAccessToken.isNullOrBlank() || !malAccessToken.isNullOrBlank() || !simklAccessToken.isNullOrBlank()
    }
}
