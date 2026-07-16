package com.fluxa.app.data.local

import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow

class InMemoryWatchlistStore : WatchlistStore {
    private val watchlist = MutableStateFlow<List<Meta>>(emptyList())
    private val progress = MutableStateFlow<List<Meta>>(emptyList())
    private val externalProgress = MutableStateFlow<List<Meta>>(emptyList())
    private val liked = MutableStateFlow<List<Meta>>(emptyList())
    private val watchedDuration = MutableStateFlow(0L)
    private val feedback = mutableMapOf<String, Boolean?>()

    override fun setActiveProfile(profileId: String) {
        Unit
    }

    override fun observeWatchlist(): Flow<List<Meta>> = watchlist
    override fun observeContinueWatching(): Flow<List<Meta>> = progress
    override fun observeExternalContinueWatching(): Flow<List<Meta>> = externalProgress
    override fun observeLiked(): Flow<List<Meta>> = liked
    override fun observeTotalWatchedDuration(): Flow<Long> = watchedDuration
    override suspend fun watchlistSnapshot(): List<Meta> = watchlist.value
    override suspend fun continueWatchingSnapshot(): List<Meta> = progress.value

    override suspend fun replaceWatchlist(items: List<Meta>) {
        watchlist.value = items.distinctBy { it.id }
    }

    override suspend fun toggleWatchlist(item: Meta) {
        watchlist.value = if (watchlist.value.any { it.id == item.id }) {
            watchlist.value.filterNot { it.id == item.id }
        } else {
            watchlist.value + item
        }
    }

    override suspend fun isInWatchlist(id: String): Boolean = watchlist.value.any { it.id == id }

    override suspend fun setFeedback(id: String, isLike: Boolean?, content: Meta?) {
        feedback[id] = isLike
        liked.value = if (isLike == true && content != null) {
            liked.value.filterNot { it.id == id } + content
        } else {
            liked.value.filterNot { it.id == id }
        }
    }

    override suspend fun feedback(id: String): Boolean? = feedback[id]

    override suspend fun saveProgress(update: PlaybackProgressUpdate) {
        val item = update.content.copy(
            timeOffset = update.positionMs,
            duration = update.durationMs,
            lastVideoId = update.videoId,
            lastStreamIndex = update.streamIndex,
            lastEpisodeName = update.episodeName,
            lastStreamUrl = update.streamUrl,
            lastStreamTitle = update.streamTitle,
            lastBingeGroup = update.bingeGroup,
            lastAudioLanguage = update.audioLanguage,
            lastSubtitleLanguage = update.subtitleLanguage
        )
        progress.value = progress.value.filterNot { it.id == item.id } + item
    }

    override suspend fun progress(id: String): Meta? = progress.value.firstOrNull { it.id == id }

    override suspend fun clearProgress(id: String) {
        progress.value = progress.value.filterNot { it.id == id }
    }

    override suspend fun clearAllProgress() {
        progress.value = emptyList()
    }
}
