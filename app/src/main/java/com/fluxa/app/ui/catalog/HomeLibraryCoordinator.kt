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
    private val watchlistManager: WatchlistManager,
    private val pushCoordinator: ExternalSyncPushCoordinator,
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
                val traktPlannedWithListedAt = if (!traktToken.isNullOrBlank()) async(Dispatchers.IO) { traktRepository.getTraktWatchlistWithListedAt(traktToken) } else null
                val traktWatched = if (!traktToken.isNullOrBlank()) async(Dispatchers.IO) { traktRepository.getTraktRecentlyWatched(traktToken, language, profile) } else null
                val traktWatchedEpisodesWithTimestamps = if (!traktToken.isNullOrBlank()) async(Dispatchers.IO) { traktRepository.getTraktWatchedEpisodesWithTimestamps(traktToken) } else null
                val traktCollection = if (!traktToken.isNullOrBlank()) async(Dispatchers.IO) { traktRepository.getTraktCollection(traktToken) } else null

                val malToken = profile?.malAccessToken
                val malWatching = if (!malToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getMalLibraryItems(malToken, "watching") } else null
                val malPlanned = if (!malToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getMalLibraryItems(malToken, "plan_to_watch") } else null
                val malCompleted = if (!malToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getMalLibraryItems(malToken, "completed") } else null

                val simklToken = profile?.simklAccessToken
                val simklWatching = if (!simklToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getSimklLibraryItems(simklToken, "watching") } else null
                val simklPlanned = if (!simklToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getSimklLibraryItems(simklToken, "plantowatch") } else null
                val simklCompleted = if (!simklToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getSimklLibraryItems(simklToken, "completed") } else null
                val simklWatchedEpisodesWithTimestamps = if (!simklToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getSimklWatchedEpisodesWithTimestamps(simklToken) } else null

                val anilistToken = profile?.anilistAccessToken
                val anilistWatchlistWithTimestamps = if (!anilistToken.isNullOrBlank()) async(Dispatchers.IO) { repository.getAnilistWatchlistWithTimestamps(anilistToken) } else null

                val traktPlannedWithTimestamps = traktPlannedWithListedAt?.await().orEmpty()

                setLibraryState(LibraryUiState(
                    continueItems = (stremioContinue.await() + externalContinue.await()).distinctBy { it.id },
                    traktPlanned = traktPlannedWithTimestamps.map { it.first },
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

                if (profile != null && !traktToken.isNullOrBlank()) {
                    scope.launch(Dispatchers.IO) {
                        runCatching { reconcileWatchlist(profile, traktPlannedWithTimestamps) }
                            .onFailure { Log.w("HomeLibrary", "Trakt watchlist reconcile failed", it) }
                    }
                    scope.launch(Dispatchers.IO) {
                        val remoteWatched = traktWatchedEpisodesWithTimestamps?.await().orEmpty()
                        runCatching { reconcileWatchedEpisodes(profile, remoteWatched) }
                            .onFailure { Log.w("HomeLibrary", "Trakt watched reconcile failed", it) }
                    }
                }
                if (profile != null && !simklToken.isNullOrBlank()) {
                    scope.launch(Dispatchers.IO) {
                        val remotePlanned = simklPlanned?.await().orEmpty()
                        runCatching { remotePlanned.forEach { watchlistManager.applyRemoteWatchlistAdd(it) } }
                            .onFailure { Log.w("HomeLibrary", "Simkl watchlist reconcile failed", it) }
                    }
                    scope.launch(Dispatchers.IO) {
                        val remoteWatched = simklWatchedEpisodesWithTimestamps?.await().orEmpty()
                        runCatching { reconcileWatchedEpisodes(profile, remoteWatched) }
                            .onFailure { Log.w("HomeLibrary", "Simkl watched reconcile failed", it) }
                    }
                }
                if (profile != null && !anilistToken.isNullOrBlank()) {
                    scope.launch(Dispatchers.IO) {
                        val remotePlanned = anilistWatchlistWithTimestamps?.await().orEmpty()
                        runCatching { reconcileWatchlist(profile, remotePlanned) }
                            .onFailure { Log.w("HomeLibrary", "AniList watchlist reconcile failed", it) }
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

    private suspend fun reconcileWatchlist(profile: UserProfile, remoteEntries: List<Pair<Meta, Long>>) {
        val localSnapshot = watchlistManager.getWatchlistMembershipSnapshot()
        val remoteMembership = remoteEntries.map { (meta, listedAtMs) ->
            ExternalSyncMergeBridge.RemoteMembershipItem(meta.id, listedAtMs)
        }
        val plan = ExternalSyncMergeBridge.mergeWatchlist(localSnapshot, remoteMembership)
        val remoteById = remoteEntries.associateBy { it.first.id }

        plan.applyLocalAdd.forEach { id ->
            remoteById[id]?.first?.let { watchlistManager.applyRemoteWatchlistAdd(it) }
        }
        plan.pushRemoteAdd.forEach { id ->
            val meta = watchlistManager.getContentMeta(id) ?: return@forEach
            pushCoordinator.pushWatchlist(profile, meta, isInWatchlist = true)
        }
        plan.pushRemoteRemove.forEach { id ->
            val meta = watchlistManager.getContentMeta(id) ?: return@forEach
            pushCoordinator.pushWatchlist(profile, meta, isInWatchlist = false)
        }
    }

    private suspend fun reconcileWatchedEpisodes(profile: UserProfile, remoteWatched: Map<String, Long>) {
        val localSnapshot = watchlistManager.getWatchedEpisodeMembershipSnapshot()
        val remoteMembership = remoteWatched.map { (videoId, watchedAtMs) ->
            ExternalSyncMergeBridge.RemoteMembershipItem(videoId, watchedAtMs)
        }
        val plan = ExternalSyncMergeBridge.mergeWatched(localSnapshot, remoteMembership)

        plan.applyLocalAdd.forEach { videoId ->
            val seriesId = TraktIntegration.showIdFromEpisodeId(videoId)
            watchlistManager.markEpisodesWatched(seriesId, listOf(videoId), watched = true)
        }
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
            this?.malAccessToken.orEmpty(),
            this?.simklAccessToken.orEmpty(),
            this?.safeLanguage.orEmpty()
        ).joinToString("|")
    }
}
