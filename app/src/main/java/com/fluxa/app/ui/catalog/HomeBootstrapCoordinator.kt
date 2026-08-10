package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.NativeHeadlessEngineResult
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeContinueWatchingSource
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.domain.discovery.effectiveHomeMetadataFeedSelection
import com.fluxa.app.domain.discovery.isMetadataFeedEnabled
import com.google.gson.Gson
import java.lang.reflect.Type
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/** Loads cached/fresh home state and coordinates first billboard hydration. */
internal class HomeBootstrapCoordinator(
    private val scope: CoroutineScope,
    private val gson: Gson,
    private val categoryListType: Type,
    private val addonListType: Type,
    private val metaListType: Type,
    private val categoryState: HomeCategoryStateStore,
    private val homeCategoryCache: HomeCategoryCache,
    private val homeBillboardCache: HomeBillboardCache,
    private val billboardState: HomeBillboardStateHolder,
    private val billboardRuntime: HomeBillboardRuntime,
    private val billboardLoader: HomeBillboardLoader,
    private val dispatchHeadless: suspend (Any) -> NativeHeadlessEngineResult,
    private val fetchExternalContinueWatching: suspend (UserProfile?) -> List<Meta>,
    private val resetScrollState: () -> Unit,
    private val setActiveProfile: (UserProfile?) -> Unit,
    private val setCategories: (List<HomeCategory>) -> Unit,
    private val setCategoriesAndCache: (List<HomeCategory>) -> Unit,
    private val setUserAddons: (List<AddonDescriptor>) -> Unit,
    private val setCurrentWatchlist: (List<Meta>) -> Unit,
    private val setWatchlist: (List<Meta>) -> Unit,
    private val setExternalContinueWatching: (List<Meta>) -> Unit,
    private val refreshDynamicRows: () -> Unit,
    private val scheduleCs3Refresh: () -> Unit,
    private val setLoading: (Boolean) -> Unit,
    private val setLoaded: (Boolean) -> Unit,
    initialProfileId: String? = null,
) {
    private var activeProfileId: String? = initialProfileId
    private var hasFetchedFreshBillboard = false

    fun load(activeProfile: UserProfile?, force: Boolean) {
        val profileChanged = activeProfile?.id != activeProfileId
        activeProfileId = activeProfile?.id
        if (profileChanged) resetScrollState()

        if (categoryState.currentCategories().isEmpty()) {
            val cachedCategories = homeCategoryCache.load(activeProfile)
            if (cachedCategories.isNotEmpty()) setCategories(cachedCategories)
        }

        scope.launch {
            setLoading(true)
            try {
                val result = dispatchHeadless(
                    mapOf(
                        "type" to "homeLoadRequested",
                        "profile" to activeProfile,
                        "language" to (activeProfile?.safeLanguage ?: "en"),
                        "force" to force,
                    ),
                )
                val home = result.state["home"] as? Map<*, *> ?: return@launch
                setActiveProfile(activeProfile)
                val rawCategories = decodeList<HomeCategory>(home["categories"], categoryListType)
                val categories = filterSelectedHomeFeeds(rawCategories, activeProfile)
                setUserAddons(decodeList(home["userAddons"], addonListType))
                setCategoriesAndCache(categories)
                setCurrentWatchlist(decodeList(home["continueWatching"], metaListType))
                setWatchlist(decodeList(home["watchlist"], metaListType))

                if (ThirdPartyProviderId.from(activeProfile?.safeContinueWatchingSource) != null) {
                    setExternalContinueWatching(fetchExternalContinueWatching(activeProfile))
                } else {
                    setExternalContinueWatching(emptyList())
                }
                refreshDynamicRows()
            } finally {
                setLoaded(true)
                setLoading(false)
            }
            scheduleCs3Refresh()
        }

        hydrateBillboard(activeProfile, force)
    }

    private fun hydrateBillboard(profile: UserProfile?, force: Boolean) {
        if (billboardState.poolValue.isEmpty()) {
            val cachedPool = homeBillboardCache.load(profile)
            if (cachedPool.isNotEmpty()) {
                billboardState.poolValue = cachedPool
                scope.launch {
                    billboardRuntime.updateContent(cachedPool.first())
                    billboardRuntime.startRotation()
                }
            }
        }
        if (force || !hasFetchedFreshBillboard) {
            scope.launch {
                billboardLoader.load(profile)
                hasFetchedFreshBillboard = true
            }
        }
    }

    private fun filterSelectedHomeFeeds(
        categories: List<HomeCategory>,
        profile: UserProfile?,
    ): List<HomeCategory> {
        val toggles = profile?.homeFeedToggles ?: return categories
        val selection = effectiveHomeMetadataFeedSelection(toggles, categories.map(HomeCategory::id))
        return categories.filter { isMetadataFeedEnabled(selection, it.id) }
    }

    private suspend fun <T> decodeList(value: Any?, type: Type): List<T> {
        if (value == null) return emptyList()
        return withContext(Dispatchers.Default) {
            runCatching { gson.fromJson<List<T>>(gson.toJsonTree(value), type) }
                .getOrDefault(emptyList())
        }
    }
}
