package com.fluxa.app.ui.catalog

import android.util.Log
import com.fluxa.app.core.rust.FluxaUniFfiCoreStateHandle
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.TraktIntegration
import com.fluxa.app.data.repository.ExternalSyncMergeBridge
import com.fluxa.app.data.repository.ExternalSyncPushCoordinator
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.TraktRepository
import com.fluxa.app.data.repository.library.ProviderAdapters
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
    val traktFavorites: List<Meta> = emptyList(),
    val simklWatching: List<Meta> = emptyList(),
    val simklPlanned: List<Meta> = emptyList(),
    val simklCompleted: List<Meta> = emptyList(),
    val anilistPlanned: List<Meta> = emptyList(),
    val anilistWatching: List<Meta> = emptyList(),
    val anilistCompleted: List<Meta> = emptyList(),
    val stremioPlanned: List<Meta> = emptyList(),
    val nuvioPlanned: List<Meta> = emptyList(),
    val errorMessage: String? = null,
    val lastLoadedProfileKey: String? = null
)

internal class HomeLibraryCoordinator(
    private val repository: StremioRepository,
    private val traktRepository: TraktRepository,
    private val watchlistManager: WatchlistManager,
    private val pushCoordinator: ExternalSyncPushCoordinator,
    private val adapters: ProviderAdapters,
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
                val profile = activeProfile
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

                val traktConnected = profile != null && adapters.trakt.isConnected(profile)
                val traktPlannedWithListedAt = if (traktConnected) async(Dispatchers.IO) { traktRepository.getTraktWatchlistWithListedAt(profile.traktAccessToken!!) } else null
                val traktWatched = if (traktConnected) async(Dispatchers.IO) { adapters.trakt.fetchWatched(profile) } else null
                val traktWatchedEpisodeTimestamps = if (traktConnected) async(Dispatchers.IO) { adapters.trakt.fetchWatchedEpisodeTimestamps(profile) } else null
                val traktCollection = if (traktConnected) async(Dispatchers.IO) { traktRepository.getTraktCollection(profile.traktAccessToken!!) } else null
                val traktFavorites = if (traktConnected && profile.integrationLibrarySource == "trakt") {
                    async(Dispatchers.IO) { adapters.trakt.fetchFavorites(profile) }
                } else null

                val simklConnected = profile != null && adapters.simkl.isConnected(profile)
                val simklWatching = if (simklConnected) async(Dispatchers.IO) { repository.getSimklLibraryItems(profile, "watching") } else null
                val simklPlanned = if (simklConnected) async(Dispatchers.IO) { adapters.simkl.fetchWatchlist(profile) } else null
                val simklCompleted = if (simklConnected) async(Dispatchers.IO) { adapters.simkl.fetchWatched(profile) } else null
                val simklWatchedEpisodeTimestamps = if (simklConnected) async(Dispatchers.IO) { adapters.simkl.fetchWatchedEpisodeTimestamps(profile) } else null

                val anilistConnected = profile != null && adapters.anilist.isConnected(profile)
                val anilistSnapshot = if (anilistConnected) async(Dispatchers.IO) { repository.getAnilistLibrarySnapshot(profile.anilistAccessToken) } else null

                val stremioPlanned = if (profile != null && adapters.stremio.isConnected(profile)) async(Dispatchers.IO) { adapters.stremio.fetchWatchlist(profile) } else null
                val nuvioPlanned = if (profile != null && adapters.nuvio.isConnected(profile)) async(Dispatchers.IO) { adapters.nuvio.fetchWatchlist(profile) } else null

                val traktPlannedWithTimestamps = traktPlannedWithListedAt?.await().orEmpty()
                val anilistLibrary = anilistSnapshot?.await()
                val anilistWatchlistWithTimestamps = anilistLibrary?.watchlist.orEmpty()

                setLibraryState(LibraryUiState(
                    continueItems = (stremioContinue.await() + externalContinue.await()).distinctBy { it.id },
                    traktPlanned = traktPlannedWithTimestamps.map { it.first },
                    traktWatched = traktWatched?.await().orEmpty(),
                    traktCollection = traktCollection?.await().orEmpty(),
                    traktFavorites = traktFavorites?.await().orEmpty(),
                    simklWatching = simklWatching?.await().orEmpty(),
                    simklPlanned = simklPlanned?.await().orEmpty(),
                    simklCompleted = simklCompleted?.await().orEmpty(),
                    anilistPlanned = anilistWatchlistWithTimestamps.map { it.first },
                    anilistWatching = anilistLibrary?.watching.orEmpty(),
                    anilistCompleted = anilistLibrary?.completed.orEmpty(),
                    stremioPlanned = stremioPlanned?.await().orEmpty(),
                    nuvioPlanned = nuvioPlanned?.await().orEmpty(),
                    lastLoadedProfileKey = profileKey
                ))

                if (profile != null && traktConnected) {
                    scope.launch(Dispatchers.IO) {
                        runCatching { pushLocalWatchlistDelta(profile, traktPlannedWithTimestamps) }
                            .onFailure { Log.w("HomeLibrary", "Trakt watchlist push failed", it) }
                    }
                    scope.launch(Dispatchers.IO) {
                        val remoteWatched = traktWatchedEpisodeTimestamps?.await().orEmpty()
                        runCatching { pushLocalWatchedDelta(profile, remoteWatched) }
                            .onFailure { Log.w("HomeLibrary", "Trakt watched push failed", it) }
                    }
                }
                if (profile != null && simklConnected) {
                    scope.launch(Dispatchers.IO) {
                        val remoteWatched = simklWatchedEpisodeTimestamps?.await().orEmpty()
                        runCatching { pushLocalWatchedDelta(profile, remoteWatched) }
                            .onFailure { Log.w("HomeLibrary", "Simkl watched push failed", it) }
                    }
                }
                if (profile != null && anilistConnected) {
                    scope.launch(Dispatchers.IO) {
                        runCatching { pushLocalWatchlistDelta(profile, anilistWatchlistWithTimestamps) }
                            .onFailure { Log.w("HomeLibrary", "AniList watchlist push failed", it) }
                    }
                }
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

    private suspend fun pushLocalWatchlistDelta(profile: UserProfile, remoteEntries: List<Pair<Meta, Long>>) {
        val localSnapshot = watchlistManager.getWatchlistMembershipSnapshot()
        val remoteMembership = remoteEntries.map { (meta, listedAtMs) ->
            ExternalSyncMergeBridge.RemoteMembershipItem(meta.id, listedAtMs)
        }
        val plan = ExternalSyncMergeBridge.mergeWatchlist(localSnapshot, remoteMembership)

        plan.pushRemoteAdd.forEach { id ->
            val meta = watchlistManager.getContentMeta(id) ?: return@forEach
            pushCoordinator.pushWatchlist(profile, meta, isInWatchlist = true)
        }
        plan.pushRemoteRemove.forEach { id ->
            val meta = watchlistManager.getContentMeta(id) ?: return@forEach
            pushCoordinator.pushWatchlist(profile, meta, isInWatchlist = false)
        }
    }

    private suspend fun pushLocalWatchedDelta(profile: UserProfile, remoteWatched: Map<String, Long>) {
        val localSnapshot = watchlistManager.getWatchedEpisodeMembershipSnapshot()
        val remoteMembership = remoteWatched.map { (videoId, watchedAtMs) ->
            ExternalSyncMergeBridge.RemoteMembershipItem(videoId, watchedAtMs)
        }
        val plan = ExternalSyncMergeBridge.mergeWatched(localSnapshot, remoteMembership)

        plan.pushRemoteAdd.forEach { videoId ->
            val seriesId = TraktIntegration.showIdFromEpisodeId(videoId)
            val meta = watchlistManager.getContentMeta(seriesId) ?: return@forEach
            pushCoordinator.pushMarkWatched(profile, meta, listOf(Video(id = videoId)), watched = true)
        }
        plan.pushRemoteRemove.forEach { videoId ->
            val seriesId = TraktIntegration.showIdFromEpisodeId(videoId)
            val meta = watchlistManager.getContentMeta(seriesId) ?: return@forEach
            pushCoordinator.pushMarkWatched(profile, meta, listOf(Video(id = videoId)), watched = false)
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
            this?.simklAccessToken.orEmpty(),
            this?.safeLanguage.orEmpty()
        ).joinToString("|")
    }
}
