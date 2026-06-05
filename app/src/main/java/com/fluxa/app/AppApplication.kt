package com.fluxa.app

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.app.Application
import dagger.hilt.android.HiltAndroidApp
import coil3.ImageLoader
import coil3.PlatformContext
import coil3.SingletonImageLoader
import coil3.disk.DiskCache
import coil3.svg.SvgDecoder
import coil3.gif.AnimatedImageDecoder
import coil3.memory.MemoryCache
import coil3.request.crossfade
import coil3.network.okhttp.OkHttpNetworkFetcherFactory
import okio.Path.Companion.toOkioPath
import com.lagradost.cloudstream3.AcraApplication
import com.lagradost.cloudstream3.app
import com.fluxa.app.ui.catalog.AppStrings
import com.fluxa.app.ui.catalog.EpisodeReleaseWorker
import com.fluxa.app.plugins.PluginAutoUpdateWorker
import okhttp3.OkHttpClient
import androidx.work.Configuration
import androidx.hilt.work.HiltWorkerFactory
import javax.inject.Inject
import javax.inject.Named

@HiltAndroidApp
class AppApplication : Application(), SingletonImageLoader.Factory, Configuration.Provider {

    @Inject lateinit var workerFactory: HiltWorkerFactory
    @Inject @Named("ImageClient") lateinit var imageClient: OkHttpClient


    override fun onCreate() {
        super.onCreate()

        // Initialize CloudStream3 context
        AcraApplication.init(this)
        // CloudStream library's app singleton is auto-initialized by the library

        AppStrings.initialize(this)
        PluginAutoUpdateWorker.enqueue(this)
        EpisodeReleaseWorker.enqueue(this)
        com.fluxa.app.player.TorrentStreamManager.getInstance(this).startEngineEarly(this)
    }

    override fun onTerminate() {
        super.onTerminate()
    }

    override fun newImageLoader(context: PlatformContext): ImageLoader {
        return ImageLoader.Builder(context)
            .memoryCache {
                MemoryCache.Builder()
                    .maxSizePercent(context, 0.30)
                    .build()
            }
            .diskCache {
                DiskCache.Builder()
                    .directory(cacheDir.resolve("image_cache").toOkioPath())
                    .maxSizeBytes(256L * 1024L * 1024L)
                    .build()
            }
            .crossfade(true)
            // Coil3 needs an explicit network backend; provide our OkHttp client as the fetcher.
            .components {
                add(OkHttpNetworkFetcherFactory(callFactory = { imageClient }))
                add(AnimatedImageDecoder.Factory())
                add(SvgDecoder.Factory())
            }
            .build()
    }

    override val workManagerConfiguration: Configuration
        get() =
        Configuration.Builder()
            .setWorkerFactory(workerFactory)
            .build()
}
