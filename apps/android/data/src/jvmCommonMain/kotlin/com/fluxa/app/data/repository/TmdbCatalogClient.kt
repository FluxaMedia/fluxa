package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.TmdbMeta
import com.fluxa.app.data.remote.TmdbService
import com.fluxa.app.common.AppStrings
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

internal class TmdbCatalogClient(
    private val tmdbService: TmdbService
) {
    suspend fun getRecommendations(type: String, id: String, language: String = "en"): List<Meta> = withContext(Dispatchers.IO) {
        try {
            val tmdbId = if (id.startsWith("tt")) findTmdbId(type, id) else id
            if (tmdbId == null) return@withContext emptyList()
            tmdbService.getRecommendations(if (type == "series") "tv" else type, tmdbId, getTmdbLang(language)).results
                .map { it.toMeta(language).copy(type = type) }
        } catch (e: Exception) {
            emptyList()
        }
    }

    suspend fun getSimilar(type: String, id: String, language: String = "en"): List<Meta> = withContext(Dispatchers.IO) {
        try {
            val tmdbId = if (id.startsWith("tt")) findTmdbId(type, id) else id
            if (tmdbId == null) return@withContext emptyList()
            val tmdbType = if (type == "series") "tv" else type
            val recommendations = tmdbService.getRecommendations(tmdbType, tmdbId, getTmdbLang(language)).results
            (if (recommendations.isNotEmpty()) recommendations else tmdbService.getSimilar(tmdbType, tmdbId, getTmdbLang(language)).results)
                .map { it.toMeta(language).copy(type = type) }
        } catch (e: Exception) {
            emptyList()
        }
    }

    suspend fun findTmdbId(type: String, imdbId: String): String? = withContext(Dispatchers.IO) {
        try {
            val result = tmdbService.findById(imdbId)
            if (type == "movie") result.movieResults.firstOrNull()?.id?.toString() else result.tvResults.firstOrNull()?.id?.toString()
        } catch (e: Exception) {
            null
        }
    }

    private fun getTmdbLang(lang: String?): String = TmdbLanguageResolver.languageTag(lang)

    private fun TmdbMeta.toMeta(language: String, englishFallback: TmdbMeta? = null): Meta {
        val resolvedType = if (media_type == "tv" || first_air_date != null) "series" else "movie"
        val localizedName = localizedTitleWithEnglishFallback(
            localized = title ?: name,
            original = original_name,
            english = englishFallback?.title ?: englishFallback?.name
        )

        return Meta(
            id = id.toString(),
            name = localizedName ?: AppStrings.t(language, "auto.unknown"),
            type = resolvedType,
            poster = null,
            releaseInfo = (release_date ?: first_air_date)?.take(4),
            released = release_date ?: first_air_date,
            originalName = original_name
        )
    }

    private fun localizedTitleWithEnglishFallback(localized: String?, original: String?, english: String?): String? {
        val value = localized?.takeIf { it.isNotBlank() }
        val englishValue = english?.takeIf { it.isNotBlank() }
        if (value == null) return englishValue
        if (englishValue != null && (value == original || value.hasCyrillic())) return englishValue
        return value
    }

    private fun String.hasCyrillic(): Boolean = any { it in '\u0400'..'\u04FF' }
}
