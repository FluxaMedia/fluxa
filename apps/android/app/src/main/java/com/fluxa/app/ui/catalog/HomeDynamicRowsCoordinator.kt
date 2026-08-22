package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import java.util.concurrent.atomic.AtomicLong

internal class HomeDynamicRowsCoordinator(
    private val scope: CoroutineScope,
    private val categories: () -> List<HomeCategory>,
    private val setCategories: (List<HomeCategory>) -> Unit,
    private val activeProfile: () -> UserProfile?,
    private val buildUserCollectionHomeCategories: (UserProfile?, Boolean?) -> List<HomeCategory>,
    private val buildContinueWatchingItems: (String) -> List<Meta>,
    private val isUpcoming: (Meta) -> Boolean,
    private val optimizeHomeCategories: (List<HomeCategory>, String) -> List<HomeCategory>
) {
    private var refreshJob: Job? = null
    private val refreshGeneration = AtomicLong(0)

    fun refresh() {
        val generation = refreshGeneration.incrementAndGet()
        refreshJob?.cancel()
        refreshJob = scope.launch(Dispatchers.IO) {
            val currentCategories = categories()
            val profile = activeProfile()
            val lang = profile?.safeLanguage ?: "en"

            val staticCategories = currentCategories
                .filterNot {
                    it.id == "watchlist" ||
                        it.id == "library" ||
                        it.id == "continue_watching" ||
                        it.id == "upcoming" ||
                        it.type == "collection" ||
                        it.type == "collection_folder"
                }
                .toMutableList()
            val aboveContinueWatching = buildUserCollectionHomeCategories(profile, true)
            val belowContinueWatching = buildUserCollectionHomeCategories(profile, false)

            val allContinueWatching = if (profile?.safeContinueWatchingEnabled != false) {
                buildContinueWatchingItems(lang)
            } else {
                emptyList()
            }
            val upcomingEnabled = profile?.safeUpcomingRowEnabled == true
            val upcoming = if (upcomingEnabled) allContinueWatching.filter(isUpcoming) else emptyList()
            val continueWatching = if (upcomingEnabled) allContinueWatching.filterNot(isUpcoming) else allContinueWatching
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
            if (upcoming.isNotEmpty()) {
                staticCategories.add(
                    insertIndex,
                    HomeCategory(
                        name = AppStrings.t(lang, "settings.upcoming_row"),
                        items = upcoming,
                        id = "upcoming",
                        type = "upcoming",
                        canLoadMore = false
                    )
                )
                insertIndex += 1
            }
            staticCategories.addAll(insertIndex, belowContinueWatching)

            if (generation == refreshGeneration.get()) {
                setCategories(optimizeHomeCategories(staticCategories, lang))
            }
        }
    }
}
