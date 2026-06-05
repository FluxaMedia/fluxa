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

@Entity(tableName = "content_items", primaryKeys = ["profileId", "contentId"])
data class ContentItemEntity(
    val profileId: String,
    val contentId: String,
    val name: String,
    val type: String,
    val poster: String?,
    val background: String?,
    val logo: String?,
    val description: String?,
    val rating: String?,
    val releaseInfo: String?,
    val updatedAt: Long = System.currentTimeMillis()
)

@Entity(
    tableName = "watchlist_entries",
    primaryKeys = ["profileId", "contentId"],
    indices = [Index(value = ["profileId", "addedAt"], name = "index_watchlist_entries_profile_addedAt", orders = [Index.Order.ASC, Index.Order.DESC])]
)
data class WatchlistEntryEntity(
    val profileId: String,
    val contentId: String,
    val addedAt: Long = System.currentTimeMillis()
)

@Entity(
    tableName = "playback_progress",
    primaryKeys = ["profileId", "contentId"],
    indices = [Index(value = ["profileId", "updatedAt"], name = "index_playback_progress_profile_updatedAt", orders = [Index.Order.ASC, Index.Order.DESC])]
)
data class PlaybackProgressEntity(
    val profileId: String,
    val contentId: String,
    val videoId: String?,
    val timeOffset: Long,
    val duration: Long,
    val lastStreamIndex: Int?,
    val lastEpisodeName: String?,
    val lastStreamUrl: String?,
    val lastStreamTitle: String?,
    val lastBingeGroup: String? = null,
    val continueWatchingPoster: String?,
    val continueWatchingBackground: String?,
    val updatedAt: Long = System.currentTimeMillis()
)

@Entity(
    tableName = "user_feedback",
    primaryKeys = ["profileId", "contentId"],
    indices = [Index(value = ["profileId", "updatedAt"], name = "index_user_feedback_profile_updatedAt", orders = [Index.Order.ASC, Index.Order.DESC])]
)
data class UserFeedbackEntity(
    val profileId: String,
    val contentId: String,
    val isLiked: Boolean?,
    val updatedAt: Long = System.currentTimeMillis()
)

@Entity(tableName = "track_preferences", primaryKeys = ["profileId", "contentId"])
data class TrackPreferenceEntity(
    val profileId: String,
    val contentId: String,
    val lastAudioLanguage: String?,
    val lastSubtitleLanguage: String?,
    val updatedAt: Long = System.currentTimeMillis()
)

@Entity(
    tableName = "watched_episodes",
    primaryKeys = ["profileId", "seriesId", "videoId"],
    indices = [Index(value = ["profileId", "seriesId"], name = "index_watched_episodes_profile_series")]
)
data class WatchedEpisodeEntity(
    val profileId: String,
    val seriesId: String,
    val videoId: String,
    val watchedAt: Long = System.currentTimeMillis()
)

@Entity(
    tableName = "watched_content_durations",
    primaryKeys = ["profileId", "source", "contentId", "videoId"],
    indices = [
        Index(value = ["profileId"], name = "index_watched_content_durations_profile"),
        Index(value = ["profileId", "source"], name = "index_watched_content_durations_profile_source")
    ]
)
data class WatchedContentDurationEntity(
    val profileId: String,
    val source: String,
    val contentId: String,
    val videoId: String,
    val duration: Long,
    val watchedAt: Long = System.currentTimeMillis()
)

@Entity(
    tableName = "external_playback_progress",
    primaryKeys = ["profileId", "provider", "contentId"],
    indices = [Index(value = ["profileId", "syncedAt"], name = "index_external_playback_progress_profile_syncedAt", orders = [Index.Order.ASC, Index.Order.DESC])]
)
data class ExternalPlaybackProgressEntity(
    val profileId: String,
    val provider: String,
    val contentId: String,
    val name: String,
    val type: String,
    val poster: String?,
    val background: String?,
    val logo: String?,
    val description: String?,
    val rating: String?,
    val releaseInfo: String?,
    val videoId: String?,
    val timeOffset: Long,
    val duration: Long,
    val lastEpisodeName: String?,
    val reason: String?,
    val continueWatchingPoster: String?,
    val continueWatchingBackground: String?,
    val syncedAt: Long = System.currentTimeMillis()
)

@Entity(
    tableName = "external_watched_episodes",
    primaryKeys = ["profileId", "provider", "seriesId", "videoId"],
    indices = [Index(value = ["profileId", "provider", "seriesId"], name = "index_external_watched_episodes_profile_provider_series")]
)
data class ExternalWatchedEpisodeEntity(
    val profileId: String,
    val provider: String,
    val seriesId: String,
    val videoId: String,
    val syncedAt: Long = System.currentTimeMillis()
)

data class ContentStateRow(
    val id: String,
    val name: String,
    val type: String,
    val poster: String?,
    val background: String?,
    val logo: String?,
    val description: String?,
    val rating: String?,
    val releaseInfo: String?,
    val timeOffset: Long?,
    val duration: Long?,
    val lastVideoId: String?,
    val lastStreamIndex: Int?,
    val lastEpisodeName: String?,
    val lastStreamUrl: String?,
    val lastStreamTitle: String?,
    val lastBingeGroup: String?,
    val lastAudioLanguage: String?,
    val lastSubtitleLanguage: String?,
    val continueWatchingPoster: String?,
    val continueWatchingBackground: String?,
    val isLiked: Boolean?,
    val sortAt: Long
)
