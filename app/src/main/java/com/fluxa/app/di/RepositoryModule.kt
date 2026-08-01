package com.fluxa.app.di

import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.StreamDiscoveryMemoryCache
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import com.fluxa.app.data.remote.TraktApi
import com.fluxa.app.data.remote.TmdbService
import android.content.Context
import com.google.gson.Gson
import dagger.hilt.android.qualifiers.ApplicationContext

@Module
@InstallIn(SingletonComponent::class)
object RepositoryModule {

    @Provides
    @Singleton
    fun provideAddonRepository(
        manifestClient: StremioAddonManifestClient,
        resourceClient: StremioAddonResourceClient
    ): AddonRepository {
        return AddonRepository(manifestClient, resourceClient)
    }

    @Provides
    @Singleton
    fun provideTraktRepository(
        @ApplicationContext context: Context,
        traktApi: TraktApi,
        addonRepository: AddonRepository,
        externalLibraryClient: ExternalLibraryClient,
        traktSyncClient: TraktSyncClient,
        gson: Gson
    ): TraktRepository {
        return TraktRepository(context, traktApi, addonRepository, externalLibraryClient, traktSyncClient, gson)
    }

    @Provides
    @Singleton
    fun provideTmdbRepository(
        tmdbService: TmdbService
    ): TmdbRepository {
        return TmdbRepository(tmdbService)
    }

    @Provides
    @Singleton
    fun provideStreamDiscoveryMemoryCache(): StreamDiscoveryMemoryCache {
        return StreamDiscoveryMemoryCache()
    }
}
