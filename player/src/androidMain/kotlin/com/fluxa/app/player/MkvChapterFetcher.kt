package com.fluxa.app.player

import com.fluxa.app.shared.feature.player.Chapter

import com.fluxa.app.core.rust.FluxaStreamingNative
import com.google.gson.Gson
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request

object MkvChapterFetcher {
    private const val HEAD_BYTES = 64L * 1024
    private const val SEEK_HINT_BYTES = 256L * 1024
    private val gson = Gson()
    private val client = OkHttpClient.Builder().build()

    suspend fun fetch(url: String, headers: Map<String, String> = emptyMap()): List<Chapter> = withContext(Dispatchers.IO) {
        runCatching {
            val requestHeaders = StreamRequestPolicy.headersFor(url, headers)
            val referer = StreamRequestPolicy.refererFor(url)

            val headBytes = fetchRange(url, requestHeaders, referer, 0, HEAD_BYTES) ?: return@runCatching emptyList()
            val scan = parseScanJson(FluxaStreamingNative.parseMkvChapters(headBytes))
            if (scan.chapters.isNotEmpty()) return@runCatching scan.chapters

            val seekOffset = scan.seekOffset ?: return@runCatching emptyList()
            val hintBytes = fetchRange(url, requestHeaders, referer, seekOffset, SEEK_HINT_BYTES) ?: return@runCatching emptyList()
            parseChaptersJson(FluxaStreamingNative.parseMkvChaptersAtOffset(hintBytes))
        }.getOrDefault(emptyList())
    }

    private fun fetchRange(url: String, headers: Map<String, String>, referer: String?, start: Long, length: Long): ByteArray? {
        val requestBuilder = Request.Builder()
            .url(url)
            .header("Range", "bytes=$start-${start + length - 1}")
        headers.forEach { (key, value) -> requestBuilder.header(key, value) }
        referer?.let { requestBuilder.header("Referer", it) }

        return client.newCall(requestBuilder.build()).execute().use { response ->
            if (!response.isSuccessful) return@use null
            if (response.code != 206 && (response.body.contentLength() < 0 || response.body.contentLength() > length)) {
                return@use null
            }
            response.body.bytes()
        }
    }

    private data class ChapterEntry(val title: String?, val startMs: Long?)
    private data class ChapterScanResponse(val chapters: List<ChapterEntry>?, val seekOffset: Long?)

    private data class ChapterScan(val chapters: List<Chapter>, val seekOffset: Long?)

    private fun parseScanJson(json: String): ChapterScan {
        val response = runCatching { gson.fromJson(json, ChapterScanResponse::class.java) }.getOrNull()
            ?: return ChapterScan(emptyList(), null)
        return ChapterScan(response.chapters.orEmpty().toChapters(), response.seekOffset)
    }

    private fun parseChaptersJson(json: String): List<Chapter> {
        val entries = runCatching {
            gson.fromJson(json, Array<ChapterEntry>::class.java)
        }.getOrNull() ?: return emptyList()
        return entries.toList().toChapters()
    }

    private fun List<ChapterEntry>.toChapters(): List<Chapter> = mapNotNull { entry ->
        val startMs = entry.startMs ?: return@mapNotNull null
        Chapter(title = entry.title.orEmpty(), startMs = startMs)
    }
}
