package com.fluxa.app.data.local

import androidx.room.Database
import androidx.room.RoomDatabase

@Database(
    entities = [
        ContentItemEntity::class,
        WatchlistEntryEntity::class,
        PlaybackProgressEntity::class,
        UserFeedbackEntity::class,
        TrackPreferenceEntity::class,
        WatchedEpisodeEntity::class,
        WatchedContentDurationEntity::class,
        ExternalPlaybackProgressEntity::class,
        ExternalWatchedEpisodeEntity::class,
        WatchlistRemovalEntity::class,
        WatchedEpisodeRemovalEntity::class
    ],
    version = 16,
    exportSchema = true
)
abstract class AppDatabase : RoomDatabase() {
    abstract fun watchlistDao(): WatchlistDao
}
