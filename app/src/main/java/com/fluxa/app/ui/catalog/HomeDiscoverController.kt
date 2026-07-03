package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.core.rust.FluxaCoreStateHandle
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.domain.discovery.DiscoverCatalogContentLoader
import com.fluxa.app.domain.discovery.DiscoverCatalogOption
import com.fluxa.app.domain.discovery.DiscoverRequest
import com.fluxa.app.domain.discovery.buildDiscoverCatalogOptions
import com.google.gson.Gson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import java.util.Locale

internal class HomeDiscoverController(
    private val addonRepository: AddonRepository,
    private val discoverCatalogContentLoader: DiscoverCatalogContentLoader,
    private val scope: CoroutineScope,
    private val activeProfile: () -> UserProfile?,
    private val userAddons: () -> List<AddonDescriptor>,
    private val setUserAddons: (List<AddonDescriptor>) -> Unit,
    private val normalizeCatalogItems: suspend (List<Meta>, String, String, String?) -> List<Meta>,
    private val coreState: FluxaCoreStateHandle
) {
    private val gson = Gson()
    private val _results = MutableStateFlow<List<Meta>>(emptyList())
    val results: StateFlow<List<Meta>> = _results.asStateFlow()

    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    private val _genres = MutableStateFlow<List<DiscoverGenreOption>>(emptyList())
    val genres: StateFlow<List<DiscoverGenreOption>> = _genres.asStateFlow()

    private val _catalogs = MutableStateFlow<List<DiscoverCatalogOption>>(emptyList())
    val catalogs: StateFlow<List<DiscoverCatalogOption>> = _catalogs.asStateFlow()

    fun discover(type: String, catalogKey: String?, genre: String?, year: String?, rating: Float?, provider: String?, region: String?) {
        scope.launch {
            dispatchDiscover("setDiscoverLoading", true)
            try {
                val language = activeProfile()?.safeLanguage ?: "en"
                val results = discoverCatalogContentLoader.discover(
                    request = DiscoverRequest(
                        type = type,
                        catalogKey = catalogKey,
                        genre = genre,
                        year = year,
                        rating = rating,
                        provider = provider,
                        region = region
                    ),
                    catalogOptions = _catalogs.value
                ) { items, _, catalogId, selectedGenre ->
                    normalizeCatalogItems(items, catalogId, language, selectedGenre)
                }
                dispatchDiscover("setDiscoverResults", results)
            } catch (e: Exception) {
                e.printStackTrace()
            } finally {
                dispatchDiscover("setDiscoverLoading", false)
            }
        }
    }

    fun clearGenres() {
        scope.launch {
            dispatchDiscover("setDiscoverGenres", emptyList<DiscoverGenreOption>())
        }
    }

    fun loadCatalogFilters(type: String, selectedCatalogKey: String?) {
        scope.launch {
            val profile = activeProfile()
            val language = profile?.safeLanguage ?: "en"
            val allLabel = AppStrings.t(language, "auto.all")
            val addons = userAddons().ifEmpty {
                addonRepository.getUserAddons(profile?.authKey.orEmpty(), profile?.safeLocalAddons)
                    .also(setUserAddons)
            }
            val catalogOptions = buildDiscoverCatalogOptions(addons, type)
            dispatchDiscover("setDiscoverCatalogs", catalogOptions)
            val selectedCatalog = catalogOptions.firstOrNull { it.key == selectedCatalogKey }
            val selectedGenres = selectedCatalog?.genres.orEmpty()
                .distinct()
                .sortedBy { it.lowercase(Locale.ROOT) }
                .map { DiscoverGenreOption(it, it) }
            val genres = if (selectedCatalog == null || selectedGenres.isEmpty()) {
                emptyList()
            } else {
                val includeAll = !selectedCatalog.requiresGenre
                if (includeAll) listOf(DiscoverGenreOption(null, allLabel)) + selectedGenres else selectedGenres
            }
            dispatchDiscover("setDiscoverGenres", genres)
        }
    }

    private fun dispatchDiscover(type: String, value: Any?) {
        val snapshotJson = coreState.dispatch(CoreAction(type = type, value = value))
        val snapshot = gson.fromJson(snapshotJson, CoreStateSnapshot::class.java)?.discover ?: return
        _results.value = snapshot.results
        _isLoading.value = snapshot.isLoading
        _genres.value = snapshot.genres
        _catalogs.value = snapshot.catalogs
    }

    private data class CoreAction(
        val type: String,
        val value: Any?
    )

    private data class CoreStateSnapshot(
        val discover: CoreDiscoverSnapshot = CoreDiscoverSnapshot()
    )

    private data class CoreDiscoverSnapshot(
        val results: List<Meta> = emptyList(),
        val isLoading: Boolean = false,
        val genres: List<DiscoverGenreOption> = emptyList(),
        val catalogs: List<DiscoverCatalogOption> = emptyList()
    )
}
