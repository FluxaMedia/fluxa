package com.fluxa.app.di

import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.StreamDiscoveryMemoryCache
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Named
import javax.inject.Singleton
import com.fluxa.app.data.remote.ExternalSyncApi
import com.fluxa.app.data.remote.TmdbService
import com.fluxa.app.data.platform.AndroidPlatformFileStore
import com.fluxa.app.data.platform.AndroidPlatformKeyValueStore
import com.fluxa.app.data.platform.AndroidPlatformSecureStore
import com.fluxa.app.data.platform.PlatformFileStore
import com.fluxa.app.data.platform.PlatformKeyValueStore
import com.fluxa.app.data.platform.PlatformSecureStore
import android.content.Context
import com.google.gson.Gson
import dagger.hilt.android.qualifiers.ApplicationContext

@Module
@InstallIn(SingletonComponent::class)
object RepositoryModule {

    @Provides
    @Singleton
    fun providePlatformFileStore(@ApplicationContext context: Context): PlatformFileStore {
        return AndroidPlatformFileStore(context.cacheDir)
    }

    @Provides
    @Singleton
    @Named("SimklSyncDelta")
    fun provideSimklSyncDeltaStore(@ApplicationContext context: Context): PlatformKeyValueStore {
        return AndroidPlatformKeyValueStore(context.getSharedPreferences("simkl_sync_delta", Context.MODE_PRIVATE))
    }

    @Provides
    @Singleton
    @Named("TraktSyncDelta")
    fun provideTraktSyncDeltaStore(@ApplicationContext context: Context): PlatformKeyValueStore {
        return AndroidPlatformKeyValueStore(context.getSharedPreferences("trakt_sync_delta", Context.MODE_PRIVATE))
    }

    @Provides
    @Singleton
    @Named("NuvioSyncDelta")
    fun provideNuvioSyncDeltaStore(@ApplicationContext context: Context): PlatformKeyValueStore {
        return AndroidPlatformKeyValueStore(context.getSharedPreferences("nuvio_sync_delta", Context.MODE_PRIVATE))
    }

    @Provides
    @Singleton
    @Named("PluginRepositoryState")
    fun providePluginRepositoryStateStore(@ApplicationContext context: Context): PlatformKeyValueStore {
        return AndroidPlatformKeyValueStore(context.getSharedPreferences("fluxa_plugin_repository_manager", Context.MODE_PRIVATE))
    }

    @Provides
    @Singleton
    @Named("ProfilePrefs")
    fun provideProfilePrefsStore(@ApplicationContext context: Context): PlatformKeyValueStore {
        return AndroidPlatformKeyValueStore(context.getSharedPreferences("fluxa_profiles", Context.MODE_PRIVATE))
    }

    @Provides
    @Singleton
    fun providePlatformSecureStore(impl: AndroidPlatformSecureStore): PlatformSecureStore = impl

    @Provides
    @Singleton
    fun provideOAuthClientConfig(): OAuthClientConfig {
        return OAuthClientConfig(
            traktClientId = com.fluxa.app.BuildConfig.TRAKT_CLIENT_ID,
            traktClientSecret = com.fluxa.app.BuildConfig.TRAKT_CLIENT_SECRET.takeIf { it.isNotBlank() },
            simklClientId = com.fluxa.app.BuildConfig.SIMKL_CLIENT_ID,
            anilistClientId = com.fluxa.app.BuildConfig.ANILIST_CLIENT_ID
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
