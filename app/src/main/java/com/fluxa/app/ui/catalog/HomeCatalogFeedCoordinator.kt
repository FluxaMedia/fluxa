package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import android.util.Log
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.domain.discovery.MetadataFeedOption
import com.fluxa.app.domain.discovery.buildMetadataFeedOptions
import com.fluxa.app.domain.discovery.effectiveHomeMetadataFeedSelection
import com.fluxa.app.domain.discovery.isMetadataFeedEnabled
import com.fluxa.app.domain.discovery.metadataFeedHomeTitle
import com.fluxa.app.domain.discovery.orderedMetadataFeeds
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch

internal class HomeCatalogFeedCoordinator(
    private val repository: StremioRepository,
    private val addonRepository: AddonRepository,
    private val scope: CoroutineScope,
    private val userAddons: () -> List<AddonDescriptor>,
    private val setUserAddons: (List<AddonDescriptor>) -> Unit,
    private val continueWatchingItems: (String) -> List<Meta>,
    private val normalizeCatalogItems: suspend (List<Meta>, String, String, String?) -> List<Meta>,
    private val setCategories: (List<HomeCategory>) -> Unit,
    private val currentCategories: () -> List<HomeCategory>
) {
    suspend fun buildInitialCategories(profile: UserProfile?): List<HomeCategory> {
        val lang = profile?.safeLanguage ?: "en"
        val categories = mutableListOf<HomeCategory>()

        categories.addAll(buildUserCollectionHomeCategories(profile, showAboveContinueWatching = true))

        val continueWatching = if (profile?.safeContinueWatchingEnabled != false) continueWatchingItems(lang) else emptyList()
        if (continueWatching.isNotEmpty()) {
            categories.add(
                HomeCategory(
                    AppStrings.t(lang, "auto.continue_watching"),
                    continueWatching,
                    "continue_watching",
                    "continue_watching",
                    canLoadMore = false
                )
            )
        }
        categories.addAll(buildUserCollectionHomeCategories(profile, showAboveContinueWatching = false))
        val initialFeeds = getMetadataFeeds(profile)
            .let { orderedMetadataFeeds(it, profile?.homeFeedOrder) }
            .let { feeds ->
                val availableKeys = feeds.map { it.key }
                val selectedKeys = effectiveHomeMetadataFeedSelection(profile?.homeFeedToggles, availableKeys)
                feeds.filter { isMetadataFeedEnabled(selectedKeys, it.key) }
            }
            .take(2)
        initialFeeds.map { feed ->
            scope.async(Dispatchers.IO) { fetchAddonFeedCategory(feed, lang) }
        }.awaitAll().filterNotNull().let(categories::addAll)
        return categories
    }

    fun buildUserCollectionHomeCategories(
        profile: UserProfile?,
        showAboveContinueWatching: Boolean? = null
    ): List<HomeCategory> {
        val addons = userAddons()
        val lang = profile?.safeLanguage ?: "en"
        val collections = profile?.safeLibraryCollections.orEmpty()
            .filter { it.folders.orEmpty().isNotEmpty() }
            .filter { collection ->
                showAboveContinueWatching == null || collection.showOnHome == showAboveContinueWatching
            }
        return collections.flatMap { collection ->
            val folderSources = collection.folders.orEmpty().associateWith { folder ->
                folder.catalogSources.orEmpty().mapNotNull { source ->
                    val addon = resolveCollectionSourceAddon(source, addons) ?: return@mapNotNull null
                    val catalogName = addon.manifest.catalogs.orEmpty()
                        .firstOrNull { catalog ->
                            catalog.id == source.catalogId &&
                                normalizeCollectionContentType(catalog.type) == normalizeCollectionContentType(source.type)
                        }
                        ?.name
                        ?.takeIf(String::isNotBlank)
                    HomeCatalogSource(
                        transportUrl = addon.transportUrl,
                        catalogId = source.catalogId,
                        type = source.type,
                        genre = normalizeCollectionGenre(source.genre ?: folder.genre),
                        displayName = source.displayName?.takeIf(String::isNotBlank) ?: catalogName,
                        emoji = folder.coverEmoji?.takeIf(String::isNotBlank)
                    )
                }
            }
            val folderResultCategories = collection.folders.orEmpty().mapNotNull { folder ->
                val sources = folderSources[folder].orEmpty()
                val remoteSources = folder.sources.orEmpty()
                if (sources.isEmpty() && remoteSources.isEmpty()) return@mapNotNull null
                HomeCategory(
                    name = folder.title,
                    items = emptyList(),
                    id = folder.id,
                    type = "collection_folder",
                    catalogId = sources.firstOrNull()?.catalogId ?: folder.id,
                    addonTransportUrl = sources.firstOrNull()?.transportUrl,
                    addonGenre = folder.genre,
                    catalogSources = sources,
                    showAllSourcesTab = collection.showAllTab == true,
                    folderViewMode = collection.viewMode,
                    folderHeroImageUrl = folder.effectiveImageUrl(),
                    remoteSources = remoteSources
                )
            }
            val allSources = collection.folders.orEmpty().flatMap { folderSources[it].orEmpty() }
            val allRemoteSources = collection.folders.orEmpty().flatMap { it.sources.orEmpty() }
            val allCategoryId = "${collection.id}.all"
            val allResultCategory = if (collection.showAllTab == true && (allSources.isNotEmpty() || allRemoteSources.isNotEmpty())) {
                HomeCategory(
                    name = AppStrings.t(lang, "auto.all"),
                    items = emptyList(),
                    id = allCategoryId,
                    type = "collection_folder",
                    catalogId = allSources.firstOrNull()?.catalogId ?: allCategoryId,
                    addonTransportUrl = allSources.firstOrNull()?.transportUrl,
                    addonGenre = null,
                    catalogSources = allSources,
                    remoteSources = allRemoteSources
                )
            } else {
                null
            }
            val allCard = if (allResultCategory != null) {
                listOf(
                    Meta(
                        id = allCategoryId,
                        name = AppStrings.t(lang, "auto.all"),
                        type = "catalog_folder",
                        poster = collection.imageUrl,
                        background = collection.imageUrl,
                        reason = null,
                        coverEmoji = "*",
                        focusGlowEnabled = collection.focusGlowEnabled
                    )
                )
            } else {
                emptyList()
            }
            listOf(HomeCategory(
                name = collection.title,
                items = allCard + collection.folders.orEmpty().map { folder ->
                    Meta(
                        id = folder.id,
                        name = folder.title,
                        type = "catalog_folder",
                        poster = folder.effectiveImageUrl(),
                        background = folder.heroBackdropUrl ?: folder.effectiveImageUrl(),
                        logo = folder.titleLogoUrl,
                        releaseInfo = folder.catalogTitle,
                        reason = collection.effectiveFolderShape(folder),
                        focusGifUrl = folder.focusGifUrl.takeIf { folder.focusGifEnabled != false },
                        coverEmoji = folder.coverEmoji,
                        hideTitle = folder.hideTitle,
                        focusGlowEnabled = collection.focusGlowEnabled
                    )
                },
                type = "collection",
                id = collection.id,
                canLoadMore = false
            )) + listOfNotNull(allResultCategory) + folderResultCategories
        }
    }

    suspend fun fetchFolderSections(
        folder: com.fluxa.app.data.local.LibraryUserCollectionFolder,
        lang: String
    ): List<Pair<String, List<Meta>>> {
        val addons = userAddons()
        return coroutineScope {
            folder.catalogSources.orEmpty().map { source ->
                async(Dispatchers.IO) {
                    val addon = resolveCollectionSourceAddon(source, addons) ?: return@async null
                    val catalogName = addon.manifest.catalogs.orEmpty()
                        .firstOrNull { catalog ->
                            catalog.id == source.catalogId &&
                                normalizeCollectionContentType(catalog.type) == normalizeCollectionContentType(source.type)
                        }
                        ?.name
                        ?.takeIf(String::isNotBlank)
                    val baseName = normalizeCollectionGenre(source.genre)
                        ?: source.displayName?.takeIf(String::isNotBlank)
                        ?: catalogName?.takeUnless { it.equals(source.type, ignoreCase = true) }
                        ?: folder.title
                    val emojiPrefix = folder.coverEmoji?.takeIf(String::isNotBlank)?.let { "$it " }.orEmpty()
                    val label = emojiPrefix + folderSectionTitle(baseName, source.type, lang)
                    val items = try {
                        normalizeCatalogItems(
                            addonRepository.getAddonCatalog(
                                addon.transportUrl,
                                source.type,
                                source.catalogId,
                                skip = 0,
                                genre = normalizeCollectionGenre(source.genre ?: folder.genre)
                            ),
                            source.catalogId,
                            lang,
                            source.genre
                        )
                    } catch (e: Exception) {
                        emptyList()
                    }
                    label to items
                }
            }.awaitAll().filterNotNull().filter { it.second.isNotEmpty() }
        }
    }

    private fun resolveCollectionSourceAddon(
        source: com.fluxa.app.data.local.LibraryCatalogSource,
        addons: List<AddonDescriptor>
    ): AddonDescriptor? {
        val addonId = source.addonId
        if (!addonId.isNullOrBlank()) {
            val normalizedAddonId = normalizeAddonIdentity(addonId)
            return addons.firstOrNull { addon ->
                addon.manifest.id.equals(addonId, ignoreCase = true) ||
                    addon.transportUrl.contains(addonId, ignoreCase = true) ||
                    normalizeAddonIdentity(addon.manifest.id) == normalizedAddonId ||
                    normalizeAddonIdentity(addon.transportUrl).contains(normalizedAddonId)
            }
        }
        return addons.firstOrNull { addon ->
            addon.manifest.catalogs.orEmpty().any { catalog ->
                catalog.id == source.catalogId && normalizeCollectionContentType(catalog.type) == normalizeCollectionContentType(source.type)
            }
        }
    }

    private fun normalizeCollectionContentType(value: String?): String? {
        return when (value?.trim()?.lowercase()) {
            "movie", "movies" -> "movie"
            "series", "tv", "show", "shows" -> "series"
            else -> value?.trim()?.lowercase()?.takeIf { it.isNotBlank() }
        }
    }

    private fun normalizeCollectionGenre(value: String?): String? {
        return value?.trim()?.takeIf { it.isNotBlank() && !it.equals("none", ignoreCase = true) }
    }

    private fun normalizeAddonIdentity(value: String?): String {
        return value.orEmpty().lowercase().filter(Char::isLetterOrDigit)
    }

    fun loadRemainingCatalogs(profile: UserProfile?) {
        val lang = profile?.safeLanguage ?: "en"

        scope.launch(Dispatchers.IO) {
            try {
                val enabledFeeds = getMetadataFeeds(profile)
                    .let { orderedMetadataFeeds(it, profile?.homeFeedOrder) }
                    .let { feeds ->
                        val availableKeys = feeds.map { it.key }
                        val selectedKeys = effectiveHomeMetadataFeedSelection(profile?.homeFeedToggles, availableKeys)
                        feeds.filter { isMetadataFeedEnabled(selectedKeys, it.key) }
                    }
                    .drop(2)
                val firstWave = enabledFeeds.take(8).map { feed ->
                    async { fetchAddonFeedCategory(feed, lang) }
                }
                setCategories(optimizeHomeCategories(currentCategories() + firstWave.awaitAll().filterNotNull(), lang))

                val secondWave = enabledFeeds.drop(8).map { feed ->
                    async { fetchAddonFeedCategory(feed, lang) }
                }
                setCategories(optimizeHomeCategories(currentCategories() + secondWave.awaitAll().filterNotNull(), lang))
            } catch (e: Exception) {
                Log.e("HomeViewModel", "Remaining catalogs failed", e)
            }
        }
    }

    suspend fun getMetadataFeeds(profile: UserProfile?): List<MetadataFeedOption> {
        val addons = userAddons().ifEmpty {
            addonRepository.getUserAddons(profile?.authKey ?: "", profile?.safeLocalAddons)
                .also(setUserAddons)
        }
        return buildMetadataFeedOptions(addons, profile?.safeLanguage ?: "en")
    }

    suspend fun fetchAddonFeedCategory(feed: MetadataFeedOption, lang: String): HomeCategory? {
        return try {
            val items = addonRepository.getAddonCatalog(feed.transportUrl, feed.type, feed.id, genre = feed.genre)
            val normalized = normalizeCatalogItems(items, feed.id, lang, feed.genre)
            if (normalized.isEmpty()) return null
            HomeCategory(
                name = metadataFeedHomeTitle(feed.label),
                items = normalized,
                id = feed.key,
                type = feed.type,
                semanticName = metadataFeedHomeTitle(feed.label),
                movieGenre = if (feed.type == "movie") feed.genre else null,
                seriesGenre = if (feed.type == "series") feed.genre else null,
                catalogId = feed.id,
                addonTransportUrl = feed.transportUrl,
                addonGenre = feed.genre
            )
        } catch (e: Exception) {
            null
        }
    }

    fun optimizeHomeCategories(categories: List<HomeCategory>, lang: String): List<HomeCategory> {
        val hiddenCollectionFolderCategories = categories.filter { it.isHiddenCollectionFolderCategory() }
        val visibleCategories = categories.filterNot { it.isHiddenCollectionFolderCategory() }
        return visibleCategories + hiddenCollectionFolderCategories.distinctBy { it.id }
    }
}

private fun HomeCategory.isHiddenCollectionFolderCategory(): Boolean {
    return type == "collection_folder"
}
