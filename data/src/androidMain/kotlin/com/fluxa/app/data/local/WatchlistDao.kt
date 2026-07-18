package com.fluxa.app.data.local

import androidx.room.Dao
import androidx.room.Database
import androidx.room.Entity
import androidx.room.Index
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.PrimaryKey
import androidx.room.Query
import androidx.room.RoomDatabase
import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase
import kotlinx.coroutines.flow.Flow

@Dao
interface WatchlistDao {
    @Query(
        """
        SELECT
            c.contentId AS id,
            c.name AS name,
            c.type AS type,
            c.poster AS poster,
            c.background AS background,
            c.logo AS logo,
            c.description AS description,
            c.rating AS rating,
            c.releaseInfo AS releaseInfo,
            p.timeOffset AS timeOffset,
            p.duration AS duration,
            p.videoId AS lastVideoId,
            p.lastStreamIndex AS lastStreamIndex,
            p.lastEpisodeName AS lastEpisodeName,
            p.lastStreamUrl AS lastStreamUrl,
            p.lastStreamTitle AS lastStreamTitle,
            p.lastBingeGroup AS lastBingeGroup,
            t.lastAudioLanguage AS lastAudioLanguage,
            t.lastSubtitleLanguage AS lastSubtitleLanguage,
            p.continueWatchingPoster AS continueWatchingPoster,
            p.continueWatchingBackground AS continueWatchingBackground,
            f.isLiked AS isLiked,
            w.addedAt AS sortAt
        FROM watchlist_entries w
        INNER JOIN content_items c ON c.profileId = w.profileId AND c.contentId = w.contentId
        LEFT JOIN playback_progress p ON p.profileId = c.profileId AND p.contentId = c.contentId
        LEFT JOIN user_feedback f ON f.profileId = c.profileId AND f.contentId = c.contentId
        LEFT JOIN track_preferences t ON t.profileId = c.profileId AND t.contentId = c.contentId
        WHERE w.profileId = :profileId
        ORDER BY w.addedAt DESC
        """
    )
    fun observeWatchlist(profileId: String): Flow<List<ContentStateRow>>

    @Query(
        """
        SELECT
            c.contentId AS id,
            c.name AS name,
            c.type AS type,
            c.poster AS poster,
            c.background AS background,
            c.logo AS logo,
            c.description AS description,
            c.rating AS rating,
            c.releaseInfo AS releaseInfo,
            p.timeOffset AS timeOffset,
            p.duration AS duration,
            p.videoId AS lastVideoId,
            p.lastStreamIndex AS lastStreamIndex,
            p.lastEpisodeName AS lastEpisodeName,
            p.lastStreamUrl AS lastStreamUrl,
            p.lastStreamTitle AS lastStreamTitle,
            p.lastBingeGroup AS lastBingeGroup,
            t.lastAudioLanguage AS lastAudioLanguage,
            t.lastSubtitleLanguage AS lastSubtitleLanguage,
            p.continueWatchingPoster AS continueWatchingPoster,
            p.continueWatchingBackground AS continueWatchingBackground,
            f.isLiked AS isLiked,
            p.updatedAt AS sortAt
        FROM playback_progress p
        INNER JOIN content_items c ON c.profileId = p.profileId AND c.contentId = p.contentId
        LEFT JOIN user_feedback f ON f.profileId = c.profileId AND f.contentId = c.contentId
        LEFT JOIN track_preferences t ON t.profileId = c.profileId AND t.contentId = c.contentId
        WHERE p.profileId = :profileId AND (
            (p.timeOffset > 0 AND p.duration > 0) OR
            (c.type IN ('series', 'tv', 'anime') AND p.videoId IS NOT NULL AND p.timeOffset <= 0 AND p.duration <= 0)
        )
        ORDER BY p.updatedAt DESC
        """
    )
    fun observeContinueWatching(profileId: String): Flow<List<ContentStateRow>>

