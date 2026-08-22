package com.fluxa.app.ui.catalog

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class HomeCategoryStateStore {
    private val categoriesState = MutableStateFlow<List<HomeCategory>>(emptyList())
    private val folderCategoriesState = MutableStateFlow<Map<String, HomeCategory>>(emptyMap())
    private var renderSignature: Int? = null

    val categories: StateFlow<List<HomeCategory>> = categoriesState.asStateFlow()
    val folderCategories: StateFlow<Map<String, HomeCategory>> = folderCategoriesState.asStateFlow()

    fun currentCategories(): List<HomeCategory> = categoriesState.value

    fun currentFolderCategories(): Map<String, HomeCategory> = folderCategoriesState.value

    fun replaceFolderCategories(categories: Map<String, HomeCategory>) {
        folderCategoriesState.value = categories
    }

    fun setCategories(categories: List<HomeCategory>) {
        val hiddenFolders = categories.filter { it.type == "collection_folder" }
        val visible = categories.filterNot { it.type == "collection_folder" || it.type == "collection" }
        when {
            categories.isEmpty() -> folderCategoriesState.value = emptyMap()
            hiddenFolders.isNotEmpty() || categories.any { it.type == "collection" } -> {
                val existing = folderCategoriesState.value
                folderCategoriesState.value = hiddenFolders.associate { incoming ->
                    val previous = existing[incoming.id]
                    incoming.id to if (previous == null || (previous.items.isEmpty() && !previous.folderSourcesLoading)) {
                        incoming
                    } else {
                        incoming.copy(
                            items = previous.items,
                            skip = previous.skip,
                            canLoadMore = previous.canLoadMore,
                            resultSources = previous.resultSources,
                            folderSourcesLoading = previous.folderSourcesLoading
                        )
                    }
                }
            }
        }
        val signature = visible.renderSignature()
        if (renderSignature != signature) {
            renderSignature = signature
            categoriesState.value = visible
        }
    }

    private fun List<HomeCategory>.renderSignature(): Int {
        var result = size
        for (category in this) {
            result = 31 * result + category.id.hashCode()
            result = 31 * result + category.name.hashCode()
            result = 31 * result + category.type.hashCode()
            result = 31 * result + (category.addonIconUrl?.hashCode() ?: 0)
            result = 31 * result + category.items.size
            result = 31 * result + category.skip
            result = 31 * result + category.canLoadMore.hashCode()
            val visibleCount = minOf(category.items.size, 24)
            for (index in 0 until visibleCount) {
                val item = category.items[index]
                result = 31 * result + item.id.hashCode()
                result = 31 * result + item.type.hashCode()
                result = 31 * result + item.name.hashCode()
                result = 31 * result + (item.poster?.hashCode() ?: 0)
                result = 31 * result + (item.background?.hashCode() ?: 0)
                result = 31 * result + (item.logo?.hashCode() ?: 0)
                result = 31 * result + (item.releaseInfo?.hashCode() ?: 0)
                result = 31 * result + (item.reason?.hashCode() ?: 0)
                result = 31 * result + (item.timeOffset?.hashCode() ?: 0)
                result = 31 * result + (item.duration?.hashCode() ?: 0)
                result = 31 * result + (item.lastVideoId?.hashCode() ?: 0)
                result = 31 * result + (item.lastEpisodeName?.hashCode() ?: 0)
                result = 31 * result + (item.continueWatchingPoster?.hashCode() ?: 0)
                result = 31 * result + (item.continueWatchingBackground?.hashCode() ?: 0)
            }
        }
        return result
    }
}
