package com.fluxa.app.data.local

import androidx.room.ConstructedBy
import androidx.room.Database
import androidx.room.RoomDatabase
import androidx.room.RoomDatabaseConstructor

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
    version = 17,
    exportSchema = true
)
@ConstructedBy(AppDatabaseConstructor::class)
abstract class AppDatabase : RoomDatabase() {
    abstract fun watchlistDao(): WatchlistDao
}

expect object AppDatabaseConstructor : RoomDatabaseConstructor<AppDatabase> {
    override fun initialize(): AppDatabase
}
