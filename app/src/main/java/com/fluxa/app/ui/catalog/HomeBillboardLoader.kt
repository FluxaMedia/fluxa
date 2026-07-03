package com.fluxa.app.ui.catalog

import android.util.Log
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
    private val setPool: (List<Meta>) -> Unit,
    private val updateContent: suspend (Meta) -> Unit,
    private val normalizePool: (List<Meta>) -> List<Meta>,
    private val startRotation: () -> Unit
) {
    suspend fun load(profile: UserProfile?) {
        val lang = profile?.safeLanguage ?: "en"
        try {
            val enabledFeeds = getMetadataFeeds(profile)
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
                        addonRepository.getAddonCatalog(feed.transportUrl, feed.type, feed.id, genre = feed.genre).take(10)
                    }
                }
                .awaitAll()
                .flatten()

            val pool = spotlightCandidates
                .distinctBy(HomeBillboardRanking::contentIdentityKey)
                .map { assignHomeBadge(it, lang) }
                .sortedByDescending { HomeBillboardRanking.scoreCandidate(it) }
                .take(8)
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
}
