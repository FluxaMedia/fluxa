package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken

private data class NativeHomeOptimizeRequest(
    val categories: List<HomeCategory>,
    val preferredOrderLabels: List<String>,
    val preferredGenres: Map<String, Int>,
    val preferredTypes: Map<String, Int>,
    val priorityLabels: NativeHomePriorityLabels
)

private data class NativeHomePriorityLabels(
    val trendingNow: String,
    val popularForYou: String,
    val mostWatched: String
)

object HomeRowRankingPolicy {
    private val gson = Gson()
    private val homeCategoryListType = object : TypeToken<List<HomeCategory>>() {}.type
    private val metaListType = object : TypeToken<List<Meta>>() {}.type

    fun optimize(
        categories: List<HomeCategory>,
        lang: String,
        preferredOrderLabels: List<String>,
        preferredGenres: Map<String, Int>,
        preferredTypes: Map<String, Int>
    ): List<HomeCategory> {
        val originalItemsById = categories.associateBy { it.id }
        val request = NativeHomeOptimizeRequest(
            categories = categories,
            preferredOrderLabels = preferredOrderLabels,
            preferredGenres = preferredGenres,
            preferredTypes = preferredTypes,
            priorityLabels = priorityLabels(lang)
        )
        val json = FluxaCoreNative.optimizeHomeRowsJson(gson.toJson(request))
        val optimized = gson.fromJson<List<HomeCategory>>(json, homeCategoryListType) ?: emptyList()
        // Rust may reorder items within categories for scoring — restore original item lists
        // so Compose LazyRow doesn't treat reordered items as prepended and jump scroll.
        return optimized.map { category ->
            val original = originalItemsById[category.id]
            if (original != null) category.copy(items = original.items) else category
        }
    }

    fun curateItems(category: HomeCategory, lang: String): List<Meta> {
        val json = FluxaCoreNative.curateHomeItemsJson(gson.toJson(category))
        return gson.fromJson<List<Meta>>(json, metaListType) ?: emptyList()
    }

    fun prioritizeRows(
        categories: List<HomeCategory>,
        preferredOrderLabels: List<String>,
        preferredGenres: Map<String, Int> = emptyMap(),
        preferredTypes: Map<String, Int> = emptyMap(),
        lang: String = "en"
    ): List<HomeCategory> {
        val originalItemsById = categories.associateBy { it.id }
        val json = FluxaCoreNative.prioritizeHomeRowsJson(
            categoriesJson = gson.toJson(categories),
            preferredOrderLabelsJson = gson.toJson(preferredOrderLabels),
            preferredGenresJson = gson.toJson(preferredGenres),
            preferredTypesJson = gson.toJson(preferredTypes),
            priorityLabelsJson = gson.toJson(priorityLabels(lang))
        )
        val result = gson.fromJson<List<HomeCategory>>(json, homeCategoryListType) ?: emptyList()
        return result.map { category ->
            val original = originalItemsById[category.id]
            if (original != null) category.copy(items = original.items) else category
        }
    }

    fun personalizationScore(
        category: HomeCategory,
        preferredGenres: Map<String, Int>,
        preferredTypes: Map<String, Int>,
        lang: String
    ): Int {
        return FluxaCoreNative.homePersonalizationScoreJson(
            categoryJson = gson.toJson(category),
            preferredGenresJson = gson.toJson(preferredGenres),
            preferredTypesJson = gson.toJson(preferredTypes),
            priorityLabelsJson = gson.toJson(priorityLabels(lang))
        )
    }

    fun overlapRatio(first: HomeCategory, second: HomeCategory): Float {
        return FluxaCoreNative.homeOverlapRatioJson(gson.toJson(first), gson.toJson(second))
    }

    fun normalizeKey(value: String): String {
        val sb = StringBuilder()
        var lastSpace = false
        for (ch in value.lowercase()) {
            val n = when (ch) {
                'ç' -> 'c'; 'ğ' -> 'g'; 'ı' -> 'i'; 'ö' -> 'o'; 'ş' -> 's'; 'ü' -> 'u'
                else -> if (ch in 'a'..'z' || ch in '0'..'9') ch else ' '
            }
            if (n == ' ') { if (!lastSpace) { sb.append(' '); lastSpace = true } }
            else { sb.append(n); lastSpace = false }
        }
        return sb.toString().trim()
    }

    private fun priorityLabels(lang: String): NativeHomePriorityLabels {
        return NativeHomePriorityLabels(
            trendingNow = AppStrings.t(lang, "auto.trending_now"),
            popularForYou = AppStrings.t(lang, "auto.popular_for_you"),
            mostWatched = AppStrings.t(lang, "auto.most_watched")
        )
    }
}
