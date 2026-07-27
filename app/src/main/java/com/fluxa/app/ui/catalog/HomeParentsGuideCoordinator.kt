package com.fluxa.app.ui.catalog

import com.fluxa.app.core.StremioId
import com.fluxa.app.data.remote.ParentsGuideCategory
import com.fluxa.app.data.remote.ImdbApiService
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import java.util.concurrent.ConcurrentHashMap

internal class HomeParentsGuideCoordinator(
    private val scope: CoroutineScope,
    private val imdbApiService: ImdbApiService
) {
    private val cache = ConcurrentHashMap<String, List<ParentsGuideCategory>>()
    private val mutableState = MutableStateFlow<List<ParentsGuideCategory>>(emptyList())
    val state: StateFlow<List<ParentsGuideCategory>> = mutableState.asStateFlow()

    fun load(metaId: String) {
        val imdbId = StremioId.imdbId(metaId)
        if (imdbId == null) {
            mutableState.value = emptyList()
            return
        }
        cache[imdbId]?.let {
            mutableState.value = it
            return
        }
        scope.launch {
            try {
                val guide = imdbApiService.getParentsGuide(imdbId).parentsGuide
                cache[imdbId] = guide
                mutableState.value = guide
            } catch (_: Exception) {
                mutableState.value = emptyList()
            }
        }
    }
}
