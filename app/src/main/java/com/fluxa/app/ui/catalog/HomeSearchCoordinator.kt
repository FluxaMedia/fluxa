package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.local.safeLocalAddons
import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

/** Owns home-search debounce, result projection and profile-scoped history persistence. */
internal class HomeSearchCoordinator(
    private val scope: CoroutineScope,
    private val platformContentGateway: HomePlatformContentGateway,
    private val searchHistoryStore: SearchHistoryStore,
    private val state: HomeSearchFocusStateHolder,
    private val activeProfile: () -> UserProfile?,
) {
    private var searchJob: Job? = null
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    fun search(query: String) {
        searchJob?.cancel()
        if (query.isEmpty()) {
            state.searchResultsValue = emptyList()
            state.searchRowsValue = emptyList()
            _isLoading.value = false
            return
        }
        searchJob = scope.launch {
            try {
                _isLoading.value = true
                delay(400)
                if (!isActive) return@launch
                val profile = activeProfile()
                val rows = platformContentGateway.searchRows(
                    query = query.trim(),
                    language = profile?.safeLanguage ?: "en",
                    authKey = profile?.authKey.orEmpty(),
                    localAddons = profile?.safeLocalAddons.orEmpty(),
                )
                if (!isActive) return@launch
                state.searchRowsValue = rows
                state.searchResultsValue = rows.flatMap { it.items }.distinctBy { it.id }.take(80)
            } catch (error: Exception) {
                error.printStackTrace()
            } finally {
                _isLoading.value = false
            }
        }
    }

    fun addToHistory(meta: Meta) {
        val current = state.searchHistoryValue.toMutableList()
        val existingIndex = current.indexOfFirst {
            it.id == meta.id || it.name.equals(meta.name, ignoreCase = true)
        }
        if (existingIndex != -1) current.removeAt(existingIndex)
        val updated = (
            listOf(meta.copy(description = null, cast = null, ratings = null, awards = null)) + current
        ).take(10)
        searchHistoryStore.save(updated, activeProfile())
        state.searchHistoryValue = updated
    }

    fun recordSelection(id: String, type: String) {
        val selected = state.searchResultsValue.firstOrNull { it.id == id && it.type == type }
            ?: state.searchResultsValue.firstOrNull { it.id == id }
        selected?.let(::addToHistory)
    }

    fun clearHistory() {
        searchHistoryStore.save(emptyList(), activeProfile())
        state.searchHistoryValue = emptyList()
    }

    fun close() {
        searchJob?.cancel()
    }
}
