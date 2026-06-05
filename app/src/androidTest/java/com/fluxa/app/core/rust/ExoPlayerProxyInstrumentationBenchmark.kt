package com.fluxa.app.core.rust

import android.graphics.SurfaceTexture
import android.view.Surface
import androidx.media3.common.MediaItem
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import org.junit.After
import org.junit.Assert.assertTrue
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class ExoPlayerProxyInstrumentationBenchmark {
    private val sessions = mutableListOf<FluxaLocalStreamServer.Session>()
    private val players = mutableListOf<ExoPlayer>()
    private val surfaces = mutableListOf<Surface>()
    private val textures = mutableListOf<SurfaceTexture>()

    @After
    fun tearDown() {
        players.forEach { it.release() }
        surfaces.forEach { it.release() }
        textures.forEach { it.release() }
        sessions.forEach { it.stop() }
    }

    @Test
    fun benchmarkExoPlayerFirstFrameAndSeekDelaysThroughRustProxy() {
        val args = InstrumentationRegistry.getArguments()
        val mediaUrl = args.getString("mediaUrl").orEmpty()
        assumeTrue("Pass -e mediaUrl <direct media url> to run this benchmark.", mediaUrl.isNotBlank())

        val session = FluxaLocalStreamServer.start(mediaUrl, emptyMap())
        assumeTrue("Rust local stream server did not start.", session != null)
        sessions += session!!

        val player = newPlayer()
        players += player

        val firstFrameMs = measureFirstFrameMs(player, session.url)
        println("exo-rust-proxy-time-to-first-frame-ms=$firstFrameMs")

        listOf(
            "seek-10s" to 10_000L,
            "seek-5m" to 5 * 60_000L,
            "seek-30m" to 30 * 60_000L
        ).forEach { (label, positionMs) ->
            val delayMs = measureSeekReadyMs(player, positionMs)
            println("exo-rust-proxy-$label-ready-ms=$delayMs")
        }
    }

    private fun newPlayer(): ExoPlayer {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        val texture = SurfaceTexture(0)
        val surface = Surface(texture)
        textures += texture
        surfaces += surface
        return ExoPlayer.Builder(context)
            .build()
            .also { it.setVideoSurface(surface) }
    }

    private fun measureFirstFrameMs(player: ExoPlayer, url: String): Long {
        val rendered = CountDownLatch(1)
        var failure: PlaybackException? = null
        val listener = object : Player.Listener {
            override fun onRenderedFirstFrame() {
                rendered.countDown()
            }

            override fun onPlayerError(error: PlaybackException) {
                failure = error
                rendered.countDown()
            }
        }
        player.addListener(listener)
        val start = System.nanoTime()
        player.setMediaItem(MediaItem.fromUri(url))
        player.prepare()
        player.playWhenReady = true
        assertTrue("Timed out waiting for first frame. error=$failure", rendered.await(60, TimeUnit.SECONDS))
        player.removeListener(listener)
        failure?.let { throw it }
        return (System.nanoTime() - start) / 1_000_000L
    }

    private fun measureSeekReadyMs(player: ExoPlayer, positionMs: Long): Long {
        val ready = CountDownLatch(1)
        var failure: PlaybackException? = null
        val listener = object : Player.Listener {
            override fun onPlaybackStateChanged(playbackState: Int) {
                if (playbackState == Player.STATE_READY && player.playWhenReady) {
                    ready.countDown()
                }
            }

            override fun onPlayerError(error: PlaybackException) {
                failure = error
                ready.countDown()
            }
        }
        player.addListener(listener)
        val start = System.nanoTime()
        player.seekTo(positionMs)
        player.playWhenReady = true
        assertTrue("Timed out waiting for seek readiness at $positionMs ms. error=$failure", ready.await(60, TimeUnit.SECONDS))
        player.removeListener(listener)
        failure?.let { throw it }
        return (System.nanoTime() - start) / 1_000_000L
    }
}
