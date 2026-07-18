package com.fluxa.app.data.local

import android.content.Context
import androidx.room.Room
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map

import javax.inject.Inject
import javax.inject.Singleton

@OptIn(ExperimentalCoroutinesApi::class)
@Singleton
class WatchlistManager @Inject constructor(
    private val dao: WatchlistDao
) {
    private val _activeProfileId = MutableStateFlow("guest")

    fun setActiveProfile(profileId: String) {
        val normalized = profileId.ifBlank { "guest" }
        if (_activeProfileId.value != normalized) {
            _activeProfileId.value = normalized
        }
    }

    private fun pid() = _activeProfileId.value

    fun getWatchlistFlow(): Flow<List<Meta>> {
        return _activeProfileId.flatMapLatest { profileId ->
            dao.observeWatchlist(profileId).map { rows -> rows.map { it.toMeta() } }
        }
    }

    fun getContinueWatchingFlow(): Flow<List<Meta>> {
        return _activeProfileId.flatMapLatest { profileId ->
            dao.observeContinueWatching(profileId).map { rows -> rows.map { it.toMeta() } }
        }
    }

    fun getExternalContinueWatchingFlow(): Flow<List<Meta>> {
        return _activeProfileId.flatMapLatest { profileId ->
            dao.observeExternalContinueWatching(profileId).map { rows -> rows.map { it.toMeta() } }
        }
    }

    fun getExternalContinueWatchingFlowByProvider(provider: String): Flow<List<Meta>> {
        return _activeProfileId.flatMapLatest { profileId ->
            dao.observeExternalContinueWatchingByProvider(profileId, provider).map { rows -> rows.map { it.toMeta() } }
        }
    }

    fun getLikedFlow(): Flow<List<Meta>> {
        return _activeProfileId.flatMapLatest { profileId ->
            dao.observeLiked(profileId).map { rows -> rows.map { it.toMeta() } }
        }
    }

    fun getTotalWatchedContentDurationFlow(): Flow<Long> {
        return _activeProfileId.flatMapLatest { profileId ->
            dao.observeTotalWatchedContentDuration(profileId)
        }
    }

    suspend fun getWatchlistSnapshot(): List<Meta> {
        return dao.getWatchlistSnapshot(pid()).map { it.toMeta() }
    }

    suspend fun replaceWatchlist(items: List<Meta>) {
        val profileId = pid()
        val itemIds = items.map { it.id }.toSet()
        dao.getWatchlistSnapshot(profileId)
            .filter { it.id !in itemIds }
            .forEach { dao.deleteWatchlistEntry(profileId, it.id) }
        items.forEach { item ->
            dao.upsertContent(item.toContentItemEntity(profileId))
            dao.upsertWatchlistEntry(WatchlistEntryEntity(profileId, item.id))
        }
    }

    suspend fun getContinueWatchingSnapshot(): List<Meta> {
        return dao.getContinueWatchingSnapshot(pid()).map { it.toMeta() }
    }

    suspend fun getExternalContinueWatchingSnapshot(): List<Meta> {
        return dao.getExternalContinueWatchingSnapshot(pid()).map { it.toMeta() }
    }

    suspend fun toggleWatchlist(item: Meta) {
        val profileId = pid()
        dao.upsertContent(item.toContentItemEntity(profileId))
        if (dao.isInWatchlist(profileId, item.id)) {
            dao.deleteWatchlistEntry(profileId, item.id)
            dao.upsertWatchlistRemoval(WatchlistRemovalEntity(profileId, item.id))
        } else {
            dao.clearWatchlistRemoval(profileId, item.id)
            dao.upsertWatchlistEntry(WatchlistEntryEntity(profileId, item.id))
        }
    }

    suspend fun isInWatchlist(id: String): Boolean {
        return dao.isInWatchlist(pid(), id)
    }

    suspend fun setFeedback(id: String, isLike: Boolean?, metaIfNew: Meta? = null) {
        val profileId = pid()
        metaIfNew?.let { dao.upsertContent(it.toContentItemEntity(profileId)) }
        dao.upsertFeedback(UserFeedbackEntity(profileId, id, isLike))
    }

    suspend fun savePlaybackProgress(
        meta: Meta,
        timeOffset: Long,
        duration: Long,
        lastVideoId: String? = null,
        lastStreamIndex: Int? = null,
        lastEpisodeName: String? = null,
        lastStreamUrl: String? = null,
        lastStreamTitle: String? = null,
        lastBingeGroup: String? = null,
        lastAudioLanguage: String? = null,
        lastSubtitleLanguage: String? = null,
        updatedAt: Long = System.currentTimeMillis()
    ) {
        val profileId = pid()
        val existing = dao.getContentState(profileId, meta.id)
        val resolvedVideoId = lastVideoId ?: existing?.lastVideoId
        val videoChanged = lastVideoId != null && lastVideoId != existing?.lastVideoId

        dao.upsertContent(meta.toContentItemEntity(profileId))
        dao.upsertPlaybackProgress(
            PlaybackProgressEntity(
                profileId = profileId,
                contentId = meta.id,
                videoId = resolvedVideoId,
                timeOffset = timeOffset,
                duration = duration,
                lastStreamIndex = lastStreamIndex ?: existing?.lastStreamIndex,
                lastEpisodeName = if (videoChanged) lastEpisodeName else lastEpisodeName ?: existing?.lastEpisodeName,
                lastStreamUrl = lastStreamUrl ?: existing?.lastStreamUrl,
                lastStreamTitle = lastStreamTitle ?: existing?.lastStreamTitle,
                lastBingeGroup = lastBingeGroup ?: existing?.lastBingeGroup,
                continueWatchingPoster = meta.continueWatchingPoster ?: existing?.continueWatchingPoster,
                continueWatchingBackground = meta.continueWatchingBackground ?: existing?.continueWatchingBackground,
                updatedAt = updatedAt
            )
        )
        if (lastAudioLanguage != null || lastSubtitleLanguage != null || existing?.lastAudioLanguage != null || existing?.lastSubtitleLanguage != null) {
            dao.upsertTrackPreference(
                TrackPreferenceEntity(
                    profileId = profileId,
                    contentId = meta.id,
                    lastAudioLanguage = lastAudioLanguage ?: existing?.lastAudioLanguage,
                    lastSubtitleLanguage = lastSubtitleLanguage ?: existing?.lastSubtitleLanguage
                )
            )
        }
    }

    suspend fun updateContinueWatchingArtwork(id: String, poster: String?, background: String?) {
        val profileId = pid()
        val existing = dao.getContentState(profileId, id) ?: return
        dao.upsertPlaybackProgress(
            PlaybackProgressEntity(
                profileId = profileId,
                contentId = id,
                videoId = existing.lastVideoId,
                timeOffset = existing.timeOffset ?: 0L,
                duration = existing.duration ?: 0L,
                lastStreamIndex = existing.lastStreamIndex,
                lastEpisodeName = existing.lastEpisodeName,
                lastStreamUrl = existing.lastStreamUrl,
                lastStreamTitle = existing.lastStreamTitle,
                continueWatchingPoster = poster,
                continueWatchingBackground = background,
                updatedAt = existing.sortAt
            )
        )
    }

    suspend fun getFeedback(id: String): Boolean? {
        return dao.getFeedback(pid(), id)
    }

    suspend fun getPlaybackProgress(id: String): Meta? {
        return dao.getContentState(pid(), id)?.toMeta()?.takeIf {
            (it.timeOffset ?: 0L) > 0L || it.lastVideoId != null || it.lastStreamUrl != null
        }
    }

    suspend fun getLocalWatchedVideoIds(seriesId: String): Set<String> {
        return dao.getWatchedEpisodeIds(pid(), seriesId).toSet()
    }

    suspend fun markEpisodeWatched(seriesId: String, videoId: String, watched: Boolean = true): Set<String> {
        return markEpisodesWatched(seriesId, listOf(videoId), watched)
    }

    suspend fun recordWatchedContentDuration(contentId: String, videoId: String?, duration: Long) {
        val normalizedDuration = duration.takeIf { it > 0L } ?: return
        val normalizedVideoId = videoId?.takeIf { it.isNotBlank() } ?: contentId
        dao.upsertWatchedContentDuration(
            WatchedContentDurationEntity(
                profileId = pid(),
                source = "local",
                contentId = contentId,
                videoId = normalizedVideoId,
                duration = normalizedDuration
            )
        )
    }

    suspend fun replaceExternalWatchedContentDurations(provider: String, records: Collection<WatchedContentDurationRecord>) {
        val profileId = pid()
        val source = provider.ifBlank { "external" }
        dao.deleteWatchedContentDurationsBySource(profileId, source)
        val now = System.currentTimeMillis()
        dao.upsertWatchedContentDurations(
            records.mapNotNull { record ->
                val duration = record.duration.takeIf { it > 0L } ?: return@mapNotNull null
                val contentId = record.contentId.takeIf { it.isNotBlank() } ?: return@mapNotNull null
                WatchedContentDurationEntity(
                    profileId = profileId,
                    source = source,
                    contentId = contentId,
                    videoId = record.videoId?.takeIf { it.isNotBlank() } ?: contentId,
                    duration = duration,
                    watchedAt = now
                )
            }
        )
    }

    suspend fun markEpisodesWatched(
        seriesId: String,
        videoIds: Collection<String>,
        watched: Boolean = true
    ): Set<String> {
        val profileId = pid()
        val cleanVideoIds = videoIds.filter { it.isNotBlank() }.distinct()
        if (cleanVideoIds.isEmpty()) return getLocalWatchedVideoIds(seriesId)
        if (watched) {
            dao.clearWatchedEpisodeRemovals(profileId, seriesId, cleanVideoIds)
            dao.upsertWatchedEpisodes(cleanVideoIds.map { WatchedEpisodeEntity(profileId, seriesId, it) })
        } else {
            dao.deleteWatchedEpisodes(profileId, seriesId, cleanVideoIds)
            dao.upsertWatchedEpisodeRemovals(cleanVideoIds.map { WatchedEpisodeRemovalEntity(profileId, seriesId, it) })
        }
        return getLocalWatchedVideoIds(seriesId)
    }

    suspend fun markSeasonWatched(seriesId: String, episodes: List<Video>): Set<String> {
        return markEpisodesWatched(seriesId, episodes.mapNotNull { it.id })
    }

    suspend fun clearPlaybackProgress(id: String) {
        val profileId = pid()
        dao.deletePlaybackProgress(profileId, id)
        dao.deleteTrackPreference(profileId, id)
    }

    suspend fun clearAllPlaybackProgress() {
        dao.deleteAllPlaybackProgress(pid())
    }

    suspend fun clearAllWatchedEpisodes() {
        dao.deleteAllWatchedEpisodes(pid())
    }

    suspend fun replaceExternalContinueWatching(items: List<Meta>) {
        replaceExternalContinueWatching(items.map { it.externalProviderKey() }.toSet(), items)
    }

    suspend fun replaceExternalContinueWatching(providers: Set<String>, items: List<Meta>) {
        val profileId = pid()
        providers.forEach { dao.deleteExternalPlaybackProgressByProvider(profileId, it) }
        val now = System.currentTimeMillis()
        val kept = items.filter { it.id.isNotBlank() && (it.timeOffset ?: 0L) > 0L && (it.duration ?: 0L) > 0L }
        dao.upsertExternalPlaybackProgress(kept.map { it.toExternalPlaybackProgressEntity(profileId, now) })
    }

    suspend fun replaceExternalWatchedEpisodes(provider: String, episodesBySeries: Map<String, Set<String>>) {
        val profileId = pid()
        val normalizedProvider = provider.ifBlank { "external" }
        val now = System.currentTimeMillis()
        dao.deleteExternalWatchedEpisodes(profileId, normalizedProvider)
        dao.upsertExternalWatchedEpisodes(
            episodesBySeries.flatMap { (seriesId, videoIds) ->
                videoIds
                    .filter { it.isNotBlank() }
                    .distinct()
                    .map { videoId ->
                        ExternalWatchedEpisodeEntity(
                            profileId = profileId,
                            provider = normalizedProvider,
                            seriesId = seriesId,
                            videoId = videoId,
                            syncedAt = now
                        )
                    }
            }
        )
    }

    private fun ContentStateRow.toMeta() = Meta(
        id = id,
        name = name,
        type = type,
        poster = poster,
        background = background,
        logo = logo,
        description = description,
        imdbRating = rating,
        releaseInfo = releaseInfo,
        timeOffset = timeOffset,
        duration = duration,
        lastVideoId = lastVideoId,
        lastStreamIndex = lastStreamIndex,
        lastEpisodeName = lastEpisodeName,
        lastStreamUrl = lastStreamUrl,
        lastStreamTitle = lastStreamTitle,
        lastBingeGroup = lastBingeGroup,
        lastAudioLanguage = lastAudioLanguage,
        lastSubtitleLanguage = lastSubtitleLanguage,
        lastWatchedAt = sortAt,
        continueWatchingPoster = continueWatchingPoster,
        continueWatchingBackground = continueWatchingBackground
    )

    private fun Meta.toContentItemEntity(profileId: String) = ContentItemEntity(
        profileId = profileId,
        contentId = id,
        name = name,
        type = type,
        poster = poster,
        background = background,
        logo = logo,
        description = description,
        rating = imdbRating,
        releaseInfo = releaseInfo
    )

    private fun ExternalPlaybackProgressEntity.toMeta() = Meta(
        id = contentId,
        name = name,
        type = type,
        poster = poster,
        background = background,
        logo = logo,
        description = description,
        imdbRating = rating,
        releaseInfo = releaseInfo,
        timeOffset = timeOffset,
        duration = duration,
        lastVideoId = videoId,
        lastEpisodeName = lastEpisodeName,
        reason = reason,
        lastWatchedAt = syncedAt,
        continueWatchingPoster = continueWatchingPoster,
        continueWatchingBackground = continueWatchingBackground
    )

    private fun Meta.toExternalPlaybackProgressEntity(profileId: String, fallbackSyncedAt: Long) = ExternalPlaybackProgressEntity(
        profileId = profileId,
        provider = externalProviderKey(),
        contentId = id,
        name = name,
        type = type,
        poster = poster,
        background = background,
        logo = logo,
        description = description,
        rating = imdbRating,
        releaseInfo = releaseInfo,
        videoId = lastVideoId,
        timeOffset = timeOffset ?: 0L,
        duration = duration ?: 0L,
        lastEpisodeName = lastEpisodeName,
        reason = reason,
        continueWatchingPoster = continueWatchingPoster,
        continueWatchingBackground = continueWatchingBackground,
        syncedAt = lastWatchedAt ?: fallbackSyncedAt
    )

    private fun Meta.externalProviderKey(): String {
        return when {
            reason.equals("Trakt.tv", ignoreCase = true) -> "trakt"
            reason.equals("MyAnimeList", ignoreCase = true) -> "mal"
            reason.equals("Simkl", ignoreCase = true) -> "simkl"
            reason.equals("Nuvio", ignoreCase = true) -> "nuvio"
            reason.equals("AniList", ignoreCase = true) -> "anilist"
            id.startsWith("mal:", ignoreCase = true) -> "mal"
            id.startsWith("simkl:", ignoreCase = true) -> "simkl"
            id.startsWith("anilist:", ignoreCase = true) -> "anilist"
            else -> "nuvio"
        }
    }
}
