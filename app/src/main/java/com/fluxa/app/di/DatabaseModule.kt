package com.fluxa.app.di

import android.content.Context
import androidx.room.Room
import com.fluxa.app.data.local.AppDatabase
import com.fluxa.app.data.local.WatchlistDao
import com.fluxa.app.data.local.WatchlistManager
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object DatabaseModule {

    @Provides
    @Singleton
    fun provideWatchlistDatabase(@ApplicationContext context: Context): AppDatabase {
        return Room.databaseBuilder(
            context,
            AppDatabase::class.java,
            "watchlist_db"
        ).build()
    }

    @Provides
    @Singleton
    fun provideWatchlistDao(database: AppDatabase): WatchlistDao {
        return database.watchlistDao()
    }

    @Provides
    @Singleton
    fun provideWatchlistManager(dao: WatchlistDao): WatchlistManager {
        return WatchlistManager(dao)
    }
}
