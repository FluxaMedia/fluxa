package com.fluxa.app.ui.catalog

import android.util.Log
import com.fluxa.app.core.rust.FluxaUniFfiCoreStateHandle
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.providerAccountId
import com.fluxa.app.data.local.safeContinueWatchingSource
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.library.ThirdPartyProviderRepository
import com.fluxa.app.data.repository.library.ThirdPartyProviderSnapshot
import com.google.gson.Gson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class LibraryUiState(
    val isLoading: Boolean = false,
    val continueItems: List<Meta> = emptyList(),
    val traktPlanned: List<Meta> = emptyList(),
    val traktWatched: List<Meta> = emptyList(),
    val traktCollection: List<Meta> = emptyList(),
    val traktFavorites: List<Meta> = emptyList(),
    val simklWatching: List<Meta> = emptyList(),
    val simklPlanned: List<Meta> = emptyList(),
    val simklCompleted: List<Meta> = emptyList(),
    val anilistPlanned: List<Meta> = emptyList(),
    val anilistWatching: List<Meta> = emptyList(),
    val anilistCompleted: List<Meta> = emptyList(),
    val stremioPlanned: List<Meta> = emptyList(),
    val stremioContinue: List<Meta> = emptyList(),
    val nuvioPlanned: List<Meta> = emptyList(),
    val nuvioContinue: List<Meta> = emptyList(),
    val traktContinue: List<Meta> = emptyList(),
    val simklContinue: List<Meta> = emptyList(),
    val anilistContinue: List<Meta> = emptyList(),
    val errorMessage: String? = null,
    val lastLoadedProfileKey: String? = null
)

/**
 * Android presentation facade over the JVM-common third-party repository.
 * Provider snapshots remain isolated; this class only maps each snapshot into
 * the legacy UI fields expected by existing screens.
 */
internal class HomeLibraryCoordinator(
    private val providerRepository: ThirdPartyProviderRepository,
    private val scope: CoroutineScope,
    private val coreState: FluxaUniFfiCoreStateHandle,
    private val gson: Gson
) {
    private val _state = MutableStateFlow(LibraryUiState())
    val state: StateFlow<LibraryUiState> = _state.asStateFlow()

    fun load(activeProfile: UserProfile?, force: Boolean = false) {
        val profileKey = activeProfile.libraryProfileKey()
        val current = _state.value
        if (current.isLoading || (!force && current.lastLoadedProfileKey == profileKey)) return

        scope.launch {
            setLibraryState(current.copy(isLoading = true, errorMessage = null))
            try {
                if (activeProfile == null) {
                    setLibraryState(LibraryUiState(lastLoadedProfileKey = profileKey))
                    return@launch
                }

                val snapshots = providerRepository.loadConnected(activeProfile, refresh = true)
                val selectedContinueProvider = ThirdPartyProviderId.from(
                    activeProfile.safeContinueWatchingSource
                )
                val selectedContinue = selectedContinueProvider
                    ?.let { snapshots[it.key]?.continueWatching }
                    .orEmpty()

                setLibraryState(
                    snapshots.toLegacyUiState(
                        selectedContinue = selectedContinue,
                        profileKey = profileKey
                    )
                )
            } catch (error: Exception) {
                Log.w("HomeLibrary", "Failed to load isolated provider snapshots", error)
                setLibraryState(
                    _state.value.copy(
                        isLoading = false,
                        errorMessage = error.message,
                        lastLoadedProfileKey = profileKey
                    )
                )
            }
        }
    }

    private fun setLibraryState(value: LibraryUiState) {
        val snapshotJson = coreState.dispatch(CoreAction(type = "setLibraryUiState", value = value))
        val snapshot = gson.fromJson(snapshotJson, CoreStateSnapshot::class.java)?.library ?: return
        _state.value = snapshot.uiState
    }

    private data class CoreAction(
        val type: String,
        val value: Any?
    )

    private data class CoreStateSnapshot(
        val library: CoreLibrarySnapshot = CoreLibrarySnapshot()
    )

    private data class CoreLibrarySnapshot(
        val uiState: LibraryUiState = LibraryUiState()
    )

    private fun UserProfile?.libraryProfileKey(): String {
        if (this == null) return "none"
        return buildList {
            add(id)
            add(safeLanguage)
            add(safeContinueWatchingSource)
            add(integrationLibrarySource.orEmpty())
            ThirdPartyProviderId.entries.forEach { provider ->
                add(provider.key)
                add(providerAccountId(provider).orEmpty())
            }
        }.joinToString("|")
    }
}

private fun Map<String, ThirdPartyProviderSnapshot>.toLegacyUiState(
    selectedContinue: List<Meta>,
    profileKey: String
): LibraryUiState {
    val trakt = this[ThirdPartyProviderId.TRAKT.key]
    val simkl = this[ThirdPartyProviderId.SIMKL.key]
    val anilist = this[ThirdPartyProviderId.ANILIST.key]
    val stremio = this[ThirdPartyProviderId.STREMIO.key]
    val nuvio = this[ThirdPartyProviderId.NUVIO.key]

    return LibraryUiState(
        isLoading = false,
        continueItems = selectedContinue,
        traktPlanned = trakt?.planned.orEmpty(),
        traktWatched = trakt?.completed.orEmpty(),
        traktCollection = trakt?.collection.orEmpty(),
        traktFavorites = trakt?.favorites.orEmpty(),
        simklWatching = simkl?.watching.orEmpty(),
        simklPlanned = simkl?.planned.orEmpty(),
        simklCompleted = simkl?.completed.orEmpty(),
        anilistPlanned = anilist?.planned.orEmpty(),
        anilistWatching = anilist?.watching.orEmpty(),
        anilistCompleted = anilist?.completed.orEmpty(),
        stremioPlanned = stremio?.planned.orEmpty(),
        stremioContinue = stremio?.continueWatching.orEmpty(),
        nuvioPlanned = nuvio?.planned.orEmpty(),
        nuvioContinue = nuvio?.continueWatching.orEmpty(),
        traktContinue = trakt?.continueWatching.orEmpty(),
        simklContinue = simkl?.continueWatching.orEmpty(),
        anilistContinue = anilist?.continueWatching.orEmpty(),
        lastLoadedProfileKey = profileKey
    )
}
