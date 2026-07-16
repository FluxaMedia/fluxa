package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.Meta
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

object StremioCatalogParser {
    private val json = Json { ignoreUnknownKeys = true }

    fun parse(body: String): List<Meta>? = runCatching {
        json.decodeFromString<CatalogPayload>(body).metas
    }.getOrNull()

    @Serializable
    private data class CatalogPayload(val metas: List<Meta> = emptyList())
}
