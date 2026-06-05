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
        traktApi: TraktApi,
        addonRepository: AddonRepository,
        externalLibraryClient: ExternalLibraryClient,
        traktSyncClient: TraktSyncClient
    ): TraktRepository {
        return TraktRepository(traktApi, addonRepository, externalLibraryClient, traktSyncClient)
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
