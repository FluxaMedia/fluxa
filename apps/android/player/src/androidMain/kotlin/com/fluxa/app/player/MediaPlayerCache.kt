package com.fluxa.app.player

import android.content.Context
import android.net.Uri
import androidx.media3.common.C
import androidx.media3.datasource.DataSource
import androidx.media3.datasource.DataSpec
import androidx.media3.datasource.TransferListener
import androidx.media3.datasource.cache.CacheDataSink
import androidx.media3.datasource.cache.CacheDataSource
import androidx.media3.datasource.cache.CacheWriter
import androidx.media3.datasource.cache.LeastRecentlyUsedCacheEvictor
import androidx.media3.datasource.cache.SimpleCache
import androidx.media3.database.StandaloneDatabaseProvider
import androidx.media3.datasource.okhttp.OkHttpDataSource

internal object MediaPlayerCache {
    private const val PLAYER_CACHE_FRAGMENT_BYTES = 8L * 1024L * 1024L
    private const val PREFS_PLAYER = "fluxa_player"
    private const val PREF_BW_ESTIMATE_BPS = "bw_estimate_bps"
    @Volatile private var playerDiskCache: SimpleCache? = null
    @Volatile private var lastPersistedBandwidthAtMs: Long = 0L
    @Volatile private var lastPersistedBandwidthBps: Long = 0L

    fun savedBandwidthEstimate(context: Context): Long =
        context.getSharedPreferences(PREFS_PLAYER, Context.MODE_PRIVATE)
            .getLong(PREF_BW_ESTIMATE_BPS, 0L)

    fun saveBandwidthEstimate(context: Context, bps: Long) {
        if (bps <= 0L) return
        val now = android.os.SystemClock.elapsedRealtime()
        val previous = lastPersistedBandwidthBps
        val relativeChange = if (previous > 0L) {
            kotlin.math.abs(bps - previous).toDouble() / previous.toDouble()
        } else {
            1.0
        }
        if (now - lastPersistedBandwidthAtMs < 30_000L && relativeChange < 0.25) return
        lastPersistedBandwidthAtMs = now
        lastPersistedBandwidthBps = bps
        context.applicationContext.getSharedPreferences(PREFS_PLAYER, Context.MODE_PRIVATE)
            .edit().putLong(PREF_BW_ESTIMATE_BPS, bps).apply()
    }

    fun playerCache(context: Context): SimpleCache {
        return playerDiskCache ?: synchronized(this) {
            playerDiskCache ?: SimpleCache(
                context.applicationContext.cacheDir.resolve("player_http_cache"),
                LeastRecentlyUsedCacheEvictor(playerDiskCacheBytes(context)),
                StandaloneDatabaseProvider(context.applicationContext)
            ).also { playerDiskCache = it }
        }
    }

    fun primeHttpStream(
        context: Context,
        url: String,
        headers: Map<String, String>,
        primeBytes: Long = 2L * 1024L * 1024L
    ) {
        val uri = Uri.parse(url)
        if (!shouldUsePlayerDiskCache(uri)) return
        val okHttp = PlayerHttpResources.newBuilder()
            .callTimeout(10, java.util.concurrent.TimeUnit.SECONDS)
            .apply { cronetTransportInterceptor(context)?.let { addInterceptor(it) } }
            .build()
        val upstream = OkHttpDataSource.Factory(okHttp)
            .setUserAgent(StreamRequestPolicy.DEFAULT_USER_AGENT)
            .apply { if (headers.isNotEmpty()) setDefaultRequestProperties(headers) }
        val cacheDataSource = CacheDataSource.Factory()
            .setCache(playerCache(context))
            .setUpstreamDataSourceFactory(upstream)
            .setFlags(CacheDataSource.FLAG_IGNORE_CACHE_ON_ERROR)
            .createDataSource()
        runCatching { CacheWriter(cacheDataSource, DataSpec(uri, 0L, primeBytes), null, null).cache() }
    }

    fun dataSourceFactory(
        context: Context,
        upstream: DataSource.Factory,
        shouldUseCache: () -> Boolean
    ): DataSource.Factory {
        val cachedFactory = CacheDataSource.Factory()
            .setCache(playerCache(context))
            .setUpstreamDataSourceFactory(upstream)
            .setCacheWriteDataSinkFactory(
                CacheDataSink.Factory()
                    .setCache(playerCache(context))
                    .setFragmentSize(PLAYER_CACHE_FRAGMENT_BYTES)
            )
            .setFlags(CacheDataSource.FLAG_IGNORE_CACHE_ON_ERROR)
        return DataSource.Factory {
            SelectiveCacheDataSource(cachedFactory, upstream, shouldUseCache)
        }
    }

    private fun playerDiskCacheBytes(context: Context): Long {
        val activityManager = context.getSystemService(android.app.ActivityManager::class.java)
        return when {
            activityManager?.isLowRamDevice == true -> 128L * 1024L * 1024L
            context.packageManager.hasSystemFeature(android.content.pm.PackageManager.FEATURE_LEANBACK) -> 256L * 1024L * 1024L
            else -> 512L * 1024L * 1024L
        }
    }

    private fun shouldUsePlayerDiskCache(uri: Uri): Boolean {
        val scheme = uri.scheme?.lowercase() ?: return false
        if (scheme != "http" && scheme != "https") return false
        val host = uri.host?.lowercase() ?: return false
        return host != "localhost" && host != "127.0.0.1" && host != "::1"
    }

    private class SelectiveCacheDataSource(
        private val cachedFactory: DataSource.Factory,
        private val uncachedFactory: DataSource.Factory,
        private val shouldUseCache: () -> Boolean
    ) : DataSource {
        private var active: DataSource? = null
        private val transferListeners = mutableListOf<TransferListener>()

        override fun addTransferListener(transferListener: TransferListener) {
            transferListeners += transferListener
            active?.addTransferListener(transferListener)
        }

        override fun open(dataSpec: DataSpec): Long {
            val selected = if (shouldUseCache() && shouldUsePlayerDiskCache(dataSpec.uri)) {
                cachedFactory.createDataSource()
            } else {
                uncachedFactory.createDataSource()
            }
            transferListeners.forEach(selected::addTransferListener)
            active = selected
            return selected.open(dataSpec)
        }

        override fun read(buffer: ByteArray, offset: Int, length: Int): Int =
            active?.read(buffer, offset, length) ?: C.RESULT_END_OF_INPUT

        override fun getUri(): Uri? = active?.uri

        override fun getResponseHeaders(): Map<String, List<String>> =
            active?.responseHeaders ?: emptyMap()

        override fun close() {
            active?.close()
            active = null
        }
    }
}
