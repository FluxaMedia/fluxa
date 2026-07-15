package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.AddonRepository

internal data class HomeSearchResults(
    val flatItems: List<Meta>,
    val rows: List<SearchResultRow>
)

internal class HomeSearchCoordinator(
    private val addonRepository: AddonRepository,
    private val historyStore: SearchHistoryStore
) {
    suspend fun search(query: String, profile: UserProfile?, filter: (List<Meta>) -> List<Meta>): HomeSearchResults {
        val normalizedQuery = query.trim()
        val rows = addonRepository.searchRows(
            query = normalizedQuery,
            language = profile?.safeLanguage ?: "en",
            authKey = profile?.authKey.orEmpty(),
            localAddons = profile?.safeLocalAddons.orEmpty()
        )
        val results = rows.flatMap { it.items }.distinctBy { it.id }.take(80)
        return HomeSearchResults(
            flatItems = filter(results),
            rows = rows.mapNotNull { row ->
                val filteredItems = filter(row.items)
                if (filteredItems.isEmpty()) null else row.copy(items = filteredItems)
            }
        )
    }

    fun addToHistory(meta: Meta, current: List<Meta>, profile: UserProfile?): List<Meta> {
        val updated = current.toMutableList()
        val existingIndex = updated.indexOfFirst { it.id == meta.id || it.name.equals(meta.name, ignoreCase = true) }
        if (existingIndex != -1) updated.removeAt(existingIndex)
        updated.add(0, meta.copy(description = null, cast = null, ratings = null, awards = null))
        return updated.take(10).also { historyStore.save(it, profile) }
    }
}
