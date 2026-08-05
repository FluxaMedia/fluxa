package com.fluxa.app.di

import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.StreamDiscoveryMemoryCache
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import com.fluxa.app.data.remote.ExternalSyncApi
import com.fluxa.app.data.remote.TmdbService
import android.content.Context
import com.google.gson.Gson
import dagger.hilt.android.qualifiers.ApplicationContext

@Module
@InstallIn(SingletonComponent::class)
object RepositoryModule {

    @Provides
    @Singleton
    fun provideOAuthClientConfig(): OAuthClientConfig {
        return OAuthClientConfig(
            traktClientId = com.fluxa.app.BuildConfig.TRAKT_CLIENT_ID,
            traktClientSecret = com.fluxa.app.BuildConfig.TRAKT_CLIENT_SECRET.takeIf { it.isNotBlank() },
            simklClientId = com.fluxa.app.BuildConfig.SIMKL_CLIENT_ID,
            simklClientSecret = com.fluxa.app.BuildConfig.SIMKL_CLIENT_SECRET.takeIf { it.isNotBlank() },
            anilistClientId = com.fluxa.app.BuildConfig.ANILIST_CLIENT_ID,
            anilistClientSecret = com.fluxa.app.BuildConfig.ANILIST_CLIENT_SECRET.takeIf { it.isNotBlank() }
        )
    }

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
        externalSyncApi: ExternalSyncApi,
        addonRepository: AddonRepository,
        externalLibraryClient: ExternalLibraryClient,
        traktSyncClient: TraktSyncClient,
        gson: Gson,
        oauthClientConfig: OAuthClientConfig
    ): TraktRepository {
        return TraktRepository(context, externalSyncApi, addonRepository, externalLibraryClient, traktSyncClient, gson, oauthClientConfig)
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
