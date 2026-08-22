package com.fluxa.app.ui.catalog

import android.util.Log
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.domain.discovery.MetadataFeedOption
import com.fluxa.app.domain.discovery.effectiveHomeMetadataFeedSelection
import com.fluxa.app.domain.discovery.isMetadataFeedEnabled
import com.fluxa.app.domain.discovery.orderedMetadataFeeds
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll

internal class HomeBillboardLoader(
    private val addonRepository: AddonRepository,
    private val scope: CoroutineScope,
    private val getMetadataFeeds: suspend (UserProfile?) -> List<MetadataFeedOption>,
    private val getCs3MetadataFeeds: () -> List<MetadataFeedOption>,
    private val fetchCs3FeedItems: suspend (MetadataFeedOption) -> List<Meta>,
    private val setPool: (List<Meta>) -> Unit,
    private val updateContent: suspend (Meta) -> Unit,
    private val normalizePool: (List<Meta>) -> List<Meta>,
    private val startRotation: () -> Unit
) {
    suspend fun load(profile: UserProfile?) {
        val lang = profile?.safeLanguage ?: "en"
        try {
            val enabledFeeds = (getMetadataFeeds(profile) + getCs3MetadataFeeds())
                .let { orderedMetadataFeeds(it, profile?.heroFeedOrder) }
                .let { feeds ->
                    val availableKeys = feeds.map { it.key }
                    val selectedKeys = effectiveHomeMetadataFeedSelection(profile?.heroFeedToggles, availableKeys)
                    feeds.filter { isMetadataFeedEnabled(selectedKeys, it.key) }
                }
                .take(2)

            val spotlightCandidates = enabledFeeds
                .map { feed ->
                    scope.async(Dispatchers.IO) {
                        if (feed.transportUrl.startsWith("cs3://")) {
                            fetchCs3FeedItems(feed).take(10)
                        } else {
                            addonRepository.getAddonCatalog(feed.transportUrl, feed.type, feed.id, genre = feed.genre).take(10)
                        }
                    }
                }
                .awaitAll()
                .flatten()

            val scoredCandidates = spotlightCandidates
                .distinctBy(HomeBillboardRanking::contentIdentityKey)
                .map { assignHomeBadge(it, lang) }

            val reserved = scoredCandidates
                .groupBy { it.billboardTypeGroup() }
                .values
                .flatMap { group -> group.sortedByDescending(HomeBillboardRanking::scoreCandidate).take(POOL_QUOTA_PER_TYPE) }

            val remainingSlots = (POOL_SIZE - reserved.size).coerceAtLeast(0)
            val remainder = scoredCandidates
                .minus(reserved.toSet())
                .sortedByDescending(HomeBillboardRanking::scoreCandidate)
                .take(remainingSlots)

            val pool = (reserved + remainder)
                .sortedByDescending(HomeBillboardRanking::scoreCandidate)
                .let(normalizePool)

            if (pool.isNotEmpty()) {
                setPool(pool)
                updateContent(pool[0])
                startRotation()
            }
        } catch (e: Exception) {
            Log.e("HomeBillboard", "Billboard load failed: ${e.message}")
        }
    }

    private fun Meta.billboardTypeGroup(): String = when (type) {
        "movie" -> "movie"
        "series", "tv", "anime" -> "series"
        else -> "other"
    }

    private companion object {
        const val POOL_SIZE = 8
        const val POOL_QUOTA_PER_TYPE = 4
    }
}
