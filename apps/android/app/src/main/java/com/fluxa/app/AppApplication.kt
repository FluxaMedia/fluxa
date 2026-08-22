package com.fluxa.app

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.app.Application
import android.app.ActivityManager
import dagger.hilt.android.HiltAndroidApp
import coil3.ImageLoader
import coil3.PlatformContext
import coil3.SingletonImageLoader
import coil3.disk.DiskCache
import coil3.svg.SvgDecoder
import coil3.gif.AnimatedImageDecoder
import coil3.memory.MemoryCache
import coil3.request.crossfade
import coil3.size.Precision
import coil3.network.okhttp.OkHttpNetworkFetcherFactory
import com.fluxa.app.core.rust.FluxaCoreNative
import okio.Path.Companion.toOkioPath
import com.lagradost.cloudstream3.AcraApplication
import com.lagradost.cloudstream3.app
import com.fluxa.app.common.AppStrings
import com.fluxa.app.common.initialize
import com.fluxa.app.ui.catalog.TrailerResolver
import com.fluxa.app.domain.background.BackgroundTaskScheduler
import okhttp3.OkHttpClient
import androidx.work.Configuration
import androidx.hilt.work.HiltWorkerFactory
import javax.inject.Inject
import javax.inject.Named
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

@HiltAndroidApp
class AppApplication : Application(), SingletonImageLoader.Factory, Configuration.Provider {

    @Inject lateinit var workerFactory: HiltWorkerFactory
    @Inject @Named("ImageClient") lateinit var imageClient: OkHttpClient
    @Inject lateinit var profileManager: ProfileManager
    @Inject lateinit var backgroundTaskScheduler: BackgroundTaskScheduler

    private val appScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    override fun onCreate() {
        super.onCreate()

        // Initialize CloudStream3 context
        AcraApplication.init(this)
        // CloudStream library's app singleton is auto-initialized by the library

        AppStrings.initialize(this)
        TrailerResolver.init(cacheDir)
        backgroundTaskScheduler.scheduleEpisodeReleaseChecks()
        com.fluxa.app.player.TorrentStreamManager.getInstance(this).startEngineEarly(this)
        appScope.launch { profileManager.getProfiles() }
    }

    override fun onTerminate() {
        super.onTerminate()
    }

    override fun newImageLoader(context: PlatformContext): ImageLoader {
        val activityManager = context.getSystemService(ActivityManager::class.java)
        val isLowRam = activityManager?.isLowRamDevice == true
        val isTelevision = packageManager.hasSystemFeature(android.content.pm.PackageManager.FEATURE_LEANBACK)
        val memInfo = ActivityManager.MemoryInfo()
        activityManager?.getMemoryInfo(memInfo)
        val budget = FluxaCoreNative.deviceResourceBudget(
            totalRamMb = memInfo.totalMem / (1024L * 1024L),
            heapMaxMb = Runtime.getRuntime().maxMemory() / (1024L * 1024L),
            isLowRamDevice = isLowRam,
            isTelevision = isTelevision
        )
        return ImageLoader.Builder(context)
            .memoryCache {
                MemoryCache.Builder()
                    .maxSizePercent(context, budget.imageCacheMemoryPercent)
                    .build()
            }
            .diskCache {
                DiskCache.Builder()
                    .directory(cacheDir.resolve("image_cache").toOkioPath())
                    .maxSizeBytes(if (isLowRam) 128L * 1024L * 1024L else 256L * 1024L * 1024L)
                    .build()
            }
            .crossfade(budget.imageCrossfadeEnabled)
            .precision(if (budget.imagePrecisionInexact) Precision.INEXACT else Precision.EXACT)
            .fetcherCoroutineContext(Dispatchers.IO.limitedParallelism(budget.imageDecodeConcurrency))
            .decoderCoroutineContext(Dispatchers.IO.limitedParallelism(budget.imageDecodeConcurrency))
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
