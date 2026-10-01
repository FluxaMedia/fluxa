package com.fluxa.app.ui

import android.content.ContentUris
import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.media.tv.TvContract
import android.net.Uri
import androidx.core.content.ContextCompat
import androidx.core.graphics.drawable.toBitmap
import org.json.JSONArray
import org.json.JSONObject

object TvLauncher {
    private const val PREFS = "tv_launcher"
    private const val FEED = "feed"
    private const val CHANNEL = "channel"

    fun publish(context: Context, feed: String) {
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().putString(FEED, feed).apply()
        runCatching { write(context, JSONObject(feed)) }
    }

    fun initialize(context: Context) {
        val feed = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString(FEED, null)
        runCatching { write(context, feed?.let(::JSONObject) ?: JSONObject()) }
    }

    private fun write(context: Context, feed: JSONObject) {
        writeWatchNext(context, feed.optJSONArray("watchNext") ?: JSONArray())
        val items = feed.optJSONArray("rows")?.optJSONObject(0)?.optJSONArray("items") ?: JSONArray()
        writeChannel(context, items)
    }

    private fun writeWatchNext(context: Context, items: JSONArray) {
        val resolver = context.contentResolver
        resolver.delete(TvContract.WatchNextPrograms.CONTENT_URI, null, null)
        val now = System.currentTimeMillis()
        for (index in 0 until items.length()) {
            val item = items.getJSONObject(index)
            val values = ContentValues().apply {
                put(TvContract.WatchNextPrograms.COLUMN_WATCH_NEXT_TYPE, TvContract.WatchNextPrograms.WATCH_NEXT_TYPE_CONTINUE)
                put(TvContract.WatchNextPrograms.COLUMN_LAST_ENGAGEMENT_TIME_UTC_MILLIS, now - index)
                put(TvContract.WatchNextPrograms.COLUMN_LAST_PLAYBACK_POSITION_MILLIS, item.optLong("positionMs"))
                put(TvContract.WatchNextPrograms.COLUMN_DURATION_MILLIS, item.optLong("durationMs"))
                putAll(program(context, item, TvContract.PreviewPrograms.ASPECT_RATIO_16_9))
            }
            resolver.insert(TvContract.WatchNextPrograms.CONTENT_URI, values)
        }
    }

    private fun writeChannel(context: Context, items: JSONArray) {
        val resolver = context.contentResolver
        val channel = channelId(context)
        resolver.delete(TvContract.PreviewPrograms.CONTENT_URI, "${TvContract.PreviewPrograms.COLUMN_CHANNEL_ID}=?", arrayOf(channel.toString()))
        for (index in 0 until items.length()) {
            val item = items.getJSONObject(index)
            val values = ContentValues().apply {
                put(TvContract.PreviewPrograms.COLUMN_CHANNEL_ID, channel)
                put(TvContract.PreviewPrograms.COLUMN_WEIGHT, items.length() - index)
                putAll(program(context, item, TvContract.PreviewPrograms.ASPECT_RATIO_2_3))
            }
            resolver.insert(TvContract.PreviewPrograms.CONTENT_URI, values)
        }
    }

    private fun program(context: Context, item: JSONObject, aspect: Int): ContentValues {
        val type = if (item.optString("type") == "movie") TvContract.PreviewPrograms.TYPE_MOVIE else TvContract.PreviewPrograms.TYPE_TV_SERIES
        val image = item.optString("image").takeIf(String::isNotEmpty)
        val backdrop = item.optString("backdrop").takeIf(String::isNotEmpty)
        return ContentValues().apply {
            put(TvContract.PreviewPrograms.COLUMN_TYPE, type)
            put(TvContract.PreviewPrograms.COLUMN_TITLE, item.optString("title"))
            put(TvContract.PreviewPrograms.COLUMN_SHORT_DESCRIPTION, item.optString("subtitle").takeIf(String::isNotEmpty))
            put(TvContract.PreviewPrograms.COLUMN_POSTER_ART_URI, image)
            put(TvContract.PreviewPrograms.COLUMN_POSTER_ART_ASPECT_RATIO, aspect)
            put(TvContract.PreviewPrograms.COLUMN_THUMBNAIL_URI, backdrop ?: image)
            put(
                TvContract.PreviewPrograms.COLUMN_THUMBNAIL_ASPECT_RATIO,
                if (backdrop != null) TvContract.PreviewPrograms.ASPECT_RATIO_16_9 else aspect,
            )
            put(TvContract.PreviewPrograms.COLUMN_INTERNAL_PROVIDER_ID, item.optString("id"))
            put(TvContract.PreviewPrograms.COLUMN_INTENT_URI, intentUri(context, item))
        }
    }

    private fun intentUri(context: Context, item: JSONObject): String {
        val uri = Uri.Builder()
            .scheme("app")
            .authority("play")
            .appendQueryParameter("id", item.optString("id"))
            .appendQueryParameter("type", item.optString("type"))
            .build()
        return Intent(Intent.ACTION_VIEW, uri).setPackage(context.packageName).toUri(Intent.URI_INTENT_SCHEME)
    }

    private fun channelId(context: Context): Long {
        val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        val known = prefs.getLong(CHANNEL, -1L)
        val resolver = context.contentResolver
        if (known >= 0 && resolver.query(TvContract.buildChannelUri(known), arrayOf(TvContract.Channels._ID), null, null, null)?.use { it.moveToFirst() } == true) {
            return known
        }
        val values = ContentValues().apply {
            put(TvContract.Channels.COLUMN_INPUT_ID, "")
            put(TvContract.Channels.COLUMN_TYPE, TvContract.Channels.TYPE_PREVIEW)
            put(TvContract.Channels.COLUMN_DISPLAY_NAME, context.applicationInfo.loadLabel(context.packageManager).toString())
            put(TvContract.Channels.COLUMN_APP_LINK_INTENT_URI, Intent(context, MainActivity::class.java).toUri(Intent.URI_INTENT_SCHEME))
        }
        val uri = resolver.insert(TvContract.Channels.CONTENT_URI, values) ?: error("channel insert failed")
        val id = ContentUris.parseId(uri)
        ContextCompat.getDrawable(context, com.fluxa.app.R.drawable.ic_launcher)?.let { drawable ->
            resolver.openOutputStream(TvContract.buildChannelLogoUri(id))?.use {
                drawable.toBitmap(256, 256).compress(android.graphics.Bitmap.CompressFormat.PNG, 100, it)
            }
        }
        prefs.edit().putLong(CHANNEL, id).apply()
        return id
    }
}
