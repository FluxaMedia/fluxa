package com.fluxa.app.di

import com.fluxa.app.core.background.AndroidBackgroundTaskScheduler
import com.fluxa.app.domain.background.BackgroundTaskScheduler
import com.fluxa.app.domain.playback.PlaybackProgressScheduler
import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent

@Module
@InstallIn(SingletonComponent::class)
abstract class BackgroundSchedulerModule {
    @Binds
    abstract fun bindBackgroundTaskScheduler(
        implementation: AndroidBackgroundTaskScheduler,
    ): BackgroundTaskScheduler

    @Binds
    abstract fun bindPlaybackProgressScheduler(
        implementation: AndroidBackgroundTaskScheduler,
    ): PlaybackProgressScheduler
}
