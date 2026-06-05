package com.fluxa.app.ui.catalog

import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.graphics.Color as AndroidColor
import android.view.View
import android.widget.RemoteViews
import com.fluxa.app.R
import com.fluxa.app.ui.MainActivity
import org.json.JSONArray
import org.json.JSONObject
import java.text.SimpleDateFormat
import java.util.Locale

class CalendarWidgetProvider : AppWidgetProvider() {
    override fun onUpdate(context: Context, appWidgetManager: AppWidgetManager, appWidgetIds: IntArray) {
        updateWidgets(context, appWidgetManager, appWidgetIds)
    }

    companion object {
        private const val PREFS = "fluxa_calendar_widget"
        private const val KEY_LANGUAGE = "language"
        private const val KEY_ACCENT_COLOR = "accent_color"
        private const val KEY_ROWS = "rows"
        private const val MAX_ROWS = 4
        private val ROW_CONTAINER_IDS = intArrayOf(
            R.id.calendar_widget_row_1,
            R.id.calendar_widget_row_2,
            R.id.calendar_widget_row_3,
            R.id.calendar_widget_row_4
        )
        private val ROW_DATE_IDS = intArrayOf(
            R.id.calendar_widget_row_1_date,
            R.id.calendar_widget_row_2_date,
            R.id.calendar_widget_row_3_date,
            R.id.calendar_widget_row_4_date
        )
        private val ROW_TITLE_IDS = intArrayOf(
            R.id.calendar_widget_row_1_title,
            R.id.calendar_widget_row_2_title,
            R.id.calendar_widget_row_3_title,
            R.id.calendar_widget_row_4_title
        )
        private val ROW_SUBTITLE_IDS = intArrayOf(
            R.id.calendar_widget_row_1_subtitle,
            R.id.calendar_widget_row_2_subtitle,
            R.id.calendar_widget_row_3_subtitle,
            R.id.calendar_widget_row_4_subtitle
        )
        private val ROW_EPISODE_IDS = intArrayOf(
            R.id.calendar_widget_row_1_episode,
            R.id.calendar_widget_row_2_episode,
            R.id.calendar_widget_row_3_episode,
            R.id.calendar_widget_row_4_episode
        )

        fun updateCalendar(
            context: Context,
            items: List<CalendarUpcomingItem>,
            language: String,
            accentColorArgb: Int = 0xFFFFFFFF.toInt()
        ) {
            val rows = JSONArray().apply {
                items.take(MAX_ROWS).forEach { item ->
                    put(JSONObject().apply {
                        put("date", formatWidgetDate(item.dateIso, language))
                        put("title", item.title)
                        put("subtitle", item.episodeTitle ?: item.subtitle.orEmpty())
                        put("episode", widgetEpisodeText(item))
                    })
                }
            }
            context.applicationContext.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
                .edit()
                .putString(KEY_LANGUAGE, language)
                .putInt(KEY_ACCENT_COLOR, accentColorArgb)
                .putString(KEY_ROWS, rows.toString())
                .apply()

            val manager = AppWidgetManager.getInstance(context.applicationContext)
            val ids = manager.getAppWidgetIds(ComponentName(context.applicationContext, CalendarWidgetProvider::class.java))
            updateWidgets(context.applicationContext, manager, ids)
        }

        private fun updateWidgets(context: Context, manager: AppWidgetManager, ids: IntArray) {
            val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
            val language = prefs.getString(KEY_LANGUAGE, "en") ?: "en"
            val accentColor = prefs.getInt(KEY_ACCENT_COLOR, 0xFFFFFFFF.toInt())
            val rows = parseRows(prefs.getString(KEY_ROWS, null))
            ids.forEach { id ->
                manager.updateAppWidget(id, buildViews(context, language, rows, accentColor))
            }
        }

        private fun buildViews(
            context: Context,
            language: String,
            rows: List<WidgetCalendarRow>,
            accentColor: Int
        ): RemoteViews {
            val views = RemoteViews(context.packageName, R.layout.calendar_widget)
            views.setTextViewText(R.id.calendar_widget_title, AppStrings.t(language, "nav.calendar"))
            views.setViewVisibility(R.id.calendar_widget_empty, if (rows.isEmpty()) View.VISIBLE else View.GONE)
            views.setTextViewText(R.id.calendar_widget_empty, AppStrings.t(language, "calendar.empty"))
            val highlightedTextColor = readableTextColor(accentColor)
            ROW_CONTAINER_IDS.forEachIndexed { index, containerId ->
                val row = rows.getOrNull(index)
                views.setViewVisibility(containerId, if (row == null) View.GONE else View.VISIBLE)
                if (row != null) {
                    if (index == 0) {
                        views.setInt(containerId, "setBackgroundColor", accentColor)
                        views.setTextColor(ROW_DATE_IDS[index], highlightedTextColor)
                        views.setTextColor(ROW_TITLE_IDS[index], highlightedTextColor)
                        views.setTextColor(ROW_SUBTITLE_IDS[index], withAlpha(highlightedTextColor, 0.76f))
                        views.setTextColor(ROW_EPISODE_IDS[index], withAlpha(highlightedTextColor, 0.82f))
                    }
                    views.setTextViewText(ROW_DATE_IDS[index], row.date)
                    views.setTextViewText(ROW_TITLE_IDS[index], row.title)
                    views.setTextViewText(ROW_SUBTITLE_IDS[index], row.subtitle)
                    views.setViewVisibility(ROW_SUBTITLE_IDS[index], if (row.subtitle.isBlank()) View.GONE else View.VISIBLE)
                    views.setTextViewText(ROW_EPISODE_IDS[index], row.episode)
                    views.setViewVisibility(ROW_EPISODE_IDS[index], if (row.episode.isBlank()) View.GONE else View.VISIBLE)
                }
            }
            val intent = Intent(context, MainActivity::class.java)
            val pendingIntent = PendingIntent.getActivity(
                context,
                1004,
                intent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )
            views.setOnClickPendingIntent(R.id.calendar_widget_root, pendingIntent)
            return views
        }

        private fun parseRows(raw: String?): List<WidgetCalendarRow> {
            if (raw.isNullOrBlank()) return emptyList()
            return runCatching {
                val array = JSONArray(raw)
                (0 until minOf(array.length(), MAX_ROWS)).mapNotNull { index ->
                    val item = array.optJSONObject(index) ?: return@mapNotNull null
                    WidgetCalendarRow(
                        date = item.optString("date"),
                        title = item.optString("title"),
                        subtitle = item.optString("subtitle"),
                        episode = item.optString("episode")
                    )
                }
            }.getOrElse {
                raw.split('\n')
                    .filter { it.isNotBlank() }
                    .take(MAX_ROWS)
                    .map { WidgetCalendarRow(date = "", title = it, subtitle = "", episode = "") }
            }
        }

        private fun widgetEpisodeText(item: CalendarUpcomingItem): String {
            val season = item.seasonNumber
            val episode = item.episodeNumber
            return if (season != null && episode != null) "S$season:E$episode" else ""
        }

        private fun formatWidgetDate(dateIso: String, language: String): String {
            return runCatching {
                val parsed = SimpleDateFormat("yyyy-MM-dd", Locale.US).parse(dateIso)
                parsed?.let {
                    SimpleDateFormat("d MMMM", AppStrings.locale(language)).format(it)
                }
            }.getOrNull() ?: dateIso
        }

        private fun readableTextColor(background: Int): Int {
            val red = AndroidColor.red(background) / 255.0
            val green = AndroidColor.green(background) / 255.0
            val blue = AndroidColor.blue(background) / 255.0
            val luminance = 0.299 * red + 0.587 * green + 0.114 * blue
            return if (luminance > 0.64) AndroidColor.BLACK else AndroidColor.WHITE
        }

        private fun withAlpha(color: Int, alpha: Float): Int {
            return AndroidColor.argb(
                (alpha.coerceIn(0f, 1f) * 255).toInt(),
                AndroidColor.red(color),
                AndroidColor.green(color),
                AndroidColor.blue(color)
            )
        }
    }
}

private data class WidgetCalendarRow(
    val date: String,
    val title: String,
    val subtitle: String,
    val episode: String
)
