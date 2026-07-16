package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.MetaDetailResponse
import kotlinx.serialization.json.Json

object StremioMetaParser {
    private val json = Json { ignoreUnknownKeys = true }

    fun parse(body: String): MetaDetail? = runCatching {
        json.decodeFromString<MetaDetailResponse>(body).meta
    }.getOrNull()
}
