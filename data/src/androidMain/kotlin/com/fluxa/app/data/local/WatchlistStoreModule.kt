package com.fluxa.app.data.local

import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
abstract class WatchlistStoreModule {
    @Binds
    @Singleton
    abstract fun bindWatchlistStore(store: AndroidWatchlistStore): WatchlistStore
}
