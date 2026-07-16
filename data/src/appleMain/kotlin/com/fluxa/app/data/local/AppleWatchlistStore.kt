package com.fluxa.app.data.local

import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.usePinned
import platform.Foundation.NSData
import platform.Foundation.NSUserDefaults
import platform.posix.memcpy

@OptIn(ExperimentalForeignApi::class)
class AppleWatchlistStore(
    private val defaults: NSUserDefaults = NSUserDefaults.standardUserDefaults
) : WatchlistStore {
    private val json = Json { ignoreUnknownKeys = true; encodeDefaults = true }
    private var profileId = "guest"
    private val watchlist = MutableStateFlow<List<Meta>>(emptyList())
    private val progress = MutableStateFlow<List<Meta>>(emptyList())
    private val externalProgress = MutableStateFlow<List<Meta>>(emptyList())
    private val liked = MutableStateFlow<List<Meta>>(emptyList())
    private val totalDuration = MutableStateFlow(0L)
    private val feedback = mutableMapOf<String, Boolean?>()

    init {
        migrateLegacyWatchlist()
        reload()
    }

    override fun setActiveProfile(profileId: String) {
        this.profileId = profileId.ifBlank { "guest" }
        reload()
    }

    override fun observeWatchlist(): Flow<List<Meta>> = watchlist
    override fun observeContinueWatching(): Flow<List<Meta>> = progress
    override fun observeExternalContinueWatching(): Flow<List<Meta>> = externalProgress
    override fun observeLiked(): Flow<List<Meta>> = liked
    override fun observeTotalWatchedDuration(): Flow<Long> = totalDuration
    override suspend fun watchlistSnapshot(): List<Meta> = watchlist.value
    override suspend fun continueWatchingSnapshot(): List<Meta> = progress.value

    override suspend fun replaceWatchlist(items: List<Meta>) {
        watchlist.value = items.distinctBy { it.id }
        save("watchlist", watchlist.value)
    }

    override suspend fun toggleWatchlist(item: Meta) {
        watchlist.value = if (watchlist.value.any { it.id == item.id }) {
            watchlist.value.filterNot { it.id == item.id }
        } else {
            watchlist.value + item
        }
        save("watchlist", watchlist.value)
    }

    override suspend fun isInWatchlist(id: String): Boolean = watchlist.value.any { it.id == id }

    override suspend fun setFeedback(id: String, isLike: Boolean?, content: Meta?) {
        feedback[id] = isLike
        if (isLike == true && content != null) liked.value = (liked.value.filterNot { it.id == id } + content)
        if (isLike != true) liked.value = liked.value.filterNot { it.id == id }
        save("liked", liked.value)
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
        save("progress", progress.value)
    }

    override suspend fun progress(id: String): Meta? = progress.value.firstOrNull { it.id == id }

    override suspend fun clearProgress(id: String) {
        progress.value = progress.value.filterNot { it.id == id }
        save("progress", progress.value)
    }

    override suspend fun clearAllProgress() {
        progress.value = emptyList()
        save("progress", emptyList())
    }

    private fun reload() {
        watchlist.value = load("watchlist")
        progress.value = load("progress")
        externalProgress.value = load("externalProgress")
        liked.value = load("liked")
    }

    private fun key(name: String): String = "fluxa.$profileId.$name"

    private fun load(name: String): List<Meta> = defaults.stringForKey(key(name))
        ?.let { encoded -> runCatching { json.decodeFromString<List<Meta>>(encoded) }.getOrNull() }
        .orEmpty()

    private fun save(name: String, items: List<Meta>) {
        defaults.setObject(json.encodeToString(items), key(name))
    }

    private fun migrateLegacyWatchlist() {
        if (defaults.stringForKey(key("watchlist")) != null) return
        val legacy = defaults.dataForKey("fluxa.apple.watchlist")?.decodeUtf8() ?: return
        val items = runCatching { json.decodeFromString<List<Meta>>(legacy) }.getOrNull() ?: return
        defaults.setObject(json.encodeToString(items), key("watchlist"))
        defaults.removeObjectForKey("fluxa.apple.watchlist")
    }
}

@OptIn(ExperimentalForeignApi::class)
private fun NSData.decodeUtf8(): String {
    if (length == 0uL) return ""
    val output = ByteArray(length.toInt())
    output.usePinned { pinned -> memcpy(pinned.addressOf(0), bytes, length) }
    return output.decodeToString()
}
