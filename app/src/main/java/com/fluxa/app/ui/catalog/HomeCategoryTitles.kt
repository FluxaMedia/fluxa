package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

fun homeCategoryTitleParts(category: HomeCategory, language: String?): Pair<String, String?> {
    if (category.isContinueWatchingCategory() || category.id == "library" || category.id == "watchlist" || category.id.startsWith("cs3_")) {
        return category.name to null
    }

    val baseName = if (" - " in category.name) category.name.substringAfterLast(" - ").trim() else category.name

    val typeLabel = when (category.type) {
        "movie" -> AppStrings.t(language, "auto.movies")
        "series" -> AppStrings.t(language, "auto.series")
        else -> null
    } ?: return baseName to null

    return stripTypeSuffix(baseName, category.type, language) to typeLabel
}

fun displayHomeCategoryTitle(category: HomeCategory, language: String?): String {
    val (title, typeLabel) = homeCategoryTitleParts(category, language)
    return if (typeLabel != null) "$title  —  $typeLabel" else title
}

fun folderSectionTitle(baseName: String, type: String, language: String?): String {
    val typeLabel = when (type) {
        "movie" -> AppStrings.t(language, "auto.movies")
        "series" -> AppStrings.t(language, "auto.series")
        "anime" -> AppStrings.t(language, "auto.anime")
        else -> null
    } ?: return baseName
    return "${stripTypeSuffix(baseName, type, language)}  —  $typeLabel"
}

private fun stripTypeSuffix(title: String, type: String, language: String?): String {
    val suffixes = when (type) {
        "movie" -> listOf(
            AppStrings.t(language, "auto.movies"),
            AppStrings.t(language, "auto.movie"),
            AppStrings.t("en", "auto.movies"),
            AppStrings.t("en", "auto.movie"),
            AppStrings.t("en", "auto.movies"),
            AppStrings.t("en", "auto.movie")
        )
        "series" -> listOf(
            AppStrings.t(language, "auto.tv_shows"),
            AppStrings.t(language, "auto.series"),
            AppStrings.t("en", "auto.tv_shows"),
            AppStrings.t("en", "auto.series"),
            AppStrings.t("en", "auto.tv_shows"),
            AppStrings.t("en", "auto.series")
        )
        else -> emptyList()
    }.filter { it.isNotBlank() }.distinct()

    return suffixes.fold(title.trim()) { current, suffix ->
        current.removeSuffix(" $suffix").removeSuffix(" ${suffix.lowercase()}").trim()
    }.ifBlank { title }
}
