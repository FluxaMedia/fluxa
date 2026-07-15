package com.fluxa.app.ui.catalog

import android.util.Log
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.TraktIntegration
import com.fluxa.app.data.repository.TraktRepository
import com.fluxa.app.data.repository.TraktWatchedState
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull

internal class HomeTraktCoordinator(
    private val traktRepository: TraktRepository,
    private val watchlistManager: WatchlistManager,
    private val scope: CoroutineScope,
    private val activeProfile: () -> UserProfile?,
    private val setActiveProfile: (UserProfile) -> Unit,
    private val setWatchedState: (TraktWatchedState) -> Unit,
    private val setExternalContinueWatching: (List<Meta>) -> Unit,
    private val fetchExternalContinueWatching: suspend (UserProfile?) -> List<Meta>,
    private val prefetchContinueWatchingArtwork: (List<Meta>) -> Unit,
    private val refreshDynamicRows: () -> Unit
) {
    private val refreshMutex = Mutex()

    fun refreshTokenIfNeeded(profile: UserProfile, onProfileUpdated: (UserProfile) -> Unit) {
        val refreshToken = profile.traktRefreshToken?.takeIf { it.isNotBlank() } ?: return
        val expiresAt = profile.safeTraktTokenExpiresAt
        val refreshWindowMs = 24L * 60L * 60L * 1000L
        if (!profile.traktAccessToken.isNullOrBlank() && expiresAt > System.currentTimeMillis() + refreshWindowMs) return
        scope.launch(Dispatchers.IO) {
            refreshMutex.lock()
            try {
                val latest = activeProfile()?.takeIf { it.id == profile.id } ?: profile
                val latestRefreshToken = latest.traktRefreshToken?.takeIf { it.isNotBlank() } ?: return@launch
                if (!latest.traktAccessToken.isNullOrBlank() && latest.safeTraktTokenExpiresAt > System.currentTimeMillis() + refreshWindowMs) return@launch
                runCatching {
                    val response = traktRepository.refreshTraktToken(latestRefreshToken)
                    latest.copy(
                        traktAccessToken = response.accessToken,
                        traktRefreshToken = response.refreshToken,
                        traktTokenExpiresAt = TraktIntegration.tokenExpiresAt(response.createdAt, response.expiresIn)
                    )
                }.onSuccess { updated ->
                    setActiveProfile(updated)
                    withContext(Dispatchers.Main) { onProfileUpdated(updated) }
                }.onFailure { error ->
                    Log.w("Trakt", "Token refresh failed", error)
                    val status = (error as? retrofit2.HttpException)?.code()
                    if (status == 400 || status == 401) {
                        val cleared = latest.copy(
                            traktAccessToken = null,
                            traktRefreshToken = null,
                            traktTokenExpiresAt = null
                        )
                        setActiveProfile(cleared)
                        withContext(Dispatchers.Main) { onProfileUpdated(cleared) }
                    }
                }
            } finally {
                refreshMutex.unlock()
            }
        }
    }

    suspend fun syncWatchedMirror(profile: UserProfile?) {
        val token = profile?.traktAccessToken?.takeIf { it.isNotBlank() } ?: run {
            setWatchedState(TraktWatchedState())
            return
        }
        val watchedState = withTimeoutOrNull(15_000L) {
            traktRepository.getTraktWatchedState(token)
        } ?: TraktWatchedState()
        setWatchedState(watchedState)
        watchlistManager.replaceExternalWatchedEpisodes("trakt", watchedState.episodeIdsBySeries)
        watchlistManager.replaceExternalWatchedContentDurations("trakt", watchedState.durationRecords)
    }

    fun refreshExternalContinueWatching() {
        val profile = activeProfile()
        if (!profile.hasExternalContinueProvider()) return
        scope.launch(Dispatchers.IO) {
            syncWatchedMirror(profile)
            val externalItems = fetchExternalContinueWatching(profile)
            setExternalContinueWatching(externalItems)
            prefetchContinueWatchingArtwork(externalItems)
            refreshDynamicRows()
        }
    }

    fun syncIntegration(
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        if (profile.traktAccessToken.isNullOrBlank()) {
            onComplete(false)
            return
        }
        scope.launch(Dispatchers.IO) {
            val snapshot = runCatching {
                traktRepository.getTraktSyncSnapshot(profile, profile.safeLanguage)
            }.onFailure {
                Log.w("HomeTrakt", "Failed to load Trakt sync snapshot", it)
            }.getOrNull()
            if (snapshot == null) {
                withContext(Dispatchers.Main) { onComplete(false) }
                return@launch
            }
            syncWatchedMirror(profile)
            val externalItems = fetchExternalContinueWatching(profile)
            setExternalContinueWatching(externalItems)
            prefetchContinueWatchingArtwork(externalItems)
            val updated = profile.copy(
                traktLastSyncAt = System.currentTimeMillis(),
                traktLastSyncedItems = snapshot.syncedItems,
                traktLastContinueWatchingCount = snapshot.continueWatchingCount,
                traktLastWatchlistCount = snapshot.watchlistCount
            )
            setActiveProfile(updated)
            refreshDynamicRows()
            withContext(Dispatchers.Main) {
                onProfileUpdated(updated)
                onComplete(true)
            }
        }
    }

    private fun UserProfile?.hasExternalContinueProvider(): Boolean {
        return this != null &&
            (!traktAccessToken.isNullOrBlank() || !malAccessToken.isNullOrBlank() || !simklAccessToken.isNullOrBlank())
    }
}
