package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta

const val CONTINUE_WATCHING_CATEGORY_ID = "continue_watching"

fun HomeCategory.isContinueWatchingCategory(): Boolean = id == CONTINUE_WATCHING_CATEGORY_ID

fun Meta.matchesFilter(filter: String): Boolean = when (filter) {
    "movie" -> type == "movie"
    "series" -> type == "series" || type == "tv" || type == "anime"
    else -> true
}

fun orderHomeCategories(categories: List<HomeCategory>, filter: String = "all"): List<HomeCategory> {
    return categories.mapNotNull { category ->
        val items = when {
            category.isContinueWatchingCategory() || category.id == "library" -> {
                if (filter == "all") category.items else category.items.filter { it.matchesFilter(filter) }.ifEmpty { category.items }
            }
            filter == "all" -> category.items
            else -> category.items.filter { it.matchesFilter(filter) }
        }
        when {
            items.isEmpty() && category.type != "collection_folder" -> null
            items === category.items -> category
            else -> category.copy(items = items)
        }
    }
}
