package com.fluxa.app.data.repository

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonPrimitive

data class ParsedStremioStream(
    val title: String?,
    val url: String,
    val requestHeaders: Map<String, String>
)

object StremioStreamParser {
    private val json = Json { ignoreUnknownKeys = true }

    fun parse(body: String): List<ParsedStremioStream>? = runCatching {
        val root = json.parseToJsonElement(body) as? JsonObject ?: return@runCatching null
        val streams = root["streams"] as? JsonArray ?: return@runCatching null
        streams.mapNotNull { element ->
            val stream = element as? JsonObject ?: return@mapNotNull null
            val url = playableUrl(stream) ?: return@mapNotNull null
            ParsedStremioStream(
                title = stream["title"]?.jsonPrimitive?.contentOrNull?.trim()?.takeIf { it.isNotEmpty() },
                url = url,
                requestHeaders = requestHeaders(stream)
            )
        }
    }.getOrNull()

    private fun playableUrl(stream: JsonObject): String? {
        val directUrl = stream["url"]?.jsonPrimitive?.contentOrNull
        if (directUrl != null && AllowedSchemes.any { directUrl.startsWith("$it://", true) }) return directUrl
        val infoHash = stream["infoHash"]?.jsonPrimitive?.contentOrNull?.takeIf { it.isNotBlank() } ?: return null
        val fileIndex = stream["fileIdx"]?.jsonPrimitive?.intOrNull
        return if (fileIndex == null) "stremio://torrent/$infoHash" else "stremio://torrent/$infoHash/$fileIndex"
    }

    private fun requestHeaders(stream: JsonObject): Map<String, String> {
        val behaviorHints = stream["behaviorHints"] as? JsonObject ?: return emptyMap()
        val proxyHeaders = behaviorHints["proxyHeaders"] as? JsonObject
        val request = proxyHeaders?.get("request") as? JsonObject
        return request.orEmpty().mapNotNull { (name, value) ->
            value.jsonPrimitive.contentOrNull?.let { name to it }
        }.toMap()
    }

    private val AllowedSchemes = listOf("http", "https", "magnet", "stremio")
}
