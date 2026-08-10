@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.ui.catalog

import androidx.media3.datasource.DataSource
import androidx.media3.datasource.DataSpec
import androidx.media3.datasource.TransferListener
import androidx.media3.datasource.okhttp.OkHttpDataSource
import androidx.media3.exoplayer.DefaultLoadControl
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.exoplayer.trackselection.DefaultTrackSelector
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import okhttp3.OkHttpClient

internal object TrailerResolver {

    private val httpClient = OkHttpClient.Builder().build()

    fun init(cacheDir: java.io.File) = Unit

    fun mediaDataSourceFactory(): DataSource.Factory {
        val upstream = OkHttpDataSource.Factory(httpClient)
            .setUserAgent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        return DataSource.Factory { TrailerRangeDataSource(upstream.createDataSource()) }
    }

    fun createAutoplayTrailerPlayer(context: android.content.Context, url: String): ExoPlayer {
        val trackSelector = DefaultTrackSelector(context).apply {
            setParameters(
                buildUponParameters()
                    .setMaxVideoSize(1280, 720)
                    // Home/detail autoplay trailers are muted by design. Prevent MediaCodec
                    // and the audio DSP path from decoding a track that can never be heard.
                    .setTrackTypeDisabled(C.TRACK_TYPE_AUDIO, true)
            )
        }
        val loadControl = DefaultLoadControl.Builder()
            .setBufferDurationsMs(250, 750, 150, 250)
            .setPrioritizeTimeOverSizeThresholds(true)
            .build()
        return ExoPlayer.Builder(context)
            .setTrackSelector(trackSelector)
            .setLoadControl(loadControl)
            .setMediaSourceFactory(DefaultMediaSourceFactory(mediaDataSourceFactory()))
            .setUsePlatformDiagnostics(false)
            // A background hero trailer must never trigger a TV display refresh-rate/mode
            // switch. Main content playback still owns the normal frame-rate strategy.
            .setVideoChangeFrameRateStrategy(C.VIDEO_CHANGE_FRAME_RATE_STRATEGY_OFF)
            .build()
            .apply {
                setMediaItem(MediaItem.fromUri(url))
                prepare()
            }
    }
}

internal object DetailTrailerPreloader {
    private var preparedUrl: String? = null
    private var preparedPlayer: ExoPlayer? = null

    fun preload(context: android.content.Context, url: String) {
        if (preparedUrl == url && preparedPlayer != null) return
        discard()
        preparedUrl = url
        preparedPlayer = TrailerResolver.createAutoplayTrailerPlayer(context.applicationContext, url)
    }

    fun takeOrCreate(context: android.content.Context, url: String): ExoPlayer {
        if (preparedUrl == url) {
            val player = preparedPlayer
            preparedUrl = null
            preparedPlayer = null
            if (player != null) return player
        }
        return TrailerResolver.createAutoplayTrailerPlayer(context.applicationContext, url)
    }

    fun discard() {
        preparedUrl = null
        preparedPlayer?.release()
        preparedPlayer = null
    }
}

private class TrailerRangeDataSource(
    private val delegate: DataSource
) : DataSource {
    override fun addTransferListener(transferListener: TransferListener) {
        delegate.addTransferListener(transferListener)
    }

    override fun open(dataSpec: DataSpec): Long {
        val headers = dataSpec.httpRequestHeaders
        val request = if (headers.keys.any { it.equals("Range", ignoreCase = true) }) {
            dataSpec
        } else {
            dataSpec.withRequestHeaders(headers + ("Range" to "bytes=0-"))
        }
        return delegate.open(request)
    }

    override fun read(buffer: ByteArray, offset: Int, length: Int): Int = delegate.read(buffer, offset, length)

    override fun getUri() = delegate.uri

    override fun getResponseHeaders() = delegate.responseHeaders

    override fun close() = delegate.close()
}
