package com.fluxa.app.ui.catalog

import android.util.Log
import com.fluxa.app.core.rust.FluxaUniFfiCoreStateHandle
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.TraktRepository
import com.google.gson.Gson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
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
    val malWatching: List<Meta> = emptyList(),
    val malPlanned: List<Meta> = emptyList(),
    val malCompleted: List<Meta> = emptyList(),
    val simklWatching: List<Meta> = emptyList(),
    val simklPlanned: List<Meta> = emptyList(),
    val simklCompleted: List<Meta> = emptyList(),
    val errorMessage: String? = null,
    val lastLoadedProfileKey: String? = null
)

internal class HomeLibraryCoordinator(
    private val repository: StremioRepository,
    private val traktRepository: TraktRepository,
    private val scope: CoroutineScope,
    private val coreState: FluxaUniFfiCoreStateHandle,
    private val gson: Gson
) {
    private val _state = MutableStateFlow(LibraryUiState())
    val state: StateFlow<LibraryUiState> = _state.asStateFlow()

    fun load(activeProfile: UserProfile?) {
        val profileKey = activeProfile.libraryProfileKey()
        val current = _state.value
        if (current.isLoading || current.lastLoadedProfileKey == profileKey) return

        scope.launch {
            setLibraryState(current.copy(isLoading = true, errorMessage = null))
            try {
                val profile = activeProfile
                val language = profile?.safeLanguage ?: "en"
                val stremioContinue = async(Dispatchers.IO) {
                    val authKey = profile?.authKey
                    if (!authKey.isNullOrBlank()) {
                        runCatching { repository.getLibraryItems(authKey) }
                            .onFailure { Log.w("HomeLibrary", "Failed to load Stremio library items", it) }
                            .getOrDefault(emptyList())
                    } else {
                        emptyList()
                    }
                }
                val externalContinue = async(Dispatchers.IO) {
                    profile?.let {
                        runCatching { repository.getExternalContinueWatching(it, it.safeLanguage) }
                            .onFailure { error -> Log.w("HomeLibrary", "Failed to load external continue watching", error) }
                            .getOrDefault(emptyList())
                    } ?: emptyList()
                }

                val traktToken = profile?.traktAccessToken
                val traktPlanned = if (!traktToken.isNullOrBlank()) async(Dispatchers.IO) { traktRepository.getTraktWatchlist(traktToken) } else null
                val traktWatched = if (!traktToken.isNullOrBlank()) async(Dispatchers.IO) { traktRepository.getTraktRecentlyWatched(traktToken, language, profile) } else null
                val traktCollection = if (!traktToken.isNullOrBlank()) async(Dispatchers.IO) { traktRepository.getTraktCollection(traktToken) } else null

                val malToken = profile?.malAccessToken
                val malWatching = if (!malToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getMalLibraryItems(malToken, "watching") } else null
                val malPlanned = if (!malToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getMalLibraryItems(malToken, "plan_to_watch") } else null
                val malCompleted = if (!malToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getMalLibraryItems(malToken, "completed") } else null

                val simklToken = profile?.simklAccessToken
                val simklWatching = if (!simklToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getSimklLibraryItems(simklToken, "watching") } else null
                val simklPlanned = if (!simklToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getSimklLibraryItems(simklToken, "plantowatch") } else null
                val simklCompleted = if (!simklToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getSimklLibraryItems(simklToken, "completed") } else null

                setLibraryState(LibraryUiState(
                    continueItems = (stremioContinue.await() + externalContinue.await()).distinctBy { it.id },
                    traktPlanned = traktPlanned?.await().orEmpty(),
                    traktWatched = traktWatched?.await().orEmpty(),
                    traktCollection = traktCollection?.await().orEmpty(),
                    malWatching = malWatching?.await().orEmpty(),
                    malPlanned = malPlanned?.await().orEmpty(),
                    malCompleted = malCompleted?.await().orEmpty(),
                    simklWatching = simklWatching?.await().orEmpty(),
                    simklPlanned = simklPlanned?.await().orEmpty(),
                    simklCompleted = simklCompleted?.await().orEmpty(),
                    lastLoadedProfileKey = profileKey
                ))
            } catch (e: Exception) {
                Log.w("HomeLibrary", "Failed to load library data", e)
                setLibraryState(_state.value.copy(
                    isLoading = false,
                    errorMessage = e.message,
                    lastLoadedProfileKey = profileKey
                ))
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
        return listOf(
            this?.authKey.orEmpty(),
            this?.traktAccessToken.orEmpty(),
            this?.malAccessToken.orEmpty(),
            this?.simklAccessToken.orEmpty(),
            this?.safeLanguage.orEmpty()
        ).joinToString("|")
    }
}
