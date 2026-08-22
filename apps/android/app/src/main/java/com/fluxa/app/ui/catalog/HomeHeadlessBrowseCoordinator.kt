package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.NativeHeadlessEngineResult
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.domain.discovery.DiscoverCatalogOption
import com.fluxa.app.domain.discovery.buildDiscoverCatalogOptions
import com.fluxa.app.domain.discovery.buildDiscoverContentTypes
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

internal class HomeHeadlessBrowseCoordinator(
    private val scope: CoroutineScope,
    private val gson: Gson,
    private val dispatch: suspend (Any) -> NativeHeadlessEngineResult,
    private val activeProfile: () -> UserProfile?,
    private val userAddons: () -> List<AddonDescriptor>
) {
    private val metaListType = object : TypeToken<List<Meta>>() {}.type
    private val calendarItemListType = object : TypeToken<List<CalendarUpcomingItem>>() {}.type
    private val catalogListType = object : TypeToken<List<DiscoverCatalogOption>>() {}.type
    private val genreListType = object : TypeToken<List<DiscoverGenreOption>>() {}.type
    private val contentTypeListType = object : TypeToken<List<String>>() {}.type
    private val sourceMapType = object : TypeToken<Map<String, HomeCatalogSource>>() {}.type
    private val results = MutableStateFlow<List<Meta>>(emptyList())
    private val resultSources = MutableStateFlow<Map<String, HomeCatalogSource>>(emptyMap())
    private val loading = MutableStateFlow(false)
    private val genres = MutableStateFlow<List<DiscoverGenreOption>>(emptyList())
    private val catalogs = MutableStateFlow<List<DiscoverCatalogOption>>(emptyList())
    private val contentTypes = MutableStateFlow<List<String>>(emptyList())
    private val calendarItems = MutableStateFlow<List<CalendarUpcomingItem>>(emptyList())
    private val calendarLoading = MutableStateFlow(false)
    private var discoverJob: Job? = null
    private var calendarJob: Job? = null
    private var calendarGeneration = 0L
    private var calendarRequestKey: String? = null

    val discoverUiState: StateFlow<DiscoverUiState> = combine(
        combine(results, resultSources, loading) { items, sources, pending -> Triple(items, sources, pending) },
        genres,
        catalogs,
        contentTypes
    ) { (items, sources, pending), genreOptions, catalogOptions, types ->
        DiscoverUiState(items, pending, genreOptions, catalogOptions, types, sources)
    }.stateIn(scope, SharingStarted.WhileSubscribed(5_000), DiscoverUiState())

    val discoverGenres: StateFlow<List<DiscoverGenreOption>> = genres.asStateFlow()

    val calendarUiState: StateFlow<CalendarUiState> = combine(calendarItems, calendarLoading) { items, pending ->
        CalendarUiState(items, pending)
    }.stateIn(scope, SharingStarted.WhileSubscribed(5_000), CalendarUiState())

    fun clearResults() {
        results.value = emptyList()
    }

    fun catalogOptions(type: String): List<DiscoverCatalogOption> = buildDiscoverCatalogOptions(userAddons(), type)

    fun availableContentTypes(): List<String> = buildDiscoverContentTypes(userAddons())

    fun setLoading(value: Boolean) {
        loading.value = value
    }

    fun discover(type: String, catalogKey: String?, genre: String?, year: String?, rating: Float?, provider: String?, region: String?) {
        discoverJob?.cancel()
        discoverJob = scope.launch {
            loading.value = true
            try {
                val profile = activeProfile()
                val result = dispatch(
                    mapOf(
                        "type" to "discoverRequested",
                        "contentType" to type,
                        "filters" to mapOf(
                            "catalogKey" to catalogKey,
                            "genre" to genre,
                            "year" to year,
                            "rating" to rating,
                            "provider" to provider,
                            "region" to region
                        ),
                        "profile" to profile,
                        "language" to (profile?.safeLanguage ?: "en")
                    )
                )
                val state = result.state["discover"] as? Map<*, *>
                results.value = decodeList<Meta>(state?.get("results"), metaListType).distinctBy { "${it.type}:${it.id}" }
                resultSources.value = decodeObject(state?.get("resultSources"), sourceMapType) ?: emptyMap()
            } finally {
                loading.value = false
            }
        }
    }

    fun loadMore(transportUrl: String, contentType: String, catalogId: String, genre: String?) {
        if (loading.value) return
        scope.launch {
            loading.value = true
            try {
                val result = dispatch(
                    mapOf(
                        "type" to "discoverPageRequested",
                        "transportUrl" to transportUrl,
                        "contentType" to contentType,
                        "catalogId" to catalogId,
                        "skip" to results.value.size,
                        "genre" to genre
                    )
                )
                val state = result.state["discover"] as? Map<*, *>
                val updated = decodeList<Meta>(state?.get("results"), metaListType).distinctBy { "${it.type}:${it.id}" }
                val source = HomeCatalogSource(transportUrl, catalogId, contentType, genre)
                val sources = resultSources.value.toMutableMap()
                updated.forEach { item ->
                    sources.putIfAbsent("${item.type}:${item.id}", source)
                    sources.putIfAbsent(item.id, source)
                }
                results.value = updated
                resultSources.value = sources
            } finally {
                loading.value = false
            }
        }
    }

    fun loadCalendar(profile: UserProfile?, year: Int, month: Int, plannedItems: List<Meta>) {
        val requestKey = "${profile?.id.orEmpty()}:$year:$month"
        if (calendarRequestKey == requestKey && calendarJob?.isActive == true) return
        calendarJob?.cancel()
        val generation = ++calendarGeneration
        if (calendarRequestKey != requestKey) {
            calendarItems.value = emptyList()
        }
        calendarRequestKey = requestKey
        calendarJob = scope.launch {
            calendarLoading.value = true
            try {
                val result = dispatch(
                    mapOf(
                        "type" to "calendarMonthRequested",
                        "profile" to profile,
                        "year" to year,
                        "month" to month,
                        "plannedItems" to plannedItems
                    )
                )
                val state = result.state["calendar"] as? Map<*, *>
                if (generation == calendarGeneration) {
                    calendarItems.value = decodeList(state?.get("items"), calendarItemListType)
                }
            } finally {
                if (generation == calendarGeneration) {
                    calendarLoading.value = false
                }
            }
        }
    }

    fun clearGenres() {
        genres.value = emptyList()
    }

    fun loadFilters(type: String, selectedCatalogKey: String?, onLoaded: ((List<DiscoverCatalogOption>) -> Unit)?) {
        scope.launch {
            val profile = activeProfile()
            val result = dispatch(
                mapOf(
                    "type" to "discoverCatalogFiltersRequested",
                    "contentType" to type,
                    "selectedCatalogKey" to selectedCatalogKey,
                    "profile" to profile,
                    "language" to (profile?.safeLanguage ?: "en")
                )
            )
            val state = result.state["discover"] as? Map<*, *> ?: return@launch
            val updatedCatalogs = decodeList<DiscoverCatalogOption>(state["catalogs"], catalogListType)
            catalogs.value = updatedCatalogs
            genres.value = decodeList(state["genres"], genreListType)
            contentTypes.value = decodeList(state["contentTypes"], contentTypeListType)
            onLoaded?.invoke(updatedCatalogs)
        }
    }

    private suspend fun <T> decodeList(value: Any?, type: java.lang.reflect.Type): List<T> = withContext(Dispatchers.Default) {
        if (value == null) emptyList() else runCatching { gson.fromJson<List<T>>(gson.toJson(value), type) }.getOrDefault(emptyList())
    }

    private suspend fun <T> decodeObject(value: Any?, type: java.lang.reflect.Type): T? = withContext(Dispatchers.Default) {
        if (value == null) null else runCatching { gson.fromJson<T>(gson.toJson(value), type) }.getOrNull()
    }
}
