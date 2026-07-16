package com.fluxa.app.data.local

import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AndroidWatchlistStore @Inject constructor(
    private val manager: WatchlistManager
) : WatchlistStore {
    override fun setActiveProfile(profileId: String) = manager.setActiveProfile(profileId)
    override fun observeWatchlist(): Flow<List<Meta>> = manager.getWatchlistFlow()
    override fun observeContinueWatching(): Flow<List<Meta>> = manager.getContinueWatchingFlow()
    override fun observeExternalContinueWatching(): Flow<List<Meta>> = manager.getExternalContinueWatchingFlow()
    override fun observeLiked(): Flow<List<Meta>> = manager.getLikedFlow()
    override fun observeTotalWatchedDuration(): Flow<Long> = manager.getTotalWatchedContentDurationFlow()
    override suspend fun watchlistSnapshot(): List<Meta> = manager.getWatchlistSnapshot()
    override suspend fun continueWatchingSnapshot(): List<Meta> = manager.getContinueWatchingSnapshot()
    override suspend fun replaceWatchlist(items: List<Meta>) = manager.replaceWatchlist(items)
    override suspend fun toggleWatchlist(item: Meta) = manager.toggleWatchlist(item)
    override suspend fun isInWatchlist(id: String): Boolean = manager.isInWatchlist(id)
    override suspend fun setFeedback(id: String, isLike: Boolean?, content: Meta?) = manager.setFeedback(id, isLike, content)
    override suspend fun feedback(id: String): Boolean? = manager.getFeedback(id)
    override suspend fun saveProgress(update: PlaybackProgressUpdate) {
        manager.savePlaybackProgress(
            meta = update.content,
            timeOffset = update.positionMs,
            duration = update.durationMs,
            lastVideoId = update.videoId,
            lastStreamIndex = update.streamIndex,
            lastEpisodeName = update.episodeName,
            lastStreamUrl = update.streamUrl,
            lastStreamTitle = update.streamTitle,
            lastBingeGroup = update.bingeGroup,
            lastAudioLanguage = update.audioLanguage,
            lastSubtitleLanguage = update.subtitleLanguage,
            updatedAt = update.updatedAtMillis ?: com.fluxa.app.common.epochMillisNow()
        )
    }
    override suspend fun progress(id: String): Meta? = manager.getPlaybackProgress(id)
    override suspend fun clearProgress(id: String) = manager.clearPlaybackProgress(id)
    override suspend fun clearAllProgress() = manager.clearAllPlaybackProgress()
}
