package com.fluxa.app.player

import com.fluxa.app.player.subtitle.SubtitleFetcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request

class OkHttpSubtitleFetcher(private val client: OkHttpClient) : SubtitleFetcher {
    override suspend fun fetchProgressive(
        url: String,
        headers: Map<String, String>,
        onChunk: suspend (String) -> Unit
    ) {
        withContext(Dispatchers.IO) {
            val requestBuilder = Request.Builder()
                .url(url)
                .header("User-Agent", StreamRequestPolicy.DEFAULT_USER_AGENT)
            headers.forEach { (key, value) -> requestBuilder.header(key, value) }
            client.newCall(requestBuilder.build()).execute().use { response ->
                val body = response.body
                val source = body.source()
                val buffer = ByteArray(CHUNK_BYTES)
                while (true) {
                    val read = source.read(buffer)
                    if (read == -1) break
                    onChunk(String(buffer, 0, read, Charsets.UTF_8))
                }
            }
        }
    }

    private companion object {
        const val CHUNK_BYTES = 16 * 1024
    }
}