    @Query(
        """
        SELECT
            c.contentId AS id,
            c.name AS name,
            c.type AS type,
            c.poster AS poster,
            c.background AS background,
            c.logo AS logo,
            c.description AS description,
            c.rating AS rating,
            c.releaseInfo AS releaseInfo,
            p.timeOffset AS timeOffset,
            p.duration AS duration,
            p.videoId AS lastVideoId,
            p.lastStreamIndex AS lastStreamIndex,
            p.lastEpisodeName AS lastEpisodeName,
            p.lastStreamUrl AS lastStreamUrl,
            p.lastStreamTitle AS lastStreamTitle,
            p.lastBingeGroup AS lastBingeGroup,
            t.lastAudioLanguage AS lastAudioLanguage,
            t.lastSubtitleLanguage AS lastSubtitleLanguage,
            p.continueWatchingPoster AS continueWatchingPoster,
            p.continueWatchingBackground AS continueWatchingBackground,
            f.isLiked AS isLiked,
            f.updatedAt AS sortAt
        FROM user_feedback f
        INNER JOIN content_items c ON c.profileId = f.profileId AND c.contentId = f.contentId
        LEFT JOIN playback_progress p ON p.profileId = c.profileId AND p.contentId = c.contentId
        LEFT JOIN track_preferences t ON t.profileId = c.profileId AND t.contentId = c.contentId
        WHERE f.profileId = :profileId AND f.isLiked = 1
        ORDER BY f.updatedAt DESC
        """
    )
    fun observeLiked(profileId: String): Flow<List<ContentStateRow>>

    @Query(
        """
        SELECT
            c.contentId AS id,
            c.name AS name,
            c.type AS type,
            c.poster AS poster,
            c.background AS background,
            c.logo AS logo,
            c.description AS description,
            c.rating AS rating,
            c.releaseInfo AS releaseInfo,
            p.timeOffset AS timeOffset,
            p.duration AS duration,
            p.videoId AS lastVideoId,
            p.lastStreamIndex AS lastStreamIndex,
            p.lastEpisodeName AS lastEpisodeName,
            p.lastStreamUrl AS lastStreamUrl,
            p.lastStreamTitle AS lastStreamTitle,
            p.lastBingeGroup AS lastBingeGroup,
            t.lastAudioLanguage AS lastAudioLanguage,
            t.lastSubtitleLanguage AS lastSubtitleLanguage,
            p.continueWatchingPoster AS continueWatchingPoster,
            p.continueWatchingBackground AS continueWatchingBackground,
            f.isLiked AS isLiked,
            w.addedAt AS sortAt
        FROM watchlist_entries w
        INNER JOIN content_items c ON c.profileId = w.profileId AND c.contentId = w.contentId
        LEFT JOIN playback_progress p ON p.profileId = c.profileId AND p.contentId = c.contentId
        LEFT JOIN user_feedback f ON f.profileId = c.profileId AND f.contentId = c.contentId
        LEFT JOIN track_preferences t ON t.profileId = c.profileId AND t.contentId = c.contentId
        WHERE w.profileId = :profileId
        ORDER BY w.addedAt DESC
        """
    )
    suspend fun getWatchlistSnapshot(profileId: String): List<ContentStateRow>

    @Query(
        """
        SELECT
            c.contentId AS id,
            c.name AS name,
            c.type AS type,
            c.poster AS poster,
            c.background AS background,
            c.logo AS logo,
            c.description AS description,
            c.rating AS rating,
            c.releaseInfo AS releaseInfo,
            p.timeOffset AS timeOffset,
            p.duration AS duration,
            p.videoId AS lastVideoId,
            p.lastStreamIndex AS lastStreamIndex,
            p.lastEpisodeName AS lastEpisodeName,
            p.lastStreamUrl AS lastStreamUrl,
            p.lastStreamTitle AS lastStreamTitle,
            p.lastBingeGroup AS lastBingeGroup,
            t.lastAudioLanguage AS lastAudioLanguage,
            t.lastSubtitleLanguage AS lastSubtitleLanguage,
            p.continueWatchingPoster AS continueWatchingPoster,
            p.continueWatchingBackground AS continueWatchingBackground,
            f.isLiked AS isLiked,
            p.updatedAt AS sortAt
        FROM playback_progress p
        INNER JOIN content_items c ON c.profileId = p.profileId AND c.contentId = p.contentId
        LEFT JOIN user_feedback f ON f.profileId = c.profileId AND f.contentId = c.contentId
        LEFT JOIN track_preferences t ON t.profileId = c.profileId AND t.contentId = c.contentId
        WHERE p.profileId = :profileId AND (
            (p.timeOffset > 0 AND p.duration > 0) OR
            (c.type IN ('series', 'tv', 'anime') AND p.videoId IS NOT NULL AND p.timeOffset <= 0 AND p.duration <= 0)
        )
        ORDER BY p.updatedAt DESC
        """
    )
    suspend fun getContinueWatchingSnapshot(profileId: String): List<ContentStateRow>

