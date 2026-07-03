package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

internal class HomeDynamicRowsCoordinator(
    private val scope: CoroutineScope,
    private val categories: () -> List<HomeCategory>,
    private val setCategories: (List<HomeCategory>) -> Unit,
    private val activeProfile: () -> UserProfile?,
    private val buildUserCollectionHomeCategories: (UserProfile?, Boolean?) -> List<HomeCategory>,
    private val buildContinueWatchingItems: (String) -> List<Meta>,
    private val optimizeHomeCategories: (List<HomeCategory>, String) -> List<HomeCategory>
) {
    fun refresh() {
        val currentCategories = categories()
        scope.launch(Dispatchers.IO) {
            val profile = activeProfile()
            val lang = profile?.safeLanguage ?: "en"

            val staticCategories = currentCategories
                .filterNot {
                    it.id == "watchlist" ||
                        it.id == "library" ||
                        it.id == "continue_watching" ||
                        it.type == "collection" ||
                        it.type == "collection_folder"
                }
                .toMutableList()
            val aboveContinueWatching = buildUserCollectionHomeCategories(profile, true)
            val belowContinueWatching = buildUserCollectionHomeCategories(profile, false)

            val continueWatching = if (profile?.safeContinueWatchingEnabled != false) buildContinueWatchingItems(lang) else emptyList()
            var insertIndex = 0
            staticCategories.addAll(insertIndex, aboveContinueWatching)
            insertIndex += aboveContinueWatching.size
            if (continueWatching.isNotEmpty()) {
                staticCategories.add(
                    insertIndex,
                    HomeCategory(
                        name = AppStrings.t(lang, "auto.continue_watching"),
                        items = continueWatching,
                        id = "continue_watching",
                        type = "continue_watching",
                        canLoadMore = false
                    )
                )
                insertIndex += 1
            }
            staticCategories.addAll(insertIndex, belowContinueWatching)

            setCategories(optimizeHomeCategories(staticCategories, lang))
        }
    }
}
