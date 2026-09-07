package com.fluxa.app.ui.catalog

import android.content.ContentValues
import android.content.Context
import android.media.tv.TvContract
import android.net.Uri
import android.util.Log
import com.fluxa.app.BuildConfig
import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.remote.Meta

internal class AndroidTvLauncherPublisher(context: Context) {
    private val appContext = context.applicationContext
    private val preferences = appContext.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)

    fun publish(
        categories: List<HomeCategory>,
        billboardItems: List<Meta>,
        watchlist: List<Meta>,
        language: String?,
    ) {
        if (!BuildConfig.IS_TV) return
        runCatching {
            val rows = buildRows(categories, billboardItems, watchlist, language)
                .filter { it.items.isNotEmpty() }
                .distinctBy { it.id }
                .take(MAX_CHANNELS)
            val activeChannelKeys = rows.mapTo(linkedSetOf()) { it.id }

            rows.forEach { row -> publishChannel(row) }
            removeStaleChannels(activeChannelKeys)
            preferences.edit().putStringSet(CHANNEL_KEYS, activeChannelKeys).apply()
        }.onFailure { error ->
            Log.w(TAG, "Unable to publish TV launcher content", error)
        }
    }

    private fun publishChannel(row: LauncherRow) {
        val channelUri = upsertChannel(row)
        val channelId = channelUri.lastPathSegment?.toLongOrNull() ?: return
        val previousProgramKeys = preferences.getStringSet(programKeys(row.id), emptySet()).orEmpty()
        val nextProgramKeys = row.items.take(MAX_PROGRAMS).mapTo(linkedSetOf()) { programKey(row.id, it) }

        row.items.take(MAX_PROGRAMS).forEachIndexed { index, meta ->
            upsertProgram(channelId, row.id, meta, index)
        }
        previousProgramKeys.filterNot(nextProgramKeys::contains).forEach { key ->
            preferences.getString(programUriKey(key), null)?.let { uri ->
                runCatching { appContext.contentResolver.delete(Uri.parse(uri), null, null) }
            }
            preferences.edit().remove(programUriKey(key)).apply()
        }
        preferences.edit().putStringSet(programKeys(row.id), nextProgramKeys).apply()
    }

    private fun upsertChannel(row: LauncherRow): Uri {
        val values = ContentValues().apply {
            put(TvContract.Channels.COLUMN_DISPLAY_NAME, row.title)
            put(TvContract.Channels.COLUMN_DESCRIPTION, "Fluxa · ${row.title}")
            put(TvContract.Channels.COLUMN_TYPE, TvContract.Channels.TYPE_PREVIEW)
            put(TvContract.Channels.COLUMN_BROWSABLE, 1)
            put(TvContract.Channels.COLUMN_SEARCHABLE, 1)
        }
        val key = channelUriKey(row.id)
        val existing = preferences.getString(key, null)?.let(Uri::parse)
        if (existing != null && runCatching { appContext.contentResolver.update(existing, values, null, null) }.getOrDefault(0) > 0) {
            return existing
        }
        val inserted = appContext.contentResolver.insert(TvContract.Channels.CONTENT_URI, values)
        if (inserted != null) preferences.edit().putString(key, inserted.toString()).apply()
        return inserted ?: existing ?: Uri.EMPTY
    }

    private fun upsertProgram(channelId: Long, rowId: String, meta: Meta, index: Int) {
        val key = programKey(rowId, meta)
        val values = ContentValues().apply {
            put(TvContract.PreviewPrograms.COLUMN_CHANNEL_ID, channelId)
            put(TvContract.PreviewPrograms.COLUMN_INTERNAL_PROVIDER_ID, key)
            put(TvContract.PreviewPrograms.COLUMN_TITLE, meta.name)
            put(TvContract.PreviewPrograms.COLUMN_SHORT_DESCRIPTION, meta.lastEpisodeName ?: meta.description.orEmpty())
            put(TvContract.PreviewPrograms.COLUMN_LONG_DESCRIPTION, meta.description.orEmpty())
            put(TvContract.PreviewPrograms.COLUMN_POSTER_ART_URI, meta.continueWatchingPoster ?: meta.poster.orEmpty())
            put(TvContract.PreviewPrograms.COLUMN_THUMBNAIL_URI, meta.poster.orEmpty())
            put(TvContract.PreviewPrograms.COLUMN_INTENT_URI, playbackUri(meta).toString())
            put(TvContract.PreviewPrograms.COLUMN_BROWSABLE, 1)
            put(TvContract.PreviewPrograms.COLUMN_TYPE, programType(meta))
            meta.timeOffset?.let { put(TvContract.PreviewPrograms.COLUMN_LAST_PLAYBACK_POSITION_MILLIS, it.coerceAtLeast(0L)) }
            meta.duration?.takeIf { it > 0L }?.let { put(TvContract.PreviewPrograms.COLUMN_DURATION_MILLIS, it) }
            meta.lastVideoId?.let { videoId ->
                put(TvContract.PreviewPrograms.COLUMN_SERIES_ID, meta.id)
                put(TvContract.PreviewPrograms.COLUMN_EPISODE_TITLE, meta.lastEpisodeName ?: meta.name)
                put(TvContract.PreviewPrograms.COLUMN_CONTENT_ID, videoId)
            }
        }
        val existing = preferences.getString(programUriKey(key), null)?.let(Uri::parse)
        val updated = existing != null && runCatching {
            appContext.contentResolver.update(existing, values, null, null) > 0
        }.getOrDefault(false)
        if (updated) return
        val inserted = appContext.contentResolver.insert(TvContract.PreviewPrograms.CONTENT_URI, values)
        if (inserted != null) preferences.edit().putString(programUriKey(key), inserted.toString()).apply()
    }

    private fun removeStaleChannels(activeKeys: Set<String>) {
        val previousKeys = preferences.getStringSet(CHANNEL_KEYS, emptySet()).orEmpty()
        previousKeys.filterNot(activeKeys::contains).forEach { key ->
            preferences.getString(channelUriKey(key), null)?.let { uri ->
                runCatching { appContext.contentResolver.delete(Uri.parse(uri), null, null) }
            }
            preferences.edit().remove(channelUriKey(key)).apply()
        }
    }

    private fun buildRows(
        categories: List<HomeCategory>,
        billboardItems: List<Meta>,
        watchlist: List<Meta>,
        language: String?,
    ): List<LauncherRow> {
        val rows = categories.map { category ->
            LauncherRow(category.id, category.name, category.items.filterNot { it.type == "catalog_folder" })
        }.toMutableList()
        if (watchlist.isNotEmpty() && rows.none { it.id == "watchlist" }) {
            rows += LauncherRow("watchlist", AppStrings.t(language, "auto.watchlist"), watchlist)
        }
        if (billboardItems.isNotEmpty() && rows.none { it.id == "fluxa_recommendations" }) {
            rows.add(0, LauncherRow("fluxa_recommendations", AppStrings.t(language, "auto.popular_for_you"), billboardItems))
        }
        return rows
    }

    private fun playbackUri(meta: Meta): Uri = Uri.Builder()
        .scheme("app")
        .authority("play")
        .appendQueryParameter("id", meta.id)
        .appendQueryParameter("type", meta.type)
        .apply { meta.lastVideoId?.let { appendQueryParameter("videoId", it) } }
        .apply {
            val durationMs = meta.duration
            val progressPercent = meta.resumeProgressPercent
            val positionMs = meta.timeOffset ?: if (progressPercent != null && durationMs != null) {
                (durationMs * (progressPercent / 100f)).toLong()
            } else {
                null
            }
            positionMs?.let { appendQueryParameter("positionMs", it.coerceAtLeast(0L).toString()) }
        }
        .apply { meta.lastStreamUrl?.let { appendQueryParameter("streamUrl", it) } }
        .apply { meta.lastStreamTitle?.let { appendQueryParameter("streamTitle", it) } }
        .build()

    private fun programType(meta: Meta): Int = when {
        !meta.lastVideoId.isNullOrBlank() -> TvContract.PreviewPrograms.TYPE_TV_EPISODE
        meta.type.equals("movie", ignoreCase = true) -> TvContract.PreviewPrograms.TYPE_MOVIE
        else -> TvContract.PreviewPrograms.TYPE_TV_SERIES
    }

    private fun channelUriKey(id: String): String = "channel:$id"
    private fun programUriKey(key: String): String = "program_uri:$key"
    private fun programKeys(id: String): String = "program_keys:$id"
    private fun programKey(rowId: String, meta: Meta): String = "$rowId:${meta.type}:${meta.id}"

    private data class LauncherRow(val id: String, val title: String, val items: List<Meta>)

    companion object {
        private const val MAX_CHANNELS = 20
        private const val MAX_PROGRAMS = 30
        private const val CHANNEL_KEYS = "channel_keys"
        private const val PREFERENCES = "fluxa_tv_launcher"
        private const val TAG = "FluxaTvLauncher"
    }
}