    @Query(
        """
        SELECT
            c.contentId AS id,
            c.name AS name,
            c.type AS type,
            c.poster AS poster,
            c.background AS background,
            c.logo AS logo,
            c.description AS description,
            c.rating AS rating,
            c.releaseInfo AS releaseInfo,
            p.timeOffset AS timeOffset,
            p.duration AS duration,
            p.videoId AS lastVideoId,
            p.lastStreamIndex AS lastStreamIndex,
            p.lastEpisodeName AS lastEpisodeName,
            p.lastStreamUrl AS lastStreamUrl,
            p.lastStreamTitle AS lastStreamTitle,
            p.lastBingeGroup AS lastBingeGroup,
            t.lastAudioLanguage AS lastAudioLanguage,
            t.lastSubtitleLanguage AS lastSubtitleLanguage,
            p.continueWatchingPoster AS continueWatchingPoster,
            p.continueWatchingBackground AS continueWatchingBackground,
            f.isLiked AS isLiked,
            COALESCE(p.updatedAt, c.updatedAt) AS sortAt
        FROM content_items c
        LEFT JOIN playback_progress p ON p.profileId = c.profileId AND p.contentId = c.contentId
        LEFT JOIN user_feedback f ON f.profileId = c.profileId AND f.contentId = c.contentId
        LEFT JOIN track_preferences t ON t.profileId = c.profileId AND t.contentId = c.contentId
        WHERE c.profileId = :profileId AND c.contentId = :contentId
        LIMIT 1
        """
    )
    suspend fun getContentState(profileId: String, contentId: String): ContentStateRow?

    @Query("SELECT contentId FROM watchlist_entries WHERE profileId = :profileId")
    fun observeWatchlistIds(profileId: String): Flow<List<String>>

    @Query("SELECT * FROM watchlist_entries WHERE profileId = :profileId")
    suspend fun getWatchlistEntries(profileId: String): List<WatchlistEntryEntity>

    @Query("SELECT * FROM watched_episodes WHERE profileId = :profileId AND seriesId = :seriesId")
    suspend fun getWatchedEpisodeEntries(profileId: String, seriesId: String): List<WatchedEpisodeEntity>

    @Query("SELECT * FROM watched_episodes WHERE profileId = :profileId")
    suspend fun getAllWatchedEpisodeEntries(profileId: String): List<WatchedEpisodeEntity>

    @Query("SELECT * FROM watched_episode_removals WHERE profileId = :profileId")
    suspend fun getAllWatchedEpisodeRemovals(profileId: String): List<WatchedEpisodeRemovalEntity>

    @Query("SELECT EXISTS(SELECT 1 FROM watchlist_entries WHERE profileId = :profileId AND contentId = :contentId)")
    suspend fun isInWatchlist(profileId: String, contentId: String): Boolean

    @Query("SELECT isLiked FROM user_feedback WHERE profileId = :profileId AND contentId = :contentId LIMIT 1")
    suspend fun getFeedback(profileId: String, contentId: String): Boolean?

    @Query(
        """
        SELECT videoId FROM watched_episodes WHERE profileId = :profileId AND seriesId = :seriesId
        UNION
        SELECT videoId FROM external_watched_episodes WHERE profileId = :profileId AND seriesId = :seriesId
        """
    )
    suspend fun getWatchedEpisodeIds(profileId: String, seriesId: String): List<String>

    @Query(
        """
        SELECT COALESCE(SUM(duration), 0) FROM (
            SELECT MAX(duration) AS duration
            FROM watched_content_durations
            WHERE profileId = :profileId
            GROUP BY contentId, videoId
        )
        """
    )
    fun observeTotalWatchedContentDuration(profileId: String): Flow<Long>

    @Query(
        """
        SELECT * FROM external_playback_progress
        WHERE profileId = :profileId AND timeOffset > 0 AND duration > 0
        ORDER BY syncedAt DESC
        """
    )
    fun observeExternalContinueWatching(profileId: String): Flow<List<ExternalPlaybackProgressEntity>>

