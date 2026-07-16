package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.core.StremioId
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import com.fluxa.app.domain.discovery.Cs3CatalogFeedDescriptor
import com.lagradost.cloudstream3.MainAPI

internal fun preferredHomeRowLabels(lang: String): List<String> {
    return listOf(
        AppStrings.t(lang, "auto.trending_now"),
        AppStrings.t(lang, "auto.popular_for_you"),
        AppStrings.t(lang, "auto.most_watched"),
        AppStrings.t(lang, "auto.action_adventure"),
        AppStrings.t(lang, "auto.sci_fi_fantasy")
    )
}

internal fun assignHomeBadge(meta: Meta, lang: String): Meta {
    return meta.copy(
        homeBadge = meta.homeBadge?.takeUnless { badge ->
            badge.equals(AppStrings.t(lang, "auto.watched"), ignoreCase = true) ||
                badge.equals("Watched", ignoreCase = true) ||
                badge.equals("İzlendi", ignoreCase = true) ||
                badge.equals("Izlendi", ignoreCase = true)
        }
    )
}

internal fun isCompleted(meta: Meta?): Boolean {
    val progress = meta?.timeOffset ?: return false
    val duration = meta.duration ?: return false
    if (duration <= 0L) return false
    return progress >= (duration * 0.9f).toLong()
}

internal fun watchedSeasonFromVideoId(videoId: String?): Int? {
    return StremioId.parseEpisodeLocator(videoId)?.first
}

internal fun parseEpisodeLocator(videoId: String): Pair<Int, Int>? {
    return StremioId.parseEpisodeLocator(videoId)
}

internal fun formatSeasonEpisode(videoId: String, lang: String): String? {
    val (season, episode) = com.fluxa.app.core.StremioId.parseEpisodeLocator(videoId) ?: return null
    return AppStrings.format(lang, "format.season_episode_code", season.toString(), episode.toString())
}

internal fun isRecentlyReleased(dateStr: String?, windowDays: Int): Boolean {
    return ReleaseDateUtils.isRecentlyReleased(dateStr, windowDays)
}

internal fun isUpcomingRelease(dateStr: String?): Boolean {
    return ReleaseDateUtils.isUpcoming(dateStr)
}

internal fun List<MainAPI>.toCs3CatalogFeedDescriptors(): List<Cs3CatalogFeedDescriptor> {
    return filter { it.hasMainPage }.flatMap { api ->
        api.mainPage.mapIndexed { index, page ->
            Cs3CatalogFeedDescriptor(
                pluginName = api.name,
                catalogName = page.name.takeIf { it.isNotBlank() } ?: api.name,
                catalogIndex = index
            )
        }
    }
}

internal fun Video.continueWatchingTitleForHome(): String {
    val seasonEpisode = if (season != null && number != null) "S$season, E$number" else null
    val title = name?.trim()?.takeIf { it.isNotBlank() }
    return when {
        seasonEpisode != null && title != null -> "$seasonEpisode: $title"
        seasonEpisode != null -> seasonEpisode
        else -> title.orEmpty()
    }
}
