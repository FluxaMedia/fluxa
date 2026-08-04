package com.fluxa.app.core.rust

import android.content.Context
import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.ui.catalog.CalendarUpcomingItem
import com.fluxa.app.ui.catalog.CalendarWidgetProvider
import com.fluxa.app.ui.catalog.EpisodeCalendarLoader
import com.fluxa.app.ui.catalog.EpisodeNotificationHelper
import com.google.gson.Gson

internal class AndroidCalendarEffectHandler(
    private val context: Context,
    private val repository: StremioRepository,
    private val watchlistManager: WatchlistManager,
    private val gson: Gson
) {
    suspend fun execute(effect: NativeHeadlessEffect): HeadlessEffectCompletion = when (effect.type) {
        "readCalendarMonth" -> readMonth(effect)
        "replaceExternalContinueWatching" -> replaceExternalContinueWatching(effect)
        "updateCalendarWidget" -> updateWidget(effect)
        "notifyReleasedEpisodes" -> notifyReleasedEpisodes(effect)
        else -> failure(effect, "unsupported_calendar_effect")
    }

    private suspend fun readMonth(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val profile = effect.payload.parseProfile(gson)
        val year = effect.payload.number("year")?.toInt() ?: return failure(effect, "missing_year")
        val month = effect.payload.number("month")?.toInt() ?: return failure(effect, "missing_month")
        val plannedItems = effect.payload.list("plannedItems").mapNotNull { raw ->
            runCatching { gson.fromJson(gson.toJsonTree(raw), Meta::class.java) }.getOrNull()
        }
        val result = EpisodeCalendarLoader(repository, watchlistManager).loadMonth(profile, year, month, plannedItems)
        return success(
            effect,
            mapOf(
                "items" to result.items,
                "localItems" to result.localItems,
                "externalItems" to result.externalItems
            )
        )
    }

    private suspend fun replaceExternalContinueWatching(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val items = effect.payload.list("items").mapNotNull { raw ->
            runCatching { gson.fromJson(gson.toJsonTree(raw), Meta::class.java) }.getOrNull()
        }
        watchlistManager.replaceExternalContinueWatching(items)
        return success(effect, mapOf("count" to items.size))
    }

    private fun updateWidget(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val profile = effect.payload.parseProfile(gson)
        val items = calendarItems(effect)
        CalendarWidgetProvider.updateCalendar(
            context = context,
            items = items,
            language = profile?.safeLanguage ?: "en",
            accentColorArgb = profile?.safeAccentColorArgb ?: 0xFFFFFFFF.toInt()
        )
        return success(effect, mapOf("count" to items.size))
    }

    private suspend fun notifyReleasedEpisodes(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val profile = effect.payload.parseProfile(gson)
        val items = calendarItems(effect)
        EpisodeNotificationHelper.notifyReleasedEpisodes(
            context = context,
            profile = profile,
            items = items,
            todayIso = ReleaseDateUtils.todayIso()
        )
        return success(effect, mapOf("count" to items.size))
    }

    private fun calendarItems(effect: NativeHeadlessEffect): List<CalendarUpcomingItem> =
        effect.payload.list("items").mapNotNull { raw ->
            runCatching { gson.fromJson(gson.toJsonTree(raw), CalendarUpcomingItem::class.java) }.getOrNull()
        }

    private fun success(effect: NativeHeadlessEffect, value: Any?) =
        HeadlessEffectCompletion(effectId = effect.id, status = "ok", value = value)

    private fun failure(effect: NativeHeadlessEffect, code: String) =
        HeadlessEffectCompletion(effectId = effect.id, status = "error", error = mapOf("code" to code))
}