    @Query(
        """
        SELECT * FROM external_playback_progress
        WHERE profileId = :profileId AND timeOffset > 0 AND duration > 0
        ORDER BY syncedAt DESC
        """
    )
    suspend fun getExternalContinueWatchingSnapshot(profileId: String): List<ExternalPlaybackProgressEntity>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertContent(item: ContentItemEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertWatchlistEntry(entry: WatchlistEntryEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertPlaybackProgress(progress: PlaybackProgressEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertFeedback(feedback: UserFeedbackEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertTrackPreference(preference: TrackPreferenceEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertWatchedEpisodes(episodes: List<WatchedEpisodeEntity>)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertWatchedContentDuration(item: WatchedContentDurationEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertWatchedContentDurations(items: List<WatchedContentDurationEntity>)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertExternalPlaybackProgress(items: List<ExternalPlaybackProgressEntity>)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertExternalWatchedEpisodes(items: List<ExternalWatchedEpisodeEntity>)

    @Query("DELETE FROM watchlist_entries WHERE profileId = :profileId AND contentId = :contentId")
    suspend fun deleteWatchlistEntry(profileId: String, contentId: String)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertWatchlistRemoval(removal: WatchlistRemovalEntity)

    @Query("DELETE FROM watchlist_removals WHERE profileId = :profileId AND contentId = :contentId")
    suspend fun clearWatchlistRemoval(profileId: String, contentId: String)

    @Query("SELECT * FROM watchlist_removals WHERE profileId = :profileId")
    suspend fun getWatchlistRemovals(profileId: String): List<WatchlistRemovalEntity>

    @Query("DELETE FROM playback_progress WHERE profileId = :profileId AND contentId = :contentId")
    suspend fun deletePlaybackProgress(profileId: String, contentId: String)

    @Query("DELETE FROM playback_progress WHERE profileId = :profileId")
    suspend fun deleteAllPlaybackProgress(profileId: String)

    @Query("DELETE FROM track_preferences WHERE profileId = :profileId AND contentId = :contentId")
    suspend fun deleteTrackPreference(profileId: String, contentId: String)

    @Query("DELETE FROM watched_episodes WHERE profileId = :profileId AND seriesId = :seriesId AND videoId IN (:videoIds)")
    suspend fun deleteWatchedEpisodes(profileId: String, seriesId: String, videoIds: List<String>)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertWatchedEpisodeRemovals(removals: List<WatchedEpisodeRemovalEntity>)

    @Query("DELETE FROM watched_episode_removals WHERE profileId = :profileId AND seriesId = :seriesId AND videoId IN (:videoIds)")
    suspend fun clearWatchedEpisodeRemovals(profileId: String, seriesId: String, videoIds: List<String>)

    @Query("SELECT * FROM watched_episode_removals WHERE profileId = :profileId AND seriesId = :seriesId")
    suspend fun getWatchedEpisodeRemovals(profileId: String, seriesId: String): List<WatchedEpisodeRemovalEntity>

    @Query("DELETE FROM watched_episodes WHERE profileId = :profileId")
    suspend fun deleteAllWatchedEpisodes(profileId: String)

    @Query("DELETE FROM external_playback_progress WHERE profileId = :profileId")
    suspend fun deleteExternalPlaybackProgress(profileId: String)

    @Query("DELETE FROM external_playback_progress WHERE profileId = :profileId AND provider = :provider")
    suspend fun deleteExternalPlaybackProgressByProvider(profileId: String, provider: String)

    @Query(
        """
        SELECT * FROM external_playback_progress
        WHERE profileId = :profileId AND provider = :provider AND timeOffset > 0 AND duration > 0
        ORDER BY syncedAt DESC
        """
    )
    fun observeExternalContinueWatchingByProvider(profileId: String, provider: String): Flow<List<ExternalPlaybackProgressEntity>>

    @Query("DELETE FROM external_watched_episodes WHERE profileId = :profileId AND provider = :provider")
    suspend fun deleteExternalWatchedEpisodes(profileId: String, provider: String)

    @Query("DELETE FROM watched_content_durations WHERE profileId = :profileId AND source = :source")
    suspend fun deleteWatchedContentDurationsBySource(profileId: String, source: String)
}
