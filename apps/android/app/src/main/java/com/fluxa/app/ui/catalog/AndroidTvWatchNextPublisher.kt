package com.fluxa.app.ui.catalog

import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.media.tv.TvContract
import android.net.Uri
import android.util.Log
import com.fluxa.app.data.remote.Meta

internal class AndroidTvWatchNextPublisher(context: Context) {
    private val appContext = context.applicationContext
    private val preferences = appContext.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)

    fun publish(
        meta: Meta,
        videoId: String?,
        positionMs: Long,
        durationMs: Long,
        streamUrl: String?,
        streamTitle: String?,
    ) {
        if (!com.fluxa.app.BuildConfig.IS_TV) return
        val internalId = internalId(meta, videoId)
        val values = ContentValues().apply {
            put(TvContract.WatchNextPrograms.COLUMN_INTERNAL_PROVIDER_ID, internalId)
            put(TvContract.WatchNextPrograms.COLUMN_WATCH_NEXT_TYPE, TvContract.WatchNextPrograms.WATCH_NEXT_TYPE_CONTINUE)
            put(TvContract.WatchNextPrograms.COLUMN_TITLE, meta.name)
            put(TvContract.WatchNextPrograms.COLUMN_LONG_DESCRIPTION, meta.description.orEmpty())
            put(TvContract.WatchNextPrograms.COLUMN_SHORT_DESCRIPTION, streamTitle.orEmpty())
            put(TvContract.WatchNextPrograms.COLUMN_POSTER_ART_URI, meta.poster.orEmpty())
            put(TvContract.WatchNextPrograms.COLUMN_INTENT_URI, playbackUri(meta, videoId, positionMs, streamUrl, streamTitle).toString())
            put(TvContract.WatchNextPrograms.COLUMN_LAST_PLAYBACK_POSITION_MILLIS, positionMs.coerceAtLeast(0L))
            put(TvContract.WatchNextPrograms.COLUMN_DURATION_MILLIS, durationMs.coerceAtLeast(0L))
            put(TvContract.WatchNextPrograms.COLUMN_LAST_ENGAGEMENT_TIME_UTC_MILLIS, System.currentTimeMillis())
            put(TvContract.WatchNextPrograms.COLUMN_BROWSABLE, 1)
            put(TvContract.WatchNextPrograms.COLUMN_TYPE, programType(meta, videoId))
        }
        runCatching {
            val existingUri = preferences.getString(uriKey(internalId), null)?.let(Uri::parse)
            val updated = existingUri != null && appContext.contentResolver.update(existingUri, values, null, null) > 0
            if (updated) return
            val insertedUri = appContext.contentResolver.insert(TvContract.WatchNextPrograms.CONTENT_URI, values)
            if (insertedUri != null) preferences.edit().putString(uriKey(internalId), insertedUri.toString()).apply()
        }.onFailure { error ->
            Log.w(TAG, "Unable to publish Watch Next program", error)
        }
    }

    fun remove(meta: Meta, videoId: String?) {
        if (!com.fluxa.app.BuildConfig.IS_TV) return
        val internalId = internalId(meta, videoId)
        runCatching {
            preferences.getString(uriKey(internalId), null)?.let { appContext.contentResolver.delete(Uri.parse(it), null, null) }
            preferences.edit().remove(uriKey(internalId)).apply()
        }.onFailure { error ->
            Log.w(TAG, "Unable to remove Watch Next program", error)
        }
    }

    private fun playbackUri(meta: Meta, videoId: String?, positionMs: Long, streamUrl: String?, streamTitle: String?): Uri =
        Uri.Builder()
            .scheme("app")
            .authority("play")
            .appendQueryParameter("id", meta.id)
            .appendQueryParameter("type", meta.type)
            .apply { videoId?.takeIf(String::isNotBlank)?.let { appendQueryParameter("videoId", it) } }
            .appendQueryParameter("positionMs", positionMs.coerceAtLeast(0L).toString())
            .apply { streamUrl?.takeIf(String::isNotBlank)?.let { appendQueryParameter("streamUrl", it) } }
            .apply { streamTitle?.takeIf(String::isNotBlank)?.let { appendQueryParameter("streamTitle", it) } }
            .build()

    private fun programType(meta: Meta, videoId: String?): Int = when {
        !videoId.isNullOrBlank() -> TvContract.WatchNextPrograms.TYPE_TV_EPISODE
        meta.type.equals("movie", ignoreCase = true) -> TvContract.WatchNextPrograms.TYPE_MOVIE
        else -> TvContract.WatchNextPrograms.TYPE_TV_SERIES
    }

    private fun internalId(meta: Meta, videoId: String?): String = "${meta.type}:${meta.id}:${videoId.orEmpty()}"

    private fun uriKey(internalId: String): String = "program:$internalId"

    companion object {
        private const val PREFERENCES = "fluxa_tv_watch_next"
        private const val TAG = "FluxaWatchNext"
    }
}
