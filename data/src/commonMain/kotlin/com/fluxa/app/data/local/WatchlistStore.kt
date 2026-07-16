package com.fluxa.app.data.local

import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.flow.Flow

data class PlaybackProgressUpdate(
    val content: Meta,
    val positionMs: Long,
    val durationMs: Long,
    val videoId: String? = null,
    val streamIndex: Int? = null,
    val episodeName: String? = null,
    val streamUrl: String? = null,
    val streamTitle: String? = null,
    val bingeGroup: String? = null,
    val audioLanguage: String? = null,
    val subtitleLanguage: String? = null,
    val updatedAtMillis: Long? = null
)

interface WatchlistStore {
    fun setActiveProfile(profileId: String)
    fun observeWatchlist(): Flow<List<Meta>>
    fun observeContinueWatching(): Flow<List<Meta>>
    fun observeExternalContinueWatching(): Flow<List<Meta>>
    fun observeLiked(): Flow<List<Meta>>
    fun observeTotalWatchedDuration(): Flow<Long>
    suspend fun watchlistSnapshot(): List<Meta>
    suspend fun continueWatchingSnapshot(): List<Meta>
    suspend fun replaceWatchlist(items: List<Meta>)
    suspend fun toggleWatchlist(item: Meta)
    suspend fun isInWatchlist(id: String): Boolean
    suspend fun setFeedback(id: String, isLike: Boolean?, content: Meta? = null)
    suspend fun feedback(id: String): Boolean?
    suspend fun saveProgress(update: PlaybackProgressUpdate)
    suspend fun progress(id: String): Meta?
    suspend fun clearProgress(id: String)
    suspend fun clearAllProgress()
}
