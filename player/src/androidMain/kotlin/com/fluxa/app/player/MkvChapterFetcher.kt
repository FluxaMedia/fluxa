package com.fluxa.app.player

import com.fluxa.app.core.rust.FluxaStreamingNative
import com.google.gson.Gson
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request

/**
 * ExoPlayer's MatroskaExtractor doesn't surface the Chapters element, so we read a
 * prefix of the file ourselves over HTTP and hand the bytes to the Rust EBML parser.
 * Chapters always precede Cluster (frame) data in mkvmerge/ffmpeg output, so a few
 * megabytes is normally enough — if Chapters isn't found in that prefix, there's
 * either none or the file was muxed unusually and we just report no chapters.
 */
object MkvChapterFetcher {
    private const val PREFIX_BYTES = 4L * 1024 * 1024
    private val gson = Gson()
    private val client = OkHttpClient.Builder().build()

    suspend fun fetch(url: String, headers: Map<String, String> = emptyMap()): List<Chapter> = withContext(Dispatchers.IO) {
        runCatching {
            val requestBuilder = Request.Builder()
                .url(url)
                .header("Range", "bytes=0-${PREFIX_BYTES - 1}")
            headers.forEach { (key, value) -> requestBuilder.header(key, value) }

            val bytes = client.newCall(requestBuilder.build()).execute().use { response ->
                if (!response.isSuccessful) return@use null
                response.body.bytes()
            } ?: return@runCatching emptyList()

            val json = FluxaStreamingNative.parseMkvChapters(bytes)
            parseChaptersJson(json)
        }.getOrDefault(emptyList())
    }

    private data class ChapterEntry(val title: String?, val startMs: Long?)

    private fun parseChaptersJson(json: String): List<Chapter> {
        val entries = runCatching {
            gson.fromJson(json, Array<ChapterEntry>::class.java)
        }.getOrNull() ?: return emptyList()
        return entries.mapNotNull { entry ->
            val startMs = entry.startMs ?: return@mapNotNull null
            Chapter(title = entry.title.orEmpty(), startMs = startMs)
        }
    }
}
