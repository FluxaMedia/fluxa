@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.core.rust

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import com.fluxa.app.ui.catalog.HomeCategory
import com.fluxa.app.domain.discovery.buildMetadataFeedOptions
import com.fluxa.app.domain.discovery.effectiveHomeMetadataFeedSelection
import com.fluxa.app.domain.discovery.isMetadataFeedEnabled
import com.fluxa.app.domain.discovery.orderedMetadataFeeds
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit

internal suspend fun FluxaAndroidHeadlessEnvironment.readHomeBootstrap(effect: NativeHeadlessEffect): HeadlessEffectCompletion = coroutineScope {
    val profile = effect.payload.profile()
    profile?.id?.let(watchlistManager::setActiveProfile)
    val addons = addonRepository.getUserAddons(profile?.authKey.orEmpty(), profile?.safeLocalAddons.orEmpty())
    val language = effect.payload.string("language", profile?.safeLanguage ?: "en")
    val allFeeds = buildMetadataFeedOptions(addons, language)
    val metadataFeeds = orderedMetadataFeeds(allFeeds, profile?.homeFeedOrder).let { feeds ->
        val availableKeys = feeds.map { it.key }
        val selectedKeys = effectiveHomeMetadataFeedSelection(profile?.homeFeedToggles, availableKeys)
        feeds.filter { isMetadataFeedEnabled(selectedKeys, it.key) }
    }
    val semaphore = Semaphore(8)
    val categories = metadataFeeds.map { feed ->
        async {
            val items = semaphore.withPermit {
                runCatching {
                    addonRepository.getAddonCatalog(
                        transportUrl = feed.transportUrl,
                        type = feed.type,
                        id = feed.id,
                        genre = feed.genre
                    )
                }.getOrDefault(emptyList())
            }
            if (items.isEmpty()) null else {
                HomeCategory(
                    name = feed.label,
                    semanticName = feed.label,
                    items = items,
                    id = feed.key,
                    type = feed.type,
                    catalogId = feed.id,
                    addonTransportUrl = feed.transportUrl,
                    addonGenre = feed.genre
                )
            }
        }
    }.awaitAll().filterNotNull()
    ok(
        effect,
        mapOf(
            "categories" to categories,
            "continueWatching" to watchlistManager.getContinueWatchingSnapshot(),
            "watchlist" to watchlistManager.getWatchlistSnapshot(),
            "userAddons" to addons,
            "metadataFeeds" to metadataFeeds,
            "billboard" to categories.firstOrNull()?.items?.firstOrNull()
        )
    )
}

internal suspend fun FluxaAndroidHeadlessEnvironment.readLibraryState(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    effect.payload.stringOrNull("profileId")?.let(watchlistManager::setActiveProfile)
    return ok(
        effect,
        mapOf(
            "watchlist" to watchlistManager.getWatchlistSnapshot(),
            "continueWatching" to watchlistManager.getContinueWatchingSnapshot(),
            "liked" to emptyList<Any>(),
            "watched" to emptyMap<String, Any>()
        )
    )
}

internal suspend fun FluxaAndroidHeadlessEnvironment.writeLibraryCommand(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val command = effect.payload.objectValue("command").orEmpty()
    val profileId = effect.payload.stringOrNull("profileId")
    profileId?.let(watchlistManager::setActiveProfile)
    val profile = profileId?.let { id -> profileManager.getProfiles().firstOrNull { it.id == id } }
    val value = when (command.string("type")) {
        "toggleWatchlist" -> {
            val item = command.objectValue("item")?.let { gson.fromJson(gson.toJsonTree(it), Meta::class.java) }
            if (item != null) {
                val selectedProvider = profile?.integrationLibrarySource
                    ?.let(ThirdPartyProviderId::from)
                if (profile != null && selectedProvider != null) {
                    val identity = com.fluxa.app.data.repository.TraktIntegration.contentIdentityKey(item)
                    val snapshot = thirdPartyProviderRepository.load(profile, selectedProvider, refresh = true)
                    val wasInProviderLibrary = snapshot?.libraryItems.orEmpty().any { existing ->
                        com.fluxa.app.data.repository.TraktIntegration.contentIdentityKey(existing) == identity
                    }
                    val requestedState = !wasInProviderLibrary
                    val success = thirdPartyProviderRepository.pushWatchlist(
                        profile = profile,
                        providerId = selectedProvider,
                        expectedAccountId = profile.providerAccountId(selectedProvider),
                        item = item,
                        add = requestedState
                    )
                    val updated = thirdPartyProviderRepository.cached(profile, selectedProvider)
                    mapOf(
                        "watchlist" to updated?.libraryItems.orEmpty(),
                        "isInWatchlist" to if (success) requestedState else wasInProviderLibrary,
                        "provider" to selectedProvider.key
                    )
                } else {
                    watchlistManager.toggleWatchlist(item)
                    val isInWatchlist = watchlistManager.isInWatchlist(item.id)
                    mapOf("watchlist" to watchlistManager.getWatchlistSnapshot(), "isInWatchlist" to isInWatchlist)
                }
            } else {
                mapOf("watchlist" to watchlistManager.getWatchlistSnapshot())
            }
        }
        "markWatched" -> {
            val watched = command.boolean("watched", true)
            val seriesId = command.string("seriesId")
            val videoIds = command.list("videoIds").mapNotNull { it?.toString() }
            val localWatched = watchlistManager.markEpisodesWatched(
                seriesId = seriesId,
                videoIds = videoIds,
                watched = watched
            )
            if (profile != null) {
                val meta = command.objectValue("meta")?.let { gson.fromJson(gson.toJsonTree(it), Meta::class.java) }
                val episodes = command.list("episodes").mapNotNull {
                    runCatching { gson.fromJson(gson.toJsonTree(it), Video::class.java) }.getOrNull()
                }
                if (meta != null) {
                    primeScope.launch {
                        runCatching {
                            playbackSyncCoordinator.pushWatched(profile, meta, episodes, watched)
                        }
                    }
                }
            }
            mapOf("watchlist" to watchlistManager.getWatchlistSnapshot(), "localWatchedVideoIds" to localWatched.toList())
        }
        else -> mapOf("watchlist" to watchlistManager.getWatchlistSnapshot())
    }
    return ok(effect, value)
}

internal suspend fun FluxaAndroidHeadlessEnvironment.writeFeedback(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val meta = gson.fromJson(gson.toJsonTree(payload["meta"]), Meta::class.java)
    val value = payload["value"] as? Boolean
    val profile = payload.profile()
    val selectedProvider = profile?.integrationLibrarySource?.let(ThirdPartyProviderId::from)
    if (profile != null && meta != null && selectedProvider != null) {
        thirdPartyProviderRepository.pushFavorite(
            profile = profile,
            providerId = selectedProvider,
            expectedAccountId = profile.providerAccountId(selectedProvider),
            item = meta,
            favorite = value == true
        )
    } else {
        watchlistManager.setFeedback(payload.string("id"), value, meta)
    }
    return ok(effect, mapOf("feedback" to value, "provider" to selectedProvider?.key))
}
