package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import com.fluxa.app.core.StremioId
import com.fluxa.app.domain.playback.PlaybackSyncCoordinator
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

internal data class ContinueWatchingSnapshot(
    val localItems: List<Meta>,
    val externalItems: List<Meta>
)

internal class HomePlaybackController(
    private val watchlistManager: WatchlistManager,
    private val forgottenStore: ForgottenContinueWatchingStore,
    private val playbackSyncCoordinator: PlaybackSyncCoordinator,
    private val scope: CoroutineScope,
    private val activeProfile: () -> UserProfile?,
    private val localContinueWatching: () -> List<Meta>,
    private val externalContinueWatching: () -> List<Meta>,
    private val onContinueWatchingChanged: (ContinueWatchingSnapshot) -> Unit,
    private val refreshDynamicRows: () -> Unit
) {
    private val forgottenKeys = mutableSetOf<String>()

    fun loadForgotten(profile: UserProfile?) {
        forgottenKeys.clear()
        forgottenKeys.addAll(forgottenStore.load(profile))
    }

    fun isForgotten(meta: Meta): Boolean {
        return ContinueWatchingListMerger.identityKey(meta) in forgottenKeys
    }

    fun saveProgress(
        meta: Meta,
        timeOffset: Long,
        duration: Long,
        videoId: String? = null,
        streamIndex: Int? = null,
        episodeName: String? = null,
        lastStreamUrl: String? = null,
        lastStreamTitle: String? = null,
        lastBingeGroup: String? = null,
        lastAudioLanguage: String? = null,
        lastSubtitleLanguage: String? = null,
        scrobbleTraktPause: Boolean = true
    ) {
        scope.launch {
            val profile = activeProfile()
            syncWatchlistProfile(profile)
            forgottenKeys.remove(ContinueWatchingListMerger.identityKey(meta))
            forgottenStore.save(profile, forgottenKeys.toSet())
            watchlistManager.savePlaybackProgress(
                meta,
                timeOffset,
                duration,
                videoId,
                streamIndex,
                episodeName,
                lastStreamUrl,
                lastStreamTitle,
                lastBingeGroup,
                lastAudioLanguage,
                lastSubtitleLanguage
            )
            if (profile != null) {
                playbackSyncCoordinator.scheduleProgress(
                    profile = profile,
                    meta = meta,
                    videoId = videoId,
                    positionMs = timeOffset,
                    durationMs = duration,
                    allowPauseScrobble = scrobbleTraktPause
                )
            }
        }
    }

    fun scrobble(token: String, metaType: String, itemId: String, progress: Float, action: String) {
        val profile = activeProfile() ?: return
        if (token.isBlank()) return
        playbackSyncCoordinator.scheduleTraktScrobble(
            profile = profile,
            mediaType = metaType,
            mediaId = itemId,
            progress = progress,
            action = action
        )
    }

    fun markWatched(meta: Meta, videoId: String? = null, episodeName: String? = null, nextEpisode: Video? = null) {
        scope.launch {
            val profile = activeProfile()
            syncWatchlistProfile(profile)
            val episodes = if (meta.type == "series" && !videoId.isNullOrBlank()) {
                val parsed = StremioId.parseEpisodeLocator(videoId)
                listOf(
                    Video(
                        id = videoId,
                        name = episodeName,
                        season = parsed?.first,
                        number = parsed?.second,
                        released = null,
                        thumbnail = meta.background
                    )
                )
            } else {
                emptyList()
            }
            if (meta.type == "series" && !videoId.isNullOrBlank()) {
                watchlistManager.markEpisodeWatched(meta.id, videoId, true)
            }
            if (meta.type == "series" && nextEpisode != null) {
                watchlistManager.savePlaybackProgress(
                    meta.copy(
                        continueWatchingPoster = nextEpisode.thumbnail ?: meta.continueWatchingPoster,
                        continueWatchingBackground = nextEpisode.thumbnail ?: meta.continueWatchingBackground
                    ),
                    timeOffset = 0L,
                    duration = 0L,
                    lastVideoId = nextEpisode.id,
                    lastEpisodeName = nextEpisode.continueWatchingTitle()
                )
            }
            if (profile != null) {
                playbackSyncCoordinator.pushWatched(profile, meta, episodes, watched = true)
            }
        }
    }

    fun forgetProgress(meta: Meta) {
        scope.launch {
            val profile = activeProfile()
            syncWatchlistProfile(profile)
            val forgottenKey = ContinueWatchingListMerger.identityKey(meta)
            val localSource = playbackSyncCoordinator.isLocalContinueWatchingSource(profile)
            forgottenKeys.add(forgottenKey)
            forgottenStore.save(profile, forgottenKeys.toSet())
            watchlistManager.clearPlaybackProgress(meta.id)
            val providerCleared = if (profile != null) {
                playbackSyncCoordinator.clearProgress(profile, meta)
            } else {
                false
            }
            val localItems = if (localSource) {
                localContinueWatching().map { item ->
                    if (ContinueWatchingListMerger.identityKey(item) == forgottenKey) {
                        item.copy(
                            timeOffset = 0L,
                            duration = 0L,
                            lastVideoId = null,
                            lastStreamIndex = null,
                            lastEpisodeName = null,
                            lastStreamUrl = null,
                            lastStreamTitle = null,
                            lastAudioLanguage = null,
                            lastSubtitleLanguage = null
                        )
                    } else item
                }
            } else localContinueWatching()
            val externalItems = when {
                localSource -> externalContinueWatching()
                providerCleared && profile != null ->
                    playbackSyncCoordinator.cachedContinueWatching(profile)
                else -> externalContinueWatching()
            }
            onContinueWatchingChanged(ContinueWatchingSnapshot(localItems, externalItems))
            refreshDynamicRows()
        }
    }

    private fun syncWatchlistProfile(profile: UserProfile?) {
        profile?.id?.let(watchlistManager::setActiveProfile)
    }
}

private fun Video.continueWatchingTitle(): String {
    val seasonEpisode = if (season != null && number != null) "S$season:E$number" else null
    return listOfNotNull(seasonEpisode, name?.takeIf { it.isNotBlank() }).joinToString(" ").ifBlank { name.orEmpty() }
}
